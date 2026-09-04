# Contratos IPC

> Estado: `[IMPLEMENTED]` com modelo de erro `[PARTIAL]`.

## Fronteira

Tauri IPC liga o frontend TypeScript ao backend Rust. Mesmo em um aplicativo local, essa fronteira recebe dados controláveis pelo frontend e deve validá-los.

```text
gitServices.ts → invoke(command, payload) → commands.rs → service.rs
```

## Contratos atuais

| Command | Entrada | Saída |
|---|---|---|
| `open_project` | `path` | `ProjectOpenResult` |
| `create_repository` | `path`, `branch`, `message` | `ProjectOpenResult` |
| `get_repository_details` | `path` | `GitRepositoryDetails` |
| `get_repository_graph` | `path` | `GitGraph` |
| `stage_file` | `path`, `file` | `void` |
| `stage_all` | `path` | `void` |
| `unstage_file` | `path`, `file` | `void` |
| `create_commit` | `path`, `message` | `string` |
| `get_commit_diff` | `path`, `revision` | `string` |
| `create_branch_from` | `path`, `startPoint`, `branchName` | `void` |
| `switch_branch` | `path`, `branchName` | `void` |
| `get_revert_preview` | `path`, `revision` | `GitRevertPreview` |
| `revert_commit` | `path`, `revision` | `GitRevertOutcome` |
| `get_remotes` | `path` | `GitRemotesView` |
| `add_remote` | `path`, `name`, `url` | `void` |
| `remove_remote` | `path`, `name` | `void` |
| `rename_remote` | `path`, `from`, `to` | `void` |
| `set_remote_url` | `path`, `name`, `url`, `pushOnly` | `void` |
| `get_push_plan` | `path`, `remoteName?`, `sourceBranch?`, `targetBranch?` | `GitPushPlan` |
| `push_branch` | `path`, `remoteName?`, `sourceBranch?`, `targetBranch?`, `setUpstream` | `GitPushOutcome` |
| `fetch_remote` | `path`, `remoteName?`, `prune` | `GitFetchOutcome` |
| `get_pull_plan` | `path`, `remoteName?`, `remoteBranch?` | `GitPullPlan` |
| `pull_branch` | `path`, `remoteName?`, `remoteBranch?`, `strategy` | `GitPullOutcome` |
| `get_branch_comparison` | `path`, `base`, `head` | `GitBranchComparison` |
| `get_comparison_diff` | `path`, `base`, `head` | `string` (diff unificado) |
| `get_github_context` | `path` | `GithubContext` |
| `list_pull_requests` | `path`, `headBranch?` | `GithubPullRequest[]` |
| `create_pull_request` | `path`, `title`, `body`, `head`, `base?`, `draft` | `string` (URL) |
| `open_github_in_browser` | `path`, `branch?` | `string` (URL) |
| `get_pull_request` | `path`, `number` | `GithubPullRequestDetail` |
| `get_pull_request_diff` | `path`, `number` | `string` (diff unificado) |
| `get_pull_request_plan` | `path`, `number` | `GithubPullRequestPlan` |
| `merge_pull_request` | `path`, `number`, `method`, `deleteBranch`, `expectedHeadSha?` | `GithubMergeOutcome` |
| `close_pull_request` | `path`, `number`, `deleteBranch`, `expectedHeadSha?` | `GithubPullRequestDetail` |
| `get_activity_stream` | `path`, `limit?` | `ActivityStream` |

Os commands acima da linha do revert devolvem `Result<T, String>`; do revert
em diante, `Result<T, GitOperationError>`. O contrato duplo é deliberado:
migrar os antigos exigiria tocar em tudo que já funciona.

Tauri converte camelCase do payload TypeScript para os parâmetros Rust declarados nos commands conforme a integração atual.

## Tipos compartilhados manualmente

- `ProjectOpenResult`
- `GitState`
- `GitFileStatus`
- `GitCommit`
- `GitRepositoryDetails`
- `GitBranch`
- `GitGraphCommit`
- `GitGraph`

As interfaces TypeScript espelham structs Rust, mas não existe geração automática nem teste de contrato.

## Política desejada de resposta

Hoje: `Result<T, String>`.

Destino:

```ts
type BackendError = {
  code: string
  message: string
  operation: string
  recoverable: boolean
  details?: Record<string, unknown>
}
```

Não retornar tokens, credenciais, headers ou comandos com segredos.

## Validações por entrada

