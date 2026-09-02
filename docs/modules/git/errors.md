# Módulo Git — modelo de erros

> Estado atual: `[PARTIAL]` — `Result<T, String>`.  
> Estado-alvo: erros tipados e recuperáveis.

## Comportamento atual

Validações próprias retornam mensagens em português. Falhas do processo Git retornam `stderr` bruto. O frontend converte o valor recebido em texto e exibe no canvas ou inspetor.

## Taxonomia alvo

| Código | Significado | Recuperável |
|---|---|---:|
| `GIT_NOT_FOUND` | Binário Git indisponível | sim |
| `PATH_NOT_FOUND` | Caminho não existe | sim |
| `PATH_NOT_DIRECTORY` | Caminho não é pasta | sim |
| `NOT_REPOSITORY` | Pasta não é repo Git | sim |
| `UNBORN_REPOSITORY` | Repo sem primeiro commit | sim |
| `INVALID_BRANCH` | Nome/ref inválida | sim |
| `INVALID_COMMIT` | Revisão não resolve para commit | sim |
| `DIRTY_WORKTREE` | Operação bloqueada por mudanças | sim |
| `NOTHING_TO_COMMIT` | Nenhum conteúdo staged | sim |
| `IDENTITY_MISSING` | user.name/e-mail ausente | sim |
| `NO_REMOTE` | Remote/upstream ausente | sim |
| `AUTHENTICATION_FAILED` | Credencial recusada | sim |
| `NETWORK_UNAVAILABLE` | Falha de conectividade | sim |
| `REMOTE_NOT_FOUND` | Remote ou repo inexistente | sim |
| `MERGE_CONFLICT` | Integração gerou conflitos | sim, com fluxo |
| `PERMISSION_DENIED` | Sistema/remote negou acesso | depende |
| `PROCESS_TIMEOUT` | Git excedeu limite | sim |
| `OPERATION_BUSY` | Outro comando ocupa o repo | sim |
| `UNKNOWN_GIT_ERROR` | Falha não classificada | desconhecido |

## Estrutura alvo

```ts
interface GitOperationError {
  code: string
  operation: string
  message: string
  recovery?: string
  affectedState?: string[]
  details?: Record<string, unknown>
}
```

## Regras de UX

Mostrar:

1. o que ocorreu;
2. a causa provável;
3. o que foi ou não alterado;
4. como recuperar;
5. detalhes técnicos expansíveis.

Nunca mostrar apenas “ERROR”, nunca ocultar estado intermediário e nunca incluir segredos em detalhes.

## Parsing de stderr

Não espalhar comparações de strings por commands e componentes. Centralizar classificação no adapter/serviço e preservar mensagem técnica sanitizada para diagnóstico.
