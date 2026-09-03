# Arquitetura do backend

> Estado: módulo Git `[PARTIAL]`; Security Core `[PARTIAL]`; GitHub `[PARTIAL]`.
>
> O que o Core aplica hoje, o que é só fundação e o que não foi testado está em
> `../security/enforcement-state.md`. Esta página descreve a forma; aquela
> descreve o que está ligado.

## Estrutura

```text
src-tauri/src/
├── lib.rs                    inicialização e registro de commands
├── core/                     Security Core — decide autoridade, executa fronteiras
│   ├── authority.rs          tipos: ação, ator, recurso, capability, risco, confiança
│   ├── error.rs              CoreError com códigos estáveis
│   ├── path_security.rs      escopo, resolução canônica, contenção
│   ├── process.rs            ProcessBroker: a única porta de criação de processo
│   ├── state.rs              registro de workspace, autoridade, lock RAII
│   ├── events.rs             redação de segredo e journal de auditoria
│   ├── approval.rs           snapshot de preflight, digest, staleness   [preparado]
│   └── policy.rs             catálogo de ações e decisão de segurança   [preparado]
└── modules/
    ├── git/
    │   ├── commands.rs       interface Tauri
    │   ├── service.rs        orquestração
    │   ├── repository.rs     abertura/criação/estado
    │   ├── working_tree.rs   status/stage/unstage
    │   ├── commits.rs        log/commit/diff
    │   ├── branches.rs       leitura/criação/switch
    │   ├── graph.rs          grafo de commits/branches
    │   ├── models.rs         DTOs serializáveis
    │   └── git_cli.rs        adapter de processo
    ├── activity/            modelo de atividade (Temporal Matrix)
    └── github/              provider opcional baseado na CLI `gh`
```

## Camadas

### Commands

Funções `#[tauri::command]` recebem valores serializados e delegam ao service. Não devem conter regras longas.

### Service

Converte strings em `Path`, escolhe o domínio correto e agrega respostas. A validação transversal ainda é limitada.

### Domínio Git

Arquivos por conceito encapsulam argumentos, validações e parsing simples. Essa separação é a principal fronteira modular existente.

### Security Core

Fica **antes** dos módulos, não ao lado deles. A regra constitucional é
`MODULES DESCRIBE INTENT. CORE DECIDES AUTHORITY. CORE EXECUTES THROUGH
CONTROLLED BOUNDARIES.`

Dois pontos são obrigatórios hoje:

* **Autoridade** — a camada `service` traduz o `path` do IPC em workspace
  registrado (`core::state::authorize_workspace`). Um caminho que o usuário
  nunca abriu é recusado. O `path` deixou de ser credencial.
* **Processo** — nenhum módulo cria processo. `core::process` responde por
  executável, argumentos, diretório, ambiente, config forçada, tempo limite e
  teto de saída. Um teste de arquitetura falha se `Command::new` reaparecer sob
  `src/modules/`.

### Adapter `git_cli`

Descreve a intenção ("git com estes argumentos, neste diretório") e delega a
execução ao `core::process`. As assinaturas (`run`, `run_raw`,
`run_structured`) são as mesmas de antes de propósito: os chamadores não
mudaram, e a fronteira passou a valer para todos de uma vez. O adapter escolhe
o tempo limite pelo subcomando (rede recebe 180s) e converte a recusa do Core
em `io::Error` preservando o `kind`.

## Comandos registrados

- `open_project`
- `create_repository`
- `get_repository_details`
- `get_repository_graph`
- `stage_file`
- `stage_all`
- `unstage_file`
- `create_commit`
- `get_commit_diff`
- `create_branch_from`
- `switch_branch`

`stage_file` e `unstage_file` aparecem duplicados no `generate_handler!`; isso requer limpeza/auditoria, embora não seja a causa do erro TypeScript conhecido.

## Regras para extensão

1. Novo caso de uso começa pelo contrato e pelo risco.
2. Command fino, service orquestrador, domínio responsável pelo Git.
3. Toda entrada de path/ref/branch/remote deve ser validada — e todo valor que
   chega ao git como DADO passa por `core::process::operand`, que recusa
   qualquer coisa iniciada por `-`. Ausência de shell não impede injeção de
   argumento.
4. Não concatenar strings para um shell.
4b. Nenhum módulo cria processo. Se um caso novo precisa de um executável, ele
   entra no enum `ProgramId`, não em um `Command::new` local.
5. Erros futuros devem ser tipados e serializáveis.
6. Operações de rede devem ter timeout, cancelamento e categorias de erro.
7. Operações destrutivas exigem preflight que descreva impacto.

## Limitações atuais

- Lock por repositório existe (`core::state::acquire`) mas **os commands ainda
  não o adquirem**.
- Aprovação vinculada ao estado revisado existe (`core::approval`) mas **não
  está no caminho** de push, pull ou revert.
- `core::policy` avalia capability e risco, mas nenhum command o consulta.
- Erros dos commands antigos ainda são `String`; o contrato tipado convive com
  eles de propósito.
- Detecção de repo baseada em `.git` como diretório; worktrees com `.git` arquivo podem não ser reconhecidas.
- Parsing do porcelain v1 não usa `-z`, portanto nomes incomuns/renames precisam de revisão.
- Criação de repositório não possui rollback se init/add funcionarem e commit falhar.
- CSP está `null` em `tauri.conf.json` e deve ser endurecida antes de release.
