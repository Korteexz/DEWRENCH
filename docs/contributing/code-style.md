# Estilo de código

## Princípios

- nomes revelam domínio e intenção;
- funções pequenas com responsabilidade clara;
- composição antes de duplicação;
- comentários explicam “por quê”, riscos e invariantes;
- não comentar cada linha óbvia;
- tipos explícitos nas fronteiras;
- erros não são engolidos silenciosamente.

## TypeScript/React

- componentes funcionais;
- `import type` para tipos;
- lógica IPC em `services/`;
- hooks para comportamento reutilizável;
- evitar estado derivado duplicado;
- manter props e unions discriminadas;
- respeitar Oxlint e TypeScript strict/noUnusedLocals;
- acessibilidade: labels, roles, focus e teclado.

## Rust

- `Result<T, Error>` nas fronteiras; migrar de String para erro tipado;
- `Path`/`PathBuf` para filesystem;
- `Command` com `.args()`, nunca string de shell;
- validação antes de mutação;
- módulos por conceito;
- `cargo fmt` e `cargo clippy` antes de merge.

## Formatação

Não fazer refatoração/formatação massiva junto de feature pequena. Isso torna diff difícil de revisar e pode esconder regressões.

## Comentários educacionais

O projeto também serve ao aprendizado do autor. Comentários relevantes devem deixar claro qual camada está sendo alterada e quais consequências a mudança possui, sem transformar o código em tutorial redundante.
