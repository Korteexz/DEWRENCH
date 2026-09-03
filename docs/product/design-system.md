# Design system

> Estado: `[PARTIAL]` — tokens e primitivas implementados em `src/design/`;
> migração das telas em andamento (fatia 1 de 2 concluída).

## Direção visual

DEWRENCH é um **instrumento usado para observar e manipular software** — não um
dashboard SaaS futurista. A referência é instrumentação técnica: telemetria,
mission control, computer vision, sistemas industriais e científicos.

> "A machine for seeing machines."

A direção anterior (CRT/retro terminal, glow verde, scanlines como assinatura)
foi **abandonada**. O que resta dela em `src/App.css` é legado a remover.

### Princípios

1. Leitura por **contraste e traço**; cor é reservada para significado.
2. Densidade se obtém por **hierarquia e escala**, nunca reduzindo a fonte —
   10px é o piso absoluto do sistema.
3. Toda visualização representa **dado real do estado do sistema**. Número
   fictício, gráfico decorativo e animação sem evento não entram.
4. Movimento é telemetria: algo se move porque um evento **verificado** passou
   por ali.
5. Cantos retos. Um instrumento é usinado, não arredondado.
6. Seleção é comunicada por contraste e posição (barra lateral), não por cor —
   assim a cor continua livre para semântica e a leitura sobrevive a daltonismo.

### Evitar

Neon cyberpunk, glassmorphism, gradientes SaaS, cards grandes arredondados,
glow/glitch em excesso, simulação CRT pesada, hexágonos aleatórios e qualquer
decoração futurista sem função.

## Arquitetura

```text
src/design/                  não conhece Git, Tauri nem React Flow
├── tokens/                  color, type, space, motion, grid
└── primitives/              Panel, SectionHeader, TechnicalLabel, Metric,
                             StatusIndicator, TelemetryBar, DataRow, Divider,
                             CoordinateLabel, InstrumentFrame, Button
src/app/shell/               chassi: AppShell, SystemBar, ModuleRail
src/modules/<módulo>/        view-models, componentes e folha do módulo
```

**Regra de dependência**: `design/` não importa de `app/` nem de `modules/`.
Os módulos importam de `design/` — nunca o contrário. É isso que permite
acrescentar Docker, Kubernetes ou Terraform sem reescrever o shell.

## Tokens

Definidos em `src/design/tokens/`. Nenhum valor literal de cor, tamanho de
fonte, espaçamento ou duração deve existir fora desses arquivos.

### Superfícies e linhas

| Token | Papel |
|---|---|
| `--surface-void` | fundo da aplicação e campo do grafo |
| `--surface-panel` | compartimento |
| `--surface-raised` | cabeçalho de painel, linha ativa |
| `--surface-input` | campo editável |
| `--surface-hover` | estado transitório de ponteiro |
| `--line-hair` / `--line-edge` / `--line-strong` | hierarquia de traço |
| `--ink-hi` / `--ink-mid` / `--ink-low` / `--ink-faint` | pesos de tinta |

### Sinais semânticos

Cinco, e nenhum deles decora:
`--signal-active` (seleção/foco), `--signal-nominal` (estado saudável
verificado), `--signal-warn` (consequência reversível), `--signal-fault`
(falha real reportada pela ferramenta), `--signal-info` (leitura/metadado).

### Instrumento

`--instrument-git|docker|db|rrf` e `--instrument-current`, resolvido pelo
chassi via `data-module`. Trocar de módulo troca o acento da interface inteira
sem passar cor por props.

### Tipografia

Monospace é identidade (`--font-mono`), sans para texto corrente
(`--font-sans`). Escala fechada: `--t-micro` 10 · `--t-label` 11 ·
`--t-body` 12 · `--t-data` 13 · `--t-head` 15 · `--t-display` 19.

### Geometria e movimento

Unidade 4px (`--s1`…`--s8`); chassi 52px; painéis 268px/324px; junta de 1px
entre compartimentos. Movimento: `--m-instant` 90ms, `--m-signal` 160ms,
`--m-transit` 260ms, zerados sob `prefers-reduced-motion`.

## Regras de uso

1. Tela nova não inventa moldura: usa `Panel`.
2. Todo dado exibido declara de onde vem. `TelemetryBar` recebe valor **e**
   total justamente para impedir percentual fabricado.
3. Refs Git (branch, tag, hash) preservam caixa — `Main` e `main` são refs
   diferentes; uppercase decorativo ali seria informação incorreta.
4. `StatusIndicator` só pulsa (`live`) enquanto existe operação real em curso.
5. Acessibilidade tem prioridade sobre estética: contraste, foco visível,
   labels, teclado e reduced motion.

## Migração

| Área | Estado |
|---|---|
| Tokens e primitivas | `[IMPLEMENTED]` |
| Chassi / barra de sistema / trilho de módulos | `[IMPLEMENTED]` |
| Deck do Git: índice, moldura do campo, inspetor | `[IMPLEMENTED]` |
| Inspetores de objeto (project/branch/commit) | `[PARTIAL]` — pele migrada, estrutura ainda legada |
| Nós, edges e foco do grafo | `[PLANNED]` — fatia 2 |
| Home, setup e repositório sem commit | `[PLANNED]` — fatia 2 |
| Textura CRT (`CrtOverlay`) | `[PLANNED]` remover — fatia 2 |
