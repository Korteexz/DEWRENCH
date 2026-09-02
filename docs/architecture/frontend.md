# Arquitetura do frontend

> Estado: `[IMPLEMENTED/PARTIAL]`

## Responsabilidades

O frontend React apresenta o estado, recebe intenção do usuário, chama contratos IPC e atualiza a tela a partir da resposta real. Ele não deve manipular `.git`, executar o binário Git ou inferir que uma mutação funcionou.

## Estrutura atual

```text
src/
├── App.tsx
├── app/
│   ├── pages/
│   ├── components/
│   │   ├── shell/
│   │   ├── navigation/
│   │   ├── canvas/
│   │   └── effects/
│   └── graph/
└── modules/git/
    ├── adapters/
    ├── components/
    ├── hooks/
    ├── pages/
    ├── services/
    └── types/
```

## Roteamento de estado

Não há router externo. `App.tsx` seleciona a página pelo resultado de `open_project`:

- sem projeto → `HomePage`;
- `not_repository` → `RepositorySetup`;
- `unborn_repository` → `UnbornRepositoryPage`;
- `repository` → `WorkspacePage`.

O projeto selecionado existe apenas em memória. Persistência de sessões recentes é `[PLANNED]`.

## Workspace Git

`WorkspacePage` é hoje o orquestrador da tela. Ele coordena:

- leitura de detalhes e grafo;
- seleção de project, branch ou commit;
- menu contextual;
- inspetores;
- stage/unstage/commit;
- branch creation/switch;
- diff de commit;
- busy state e erros;
- refresh após mutação.

## Pipeline do grafo

```text
GitGraph do backend
  → adaptGitGraph()          cria nós e relações semânticas
  → layoutWorkspaceGraph()  calcula posições e classes visuais
  → WorkspaceCanvas         interação, foco, física e render XYFlow
```

Relações devem ser criadas no adapter; o layout reorganiza geometria, mas não inventa relações.

## Regras do frontend

1. Após mutação bem-sucedida, chamar `refresh()`.
2. Não aplicar optimistic update para histórico Git sem mecanismo explícito de reconciliação.
3. Manter chamadas Tauri em `services/`, não espalhadas por componentes.
4. Manter parsing e validação de Git no backend.
5. Componentes recebem callbacks e dados tipados.
6. Um erro de backend deve permanecer visível até nova ação ou sucesso correspondente.
7. Estados de loading e busy devem impedir dupla execução acidental.
8. Acessibilidade não pode ser removida para favorecer estética CRT.

## Concorrência implementada

- `useGitGraph` usa um `requestId` para ignorar respostas antigas.
- Detalhes e grafo são carregados em paralelo.
- `busyAction` impede duas mutações simultâneas dentro do workspace atual.

Isso não substitui lock no backend por repositório.

## Defeito atual

`[KNOWN DEFECT]` Em `WorkspacePage.tsx`, `ProjectInspector.onStage` recebe `handleUnstage`. O serviço `stageFile` existe, mas ficou importado e não utilizado. Consequências:

- `npm run build` falha com TS6133;
- o botão Stage individual solicita Unstage;
- `Stage all` continua separado e funcional.

Corrigir isso deve ser uma alteração localizada e coberta por regressão.
