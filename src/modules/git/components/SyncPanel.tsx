import { Button, Metric, MetricCluster, SectionHeader, StatusIndicator, TechnicalLabel } from '../../../design'
import { describeTracking } from '../types/remote'
import { describeFailure, isGitOperationError, type GitFailure } from '../types/revert'
import {
  STRATEGY_EXPLANATION,
  STRATEGY_LABEL,
  type GitFetchOutcome,
  type GitPullPlan,
  type GitPushPlan,
  type PullStrategy,
} from '../types/sync'
import type { GitRemotesView } from '../types/remote'
import type { GitGraphCommit } from '../types/repository'

interface SyncPanelProps {
  remotes: GitRemotesView | null
  selectedRemote: string | null
  onSelectRemote: (name: string) => void
  operation: 'idle' | 'push' | 'pull' | 'fetch'
  busy: boolean
  failure: GitFailure | null
  pushPlan: GitPushPlan | null
  pullPlan: GitPullPlan | null
  fetchOutcome: GitFetchOutcome | null
  pushSummary: string | null
  pullSummary: string | null
  onFetch: () => void
  onPreparePull: () => void
  onPreparePush: () => void
  onConfirmPush: (setUpstream: boolean) => void
  onConfirmPull: (strategy: PullStrategy) => void
  onDismiss: () => void
}

function CommitLines({ commits, limit = 8 }: { commits: GitGraphCommit[]; limit?: number }) {
  if (commits.length === 0) {
    return <p className="inspector-empty">Nenhum commit.</p>
  }

  return (
    <ul className="sync-commit-list">
      {commits.slice(0, limit).map((commit) => (
        <li key={commit.hash}>
          <code>{commit.short_hash}</code>
          <span title={commit.message}>{commit.message}</span>
        </li>
      ))}
      {commits.length > limit && (
        <li className="sync-commit-list__more">
          + {commits.length - limit} commit(s)
        </li>
      )}
    </ul>
  )
}

function Failure({ failure }: { failure: GitFailure }) {
  const typed = isGitOperationError(failure) ? failure : null

  return (
    <div className="inspector-error" role="alert">
      <p>{describeFailure(failure)}</p>
      {typed && (
        <>
          <code className="revert-error__code">{typed.code}</code>
          {typed.suggestedAction && (
            <p className="revert-error__action">{typed.suggestedAction}</p>
          )}
          {typed.affectedFiles.length > 0 && (
            <ul className="revert-panel__list">
              {typed.affectedFiles.map((file) => (
                <li key={file}><span title={file}>{file}</span></li>
              ))}
            </ul>
          )}
          {typed.details && (
            <details className="revert-error__details">
              <summary>Saída técnica do Git</summary>
              <pre>{typed.details}</pre>
            </details>
          )}
        </>
      )}
    </div>
  )
}

/**
 * Sincronização do repositório: remote, rastreamento e as três operações.
 *
 * Push e pull passam obrigatoriamente por um plano visível. Fetch executa
 * direto porque não altera nada — ele só atualiza o que o repositório local
 * sabe sobre o remote.
 */
