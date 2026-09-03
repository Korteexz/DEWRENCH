# Módulo Git — segurança operacional

> Estado: política `[PLANNED/PARTIAL]`; algumas proteções técnicas já existem.

## Classificação

### Leitura

- status, log, branches, graph, diff, remote info.
- Sem confirmação.
- Deve permitir refresh e cancelamento quando custoso.

### Mutação recuperável

- stage, unstage, commit, criar branch, fetch, pull, push comum, merge, revert.
- Exige feedback de progresso e resultado.
- Confirmação depende do preflight e impacto.

### Alto impacto

- trocar branch com risco de sobrescrita;
- apagar branch;
- abortar operações intermediárias;
- descartar mudanças selecionadas.
- Exige preview de consequência e confirmação explícita.

### Crítica/destrutiva

- `reset --hard`;
- `clean`;
- force push;
- rewrite de histórico;
- descarte em lote.
- Não existe no MVP atual e não deve ser adicionado sem política, preflight e testes dedicados.

## Proteções implementadas

- Processo Git usa argumentos separados, sem shell interpolation.
- Branch/ref são verificadas em operações específicas.
- `--` separa revisão de path no diff e path de argumentos em stage/unstage.
- A UI bloqueia segunda mutação enquanto `busyAction` está ativo.
- Abertura canonicaliza a pasta selecionada.

## Lacunas

- lock backend por repositório;
- validação consistente de path/file em toda operação;
- symlink policy;
- timeout/cancelamento;
- typed errors;
- confirmação por risco;
- audit trail;
- preflight de consequências;
- política para hooks/config remota;
- sanitização formal de logs.

## Regra central

Confirmações descrevem consequências. “Tem certeza?” é insuficiente.

Exemplo:

```text
Esta operação descartará 4 alterações locais não commitadas.
Repositório: /projeto
Branch: feature/x
Arquivos: ...
[Cancelar] [Entendo as consequências — continuar]
```

## Revert

> Estado: `[IMPLEMENTED]` para commits comuns e root commits.

`git revert` é apresentado como criação de um novo commit inverso. O histórico anterior é preservado e o commit original não é removido nem reescrito.

### Preflight

O backend valida, no preview e novamente antes da mutação: repositório válido, hash existente, objeto do tipo commit, ausência de operação intermediária (merge, revert, cherry-pick, rebase, bisect), ausência de conflitos ativos, ausência de mudanças staged, commit não-merge, identidade Git configurada e ausência de sobreposição entre alterações locais e arquivos do commit.

A confirmação descreve consequências: o que será criado, o que será preservado, quais arquivos são afetados e quais alterações locais permanecerão intocadas.

### Merge commits

`[LIMITAÇÃO CONHECIDA]` Reverter um merge exige escolher explicitamente a mainline, e essa escolha muda qual lado da história é desfeito. O DEWRENCH bloqueia com `MERGE_COMMIT_UNSUPPORTED` e explica o motivo. `-m 1` nunca é assumido, e nenhuma confirmação executável é exibida para um merge commit.

### Conflito

`[POLÍTICA]` O MVP não possui interface de resolução e continuação, portanto não deixa o repositório parado em `REVERT_HEAD`. Ao detectar conflito, o backend coleta os arquivos conflitantes, executa `git revert --abort` e só declara restauração após comprovar três condições: `REVERT_HEAD` ausente, HEAD idêntico ao anterior e status idêntico ao anterior. Comprovado, retorna `REVERT_CONFLICT_ABORTED`. Não comprovado, retorna `REVERT_CONFLICT_ABORT_FAILED`, marcado como não recuperável.

`reset --hard` nunca é usado como recuperação. Operações intermediárias que já existiam antes da ação do usuário nunca são canceladas automaticamente.
