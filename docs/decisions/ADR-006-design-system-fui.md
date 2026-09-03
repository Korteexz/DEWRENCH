# ADR-006 — Design system interno e direção FUI

- Status: aceito
- Data: 2026-09-02
- Contexto de código: `frontend/fui-redesign`

## Contexto

O frontend concentrava toda a aparência em `src/App.css` (1527 linhas) com
tokens parciais em `src/index.css` e dezenas de valores literais fora deles.
Consequências observadas na auditoria:

- a mesma cor (verde CRT) significava sucesso, seleção, grade e acento;
- tamanhos de fonte entre 5px e 8px espalhados pelas telas;
- cinco variações do mesmo conceito de "cabeçalho de painel";
- o shell importava tipos do módulo Git, e os inspetores de Git moravam dentro
  de `app/components/` — internals de módulo dentro do chassi;
- a aparência padrão do React Flow influenciava a identidade do produto.

Somado a isso, a direção visual CRT/retro terminal foi abandonada em favor de
instrumentação técnica (FUI industrial, telemetria, mission control).

## Decisão

1. Criar `src/design/` com **tokens** e **primitivas**, sem nenhuma dependência
   de `app/` ou `modules/`.
2. Mover o chassi para `src/app/shell/`, sem tipos de módulo: o módulo ativo
   injeta sua leitura na barra de sistema por slot (`systemReadout`).
3. Introduzir `src/modules/git/view/` para view-models puros, tirando regra Git
   de dentro de JSX.
4. Isolar o React Flow atrás de `GitGraphViewport` + `WorkspaceCanvas`.
5. Migrar em fatias verticais, mantendo o CSS legado vivo para as telas ainda
   não migradas, em vez de uma reescrita única.

## Alternativas consideradas

- **CSS Modules ou CSS-in-JS**: resolveriam escopo, mas trocariam a natureza do
  problema (falta de vocabulário) por infraestrutura. O problema real era
  ausência de tokens e primitivas, não colisão de nomes.
- **Reescrever o frontend inteiro de uma vez**: rejeitado — o módulo Git é o
  único fluxo funcional do produto e regressão nele custa mais do que
  incoerência visual temporária.
- **Adotar uma biblioteca de componentes pronta**: rejeitado — a identidade do
  DEWRENCH é o produto; um kit genérico entregaria exatamente o dashboard SaaS
  que a direção rejeita.

## Consequências

**Positivas**: aparência trocável sem tocar em lógica Git/Tauri; módulos
futuros herdam a linguagem; cor volta a ter significado; tipografia legível.

**Negativas / dívida assumida**: convivência temporária entre tokens novos e
aliases `--color-*` legados em `src/index.css`; `App.css` segue existindo com
canvas, nós, menu e CRT; os inspetores de objeto tiveram pele migrada mas ainda
não foram reestruturados em primitivas.

**Trabalho decorrente**: fatia 2 — nós/edges/foco do grafo, telas de entrada,
remoção do `CrtOverlay`, extração de `GitWorkspaceContainer` e mudança dos
inspetores para `modules/git/`.
