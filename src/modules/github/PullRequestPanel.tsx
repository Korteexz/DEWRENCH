import { useCallback, useEffect, useState } from 'react'

import { Button, Metric, MetricCluster, SectionHeader, StatusIndicator } from '../../design'
import DiffView from '../git/components/DiffView'
import {
  describeFailure,
  isGitOperationError,
  toGitFailure,
  type GitFailure,
} from '../git/types/revert'
import {
  closePullRequest,
  getPullRequest,
  getPullRequestDiff,
  getPullRequestPlan,
  mergePullRequest,
} from './services'
import type {
  GithubMergeOutcome,
  GithubPullRequestDetail,
  GithubPullRequestPlan,
  MergeMethod,
} from './types'

interface PullRequestPanelProps {
  projectPath: string
  number: number
  busy: boolean
  onClose: () => void
  /** Avisa o container para recarregar a lista após uma mutação. */
  onChanged: () => void
}

type Pending = 'merge' | 'close'

/**
 * Um pull request, do detalhe até a mutação.
 *
 * O fluxo de merge e de fechamento é o mesmo já usado por push e pull:
 * plano → revisão → confirmação → execução. A interface **não** decide se a
 * operação é permitida: ela exibe `blocked` e `warnings` vindos do preflight e
 * desabilita a confirmação enquanto houver bloqueio. Mesmo que passasse, o
 * backend recalcula o plano e recusa.
 */
