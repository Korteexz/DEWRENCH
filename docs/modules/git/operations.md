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

### Revert

Cria novo commit inverso e preserva histórico. Deve prever conflitos e explicar que não apaga o commit original.

### Merge

Precisa modelar fast-forward, merge commit, conflito, abort e continuação.
