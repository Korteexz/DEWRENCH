import type { ReactNode } from 'react'

import { TechnicalLabel } from './TechnicalLabel'

interface SectionHeaderProps {
  title: string
  /** Leitura numérica à direita do título (contagem real, nunca fictícia). */
  readout?: string
  actions?: ReactNode
  className?: string
}

/** Divisão interna de um painel: rótulo, régua e ações da seção. */
export function SectionHeader({
  title,
  readout,
  actions,
  className,
}: SectionHeaderProps) {
  return (
    <div className={['dw-section-header', className].filter(Boolean).join(' ')}>
      <TechnicalLabel tone="low">{title}</TechnicalLabel>
      <span className="dw-section-header__rule" aria-hidden="true" />
      {readout && (
        <TechnicalLabel
          tone="faint"
          size="micro"
          className="dw-section-header__readout"
          title={readout}
        >
          {readout}
        </TechnicalLabel>
      )}
      {actions && <div className="dw-section-header__actions">{actions}</div>}
    </div>
  )
}
