# Estratégia de testes

> Estado atual: `[MISSING]` suíte automatizada.

## Baseline auditada em 2026-09-02

- `npm run lint`: conclui com warning de import `stageFile` não usado.
- `npm run build`: falha em TS6133 pelo mesmo import.
- Rust: não verificado no ambiente desta auditoria porque `cargo` não estava instalado; isso não prova falha do código Rust.
- `lib.rs`: commands `stage_file` e `unstage_file` estão registrados duas vezes e requerem verificação.

## Pirâmide recomendada

### Unitários

- parsing de status/log/branches;
- validação de branch/ref/path;
- adapter GitGraph → WorkspaceGraph;
- layout/foco semântico;
- taxonomia de erros.

### Integração Git

Criar repos temporários isolados para init, stage, commit, branch, switch, diff, merge e erros. Nunca usar o próprio repo de desenvolvimento como fixture mutável.

### IPC/contrato

Verificar que commands, payloads e DTOs Rust/TypeScript permanecem compatíveis.

### Componentes

Testar botões, busy state, erros, seleção e callbacks. O defeito Stage→Unstage teria sido detectado aqui.

### E2E desktop

Abrir pasta temporária, criar repo, operar e confirmar estado real com Git.

## Matriz mínima antes de merge

- repo inexistente, sem Git e unborn;
- working tree limpa/suja;
- arquivo novo/modificado/removido/renomeado;
- stage individual/all e unstage;
- commit sem mensagem/sem staged/normal;
- branch válida/inválida/existente;
- switch limpo e bloqueado por mudanças;
- diff root/normal/merge;
- repo com mais de 80 commits;
- paths com espaços/unicode;
- reduced motion;
- tamanhos 800×600 e breakpoints.

## Comandos de verificação alvo

```text
npm run lint
npm run build
npm test
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

Scripts inexistentes devem ser adicionados quando a infraestrutura de testes for criada.
