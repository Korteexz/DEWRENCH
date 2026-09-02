# Ações destrutivas

> Estado: política `[DRAFT]`; ações críticas ainda não implementadas.

## Níveis

| Nível | Exemplo | UX mínima |
|---|---|---|
| Read | status, graph, diff | execução direta |
| Write | stage, commit, criar branch | progresso + resultado |
| High impact | pull/merge/switch com risco | preflight + confirmação contextual |
| Critical | reset hard, clean, force push | preview forte + confirmação explícita + log |

## Conteúdo de um preflight

- operação real que será executada;
- repositório e branch;
- arquivos/commits/refs afetados;
- destino remoto, quando houver;
- possibilidade de recuperação;
- estado sujo/conflito atual;
- alternativa mais segura, quando existir.

## Regras

1. Nunca esconder a operação destrutiva atrás de label suave como “sincronizar”.
2. “Tem certeza?” sozinho é proibido.
3. Botão de confirmação deve nomear a consequência.
4. Operação crítica nunca pode ser padrão nem acionada por atalho ambíguo.
5. Não repetir automaticamente mutação não idempotente.
6. Guardar relatório local sem segredos.
7. Quando o preflight ficar obsoleto, cancelar e recalcular.
8. Em falha intermediária, mostrar recovery/abort; não retornar simplesmente à tela normal.

## Operações atuais

O MVP atual não expõe `reset --hard`, `clean`, delete branch, force push ou rewrite de histórico. Preservar essa ausência até a infraestrutura de segurança existir.
