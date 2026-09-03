# Módulo Git — operações

> Este documento descreve o comportamento da baseline e o contrato esperado de futuras operações.

## Operações implementadas

### Open project

- Command: `open_project`
- Git: `rev-parse --verify HEAD` apenas quando existe `.git`
- Mutação: não
- Saída: nome, path canonicalizado e estado Git
- Limites: detecção não cobre todos os formatos de worktree/repo aninhado.

### Create repository

- Command: `create_repository`
- Git: `check-ref-format --branch`, `init -b`, `add .`, `commit -m`
- Mutação: alta, porém normalmente recuperável
- Valida: pasta, branch e mensagem não vazia
- Falha parcial possível: repo pode permanecer inicializado/staged se commit falhar.

### Read details

- Command: `get_repository_details`
- Git: branch atual, status porcelain v1, log de 10 commits
- Mutação: não
- Retorno: `GitRepositoryDetails`

### Read graph

- Command: `get_repository_graph`
- Git: branches locais + `log --all --topo-order -80`
- Mutação: não
- Retorno: `GitGraph`

### Stage file

- Command: `stage_file`
- Git: `add -- <file>`
- Mutação: index
- Backend: implementado
- UI: `[KNOWN DEFECT]` botão individual não chama este fluxo.

### Stage all

- Command: `stage_all`
- Git: `add -A`
- Mutação: index
- Inclui: modificados, novos e removidos; respeita `.gitignore`.

### Unstage file

- Command: `unstage_file`
- Git: `restore --staged -- <file>`
- Mutação: index; working tree preservada

### Create commit

- Command: `create_commit`
- Git: `commit -m <message>`
- Mutação: histórico e index
- Valida: mensagem não vazia
- Git decide se há conteúdo staged e se identidade está configurada.

### Create branch from

- Command: `create_branch_from`
- Git: valida branch, verifica start point, executa `branch <name> <start>`
- Mutação: refs
- Não troca automaticamente para a nova branch.

### Switch branch

- Command: `switch_branch`
- Git: `switch <branch>`
- Mutação: HEAD e possivelmente working tree
- Pode falhar quando alterações locais seriam sobrescritas.

### Commit diff

- Command: `get_commit_diff`
- Git: verifica `<revision>^{commit}`, depois `show --format= --no-ext-diff --unified=3 <revision> --`
- Mutação: não
- Retorno: patch textual bruto.

### Revert preview

- Command: `get_revert_preview`
- Git: `rev-parse --is-inside-work-tree`, `rev-parse --verify --end-of-options <rev>^{commit}`, `rev-list --parents -n 1`, `rev-parse --git-path <marcador>`, `status --porcelain=v1 -z -uall`, `var GIT_AUTHOR_IDENT`, `diff-tree --name-status --no-renames -r -z --root`
- Mutação: não
- Retorno: `GitRevertPreview`
- Erros: `GitOperationError` tipado (ver `errors.md`)

### Revert commit

- Command: `revert_commit`
- Preflight: repete integralmente o preview imediatamente antes da mutação
- Git: `revert --no-edit <hash resolvido>`; em conflito, `revert --abort`
- Mutação: working tree, index e refs. Não reescreve histórico.
- Retorno: `GitRevertOutcome`, com o hash do novo commit lido do Git após o sucesso
- Bloqueios: merge commit, operação intermediária, conflito ativo, mudanças staged, identidade ausente, sobreposição com alterações locais
- Conflito: a tentativa é abortada e o estado anterior é comprovado (HEAD, status e ausência de `REVERT_HEAD`)
- Refresh: obrigatório após sucesso

`A → B → C → D` com revert de `C` produz `A → B → C → D → C'`. O commit original permanece.

## Template obrigatório para novas operações

```text
Nome:
Entradas:
Preflight:
Comando/adapter:
Muta working tree/index/refs/metadata/remoto:
Rede:
Nível de risco:
Confirmação:
Estado intermediário:
Saída estruturada:
Erros esperados:
Recuperação:
Refresh necessário:
Testes:
```

## Operações planejadas

### Fetch

Lê remote, atualiza refs remotas, não altera working tree. Precisa tratar ausência de remote, autenticação, rede e timeout.

### Pull

Pode buscar e integrar alterações. Antes de implementar, decidir estratégia explícita (merge/rebase/ff-only), preflight e recuperação.

### Push

Deve mostrar remote, branch, upstream e commits enviados. Force push fica em nível crítico e não pertence ao fluxo padrão.

### Merge

Precisa modelar fast-forward, merge commit, conflito, abort e continuação.

## Operações de rede

> Estado: `[IMPLEMENTED]`. Ver [ADR-007](../../decisions/ADR-007-rede-como-limite-de-confianca.md).

### Remotes

`remote.rs` cuida de configuração; `sync.rs` cuida de execução. A separação é
deliberada: configurar um destino e falar com ele têm riscos diferentes.

- listagem com URL de fetch e de push, marcação de `origin` e do upstream;
- identidade extraída da URL (host, owner, repositório, provider) sem carregar
  credencial;
- adicionar, remover, renomear e trocar URL, com validação por allowlist;
- remote principal = upstream da branch atual → `origin` → único configurado.

`origin` nunca é alterado sozinho: qualquer mudança exige ação explícita, e
trocar a URL de `origin` mostra a consequência antes de aplicar.

### Push

Preflight (`get_push_plan`) devolve source, destination, ahead, behind,
upstream e a lista real de commits a enviar. O push só executa depois disso, e
recalcula o plano no momento da execução — a tela pode ter ficado aberta
enquanto o repositório mudava.

Casos tratados com código próprio: `NOTHING_TO_PUSH`, `REMOTE_NOT_FOUND`,
`UNBORN_BRANCH`, `DETACHED_HEAD`, `NON_FAST_FORWARD`, `PUSH_REJECTED`,
`AUTHENTICATION_REQUIRED`, `NETWORK_UNREACHABLE`,
`REMOTE_REPOSITORY_NOT_FOUND`. O texto original do Git fica em `details`.

Force push não existe como ação nesta versão.

### Fetch

Não altera o working tree. O relatório do que mudou é construído comparando as
refs remotas antes e depois — dado real, não interpretação do texto do Git,
que muda entre versões. `origin/HEAD` é ignorado por ser ponteiro simbólico.

A contagem total de commits recebidos é a UNIÃO entre refs, não a soma: duas
branches que avançam juntas compartilham commits.

### Pull

Preflight devolve incoming, outgoing, estratégias possíveis, alterações locais
e risco de conflito (interseção entre arquivos sujos e arquivos que chegam).

Estratégias: `fast-forward` (só quando não há commits locais pendentes),
`merge` e `rebase`. O backend recomenda, o usuário escolhe, e o `git pull`
genérico nunca é usado — seu comportamento depende de `pull.rebase`, que é
configuração global invisível.

Conflito desfaz a integração e reporta os arquivos. O repositório nunca fica
parado num merge pela metade.
