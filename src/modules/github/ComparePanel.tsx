import { useState } from 'react'

import { Button, Metric, MetricCluster, SectionHeader } from '../../design'
import DiffView from '../git/components/DiffView'
import {
  getBranchComparison,
  getComparisonDiff,
} from '../git/services/gitServices'
import type { GitBranchComparison } from '../git/types/compare'
import {
  describeFailure,
  isGitOperationError,
  toGitFailure,
  type GitFailure,
} from '../git/types/revert'

interface ComparePanelProps {
  projectPath: string
  /** Destino sugerido: a branch padrão do repositório, quando conhecida. */
  defaultBase: string | null
  /** Origem sugerida: a branch em que o usuário está. */
  defaultHead: string | null
  busy: boolean
  /** URL do repositório, para abrir a mesma comparação no GitHub. */
  webUrl: string | null
}

/**
 * Comparação entre duas referências.
 *
 * O cálculo é do Git local — funciona offline, sem `gh` e sem consumir API — e
 * usa a mesma semântica de três pontos que o GitHub, para que os dois
 * concordem sobre o mesmo par de branches.
 *
 * As referências são digitadas: o backend valida e recusa o que não existe,
 * então a interface não precisa manter uma lista paralela de branches.
 */
export default function ComparePanel({
  projectPath,
  defaultBase,
  defaultHead,
  busy,
  webUrl,
}: ComparePanelProps) {
  const [base, setBase] = useState(defaultBase ?? '')
  const [head, setHead] = useState(defaultHead ?? '')
  const [comparison, setComparison] = useState<GitBranchComparison | null>(null)
  const [loading, setLoading] = useState(false)
  const [diff, setDiff] = useState<string | null>(null)
  const [diffLoading, setDiffLoading] = useState(false)
  const [failure, setFailure] = useState<GitFailure | null>(null)

  const disabled = busy || loading || diffLoading
  const ready = base.trim().length > 0 && head.trim().length > 0

  async function handleCompare(): Promise<void> {
    setLoading(true)
    setFailure(null)
    setDiff(null)

    try {
      setComparison(await getBranchComparison(projectPath, base, head))
    } catch (error) {
      setComparison(null)
      setFailure(toGitFailure(error))
    } finally {
      setLoading(false)
    }
  }

  async function handleViewDiff(): Promise<void> {
    setDiffLoading(true)
    setFailure(null)

    try {
      setDiff(await getComparisonDiff(projectPath, base, head))
    } catch (error) {
      setFailure(toGitFailure(error))
    } finally {
      setDiffLoading(false)
    }
  }

  return (
    <section className="inspector-section compare-panel">
      <SectionHeader
        title="Compare"
        readout={comparison ? `${comparison.base}…${comparison.head}` : undefined}
      />

      <div className="remote-form compare-panel__refs">
        <input
          value={base}
          onChange={(event) => setBase(event.target.value)}
          placeholder="Destino (base)"
          aria-label="Referência de destino"
          disabled={disabled}
        />
        <input
          value={head}
          onChange={(event) => setHead(event.target.value)}
          placeholder="Origem (head)"
          aria-label="Referência de origem"
          disabled={disabled}
        />
      </div>

      <div className="sync-plan__actions">
        <Button
          variant="primary"
          onClick={() => void handleCompare()}
          disabled={disabled || !ready}
          busy={loading}
        >
          Comparar
        </Button>
        {comparison && comparison.blocked === null && (
          <Button
            onClick={() => (diff === null ? void handleViewDiff() : setDiff(null))}
            disabled={disabled}
            busy={diffLoading}
          >
            {diff === null ? 'Ver diff' : 'Fechar diff'}
          </Button>
        )}
        {webUrl && ready && (
          <a
            className="github-hint compare-panel__external"
            href={`${webUrl}/compare/${encodeURIComponent(base.trim())}...${encodeURIComponent(head.trim())}`}
            target="_blank"
            rel="noreferrer"
          >
            Abrir no GitHub
          </a>
        )}
      </div>

      {comparison && (
        <>
          {comparison.blocked && (
            <div className="inspector-error" role="alert">
              <p>{comparison.blocked}</p>
            </div>
          )}

          {comparison.warnings.map((warning) => (
            <p className="revert-panel__warning" key={warning}>{warning}</p>
          ))}

          <MetricCluster>
            <Metric
              label="À frente"
              value={comparison.ahead.toString().padStart(2, '0')}
              tone={comparison.ahead > 0 ? 'instrument' : 'neutral'}
            />
            <Metric
              label="Atrás"
              value={comparison.behind.toString().padStart(2, '0')}
              tone={comparison.behind > 0 ? 'warn' : 'neutral'}
            />
            <Metric
              label="Arquivos"
              value={comparison.files.length.toString().padStart(2, '0')}
            />
          </MetricCluster>

          {comparison.merge_base && (
            <p className="github-hint">
              Ancestral comum: <code>{comparison.merge_base.slice(0, 7)}</code>
            </p>
          )}

          {comparison.commits.length > 0 && (
            <>
              <p className="revert-panel__label">
                Commits em {comparison.head} ({comparison.commits.length})
              </p>
              <ul className="sync-commit-list">
                {comparison.commits.map((commit) => (
                  <li key={commit.hash}>
                    <code>{commit.short_hash}</code>
                    <span title={commit.message}>{commit.message}</span>
                    <small>{commit.author}</small>
                  </li>
                ))}
              </ul>
            </>
          )}

          {comparison.files.length > 0 && (
            <>
              <p className="revert-panel__label">
                Arquivos alterados ({comparison.files.length})
              </p>
              <ul className="compare-file-list">
                {comparison.files.map((file) => (
                  <li key={file.path}>
                    <code>{file.status}</code>
                    <span title={file.path}>{file.path}</span>
                    <small>
                      {file.additions === null || file.deletions === null
                        ? 'binário'
                        : `+${file.additions} / -${file.deletions}`}
                    </small>
                  </li>
                ))}
              </ul>
            </>
          )}

          {comparison.blocked === null
            && comparison.commits.length === 0
            && comparison.files.length === 0 && (
            <p className="inspector-empty">
              Nada a comparar: {comparison.head} não tem alterações além de {comparison.base}.
            </p>
          )}
        </>
      )}

      {diff !== null && (
        <div className="pr-detail__diff">
          <DiffView source={diff} />
        </div>
      )}

      {failure && (
        <div className="inspector-error" role="alert">
          <p>{describeFailure(failure)}</p>
          {isGitOperationError(failure) && failure.suggestedAction && (
            <p className="revert-error__action">{failure.suggestedAction}</p>
          )}
        </div>
      )}
    </section>
  )
}
