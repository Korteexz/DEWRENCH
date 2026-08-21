import { useState } from 'react'

import type { GitGraphCommit } from '../../../../modules/git/types/repository'

interface CommitInspectorProps {
  commit: GitGraphCommit
  diff: string | null
  diffLoading: boolean
  busy: boolean
  error: string | null
  onClose: () => void
  onViewDiff: () => void
  onCreateBranch: (name: string) => Promise<boolean>
}

export default function CommitInspector({
  commit,
  diff,
  diffLoading,
  busy,
  error,
  onClose,
  onViewDiff,
  onCreateBranch,
}: CommitInspectorProps) {
  const [branchName, setBranchName] = useState('')

  async function handleCreateBranch() {
    const created = await onCreateBranch(branchName)
    if (created) {
      setBranchName('')
    }
  }

  return (
    <aside className="canvas-inspector canvas-inspector--wide nodrag nopan" aria-label="Detalhes do commit">
      <header className="canvas-inspector__header">
        <div>
          <span>COMMIT / {commit.short_hash}</span>
          <strong>{commit.message}</strong>
        </div>
        <button type="button" aria-label="Fechar detalhes" onClick={onClose}>×</button>
      </header>

      <div className="canvas-inspector__content">
        <dl className="inspector-details">
          <div><dt>Hash</dt><dd title={commit.hash}>{commit.hash}</dd></div>
          <div><dt>Autor</dt><dd>{commit.author}</dd></div>
          <div>
            <dt>Parents</dt>
            <dd>{commit.parents.length > 0 ? commit.parents.map((hash) => hash.slice(0, 8)).join(', ') : 'root'}</dd>
          </div>
        </dl>

        <section className="inspector-section">
          <div className="inspector-section__heading">
            <h2>Diff</h2>
            <button type="button" onClick={onViewDiff} disabled={diffLoading}>
              {diffLoading ? 'Carregando…' : 'View diff'}
            </button>
          </div>
          {diff !== null && (
            <pre className="commit-diff">{diff || 'Nenhuma alteração retornada.'}</pre>
          )}
        </section>

        <section className="inspector-section">
          <h2>Nova branch a partir deste commit</h2>
          <div className="inspector-form">
            <input
              value={branchName}
              onChange={(event) => setBranchName(event.target.value)}
              placeholder="nome-da-branch"
              disabled={busy}
            />
            <button
              className="inspector-button--primary"
              type="button"
              onClick={() => void handleCreateBranch()}
              disabled={busy || branchName.trim().length === 0}
            >
              Criar em {commit.short_hash}
            </button>
          </div>
        </section>

        {error && <p className="inspector-error">{error}</p>}
      </div>
    </aside>
  )
}
