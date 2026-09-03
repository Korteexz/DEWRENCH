import { useCallback, useEffect, useState } from 'react'

import { Button, Metric, MetricCluster, SectionHeader, StatusIndicator } from '../../design'
import {
  describeFailure,
  isGitOperationError,
  toGitFailure,
  type GitFailure,
} from '../git/types/revert'
import {
  createPullRequest,
  getGithubContext,
  listPullRequests,
  openGithubInBrowser,
} from './services'
import type { GithubContext, GithubPullRequest } from './types'

interface GithubPanelProps {
  projectPath: string
  currentBranch: string | null
  busy: boolean
}

/**
 * GitHub como provider opcional.
 *
 * O painel só aparece quando algum remote aponta para o GitHub, e degrada em
 * degraus visíveis: sem `gh` instalada, mostra o que o Git local já sabe; sem
 * autenticação, explica o que falta. Nenhuma função do Git depende dele.
 */
export default function GithubPanel({
  projectPath,
  currentBranch,
  busy,
}: GithubPanelProps) {
  const [context, setContext] = useState<GithubContext | null>(null)
  const [pullRequests, setPullRequests] = useState<GithubPullRequest[]>([])
  const [loading, setLoading] = useState(false)
  const [failure, setFailure] = useState<GitFailure | null>(null)
  const [creating, setCreating] = useState(false)
  const [title, setTitle] = useState('')
  const [createdUrl, setCreatedUrl] = useState<string | null>(null)

  const load = useCallback(async () => {
    setLoading(true)
    setFailure(null)

    try {
      const next = await getGithubContext(projectPath)
      setContext(next)

      if (next.authenticated) {
        try {
          setPullRequests(await listPullRequests(projectPath, currentBranch ?? undefined))
        } catch (error) {
          // PR indisponível não invalida o contexto já lido.
          setFailure(toGitFailure(error))
        }
      } else {
        setPullRequests([])
      }
    } catch (error) {
      setContext(null)
      setFailure(toGitFailure(error))
    } finally {
      setLoading(false)
    }
  }, [currentBranch, projectPath])

  useEffect(() => {
    void load()
  }, [load])

  if (!context?.detected) {
    return null
  }

  const disabled = busy || loading || creating

  return (
    <section className="inspector-section">
      <SectionHeader
        title="GitHub"
        readout={context.owner && context.repository
          ? `${context.owner}/${context.repository}`
          : undefined}
        actions={(
          <Button onClick={() => void load()} disabled={disabled} busy={loading}>
            {loading ? 'Lendo…' : 'Atualizar'}
          </Button>
        )}
      />

      <MetricCluster>
        <Metric label="Owner" value={context.owner ?? '—'} />
        <Metric label="Repo" value={context.repository ?? '—'} />
        <Metric
          label="Default"
          value={context.default_branch ?? '—'}
          title={context.default_branch ? undefined : 'Requer gh autenticada'}
        />
      </MetricCluster>

      <div className="github-status">
        <StatusIndicator
          tone={context.cli_available ? 'nominal' : 'idle'}
          label={context.cli_available ? 'GH CLI' : 'GH AUSENTE'}
          filled={context.cli_available}
        />
        <StatusIndicator
          tone={context.authenticated ? 'nominal' : 'warn'}
          label={context.authenticated ? 'AUTENTICADO' : 'SEM SESSÃO'}
          filled={context.authenticated}
        />
      </div>

      {context.limitation && (
        <p className="github-limitation">{context.limitation}</p>
      )}

      <div className="sync-plan__actions">
        <Button
          onClick={() => void openGithubInBrowser(projectPath, currentBranch ?? undefined)}
          disabled={disabled || !context.web_url}
        >
          Abrir no GitHub
        </Button>
      </div>

      {context.authenticated && (
        <>
          <p className="revert-panel__label">
            Pull requests desta branch ({pullRequests.length})
          </p>

          {pullRequests.length === 0 ? (
            <p className="inspector-empty">
              Nenhum pull request para {currentBranch ?? 'esta branch'}.
            </p>
          ) : (
            <ul className="pr-list">
              {pullRequests.map((pr) => (
                <li key={pr.number}>
                  <span className="pr-list__number">#{pr.number}</span>
                  <span className="pr-list__title" title={pr.title}>{pr.title}</span>
                  <span className="pr-list__state" data-state={pr.state.toLowerCase()}>
                    {pr.is_draft ? 'DRAFT' : pr.state}
                  </span>
                  <small>{pr.head_branch} → {pr.base_branch}</small>
                </li>
              ))}
            </ul>
          )}

          {pullRequests.length === 0 && currentBranch && (
            <div className="remote-form">
              <input
                value={title}
                onChange={(event) => setTitle(event.target.value)}
                placeholder="Título do pull request"
                disabled={disabled}
              />
              <div className="remote-form__actions">
                <Button
                  variant="primary"
                  disabled={disabled || title.trim().length === 0}
                  busy={creating}
                  onClick={() => {
                    setCreating(true)
                    setFailure(null)
                    setCreatedUrl(null)
                    void createPullRequest(
                      projectPath,
                      title,
                      '',
                      currentBranch,
                      context.default_branch,
                      false,
                    )
                      .then((url) => {
                        setCreatedUrl(url)
                        setTitle('')
                        return load()
                      })
                      .catch((error: unknown) => setFailure(toGitFailure(error)))
                      .finally(() => setCreating(false))
                  }}
                >
                  Abrir pull request
                </Button>
              </div>
              <small className="github-hint">
                Base: {context.default_branch ?? 'padrão do repositório'} · origem: {currentBranch}
              </small>
            </div>
          )}

          {createdUrl && (
            <p className="sync-report__note">Pull request criado: {createdUrl}</p>
          )}
        </>
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
