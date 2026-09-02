import type { ElementType, ReactNode } from 'react'

interface PanelProps {
  /** Índice do compartimento no chassi (01, 02, 03…). Ensina posição. */
  index?: string
  title: string
  actions?: ReactNode
  footer?: ReactNode
  children: ReactNode
  /** Corpo sem padding, para listas que sangram até a borda do painel. */
  flush?: boolean
  scroll?: boolean
  as?: ElementType
  className?: string
  'aria-label'?: string
}

/**
 * Compartimento do instrumento.
 *
 * Painel é a única moldura do sistema: telas não inventam cabeçalho, borda ou
 * espaçamento próprios. Isso é o que permite trocar um painel por outro sem
 * renegociar a linguagem visual do resto da aplicação.
 */
export function Panel({
  index,
  title,
  actions,
  footer,
  children,
  flush = false,
  scroll = false,
  as: Tag = 'section',
  className,
  'aria-label': ariaLabel,
}: PanelProps) {
  return (
    <Tag
      className={['dw-panel', className].filter(Boolean).join(' ')}
      aria-label={ariaLabel ?? title}
    >
      <header className="dw-panel__head">
        {index && <span className="dw-panel__index">{index}</span>}
        <h2 className="dw-panel__title">{title}</h2>
        {actions && <div className="dw-panel__actions">{actions}</div>}
      </header>

      <div
        className={[
          'dw-panel__body',
          flush ? 'dw-panel__body--flush' : '',
          scroll ? 'dw-panel__body--scroll' : '',
        ].filter(Boolean).join(' ')}
      >
        {children}
      </div>

      {footer && <footer className="dw-panel__foot">{footer}</footer>}
    </Tag>
  )
}
