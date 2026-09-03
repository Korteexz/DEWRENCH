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
| `get_github_context` | `path` | `GithubContext` |
| `list_pull_requests` | `path`, `headBranch?` | `GithubPullRequest[]` |
| `create_pull_request` | `path`, `title`, `body`, `head`, `base?`, `draft` | `string` (URL) |
| `open_github_in_browser` | `path`, `branch?` | `string` (URL) |
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

- `path`: canonicalizar e confirmar escopo.
- `file`: resolver em relação ao repo e impedir escape.
- `branchName`: `git check-ref-format --branch`.
- `startPoint`/`revision`: `git rev-parse --verify` com tipo esperado.
- `remote`: allowlist de formato/protocolo e exibição clara do destino.
- `message`: não vazia; tamanho e caracteres devem ter limite razoável.

## Regra de compatibilidade

Mudança em nome, payload ou retorno IPC é breaking change interno e exige atualização simultânea de Rust, TypeScript, documentação e testes.
