# Módulo Git — arquitetura

> Estado: `[IMPLEMENTED/PARTIAL]`

## Frontend

```text
WorkspacePage
├── useGitGraph
├── gitServices
├── adaptGitGraph
├── layoutWorkspaceGraph
├── GitSidebar
├── GitGraphViewport → WorkspaceCanvas
└── GitInspectorPane → Project/Branch/CommitInspector
```

### Separações principais

- `gitServices.ts`: única camada que chama `invoke`.
- `useGitGraph.ts`: leitura e proteção contra resposta obsoleta.
- `gitGraphAdapter.ts`: converte domínio Git em nós/relações do workspace.
- `app/graph`: layout, foco, tipos e física visual.
- `WorkspacePage.tsx`: orquestra interação e mutações.

## Backend

```text
commands.rs
  → service.rs
    → repository.rs
    → working_tree.rs
    → commits.rs
    → branches.rs
    → graph.rs
      → git_cli.rs
```

## Modelos

- `GitState`: not_repository, unborn_repository, repository.
- `ProjectOpenResult`: nome, path canonicalizado, estado Git.
- `GitFileStatus`: path, status do index, status do working tree.
- `GitRepositoryDetails`: branch, arquivos, 10 commits recentes.
- `GitBranch`: nome, current, hash do head.
- `GitGraphCommit`: hash completo/curto, pais, autor, mensagem.
- `GitGraph`: branches + commits.

## Limites do adapter CLI

O adapter executa processo sem shell, o que deve ser preservado. Ele ainda precisa evoluir para:

- erro tipado;
- timeout/cancelamento;
- contexto seguro de log;
- captura estruturada de exit code;
- suporte a ambiente/autenticação controlados;
- lock operacional por repo.

## Pontos de extensão

- remote deve ser novo domínio/serviço, não código em `WorkspacePage`.
- GitHub deve depender de contratos do Git e de uma API própria, não do canvas.
- merge/revert devem produzir estados intermediários explícitos.
- parser de diff deve gerar modelo próprio antes da UI por arquivo.

## Regra anti-rewrite

Nova feature deve estender esse fluxo. Não criar uma segunda pilha paralela de commands/services/adapters sem justificar em ADR.
