import { StatusIndicator, TechnicalLabel } from '../../../design'
import type { GitRepositoryDetails } from '../types/repository'
import { summarizeWorkingTree } from '../view/workingTree'

interface GitSystemReadoutProps {
  details: GitRepositoryDetails | null
  /** Grafo já respondido pelo backend — é isso que define "linked". */
  linked: boolean
  loading: boolean
  /** Rótulo da mutação em curso, quando existir. */
  activity: string | null
}

/**
 * Leitura do módulo Git dentro da barra de sistema.
 *
 * O shell reserva o espaço; quem sabe o que é branch e working tree é o
 * módulo. Nenhum valor aqui é inventado: todos derivam da última resposta do
 * backend.
 */
export default function GitSystemReadout({
  details,
  linked,
  loading,
  activity,
}: GitSystemReadoutProps) {
  const summary = summarizeWorkingTree(details?.files)

  const tone = activity ? 'info' : linked ? 'nominal' : 'idle'
  const label = activity
    ? activity.toUpperCase()
    : loading
      ? 'READING'
      : linked
        ? 'LINKED'
        : 'NO SIGNAL'

  return (
    <div className="git-system-readout">
      <span className="git-system-readout__field">
        <TechnicalLabel tone="faint" size="micro">BRANCH</TechnicalLabel>
        <b title={details?.branch ?? undefined}>{details?.branch ?? '—'}</b>
      </span>

      <span className="git-system-readout__field">
        <TechnicalLabel tone="faint" size="micro">TREE</TechnicalLabel>
        <b data-dirty={!summary.clean}>
          {summary.clean ? 'CLEAN' : `${summary.total}Δ`}
        </b>
      </span>

      <StatusIndicator
        tone={tone}
        label={label}
        live={Boolean(activity) || loading}
        title={activity ?? undefined}
      />
    </div>
  )
}
