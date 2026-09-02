# Roadmap do produto

## Agora: módulo Git pré-1.0

O roadmap 0.5–1.0 está detalhado em [`../modules/git/roadmap.md`](../modules/git/roadmap.md). A prioridade é concluir um módulo verticalmente antes de ativar outros.

## Módulos futuros

| Módulo | Estado | Valor principal |
|---|---|---|
| Docker | `[PLANNED]` | containers, imagens, volumes e topologia |
| Database Viewer | `[PLANNED]` | estrutura, dados e relações |
| RRF | `[PLANNED]` | request/response e caminho executado |
| CI/CD | `[PLANNED]` | pipeline e falhas |
| Kubernetes | `[PLANNED]` | recursos, dependências e saúde |
| Terraform | `[PLANNED]` | plano, drift e impacto |
| System tools | `[PLANNED]` | processos, arquivos e apps |
| Plugins | `[PLANNED]` | capacidades instaláveis/removíveis |
| IA/RAG local | `[PLANNED]` | assistência opcional baseada na máquina/projeto |

## Sequência de produto

1. estabilizar Git local;
2. adicionar Git remoto e recuperação;
3. fechar segurança, testes e multiplataforma;
4. usar o segundo módulo real para validar o contrato modular;
5. formalizar plugins/eventos somente quando houver casos concretos.

## Princípio de entrega

Botão futuro pode comunicar visão, mas deve ser marcado como inativo e não simular operação. Feature é considerada ativa apenas quando UI, backend, erros, segurança, testes e documentação fecham o fluxo.