O `path` do IPC **não é credencial**. Ele é uma referência que precisa
corresponder a um workspace já registrado — o registro acontece uma vez, quando
o usuário abre o projeto (`open_project`), e é verificado em toda chamada
seguinte. Um caminho válido que o usuário nunca abriu é recusado.

- `path`: `core::state::authorize_workspace` — resolve canonicamente, compara
  com o registro e devolve a raiz REGISTRADA, que é a usada na execução.
- `file`: resolvido em relação ao repositório; `git add`/`restore` usam `--`.
- `branchName`: `core::process::operand` **antes** de qualquer execução, depois
  `git check-ref-format --branch`, depois `--` no comando que aceita.
- `startPoint`/`revision`: `operand` antes de `git rev-parse --verify`. A ordem
  importa: `rev-parse` é um processo, e um valor iniciado por `-` já teria sido
  interpretado por ele.
- `remote`: precisa existir no repositório; URL por allowlist de protocolo, com
  `ext::` e `fd::` recusados.
- `message`: não vazia; passa como argumento separado, nunca por shell.
- `base`/`head` do compare: `core::process::operand` **antes** de `git rev-parse
  --verify`; referência inexistente é recusada com `INVALID_COMMIT`.

### Validações do provider GitHub

- Referências (`headBranch`, `head`, `base`, `branch`): `core::process::operand`
  antes de virar argumento da `gh`. Um valor iniciado por `-` seria lido por ela
  como OPÇÃO.
- Texto livre (`title`, `body`): forma `--flag=valor`, de argumento único. Não
  pode ser lido como opção e não proíbe título legítimo começando com `-`.
- `number`: `u64` no contrato IPC — não há string a injetar.
- `method`: comparado com uma lista fechada (`merge`, `squash`, `rebase`); o
  que chega à `gh` é a flag constante correspondente, nunca a string recebida.
  `--admin` e `--auto` não existem no catálogo, de propósito.
- `deleteBranch`: destrutivo e opt-in; só entra na linha de comando quando a
  interface pede explicitamente.
- `expectedHeadSha`: o commit que o usuário revisou. O backend recalcula o
  preflight antes de mutar e aborta se o topo da origem mudou; a mesma exigência
  é repassada ao GitHub via `--match-head-commit`.

### Preflight das operações GitHub

`merge_pull_request` e `close_pull_request` seguem o mesmo padrão de push e
pull: `get_pull_request_plan` → confirmação na interface → execução que
**recalcula o plano** e recusa enquanto `blocked` não for nulo. `blocked`,
permissões, conflitos e estado do PR são determinados pelo backend; a interface
apenas apresenta. Isso é preflight revalidado, **não** `core::approval` — a
distinção está registrada em [`../security/enforcement-state.md`](../security/enforcement-state.md).

### Códigos de erro do Core

Recusa de segurança cruza o IPC com o código do próprio Core, sem tradução:

```text
WORKSPACE_NOT_REGISTERED   o caminho não corresponde a um projeto aberto
WORKSPACE_NOT_TRUSTED      confiança insuficiente para esta operação
PATH_OUTSIDE_SCOPE         o recurso resolvido cai fora da autoridade
PATH_UNRESOLVABLE          o caminho não resolve para um objeto real
ARGUMENT_REJECTED          o valor viraria opção do programa
EXECUTION_TIMEOUT          o processo passou do prazo e foi encerrado
EXECUTION_FAILED           o processo não pôde ser iniciado
RESOURCE_LOCKED            outro fluxo detém a autoridade sobre o recurso
APPROVAL_STALE             a aprovação não corresponde ao estado atual
APPROVAL_EXPIRED           a aprovação expirou
POLICY_DENIED              a política negou a ação
APPROVAL_REQUIRED          falta aprovação explícita
```

Os quatro últimos existem no tipo mas **nenhum fluxo os emite hoje**.

## Mudança de comportamento registrada

`open_project` passou a devolver o caminho sem o prefixo verbatim do Windows
(`\\?\C:\...` vira `C:\...`). O nome do command, o payload e o formato do
retorno são os mesmos; muda o VALOR da string. A autoridade não depende dessa
forma — ela é reresolvida canonicamente a cada chamada —, então um caminho
guardado no formato antigo continua funcionando.

## Regra de compatibilidade

Mudança em nome, payload ou retorno IPC é breaking change interno e exige atualização simultânea de Rust, TypeScript, documentação e testes.
