import type { ReactNode } from 'react'

interface DataRowProps {
  primary: ReactNode
  secondary?: ReactNode
  /** Marcador à esquerda: posição na topologia, tipo do objeto, conector. */
  lead?: ReactNode
  /** Leitura à direita: índice, contagem, unidade. */
  trail?: ReactNode
  /** Etiqueta de estado real (HEAD, CURRENT). Nunca decorativa. */
  tag?: string
  selected?: boolean
  onSelect?: () => void
  title?: string
}

/**
 * Linha de dado selecionável.
 *
 * Seleção é comunicada por contraste e por uma barra de posição à esquerda,
 * não por cor — isso mantém a leitura válida para daltônicos e preserva a cor
 * para significado semântico.
 */
export function DataRow({
  primary,
  secondary,
  lead,
  trail,
  tag,
  selected = false,
  onSelect,
  title,
}: DataRowProps) {
  return (
    <button
      className="dw-data-row"
      type="button"
      data-selected={selected}
      aria-current={selected ? 'true' : undefined}
      onClick={onSelect}
      title={title}
    >
      <span className="dw-data-row__lead" aria-hidden="true">{lead}</span>
      <span className="dw-data-row__main">
        <span className="dw-data-row__primary">{primary}</span>
        {secondary && <span className="dw-data-row__secondary">{secondary}</span>}
      </span>
      <span className="dw-data-row__trail">
        {tag ? <span className="dw-data-row__tag">{tag}</span> : trail}
      </span>
    </button>
  )
}
