# Arquitetura do backend

> Estado: módulo Git `[PARTIAL]`; Core e GitHub `[STUB]`.

## Estrutura

```text
src-tauri/src/
├── lib.rs                    inicialização e registro de commands
├── core/                     arquivos vazios
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
    └── github/               arquivos vazios
```

## Camadas

### Commands

Funções `#[tauri::command]` recebem valores serializados e delegam ao service. Não devem conter regras longas.

### Service

Converte strings em `Path`, escolhe o domínio correto e agrega respostas. A validação transversal ainda é limitada.

### Domínio Git

Arquivos por conceito encapsulam argumentos, validações e parsing simples. Essa separação é a principal fronteira modular existente.

### Adapter `git_cli`

Executa `Command::new("git").args(args).current_dir(path).output()`. O uso de argumentos separados evita interpretação por shell e reduz risco de command injection. O adapter retorna `stdout` em sucesso e `stderr` como string em falha.

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
3. Toda entrada de path/ref/branch/remote deve ser validada.
4. Não concatenar strings para um shell.
5. Erros futuros devem ser tipados e serializáveis.
6. Operações de rede devem ter timeout, cancelamento e categorias de erro.
7. Operações destrutivas exigem preflight que descreva impacto.

## Limitações atuais

- Nenhum timeout ou cancelamento de processo.
- Erros sem tipo.
- Ausência de lock por repositório.
- Validação de path inconsistente entre abertura e mutações.
- Detecção de repo baseada em `.git` como diretório; worktrees com `.git` arquivo podem não ser reconhecidas.
- Parsing do porcelain v1 não usa `-z`, portanto nomes incomuns/renames precisam de revisão.
- Criação de repositório não possui rollback se init/add funcionarem e commit falhar.
- CSP está `null` em `tauri.conf.json` e deve ser endurecida antes de release.
