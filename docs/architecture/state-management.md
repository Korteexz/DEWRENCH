# Gerenciamento de estado

> Estado: `[IMPLEMENTED]` para uma sessão/repositório; persistência `[PLANNED]`.

## Estado global atual

`App.tsx` guarda apenas `project: ProjectOpenResult | null`. Esse valor determina qual página está montada.

Não há Redux, Zustand, Context global de domínio ou armazenamento persistente.

## Estado do workspace

`WorkspacePage` guarda:

- nó selecionado;
- menu contextual;
- ação ocupada (`busyAction`);
- erro de ação;
- diff carregado;
- loading do diff.

`useGitGraph` guarda:

- detalhes do repositório;
- grafo Git;
- loading;
- erro de leitura.

## Fonte de verdade

Para estado Git, a fonte de verdade é o backend/Git CLI. O React guarda um snapshot de apresentação.

```text
mutação → aguarda backend → refresh → substitui snapshot
```

## Proteções atuais

- `requestIdRef` descarta resposta de leitura obsoleta.
- `busyAction` bloqueia mutações concorrentes iniciadas pela mesma tela.
- seleção visual deriva de `selectedNodeId`.
- a versão do layout deriva de branches/heads/commits e força reconstrução quando a topologia muda.

## Estados que ainda precisam ser modelados

- remote/upstream;
- ahead/behind;
- fetch/pull/push em andamento;
- merge/rebase/cherry-pick/revert em andamento;
- conflito e arquivos conflitantes;
- autenticação;
- processo cancelável;
- histórico de operações e recuperação;
- múltiplos workspaces recentes.

## Regras

1. Estado de domínio não deve ser duplicado em vários componentes.
2. Dados derivados devem usar seleção/memoização, não cópias manuais.
3. A UI não deve marcar operação como concluída antes do backend.
4. Mutação e leitura devem ser serializadas quando puderem competir pelo mesmo repo.
5. Erro deve pertencer à operação que o produziu, evitando mensagens antigas após nova leitura.
6. Persistência futura deve guardar preferências e caminhos recentes, nunca snapshots Git tratados como verdade.
