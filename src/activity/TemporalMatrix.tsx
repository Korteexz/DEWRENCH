import type { CSSProperties } from 'react'

import { TechnicalLabel } from '../design'
import type { TemporalCell, TemporalMatrix as Matrix } from './temporal'

interface TemporalMatrixProps {
  matrix: Matrix
  selectedKey: string | null
  onSelect: (cell: TemporalCell) => void
  onDrill: (cell: TemporalCell) => void
  canDrill: boolean
}

/**
 * Quantiza a intensidade em cinco degraus.
 *
 * Degraus discretos, e não opacidade contínua, porque o olho compara passos
 * melhor do que gradientes — e porque cada degrau é uma faixa de contagem
 * real, não uma interpolação estética.
 */
function intensityLevel(total: number, peak: number): number {
  if (total === 0 || peak === 0) {
    return 0
  }

  const ratio = total / peak
  if (ratio > 0.75) return 4
  if (ratio > 0.5) return 3
  if (ratio > 0.25) return 2
  return 1
}

/** Marcações do eixo: nem toda célula recebe rótulo, ou vira ruído. */
function shouldLabel(index: number, count: number, scale: string): boolean {
  if (scale === 'month' || scale === 'year') {
    return true
  }
  if (scale === 'hour') {
    return index % 3 === 0
  }
  return index % 5 === 0 || index === count - 1
}

export default function TemporalMatrix({
  matrix,
  selectedKey,
  onSelect,
  onDrill,
  canDrill,
}: TemporalMatrixProps) {
  if (matrix.cells.length === 0) {
    return (
      <p className="temporal-empty">
        Nenhuma atividade registrada neste intervalo.
      </p>
    )
  }

  return (
    <div className="temporal-matrix">
      {/*
        * Uma linha só, sempre: a fita é um eixo do tempo, e um eixo que quebra
        * em várias linhas deixa de ser eixo. As colunas se dividem igualmente
        * e o container ganha um teto para 12 meses não virarem blocos.
        */}
      <div
        className="temporal-matrix__cells"
        role="grid"
        aria-label="Matriz temporal"
        style={{
          gridTemplateColumns: `repeat(${matrix.cells.length}, minmax(0, 1fr))`,
          maxWidth: `${matrix.cells.length * 74}px`,
        } as CSSProperties}
      >
        {matrix.cells.map((cell, index) => {
          const level = intensityLevel(cell.total, matrix.peak)
          const merges = cell.byKind.merge ?? 0
          const reverts = cell.byKind.revert ?? 0

          return (
            <button
              key={cell.key}
              className="temporal-cell"
              type="button"
              role="gridcell"
              data-level={level}
              data-selected={cell.key === selectedKey}
              data-empty={cell.total === 0}
              aria-label={`${cell.label}: ${cell.total} evento(s)`}
              title={`${cell.label} · ${cell.total} evento(s)`}
              onClick={() => onSelect(cell)}
              onDoubleClick={() => canDrill && onDrill(cell)}
            >
              <span className="temporal-cell__fill" />
              {merges > 0 && <span className="temporal-cell__mark" data-kind="merge" />}
              {reverts > 0 && <span className="temporal-cell__mark" data-kind="revert" />}
              <span className="temporal-cell__axis" data-visible={shouldLabel(index, matrix.cells.length, matrix.scale)}>
                {cell.label}
              </span>
            </button>
          )
        })}
      </div>

      <div className="temporal-matrix__legend">
        <TechnicalLabel tone="faint" size="micro">Intensidade</TechnicalLabel>
        <span className="temporal-scale" aria-hidden="true">
          {[0, 1, 2, 3, 4].map((level) => (
            <i key={level} data-level={level} />
          ))}
        </span>
        <TechnicalLabel tone="faint" size="micro">
          pico {matrix.peak}
        </TechnicalLabel>
      </div>
    </div>
  )
}