export default function SyncPanel({
  remotes,
  selectedRemote,
  onSelectRemote,
  operation,
  busy,
  failure,
  pushPlan,
  pullPlan,
  fetchOutcome,
  pushSummary,
  pullSummary,
  onFetch,
  onPreparePull,
  onPreparePush,
  onConfirmPush,
  onConfirmPull,
  onDismiss,
}: SyncPanelProps) {
  const upstream = remotes?.upstream ?? null
  const hasRemotes = (remotes?.remotes.length ?? 0) > 0

  return (
    <section className="inspector-section">
      <SectionHeader
        title="Sincronização"
        readout={upstream?.ref_name ?? 'sem upstream'}
      />

      {!hasRemotes ? (
        <p className="inspector-empty">
          Nenhum remote configurado. Adicione um em Remotes para enviar ou
          receber commits.
        </p>
      ) : (
        <>
          <div className="sync-controls">
            <label className="sync-remote">
              <TechnicalLabel tone="low" size="micro">Remote</TechnicalLabel>
              <select
                value={selectedRemote ?? ''}
                onChange={(event) => onSelectRemote(event.target.value)}
                disabled={busy}
              >
                {remotes?.remotes.map((remote) => (
                  <option key={remote.name} value={remote.name}>
                    {remote.name}
                    {remote.is_upstream ? ' · upstream' : ''}
                  </option>
                ))}
              </select>
            </label>

            <StatusIndicator
              tone={
                !upstream || upstream.gone
                  ? 'warn'
                  : upstream.ahead > 0 || upstream.behind > 0
                    ? 'info'
                    : 'nominal'
              }
              label={describeTracking(upstream)}
              live={busy}
            />
          </div>

          <MetricCluster>
            <Metric
              label="Ahead"
              value={String(upstream?.ahead ?? 0).padStart(2, '0')}
              tone={upstream && upstream.ahead > 0 ? 'instrument' : 'neutral'}
            />
            <Metric
              label="Behind"
              value={String(upstream?.behind ?? 0).padStart(2, '0')}
              tone={upstream && upstream.behind > 0 ? 'warn' : 'neutral'}
            />
            <Metric
              label="Upstream"
              value={upstream ? (upstream.gone ? 'ausente' : 'linked') : '—'}
            />
          </MetricCluster>

          <div className="sync-actions">
            <Button onClick={onFetch} disabled={busy} busy={busy && operation === 'fetch'}>
              Fetch
            </Button>
            <Button onClick={onPreparePull} disabled={busy}>
              Pull…
            </Button>
            <Button variant="primary" onClick={onPreparePush} disabled={busy}>
              Push…
            </Button>
          </div>
        </>
      )}

      {failure && <Failure failure={failure} />}

      {pushSummary && (
        <div className="sync-report sync-report--ok" role="status">
          <strong className="sync-report__title">Push concluído</strong>
          <p className="sync-report__note">{pushSummary}</p>
        </div>
      )}

      {pullSummary && (
        <div className="sync-report sync-report--ok" role="status">
          <strong className="sync-report__title">Pull concluído</strong>
          <p className="sync-report__note">{pullSummary}</p>
        </div>
      )}

      {operation === 'fetch' && fetchOutcome && (
        <div className="sync-report" role="status">
          <strong className="sync-report__title">
            Fetch · {fetchOutcome.remote}
          </strong>

          {/* Fluxo real: quantos commits desceram para as refs remotas. */}
          <div className="sync-flow">
            <span className="sync-flow__node">REMOTE</span>
            <span className="sync-flow__pipe" data-active={fetchOutcome.received_commits > 0}>
              {fetchOutcome.received_commits} incoming
            </span>
            <span className="sync-flow__node">LOCAL REMOTE REFS</span>
          </div>

          {!fetchOutcome.had_changes && (
            <p className="sync-report__note">Nada novo: as refs já estavam atualizadas.</p>
          )}

          {fetchOutcome.updated_refs.length > 0 && (
            <ul className="sync-ref-list">
              {fetchOutcome.updated_refs.map((ref) => (
                <li key={ref.ref_name} data-kind={ref.kind}>
                  <code>{ref.kind}</code>
                  <span title={ref.ref_name}>{ref.ref_name}</span>
                  <em>{ref.received_commits > 0 ? `+${ref.received_commits}` : ''}</em>
                </li>
              ))}
            </ul>
          )}

          <p className="sync-report__note">
            O working tree não foi alterado.
          </p>
          <div className="sync-plan__actions">
            <Button onClick={onDismiss}>Fechar</Button>
          </div>
        </div>
      )}

      {operation === 'push' && pushPlan && (
        <div className="sync-plan" role="group" aria-label="Plano de push">
          <dl className="sync-plan__grid">
            <div><dt>Source</dt><dd>{pushPlan.source_branch}</dd></div>
            <div>
              <dt>Destination</dt>
              <dd>{pushPlan.remote}/{pushPlan.target_branch}</dd>
            </div>
            <div><dt>Ahead</dt><dd>{String(pushPlan.ahead).padStart(2, '0')}</dd></div>
            <div><dt>Behind</dt><dd>{String(pushPlan.behind).padStart(2, '0')}</dd></div>
            <div>
              <dt>Upstream</dt>
              <dd>
                {pushPlan.will_create_upstream
                  ? 'será criado'
                  : pushPlan.upstream?.ref_name ?? 'linked'}
              </dd>
            </div>
          </dl>

          {pushPlan.warnings.map((warning) => (
            <p className="revert-panel__warning" key={warning}>{warning}</p>
          ))}

          <p className="revert-panel__label">
            Commits a enviar ({pushPlan.commits.length})
          </p>
          <CommitLines commits={pushPlan.commits} />

          {pushPlan.blocked ? (
            <p className="revert-panel__warning">{pushPlan.blocked}</p>
          ) : null}

          <div className="sync-plan__actions">
            <Button size="md" onClick={onDismiss} disabled={busy}>Cancelar</Button>
            <Button
              size="md"
              variant="primary"
              onClick={() => onConfirmPush(pushPlan.will_create_upstream)}
              disabled={busy || Boolean(pushPlan.blocked)}
              busy={busy}
            >
              {pushPlan.will_create_upstream ? 'Enviar e criar upstream' : 'Enviar'}
            </Button>
          </div>

          <p className="revert-panel__hint">
            Equivalente a <code>
              git push{pushPlan.will_create_upstream ? ' -u' : ''} {pushPlan.remote} {pushPlan.source_branch}:{pushPlan.target_branch}
            </code>.
          </p>
        </div>
      )}

      {operation === 'pull' && pullPlan && (
        <div className="sync-plan" role="group" aria-label="Plano de pull">
          <dl className="sync-plan__grid">
            <div><dt>Branch</dt><dd>{pullPlan.branch}</dd></div>
            <div>
              <dt>Upstream</dt>
              <dd>{pullPlan.upstream?.ref_name ?? `${pullPlan.remote}/${pullPlan.branch}`}</dd>
            </div>
            <div><dt>Incoming</dt><dd>{String(pullPlan.incoming.length).padStart(2, '0')}</dd></div>
            <div><dt>Outgoing</dt><dd>{String(pullPlan.outgoing.length).padStart(2, '0')}</dd></div>
          </dl>

          {pullPlan.warnings.map((warning) => (
            <p className="revert-panel__warning" key={warning}>{warning}</p>
          ))}

          {pullPlan.conflict_risk.length > 0 && (
            <>
              <p className="revert-panel__label">
                Risco de conflito ({pullPlan.conflict_risk.length})
              </p>
              <ul className="revert-panel__list">
                {pullPlan.conflict_risk.map((file) => (
                  <li key={file}>
                    <code>local</code>
                    <span title={file}>{file}</span>
                  </li>
                ))}
              </ul>
            </>
          )}

          <p className="revert-panel__label">
            Commits a receber ({pullPlan.incoming.length})
          </p>
          <CommitLines commits={pullPlan.incoming} />

          {pullPlan.blocked ? (
            <p className="revert-panel__warning">{pullPlan.blocked}</p>
          ) : (
            <>
              <p className="revert-panel__label">Estratégia</p>
              <div className="sync-strategies">
                {pullPlan.available_strategies.map((strategy) => (
                  <div className="sync-strategy" key={strategy}>
                    <Button
                      size="md"
                      block
                      variant={strategy === pullPlan.recommended_strategy ? 'primary' : 'ghost'}
                      onClick={() => onConfirmPull(strategy)}
                      disabled={busy}
                      busy={busy}
                    >
                      {STRATEGY_LABEL[strategy]}
                      {strategy === pullPlan.recommended_strategy ? ' · recomendado' : ''}
                    </Button>
                    <small>{STRATEGY_EXPLANATION[strategy]}</small>
                  </div>
                ))}
              </div>
            </>
          )}

          <div className="sync-plan__actions">
            <Button size="md" onClick={onDismiss} disabled={busy}>Cancelar</Button>
          </div>
        </div>
      )}
    </section>
  )
}
