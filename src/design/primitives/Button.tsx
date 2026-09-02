import type { ReactNode } from 'react'

interface ButtonProps {
  children: ReactNode
  onClick?: () => void
  variant?: 'ghost' | 'primary' | 'danger'
  size?: 'sm' | 'md'
  disabled?: boolean
  /** Operação em curso: muda apenas o cursor, sem fabricar sucesso visual. */
  busy?: boolean
  title?: string
  'aria-label'?: string
}

export function Button({
  children,
  onClick,
  variant = 'ghost',
  size = 'sm',
  disabled = false,
  busy = false,
  title,
  'aria-label': ariaLabel,
}: ButtonProps) {
  return (
    <button
      className="dw-button"
      type="button"
      data-variant={variant}
      data-size={size}
      data-busy={busy}
      disabled={disabled}
      onClick={onClick}
      title={title}
      aria-label={ariaLabel}
    >
      {children}
    </button>
  )
}
