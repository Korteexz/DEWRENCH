/**
 * Moldura do campo observado: cantos e réguas.
 *
 * É puramente decorativa em pixels, mas semântica em função — delimita onde
 * termina o chassi e começa o objeto sob observação.
 */
export function InstrumentFrame({ rules = true }: { rules?: boolean }) {
  return (
    <div className="dw-frame" aria-hidden="true">
      <span className="dw-frame__corner dw-frame__corner--tl" />
      <span className="dw-frame__corner dw-frame__corner--tr" />
      <span className="dw-frame__corner dw-frame__corner--bl" />
      <span className="dw-frame__corner dw-frame__corner--br" />
      {rules && (
        <>
          <span className="dw-frame__rule dw-frame__rule--x" />
          <span className="dw-frame__rule dw-frame__rule--y" />
        </>
      )}
    </div>
  )
}
