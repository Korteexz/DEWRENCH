# ADR-002 — Git CLI como engine inicial

- Status: Accepted

## Contexto

O objetivo é visualizar e operar Git real, não reimplementar seu modelo. Usuários já possuem configuração, credenciais e comportamento esperado no Git instalado.

## Decisão

Executar o binário `git` por adapter Rust, com argumentos separados e diretório do repository como `current_dir`.

## Motivos

- compatibilidade com o ambiente normal do usuário;
- maturidade e previsibilidade;
- fácil comparação com terminal;
- evita implementar Git;
- permite inspecionar comandos e erros.

## Consequências

- Git precisa estar instalado;
- stdout/stderr precisam de parsing;
- versões/configurações podem variar;
- processos precisam de timeout e cancelamento;
- credenciais continuam sob mecanismos Git/OS.

## Regra de segurança

Não introduzir shell interpolation. User input entra como argumento separado e passa por validação semântica.

## Alternativas

libgit2/git2-rs pode ser adicionada ou substituir partes se houver benefício comprovado. A fronteira `git_cli` deve permitir evolução sem reescrever a UI.
