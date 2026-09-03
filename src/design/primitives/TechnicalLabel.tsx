import type { ReactNode } from 'react'

export type LabelTone = 'hi' | 'mid' | 'low' | 'faint' | 'instrument' | 'fault'

interface TechnicalLabelProps {
  children: ReactNode
  tone?: LabelTone
  size?: 'micro' | 'label'
  title?: string
  className?: string
}

/**
 * Rótulo técnico em caixa alta. Existe para nomear um dado, nunca para
 * decorar: se o texto não identifica uma leitura real, ele não deveria estar
 * na tela.
 */
export function TechnicalLabel({
  children,
  tone = 'low',
  size = 'label',
  title,
  className,
}: TechnicalLabelProps) {
  return (
    <span
      className={['dw-label', className].filter(Boolean).join(' ')}
      data-tone={tone}
      data-size={size}
      title={title}
    >
      {children}
    </span>
  )
}
