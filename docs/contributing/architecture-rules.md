# Regras de arquitetura

1. Entenda o fluxo existente antes de alterar.
2. Preserve comportamento fora do escopo.
3. Frontend não executa ferramenta nativa diretamente.
4. Chamadas Tauri ficam em services/adapters dedicados.
5. Commands Rust permanecem finos.
6. Regras Git ficam no domínio Git, não em componentes.
7. Use `git_cli` como adapter; não crie execução paralela improvisada.
8. Não concatenar input em shell.
9. Estado visual deriva de resposta real.
10. Adapter cria relações semânticas; layout apenas posiciona.
11. Módulos não importam internals de outros módulos.
12. Nova abstração precisa de pelo menos um problema concreto.
13. Erro e risco são partes do contrato, não detalhes posteriores.
14. Operação destrutiva exige preflight e documentação.
15. Mudança IPC atualiza Rust, TypeScript, docs e testes.
16. Não preencher stubs só para “completar arquitetura”.

## Fluxo de mudança

```text
auditar → declarar comportamento → planejar arquivos → implementar → testar → revisar diff → atualizar docs
```

## Quando criar ADR

- troca de tecnologia central;
- nova dependência arquitetural;
- mudança de fonte de verdade;
- novo limite de segurança;
- quebra de contrato de módulo/IPC;
- decisão difícil de reverter.
