/** Separador de leitura. Sem semântica de seção — use SectionHeader para isso. */
export function Divider({ className }: { className?: string }) {
  return <hr className={['dw-divider', className].filter(Boolean).join(' ')} />
}