export default function PullRequestPanel({
  projectPath,
  number,
  busy,
  onClose,
  onChanged,
}: PullRequestPanelProps) {
  const [detail, setDetail] = useState<GithubPullRequestDetail | null>(null)
  const [loading, setLoading] = useState(false)
  const [failure, setFailure] = useState<GitFailure | null>(null)

  const [diff, setDiff] = useState<string | null>(null)
  const [diffLoading, setDiffLoading] = useState(false)

  const [plan, setPlan] = useState<GithubPullRequestPlan | null>(null)
  const [pending, setPending] = useState<Pending | null>(null)
  const [planLoading, setPlanLoading] = useState(false)
  const [method, setMethod] = useState<MergeMethod | null>(null)
  const [deleteBranch, setDeleteBranch] = useState(false)
  const [running, setRunning] = useState(false)
  const [outcome, setOutcome] = useState<GithubMergeOutcome | null>(null)

  const load = useCallback(async () => {
    setLoading(true)
    setFailure(null)

    try {
      setDetail(await getPullRequest(projectPath, number))
    } catch (error) {
      setDetail(null)
      setFailure(toGitFailure(error))
    } finally {
      setLoading(false)
    }
  }, [number, projectPath])

  useEffect(() => {
    // Trocar de pull request descarta tudo que era do anterior.
    setDiff(null)
    setPlan(null)
    setPending(null)
    setMethod(null)
    setDeleteBranch(false)
    setOutcome(null)
    void load()
  }, [load])

  const disabled = busy || loading || planLoading || running

  async function handleViewDiff(): Promise<void> {
    setDiffLoading(true)
    setFailure(null)

    try {
      setDiff(await getPullRequestDiff(projectPath, number))
    } catch (error) {
      setFailure(toGitFailure(error))
    } finally {
      setDiffLoading(false)
    }
  }

  /** Preflight: nada é alterado aqui. */
  async function handlePrepare(intent: Pending): Promise<void> {
    setPlanLoading(true)
    setFailure(null)
    setOutcome(null)

    try {
      const next = await getPullRequestPlan(projectPath, number)
      setPlan(next)
      setPending(intent)
      setMethod(next.recommended_method)
      setDeleteBranch(false)
    } catch (error) {
      setPlan(null)
      setPending(null)
      setFailure(toGitFailure(error))
    } finally {
      setPlanLoading(false)
    }
  }

  function handleCancel(): void {
    setPlan(null)
    setPending(null)
    setMethod(null)
    setDeleteBranch(false)
  }

  /**
   * Confirmação. O `head_sha` enviado é o do plano REVISADO: se a branch andou
   * desde então, o backend aborta em vez de mesclar outro estado.
   */
  async function handleConfirm(): Promise<void> {
    if (!plan || !pending) {
      return
    }

    setRunning(true)
    setFailure(null)

    try {
      if (pending === 'merge') {
        if (!method) {
          return
        }

        setOutcome(
          await mergePullRequest(projectPath, number, method, deleteBranch, plan.head_sha),
        )
      } else {
        await closePullRequest(projectPath, number, deleteBranch, plan.head_sha)
        setOutcome(null)
      }

      handleCancel()
      await load()
      onChanged()
    } catch (error) {
      setFailure(toGitFailure(error))
    } finally {
      setRunning(false)
    }
  }

  const state = detail?.state ?? '—'
  const isOpen = detail?.state.toUpperCase() === 'OPEN'

  return (
    <section className="inspector-section pr-detail">
      <SectionHeader
        title={`Pull request #${number}`}
        readout={detail ? `${detail.head_branch} → ${detail.base_branch}` : undefined}
        actions={(
          <>
            <Button onClick={() => void load()} disabled={disabled} busy={loading}>
              {loading ? 'Lendo…' : 'Atualizar'}
            </Button>
            <Button onClick={onClose} disabled={running}>Fechar painel</Button>
          </>
        )}
      />

      {detail && (
        <>
          <p className="pr-detail__title" title={detail.title}>{detail.title}</p>

          <div className="github-status">
            <StatusIndicator
              tone={isOpen ? 'nominal' : 'idle'}
              label={detail.is_draft ? 'DRAFT' : state.toUpperCase()}
              filled={isOpen}
            />
            {detail.review_decision && (
              <StatusIndicator
                tone={detail.review_decision === 'APPROVED' ? 'nominal' : 'warn'}
                label={detail.review_decision}
                filled={detail.review_decision === 'APPROVED'}
              />
            )}
          </div>

          <MetricCluster>
            <Metric label="Autor" value={detail.author ?? '—'} />
            <Metric label="Commits" value={detail.commit_count.toString().padStart(2, '0')} />
            <Metric label="Arquivos" value={detail.changed_files.toString().padStart(2, '0')} />
          </MetricCluster>

          <MetricCluster>
            <Metric label="Adições" value={`+${detail.additions}`} tone="nominal" />
            <Metric label="Remoções" value={`-${detail.deletions}`} tone="warn" />
            <Metric label="Mergeable" value={detail.mergeable ?? '—'} />
          </MetricCluster>

          {detail.body.trim().length > 0 && (
            <p className="pr-detail__body">{detail.body}</p>
          )}

          <div className="sync-plan__actions">
            <Button
              onClick={() => (diff === null ? void handleViewDiff() : setDiff(null))}
              disabled={disabled}
              busy={diffLoading}
            >
              {diff === null ? 'Ver alterações' : 'Fechar diff'}
            </Button>
            {isOpen && pending === null && (
              <>
                <Button
                  variant="primary"
                  onClick={() => void handlePrepare('merge')}
                  disabled={disabled}
                >
                  Preparar merge
                </Button>
                <Button onClick={() => void handlePrepare('close')} disabled={disabled}>
                  Preparar fechamento
                </Button>
              </>
            )}
          </div>
        </>
      )}

      {diff !== null && (
        <div className="pr-detail__diff">
          <DiffView source={diff} />
        </div>
      )}

      {plan && pending && (
        <div className="sync-plan" role="group" aria-label="Confirmação da operação">
          <p className="revert-panel__label">
            {pending === 'merge' ? 'Merge' : 'Fechamento'} de #{plan.number}
            {' · '}
            {plan.head_branch} → {plan.base_branch}
          </p>

          {plan.blocked && (
            <div className="inspector-error" role="alert">
              <p>{plan.blocked}</p>
            </div>
          )}

          {plan.warnings.map((warning) => (
            <p className="revert-panel__warning" key={warning}>{warning}</p>
          ))}

          {pending === 'merge' && plan.blocked === null && (
            <div className="remote-form">
              <label className="github-hint" htmlFor={`merge-method-${plan.number}`}>
                Método
              </label>
              <select
                id={`merge-method-${plan.number}`}
                value={method ?? ''}
                disabled={disabled}
                onChange={(event) => setMethod(event.target.value as MergeMethod)}
              >
                {plan.available_methods.map((available) => (
                  <option key={available} value={available}>{available}</option>
                ))}
              </select>
            </div>
          )}

          {plan.blocked === null && (
            <label className="github-hint pr-detail__destructive">
              <input
                type="checkbox"
                checked={deleteBranch}
                disabled={disabled}
                onChange={(event) => setDeleteBranch(event.target.checked)}
              />
              Apagar a branch <code>{plan.head_branch}</code> no remoto depois
            </label>
          )}

          <div className="sync-plan__actions">
            <Button onClick={handleCancel} disabled={running}>Cancelar</Button>
            <Button
              variant="primary"
              busy={running}
              disabled={
                disabled
                || plan.blocked !== null
                || (pending === 'merge' && method === null)
              }
              onClick={() => void handleConfirm()}
            >
              {pending === 'merge'
                ? `Confirmar merge (${method ?? '—'})`
                : 'Confirmar fechamento'}
            </Button>
          </div>
        </div>
      )}

      {outcome && (
        <div className="sync-report">
          <p className="sync-report__note">
            Pull request #{outcome.number} mesclado por {outcome.method}
            {outcome.deleted_branch ? ' · branch remota apagada' : ''}
          </p>
          {outcome.notes.map((note) => (
            <p className="sync-report__note" key={note}>{note}</p>
          ))}
        </div>
      )}

      {failure && (
        <div className="inspector-error" role="alert">
          <p>{describeFailure(failure)}</p>
          {isGitOperationError(failure) && failure.details && (
            <p className="revert-error__action">{failure.details}</p>
          )}
          {isGitOperationError(failure) && failure.suggestedAction && (
            <p className="revert-error__action">{failure.suggestedAction}</p>
          )}
        </div>
      )}
    </section>
  )
}
