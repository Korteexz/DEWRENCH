import type { MetricTone } from './Metric'

interface TelemetryBarProps {
  label: string
  value: number
  total: number
  /** Número de células. Cada célula é uma unidade contável, não um gradiente. */
  segments?: number
  tone?: MetricTone
  readout?: string
}

/**
 * Barra segmentada de telemetria.
 *
 * Recebe grandezas reais (arquivos staged, commits carregados) e as discretiza.
 * Não aceita percentual pronto: quem chama precisa dizer de qual contagem o
 * valor veio, o que impede a barra de virar enfeite.
 */
export function TelemetryBar({
  label,
  value,
  total,
  segments = 24,
  tone = 'neutral',
  readout,
}: TelemetryBarProps) {
  const safeTotal = Math.max(total, 0)
  const ratio = safeTotal === 0 ? 0 : Math.min(Math.max(value / safeTotal, 0), 1)
  const filled = Math.round(ratio * segments)

  return (
    <div className="dw-telemetry" data-tone={tone}>
      <div className="dw-telemetry__head">
        <span>{label}</span>
        <span className="dw-telemetry__readout">
          {readout ?? `${value}/${safeTotal}`}
        </span>
      </div>
      <div
        className="dw-telemetry__track"
        role="meter"
        aria-label={label}
        aria-valuenow={value}
        aria-valuemin={0}
        aria-valuemax={safeTotal}
      >
        {Array.from({ length: segments }, (_unused, index) => (
          <span
            key={index}
            className="dw-telemetry__cell"
            data-on={index < filled}
          />
        ))}
      </div>
    </div>
  )
}
