# ADR-003 — XYFlow/React Flow para o canvas

- Status: Accepted

## Contexto

O workspace requer pan/zoom, nós customizados, edges, seleção, drag e evolução para outras topologias.

## Decisão

Usar `@xyflow/react` como camada de canvas e interação.

## Motivos

- nós React customizáveis;
- controles de viewport;
- eventos e estado de seleção;
- edges tipadas;
- ecossistema consolidado;
- permite separar modelo semântico, layout e render.

## Consequências

- o domínio não pode depender do formato interno do XYFlow;
- adapter/layout fazem a tradução;
- performance precisa ser testada com grafos maiores;
- física e grid customizados aumentam complexidade;
- acessibilidade do canvas exige trabalho adicional.

## Regra

XYFlow é renderer/interactor, não fonte de verdade do Git.
