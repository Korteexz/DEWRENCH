import type { ReactNode } from 'react'

export type MetricTone = 'neutral' | 'nominal' | 'warn' | 'fault' | 'instrument'

interface MetricProps {
  label: string
  value: ReactNode
  unit?: string
  tone?: MetricTone
  title?: string
}

/** Uma leitura numérica ou textual derivada do estado real da ferramenta. */
export function Metric({
  label,
  value,
  unit,
  tone = 'neutral',
  title,
}: MetricProps) {
  return (
    <div className="dw-metric" data-tone={tone}>
      <span className="dw-metric__label">{label}</span>
      <span className="dw-metric__value" title={title}>
        {value}
        {unit && <span className="dw-metric__unit">{unit}</span>}
      </span>
    </div>
  )
}

/** Agrupa métricas comparáveis em um bloco único de instrumentação. */
export function MetricCluster({ children }: { children: ReactNode }) {
  return <div className="dw-metric-cluster">{children}</div>
}
