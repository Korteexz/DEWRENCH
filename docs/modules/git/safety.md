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

`git revert` deve ser apresentado como criação de um novo commit inverso. O histórico anterior é preservado; conflitos ainda podem ocorrer.
