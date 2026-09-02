import { useState } from 'react'

import type { ProjectOpenResult } from '../../../../modules/git/types/project'
import type { GitRepositoryDetails } from '../../../../modules/git/types/repository'
import {
  hasUnstagedChanges,
  isStaged,
  statusLabel,
  summarizeWorkingTree,
} from '../../../../modules/git/view/workingTree'

interface ProjectInspectorProps {
  project: ProjectOpenResult
  details: GitRepositoryDetails | null
  loading: boolean
  busy: boolean
  error: string | null
  onClose: () => void
  onRefresh: () => void
  onStage: (file: string) => Promise<boolean>
  onStageAll: () => Promise<boolean>
  onUnstage: (file: string) => Promise<boolean>
  onCommit: (message: string) => Promise<boolean>
}

export default function ProjectInspector({
  project,
  details,
  loading,
  busy,
  error,
  onClose,
  onRefresh,
  onStage,
  onStageAll,
  onUnstage,
  onCommit,
}: ProjectInspectorProps) {
  const [message, setMessage] = useState('')
  const files = details?.files ?? []
  const summary = summarizeWorkingTree(files)
  const unstagedCount = summary.unstaged
  const stagedCount = summary.staged

  async function handleCommit() {
    const committed = await onCommit(message)
    if (committed) {
      setMessage('')
    }
  }

  return (
    <aside className="canvas-inspector nodrag nopan" aria-label="Detalhes do projeto">
      <header className="canvas-inspector__header">
        <div>
          <span>PROJECT</span>
          <strong>{project.name}</strong>
        </div>
        <button type="button" aria-label="Fechar detalhes" onClick={onClose}>×</button>
      </header>

      <div className="canvas-inspector__content">
        <dl className="inspector-summary">
          <div><dt>Branch</dt><dd>{details?.branch ?? '—'}</dd></div>
          <div><dt>Alterados</dt><dd>{files.length}</dd></div>
          <div><dt>Staged</dt><dd>{stagedCount}</dd></div>
        </dl>

        <section className="inspector-section">
          <div className="inspector-section__heading">
            <h2>Working tree</h2>

            <div className="inspector-section__actions">
              <button
                type="button"
                onClick={() => void onStageAll()}
                disabled={loading || busy || unstagedCount === 0}
              >
                Stage all ({unstagedCount})
              </button>

              <button
                type="button"
                onClick={onRefresh}
                disabled={loading || busy}
              >
                {loading ? 'Carregando…' : 'Atualizar'}
              </button>
            </div>
          </div>

          {files.length === 0 && !loading && (
            <p className="inspector-empty">Working tree limpa.</p>
          )}

          <ul className="file-status-list">
            {files.map((file) => (
              <li key={`${file.path}:${file.index_status}:${file.worktree_status}`}>
                <div className="file-status-list__file">
                  <code>{statusLabel(file)}</code>
                  <span title={file.path}>{file.path}</span>
                </div>
                <div className="file-status-list__actions">
                  {hasUnstagedChanges(file) && (
                    <button
                      type="button"
                      onClick={() => void onStage(file.path)}
                      disabled={busy}
                    >
                      Stage
                    </button>
                  )}
                  {isStaged(file) && (
                    <button
                      type="button"
                      onClick={() => void onUnstage(file.path)}
                      disabled={busy}
                    >
                      Unstage
                    </button>
                  )}
                </div>
              </li>
            ))}
          </ul>
        </section>

        <section className="inspector-section">
          <h2>Novo commit</h2>
          <div className="inspector-form">
            <input
              value={message}
              onChange={(event) => setMessage(event.target.value)}
              placeholder="Mensagem do commit"
              disabled={busy}
            />
            <button
              className="inspector-button--primary"
              type="button"
              onClick={() => void handleCommit()}
              disabled={busy || message.trim().length === 0}
            >
              Commit staged files
            </button>
          </div>
        </section>

        {details && details.commits.length > 0 && (
          <section className="inspector-section">
            <h2>Commits recentes</h2>
            <ul className="recent-commit-list">
              {details.commits.slice(0, 3).map((commit) => (
                <li key={commit.hash}>
                  <code>{commit.hash}</code>
                  <span>{commit.message}</span>
                  <small>{commit.author}</small>
                </li>
              ))}
            </ul>
          </section>
        )}

        {error && <p className="inspector-error">{error}</p>}
        <code className="canvas-inspector__path" title={project.path}>{project.path}</code>
      </div>
    </aside>
  )
}
