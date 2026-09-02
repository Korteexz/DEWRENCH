export type StatusTone = 'idle' | 'nominal' | 'warn' | 'fault' | 'info'

interface StatusIndicatorProps {
  tone: StatusTone
  label: string
  /** Marca preenchida = estado confirmado; vazada = estado apenas conhecido. */
  filled?: boolean
  /**
   * Pulso. Só deve ser ligado enquanto uma operação REAL estiver em curso —
   * ele é a evidência visual de que algo está acontecendo no backend.
   */
  live?: boolean
  title?: string
}

export function StatusIndicator({
  tone,
  label,
  filled = true,
  live = false,
  title,
}: StatusIndicatorProps) {
  return (
    <span
      className="dw-status"
      data-tone={tone}
      data-filled={filled}
      data-live={live}
      title={title}
    >
      <span className="dw-status__mark" aria-hidden="true" />
      {label}
    </span>
  )
}
