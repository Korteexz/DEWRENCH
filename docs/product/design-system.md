# Design system

> Estado: `[EXPERIMENTAL/PARTIAL]` — tokens e linguagem existem; ainda não há biblioteca formal de componentes.

## Direção visual

Interface escura, técnica, densa e fluida, inspirada em CRT/instrumentação. Glow e movimento comunicam foco e atividade. CRT é uma camada estética, não licença para reduzir legibilidade.

## Tokens atuais

### Base

| Token | Valor |
|---|---|
| background | `#070908` |
| canvas | `#090c0a` |
| panel | `#0d110f` |
| text primary | `#dce5dc` |
| text secondary | `#9da99f` |
| text muted | `#606d63` |
| success | `#55df9b` |
| warning | `#e8c969` |
| danger | `#ff746b` |
| Git | `#f08a3c` |
| Docker | `#65a9df` |
| DB | `#d5bf68` |
| RRF | `#64cbb2` |

### Tipografia

- UI: Inter, Segoe UI Variable/Segoe UI, Arial.
- Técnica: Cascadia Code, SFMono-Regular, Consolas.

O código atual não usa Roboto como fonte principal.

### Geometria

- barra superior: 72px;
- painel esquerdo: 252px;
- painel direito: 306px;
- grid: 28px;
- raios pequenos/medianos: 3px/8px.

### Motion

- fast: 120ms;
- standard: 240ms;
- spring: 420ms com easing customizado.

## Topologia da tela

- topo: identidade, módulos e repo atual;
- esquerda: índice de repository/branches/commits;
- centro: superfície de topologia;
- direita: inspetor contextual;
- diff atual: dentro do inspetor de commit.

O conceito anterior de diff viewer inferior continua uma direção possível, mas não corresponde ao código atual.

## Objetos semânticos

- projeto: orb;
- commit: ring/core;
- branch: diamond;
- merge: variação de commit e edge;
- seleção: glow + foco contextual;
- indisponível: `SOON` com feedback de toque.

## Efeitos

- scanlines e vignette CRT;
- grid computacional;
- grid deformável em canvas;
- física de nós;
- foco de vizinhança;
- LEDs/retículas/instrument labels.

## Acessibilidade

`prefers-reduced-motion` reduz animações e remove scanlines. Preservar e testar esse comportamento. Não depender apenas de cor; labels e formas carregam semântica.

## Regras para Claude Design/implementação

1. Estudar tokens e componentes antes de redesenhar.
2. Prototipar fora do frontend real quando a mudança for exploratória.
3. Implementar por componente, preservando contratos.
4. Evitar posicionamento absoluto em massa para layout estrutural.
5. Não trocar semântica por decoração.
6. Validar 800×600 e breakpoints existentes.
