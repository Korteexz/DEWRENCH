/**
 * Agregação temporal de eventos de atividade.
 *
 * Funções puras: recebem eventos, devolvem células. Nenhuma leitura de Git,
 * nenhum acesso a Tauri, nenhum React — é isso que permite testá-las e trocar
 * a visualização sem tocar na regra.
 *
 * Decisão que atravessa o arquivo: o agrupamento usa o fuso de QUEM GEROU o
 * evento (`utc_offset_minutes`), não o fuso de quem está olhando. Um commit
 * feito às 23h em São Paulo pertence àquele dia, mesmo lido de Berlim.
 */
import type { ActivityEvent } from './types'

export type TemporalScale = 'year' | 'month' | 'day' | 'hour'

export interface TemporalCell {
  /** Chave estável e ordenável: `2026`, `2026-09`, `2026-09-02`, `2026-09-02T14`. */
  key: string
  /** Início do intervalo, em epoch de segundos no fuso do evento. */
  start: number
  scale: TemporalScale
  total: number
  /** Contagem por tipo de evento, sem inventar categorias. */
  byKind: Record<string, number>
  actors: string[]
  sources: string[]
  eventIds: string[]
  /** Rótulo curto para o eixo. */
  label: string
}

export interface TemporalMatrix {
  scale: TemporalScale
  cells: TemporalCell[]
  /** Maior total entre as células — base da intensidade. */
  peak: number
  total: number
  /** Intervalo coberto, em epoch de segundos. */
  from: number | null
  to: number | null
}

const MONTHS = [
  'JAN', 'FEV', 'MAR', 'ABR', 'MAI', 'JUN',
  'JUL', 'AGO', 'SET', 'OUT', 'NOV', 'DEZ',
]

/** Data deslocada para o fuso do autor, lida com métodos UTC. */
function localParts(event: ActivityEvent) {
  const shifted = new Date((event.timestamp + event.utc_offset_minutes * 60) * 1000)

  return {
    year: shifted.getUTCFullYear(),
    month: shifted.getUTCMonth(),
    day: shifted.getUTCDate(),
    hour: shifted.getUTCHours(),
  }
}

function pad(value: number, size = 2): string {
  return String(value).padStart(size, '0')
}

function cellKey(scale: TemporalScale, parts: ReturnType<typeof localParts>): string {
  const { year, month, day, hour } = parts

  switch (scale) {
    case 'year':
      return String(year)
    case 'month':
      return `${year}-${pad(month + 1)}`
    case 'day':
      return `${year}-${pad(month + 1)}-${pad(day)}`
    default:
      return `${year}-${pad(month + 1)}-${pad(day)}T${pad(hour)}`
  }
}

function cellStart(scale: TemporalScale, parts: ReturnType<typeof localParts>): number {
  const { year, month, day, hour } = parts

  switch (scale) {
    case 'year':
      return Date.UTC(year, 0, 1) / 1000
    case 'month':
      return Date.UTC(year, month, 1) / 1000
    case 'day':
      return Date.UTC(year, month, day) / 1000
    default:
      return Date.UTC(year, month, day, hour) / 1000
  }
}

function cellLabel(scale: TemporalScale, parts: ReturnType<typeof localParts>): string {
  const { year, month, day, hour } = parts

  switch (scale) {
    case 'year':
      return String(year)
    case 'month':
      return MONTHS[month]
    case 'day':
      return pad(day)
    default:
      return `${pad(hour)}H`
  }
}

/**
 * Recorte de um período pai, para o drill-down.
 * `undefined` significa "sem recorte": a matriz cobre tudo.
 */
export interface TemporalRange {
  scale: TemporalScale
  key: string
}

/** O evento pertence ao período pai informado? */
export function matchesRange(event: ActivityEvent, range: TemporalRange | null): boolean {
  if (!range) {
    return true
  }

  const parts = localParts(event)
  return cellKey(range.scale, parts) === range.key
}

/**
 * Constrói a matriz de um nível, opcionalmente restrita a um período pai.
 *
 * Células vazias NÃO são preenchidas com zero artificial aqui: quem desenha
 * decide se quer preencher o calendário. Uma célula existir significa que
 * houve atividade real nela.
 */
