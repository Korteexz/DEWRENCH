# Visão geral da arquitetura

> Estado: `[PARTIAL]` — a separação modular já existe, enquanto partes do Core e módulos futuros ainda são stubs.

## Forma atual

DEWRENCH é um monólito desktop modular: um único aplicativo distribuível, com fronteiras internas entre shell, componentes compartilhados, módulos de produto e backend nativo.

```text
Tauri desktop application
├── React application
│   ├── app/                  shell, canvas, graph e páginas compartilhadas
│   └── modules/git/          UI, serviços e tipos específicos de Git
└── Rust backend
    ├── core/                 direção futura; atualmente vazio
    ├── modules/git/          implementação ativa
    └── modules/github/       stubs vazios
```

## Fluxo de execução

```text
ação do usuário
  → componente React
  → serviço TypeScript
  → invoke Tauri
  → command Rust
  → service Rust
  → domínio Git
  → adapter git_cli
  → processo Git
  → Result serializado
  → refresh do frontend
  → novo grafo/inspetor
```

## Fronteiras

### Application Shell

`src/app/components/shell/AppShell.tsx` possui apenas chrome global: barra do sistema, navegação de módulos, área de workspace e overlay CRT. Ele recebe o workspace como filho e não deve importar regras de Git.

### App compartilhado

`src/app/` contém páginas de entrada, componentes de canvas, layout, foco, física e tipos de grafo reutilizáveis. Hoje alguns tipos compartilhados ainda importam diretamente tipos do módulo Git; isso é aceitável na fase atual, mas é um acoplamento a observar antes do segundo módulo funcional.

### Módulo Git no frontend

`src/modules/git/` contém serviços IPC, tipos, adapter semântico, hook de leitura e componentes do workspace Git.

### Módulo Git no backend

`src-tauri/src/modules/git/` divide entrada IPC, orquestração, conceitos Git e execução do binário.

## Regras de dependência desejadas

- Módulos podem depender de contratos do Core e componentes compartilhados.
- O shell não deve depender da implementação interna de um módulo.
- Um módulo não deve importar outro módulo diretamente sem contrato explícito.
- Componentes visuais compartilhados não devem executar Git, Docker ou comandos do sistema.
- Entradas externas passam por validação antes de chegar ao adapter nativo.
- O frontend nunca fabrica sucesso; ele aguarda o backend e relê o estado.

## Estado das camadas

| Camada | Estado |
|---|---|
| Shell global | `[IMPLEMENTED]` |
| Canvas/grafo compartilhado | `[IMPLEMENTED]` + física `[EXPERIMENTAL]` |
| Git frontend | `[PARTIAL]` |
| Git backend | `[PARTIAL]` |
| Core Rust genérico | `[STUB]` |
| GitHub backend | `[STUB]` |
| Runtime formal de módulos/plugins | `[PLANNED]` |

## Dívidas arquiteturais observadas

- `WorkspacePage.tsx` ainda concentra orquestração de muitas interações.
- Erros cruzam o IPC como `String`, sem código ou metadados.
- Alguns contratos TypeScript duplicam structs Rust manualmente.
- O Core e o sistema de eventos existem apenas como arquivos vazios.
- O shell conhece `ProjectOpenResult`, que hoje pertence ao módulo Git.
- Não há testes de contrato garantindo equivalência Rust ↔ TypeScript.

Esses pontos são direção de refatoração, não autorização para reescrever a arquitetura inteira.
