import {
  Metric,
  MetricCluster,
  SectionHeader,
  TelemetryBar,
} from '../design'
import { describeCell, eventsOfCell, type TemporalCell } from './temporal'
import type { ActivityEvent } from './types'

interface TemporalInspectorProps {
  cell: TemporalCell
  events: ActivityEvent[]
  /** Maior total entre as células do mesmo nível — referência da barra. */
  peak: number
  onClose: () => void
  onDrill: (() => void) | null
}

function formatClock(event: ActivityEvent): string {
  const shifted = new Date((event.timestamp + event.utc_offset_minutes * 60) * 1000)
  const hours = String(shifted.getUTCHours()).padStart(2, '0')
  const minutes = String(shifted.getUTCMinutes()).padStart(2, '0')
  return `${hours}:${minutes}`
}

/**
 * Detalhe de um período selecionado.
 *
 * Todas as contagens vêm dos eventos reais daquele intervalo; não há valor
 * derivado de estimativa. Um período sem atividade mostra zero — que é uma
 * medição, não um vazio a preencher.
 */
export default function TemporalInspector({
  cell,
  events,
  peak,
  onClose,
  onDrill,
}: TemporalInspectorProps) {
  const periodEvents = eventsOfCell(events, cell)
  const commits = cell.byKind.commit ?? 0
  const merges = cell.byKind.merge ?? 0
  const reverts = cell.byKind.revert ?? 0
  const roots = cell.byKind.root ?? 0

  return (
    <aside className="canvas-inspector nodrag nopan" aria-label="Detalhes do período">
      <header className="canvas-inspector__header">
        <div>
          <span>PERÍODO / {cell.scale.toUpperCase()}</span>
          <strong>{describeCell(cell)}</strong>
        </div>
        <button type="button" aria-label="Fechar detalhes" onClick={onClose}>×</button>
      </header>

      <div className="canvas-inspector__content">
        <MetricCluster>
          <Metric label="Commits" value={String(commits).padStart(2, '0')} />
          <Metric label="Merges" value={String(merges).padStart(2, '0')} />
          <Metric label="Autores" value={String(cell.actors.length).padStart(2, '0')} />
        </MetricCluster>

        <section className="inspector-section">
          <SectionHeader title="Atividade" readout={`${cell.total}/${peak}`} />
          <TelemetryBar
            label="Eventos no período"
            value={cell.total}
            total={Math.max(peak, 1)}
            segments={28}
            tone={cell.total > 0 ? 'instrument' : 'neutral'}
            readout={`${cell.total}`}
          />
        </section>

        {(reverts > 0 || roots > 0) && (
          <MetricCluster>
            <Metric label="Reverts" value={String(reverts).padStart(2, '0')} tone={reverts > 0 ? 'warn' : 'neutral'} />
            <Metric label="Raízes" value={String(roots).padStart(2, '0')} />
            <Metric label="Fontes" value={cell.sources.join(', ') || '—'} />
          </MetricCluster>
        )}

        {cell.actors.length > 0 && (
          <section className="inspector-section">
            <SectionHeader title="Autores" readout={String(cell.actors.length)} />
            <ul className="temporal-actor-list">
              {cell.actors.map((actor) => (
                <li key={actor}>{actor}</li>
              ))}
            </ul>
          </section>
        )}

        <section className="inspector-section">
          <SectionHeader
            title="Eventos"
            readout={String(periodEvents.length).padStart(2, '0')}
          />

          {periodEvents.length === 0 ? (
            <p className="inspector-empty">Nenhum evento neste período.</p>
          ) : (
            <ul className="temporal-event-list">
              {periodEvents.slice(0, 40).map((event) => (
                <li key={event.id} data-kind={event.kind}>
                  <code>{event.metadata.short_hash ?? event.kind}</code>
                  <span title={event.metadata.subject ?? ''}>
                    {event.metadata.subject ?? event.kind}
                  </span>
                  <em>{formatClock(event)}</em>
                </li>
              ))}
            </ul>
          )}
        </section>

        {onDrill && cell.total > 0 && (
          <button className="temporal-drill" type="button" onClick={onDrill}>
            Detalhar este período
          </button>
        )}
      </div>
    </aside>
  )
}