export function buildMatrix(
  events: ActivityEvent[],
  scale: TemporalScale,
  range: TemporalRange | null = null,
): TemporalMatrix {
  const buckets = new Map<string, TemporalCell>()
  let from: number | null = null
  let to: number | null = null

  for (const event of events) {
    if (!matchesRange(event, range)) {
      continue
    }

    const parts = localParts(event)
    const key = cellKey(scale, parts)

    let cell = buckets.get(key)
    if (!cell) {
      cell = {
        key,
        start: cellStart(scale, parts),
        scale,
        total: 0,
        byKind: {},
        actors: [],
        sources: [],
        eventIds: [],
        label: cellLabel(scale, parts),
      }
      buckets.set(key, cell)
    }

    cell.total += 1
    cell.byKind[event.kind] = (cell.byKind[event.kind] ?? 0) + 1
    cell.eventIds.push(event.id)

    if (event.actor && !cell.actors.includes(event.actor)) {
      cell.actors.push(event.actor)
    }
    if (!cell.sources.includes(event.source)) {
      cell.sources.push(event.source)
    }

    from = from === null ? event.timestamp : Math.min(from, event.timestamp)
    to = to === null ? event.timestamp : Math.max(to, event.timestamp)
  }

  const cells = [...buckets.values()].sort((left, right) => left.key.localeCompare(right.key))
  const peak = cells.reduce((max, cell) => Math.max(max, cell.total), 0)
  const total = cells.reduce((sum, cell) => sum + cell.total, 0)

  return { scale, cells, peak, total, from, to }
}

/**
 * Preenche o intervalo com células vazias, para o eixo ter escala contínua.
 *
 * Só é usado onde a ausência de atividade é informação (um mês sem commits em
 * meio a um ano de trabalho diz algo). Células preenchidas têm total zero — e
 * zero é medição, não invenção.
 */
export function withEmptyCells(matrix: TemporalMatrix): TemporalMatrix {
  if (matrix.cells.length === 0) {
    return matrix
  }

  if (matrix.scale === 'year') {
    return fillYears(matrix)
  }

  const template = matrix.cells[0]
  const [yearText, monthText, rest] = template.key.split('-')
  const year = Number(yearText)
  const month = Number(monthText ?? '1') - 1
  const day = Number((rest ?? '01').split('T')[0])

  const slots: { key: string; start: number; label: string }[] = []

  if (matrix.scale === 'month') {
    for (let index = 0; index < 12; index += 1) {
      slots.push({
        key: `${year}-${pad(index + 1)}`,
        start: Date.UTC(year, index, 1) / 1000,
        label: MONTHS[index],
      })
    }
  } else if (matrix.scale === 'day') {
    const days = new Date(Date.UTC(year, month + 1, 0)).getUTCDate()
    for (let index = 1; index <= days; index += 1) {
      slots.push({
        key: `${year}-${pad(month + 1)}-${pad(index)}`,
        start: Date.UTC(year, month, index) / 1000,
        label: pad(index),
      })
    }
  } else {
    for (let index = 0; index < 24; index += 1) {
      slots.push({
        key: `${year}-${pad(month + 1)}-${pad(day)}T${pad(index)}`,
        start: Date.UTC(year, month, day, index) / 1000,
        label: `${pad(index)}H`,
      })
    }
  }

  const existing = new Map(matrix.cells.map((cell) => [cell.key, cell]))
  const cells = slots.map((slot) => existing.get(slot.key) ?? {
    key: slot.key,
    start: slot.start,
    scale: matrix.scale,
    total: 0,
    byKind: {},
    actors: [],
    sources: [],
    eventIds: [],
    label: slot.label,
  })

  return { ...matrix, cells }
}

function fillYears(matrix: TemporalMatrix): TemporalMatrix {
  const years = matrix.cells.map((cell) => Number(cell.key))
  const first = Math.min(...years)
  const last = Math.max(...years)
  const existing = new Map(matrix.cells.map((cell) => [cell.key, cell]))
  const cells: TemporalCell[] = []

  for (let year = first; year <= last; year += 1) {
    const key = String(year)
    cells.push(existing.get(key) ?? {
      key,
      start: Date.UTC(year, 0, 1) / 1000,
      scale: 'year',
      total: 0,
      byKind: {},
      actors: [],
      sources: [],
      eventIds: [],
      label: key,
    })
  }

  return { ...matrix, cells }
}

/** Próximo nível do drill-down, ou null quando já está no mais fino. */
export function drillInto(scale: TemporalScale): TemporalScale | null {
  switch (scale) {
    case 'year':
      return 'month'
    case 'month':
      return 'day'
    case 'day':
      return 'hour'
    default:
      return null
  }
}

/** Rótulo legível de um período selecionado. */
export function describeCell(cell: TemporalCell): string {
  const [year, month, rest] = cell.key.split('-')

  switch (cell.scale) {
    case 'year':
      return year
    case 'month':
      return `${MONTHS[Number(month) - 1]} ${year}`
    case 'day':
      return `${rest} ${MONTHS[Number(month) - 1]} ${year}`
    default: {
      const [day, hour] = (rest ?? '').split('T')
      return `${day} ${MONTHS[Number(month) - 1]} ${year} · ${hour}:00`
    }
  }
}

/** Eventos de uma célula, na ordem em que aconteceram. */
export function eventsOfCell(
  events: ActivityEvent[],
  cell: TemporalCell,
): ActivityEvent[] {
  const ids = new Set(cell.eventIds)
  return events
    .filter((event) => ids.has(event.id))
    .sort((left, right) => right.timestamp - left.timestamp)
}
