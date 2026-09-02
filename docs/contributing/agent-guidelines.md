# Diretrizes para agentes de código

Este arquivo é a constituição operacional para Claude Code, Codex e outros agentes trabalhando no DEWRENCH.

## Antes de modificar

1. Ler `docs/DEWRENCH.md` e os documentos relevantes.
2. Inspecionar a branch e o diff atual.
3. Mapear frontend → IPC → Rust → Git → retorno.
4. Confirmar no código o status da feature.
5. Listar arquivos que pretende alterar e por quê.
6. Declarar riscos, casos extremos e testes.

## Regras obrigatórias

1. Não assumir que feature planejada existe.
2. Não modificar código quando a tarefa for audit/read-only.
3. Não tocar arquivos não relacionados.
4. Preservar comportamento existente salvo instrução explícita.
5. Estender abstrações existentes antes de criar paralelas.
6. Não fabricar estado no frontend.
7. Não interpolar input em shell.
8. Validar path, ref, branch, filename e remote.
9. Não executar operação destrutiva sem autorização explícita e consequência descrita.
10. Não usar `--dangerously-skip-permissions` ou equivalente sem autorização do usuário.
11. Não colocar segredo em código, config, log, prompt ou resposta.
12. Executar verificações apropriadas após mudança.
13. Atualizar documentação quando contrato/status mudar.
14. Reportar arquivos alterados, testes e riscos não resolvidos.
15. Quando incerto, expor a incerteza; não inventar.

## Primeiro trabalho recomendado para um agente novo

Auditoria read-only:

- comparar código e documentação;
- mapear responsabilidades/dependências;
- identificar divergências e bugs;
- não alterar arquivos;
- produzir relatório priorizado com evidências.

## Formato de plano para feature

```text
Objetivo
Estado atual verificado
Contratos afetados
Arquivos previstos
Riscos e segurança
Casos de erro
Testes
Fora de escopo
```

## Formato de relatório final

```text
Resultado
Arquivos alterados e motivo
Comportamento antes/depois
Testes executados e resultado
Riscos pendentes
Documentação atualizada
```

## Baseline conhecida

Na baseline `main@eb90e2e`, stage individual está conectado ao handler errado e o build TypeScript falha por import não usado. Um agente não deve “descobrir” isso e silenciosamente refatorar outras áreas; deve propor ou executar correção localizada conforme a autorização recebida.
