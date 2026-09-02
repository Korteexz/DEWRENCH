# Limites de confiança

> Regra: dados locais também precisam de validação.

## Fronteiras

```text
usuário / frontend
        ↓ IPC
commands Tauri
        ↓
services/domínio Rust
        ↓ processo + filesystem
Git CLI / repositório local
        ↓ rede (futuro)
remote / GitHub
```

## Entradas não confiáveis

- paths e filenames;
- nomes de branch/tag/remote;
- hashes e revisões;
- mensagens de commit;
- saída/stdout/stderr do Git;
- conteúdo e configuração do repository;
- URLs de remote;
- respostas de APIs;
- dados de plugins/agentes.

## Controles por fronteira

### Frontend → IPC

Tipos, limites de tamanho, allowlist de operação e nenhum segredo desnecessário.

### IPC → domínio

Canonicalização, validação semântica e checagem de estado/precondições.

### Domínio → processo

Argumentos separados, sem shell; timeout; ambiente controlado; logs sanitizados.

### Processo → parser

Formato estável e machine-readable quando possível (`-z`, delimitadores explícitos); rejeitar resposta incoerente.

### DEWRENCH → rede

Destino exibido, TLS, credencial via mecanismo seguro, timeout, retry controlado e nenhuma mudança remota silenciosa.

## Estado atual Tauri

A capability desktop permite `core:default` e `dialog:default`. Não há permissão de shell exposta ao frontend. Commands Rust próprios continuam poderosos e devem validar inputs. A CSP está `null`, lacuna a corrigir.

## Regra para plugins

Plugin futuro não herda acesso total. Cada capability deve ser declarada e aprovada conforme filesystem, processo, rede e credenciais necessários.
