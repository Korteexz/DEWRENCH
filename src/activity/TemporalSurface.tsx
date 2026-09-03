import type { ReactNode } from 'react'

import { InstrumentFrame, StatusIndicator, TechnicalLabel } from '../design'
import TemporalMatrix from './TemporalMatrix'
import {
  describeCell,
  drillInto,
  type TemporalCell,
  type TemporalMatrix as Matrix,
  type TemporalRange,
  type TemporalScale,
} from './temporal'
import type { ActivityStream } from './types'

import './activity.css'

interface TemporalSurfaceProps {
  matrix: Matrix
  stream: ActivityStream | null
  loading: boolean
  error: string | null
  scale: TemporalScale
  range: TemporalRange | null
  selectedKey: string | null
  onScale: (scale: TemporalScale) => void
  onRange: (range: TemporalRange | null) => void
  onSelect: (cell: TemporalCell) => void
  surfaceSwitch: ReactNode
}

const SCALES: TemporalScale[] = ['year', 'month', 'day', 'hour']
const SCALE_LABEL: Record<TemporalScale, string> = {
  year: 'ANO',
  month: 'MÊS',
  day: 'DIA',
  hour: 'HORA',
}

/**
 * Superfície de análise temporal.
 *
 * Mesmo compartimento do grafo, outro instrumento sobre os mesmos dados. A
 * matriz não conhece Git: ela lê `ActivityEvent`, e é por isso que Docker,
 * CI/CD ou eventos de outra máquina poderão entrar aqui sem tocar neste
 * arquivo.
 */
export default function TemporalSurface({
  matrix,
  stream,
  loading,
  error,
  scale,
  range,
  selectedKey,
  onScale,
  onRange,
  onSelect,
  surfaceSwitch,
}: TemporalSurfaceProps) {
  const events = stream?.events ?? []
  const canDrill = drillInto(scale) !== null

  function handleDrill(cell: TemporalCell) {
    const next = drillInto(cell.scale)
    if (!next) {
      return
    }
    onRange({ scale: cell.scale, key: cell.key })
    onScale(next)
  }

  return (
    <section className="git-graph-viewport" aria-label="Matriz temporal do repositório">
      <header className="git-graph-viewport__bar">
        <span className="dw-panel__index">02</span>
        <TechnicalLabel tone="mid">Temporal matrix</TechnicalLabel>
        {surfaceSwitch}
        <span className="git-graph-viewport__bar-rule" aria-hidden="true" />
        <span className="dw-coord">
          <span><b>EV</b>{String(events.length).padStart(4, '0')}</span>
          <span><b>PK</b>{String(matrix.peak).padStart(3, '0')}</span>
        </span>
      </header>

      <div className="git-graph-viewport__field temporal-field">
        <div className="temporal-controls">
          <span className="diff-view__modes" role="group" aria-label="Escala temporal">
            {SCALES.map((item) => (
              <button
                key={item}
                className="diff-view__mode"
                type="button"
                data-active={item === scale}
                aria-pressed={item === scale}
                onClick={() => onScale(item)}
              >
                {SCALE_LABEL[item]}
              </button>
            ))}
          </span>

          <nav className="temporal-breadcrumb" aria-label="Período em foco">
            <button type="button" onClick={() => onRange(null)} data-active={range === null}>
              TUDO
            </button>
            {range && (
              <>
                <span aria-hidden="true">›</span>
                <button type="button" data-active>
                  {describeCell({
                    key: range.key,
                    scale: range.scale,
                    start: 0,
                    total: 0,
                    byKind: {},
                    actors: [],
                    sources: [],
                    eventIds: [],
                    label: range.key,
                  })}
                </button>
              </>
            )}
          </nav>

          {canDrill && (
            <TechnicalLabel tone="faint" size="micro">
              Duplo clique detalha
            </TechnicalLabel>
          )}
        </div>

        {error ? (
          <p className="temporal-empty">{error}</p>
        ) : (
          <TemporalMatrix
            matrix={matrix}
            selectedKey={selectedKey}
            onSelect={onSelect}
            onDrill={handleDrill}
            canDrill={canDrill}
          />
        )}

        <InstrumentFrame rules={false} />
      </div>

      <footer className="git-graph-viewport__foot">
        <span className="temporal-legend" aria-hidden="true">
          <span><i data-kind="volume" />VOLUME</span>
          <span><i data-kind="merge" />MERGE</span>
          <span><i data-kind="revert" />REVERT</span>
        </span>

        <span className="dw-coord">
          <span><b>SRC</b>{(stream?.sources ?? []).join('+') || '—'}</span>
          {stream?.truncated && <span><b>!</b>TETO ATINGIDO</span>}
        </span>

        <StatusIndicator
          tone={loading ? 'info' : events.length > 0 ? 'nominal' : 'idle'}
          label={loading ? 'LENDO HISTÓRICO' : `${matrix.total} EVENTOS`}
          live={loading}
        />
      </footer>
    </section>
  )
}
