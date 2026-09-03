# ADR-008 — Modelo de atividade separado das ferramentas

- Status: aceito
- Data: 2026-09-03
- Contexto de código: `frontend/fui-redesign`

## Contexto

A Temporal Matrix precisa representar atividade ao longo do tempo. O caminho
curto seria ela ler `git log` diretamente. Isso funcionaria hoje e travaria o
produto amanhã: Docker, CI/CD, deploy e colaboração entre máquinas são fontes
previstas no roadmap, e cada uma exigiria um novo componente de visualização.

## Decisão

Uma camada intermediária, em ambos os lados do IPC:

```text
estado/eventos de uma ferramenta  ->  ActivityEvent  ->  visualização
        git_activity                  activity           TemporalMatrix
```

- `modules/git/activity.rs` é o ÚNICO lugar onde hash, pais e autor viram
  evento. Depois dele, ninguém sabe que Git existe.
- `modules/activity/` mantém a lista de fontes. Acrescentar Docker significa
  acrescentar uma função que devolve `Vec<ActivityEvent>` e registrá-la.
- No frontend, `src/activity/` fica FORA de `src/modules/git`, com o modelo, a
  agregação temporal (funções puras) e os componentes.

Campos do evento: `timestamp`, `utc_offset_minutes`, `source`, `machine`,
`actor`, `module`, `kind`, `repository`, `branch`, `metadata`.

Duas escolhas explícitas:

- **Fuso do autor, não do observador.** Agrupar por dia usa
  `utc_offset_minutes` do evento: um commit feito às 23h em São Paulo pertence
  àquele dia mesmo lido de Berlim.
- **`branch` fica nulo para commits.** Um commit não guarda em qual branch foi
  feito; a relação é derivada do grafo. Preencher esse campo por heurística
  seria dado falso.

## Alternativas consideradas

- **Agregar no Rust e mandar células prontas**: rejeitado — a agregação
  precisa mudar de nível (ano/mês/dia/hora) a cada clique, e ida e volta ao
  backend por interação tornaria o drill-down lento sem ganho.
- **Persistir eventos num banco local**: prematuro. `git log` já é a fonte
  durável; um banco só se justifica quando houver fontes efêmeras (CI, deploy)
  ou eventos de outras máquinas.

## Consequências

**Positivas**: a visualização não conhece Git; novas fontes entram sem tocar
nela; a agregação é pura e testável.

**Negativas**: há um teto de 5000 eventos por coleta, e a coleta é integral a
cada leitura (sem cache incremental). Repositórios muito grandes vão precisar
de janela temporal ou paginação.

**Não implementado de propósito**: networking P2P. O modelo prevê `machine` e
`source` para que colaboração caiba depois, mas nenhum transporte foi escrito.
