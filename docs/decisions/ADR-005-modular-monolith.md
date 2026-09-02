# ADR-005 — Monólito modular

- Status: Accepted

## Contexto

A visão inclui muitos módulos, mas o produto ainda valida o primeiro vertical slice. Microserviços, runtime de plugins ou processos separados criariam custo antes de haver contratos reais.

## Decisão

Distribuir uma aplicação única com fronteiras internas fortes.

## Motivos

- build e execução simples;
- transações/estado locais diretos;
- refatoração rápida na fase inicial;
- menos infraestrutura;
- permite aprender com Git antes de generalizar.

## Consequências

- fronteiras são disciplina de código, não isolamento de processo;
- imports entre módulos precisam de revisão;
- Core só deve crescer por necessidade concreta;
- segundo módulo será teste da modularidade.

## Critério de mudança

Separar processo/plugin runtime apenas quando isolamento, atualização independente, segurança ou escala justificarem o custo.
