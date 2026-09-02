interface CoordinateLabelProps {
  /** Pares rótulo/valor. Devem vir de medições reais, não de números fictícios. */
  entries: { key: string; value: string }[]
  className?: string
}

/** Leitura de coordenadas/escala do campo observado. */
export function CoordinateLabel({ entries, className }: CoordinateLabelProps) {
  return (
    <span className={['dw-coord', className].filter(Boolean).join(' ')}>
      {entries.map((entry) => (
        <span key={entry.key}>
          <b>{entry.key}</b>
          {entry.value}
        </span>
      ))}
    </span>
  )
}
