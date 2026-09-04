import { useState } from 'react'

import { Button, Metric, MetricCluster, SectionHeader, StatusIndicator } from '../../design'
import { describeFailure, isGitOperationError } from '../git/types/revert'
import ComparePanel from './ComparePanel'
import PullRequestPanel from './PullRequestPanel'
import { openGithubInBrowser } from './services'
import { useGithub } from './useGithub'

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
 *
 * O Compare é a exceção deliberada: ele é calculado pelo Git local e continua
 * disponível mesmo sem `gh` autenticada.
 */
export default function GithubPanel({
  projectPath,
  currentBranch,
  busy,
}: GithubPanelProps) {
  const github = useGithub(projectPath)
  const [selected, setSelected] = useState<number | null>(null)
  const [creating, setCreating] = useState(false)
  const [title, setTitle] = useState('')
  const [body, setBody] = useState('')
  const [base, setBase] = useState('')
  const [draft, setDraft] = useState(false)

  const context = github.context
  const failure = github.failure

  if (!context?.detected) {
    return null
  }

  const disabled = busy || github.loading || github.busy
  const effectiveBase = base.trim().length > 0
    ? base.trim()
    : context.default_branch

  async function handleCreate(): Promise<void> {
    if (!currentBranch) {
      return
    }

    const created = await github.create(
      title,
      body,
      currentBranch,
      effectiveBase,
      draft,
    )

    if (created) {
      setTitle('')
      setBody('')
      setCreating(false)
    }
  }

  return (
    <section className="inspector-section">
      <SectionHeader
        title="GitHub"
        readout={context.owner && context.repository
          ? `${context.owner}/${context.repository}`
          : undefined}
        actions={(
          <Button
            onClick={() => void github.reload()}
            disabled={disabled}
            busy={github.loading}
          >
            {github.loading ? 'Lendo…' : 'Atualizar'}
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
          <SectionHeader
            title="Pull requests"
            readout={github.pullRequests.length.toString().padStart(2, '0')}
            actions={currentBranch
              ? (
                <Button
                  onClick={() => setCreating((value) => !value)}
                  disabled={disabled}
                >
                  {creating ? 'Cancelar' : 'Abrir pull request'}
                </Button>
              )
              : undefined}
          />

          {creating && currentBranch && (
            <div className="remote-form">
              <input
                value={title}
                onChange={(event) => setTitle(event.target.value)}
                placeholder="Título do pull request"
                aria-label="Título do pull request"
                disabled={disabled}
              />
              <textarea
                value={body}
                onChange={(event) => setBody(event.target.value)}
                placeholder="Descrição (opcional)"
                aria-label="Descrição do pull request"
                rows={3}
                disabled={disabled}
              />
              <input
                value={base}
                onChange={(event) => setBase(event.target.value)}
                placeholder={context.default_branch ?? 'Branch de destino'}
                aria-label="Branch de destino"
                disabled={disabled}
              />
              <label className="github-hint">
                <input
                  type="checkbox"
                  checked={draft}
                  onChange={(event) => setDraft(event.target.checked)}
                  disabled={disabled}
                />
                Abrir como rascunho (draft)
              </label>
              <div className="remote-form__actions">
                <Button
                  variant="primary"
                  disabled={disabled || title.trim().length === 0}
                  busy={github.busy}
                  onClick={() => void handleCreate()}
                >
                  Abrir pull request
                </Button>
              </div>
              <small className="github-hint">
                Base: {effectiveBase ?? 'padrão do repositório'} · origem: {currentBranch}
              </small>
            </div>
          )}

          {github.pullRequests.length === 0 ? (
            <p className="inspector-empty">
              Nenhum pull request neste repositório.
            </p>
          ) : (
            <ul className="pr-list">
              {github.pullRequests.map((pr) => (
                <li key={pr.number} data-selected={selected === pr.number}>
                  <span className="pr-list__number">#{pr.number}</span>
                  <button
                    type="button"
                    className="pr-list__title pr-list__open"
                    title={pr.title}
                    onClick={() => setSelected(
                      selected === pr.number ? null : pr.number,
                    )}
                  >
                    {pr.title}
                  </button>
                  <span className="pr-list__state" data-state={pr.state.toLowerCase()}>
                    {pr.is_draft ? 'DRAFT' : pr.state}
                  </span>
                  <small>{pr.head_branch} → {pr.base_branch}</small>
                </li>
              ))}
            </ul>
          )}

          {github.createdUrl && (
            <p className="sync-report__note">
              Pull request criado: {github.createdUrl}
            </p>
          )}

          {selected !== null && (
            <PullRequestPanel
              projectPath={projectPath}
              number={selected}
              busy={busy}
              onClose={() => setSelected(null)}
              onChanged={() => void github.reload()}
            />
          )}
        </>
      )}

      <ComparePanel
        projectPath={projectPath}
        defaultBase={context.default_branch}
        defaultHead={currentBranch}
        busy={busy}
        webUrl={context.web_url}
      />

      {failure && (
        <div className="inspector-error" role="alert">
          <p>{describeFailure(failure)}</p>
          {isGitOperationError(failure) && failure.suggestedAction && (
            <p className="revert-error__action">{failure.suggestedAction}</p>
          )}
          <Button onClick={github.dismiss}>Dispensar</Button>
        </div>
      )}
    </section>
  )
}
