import { useState } from 'react'

import { Button } from '../../../../design'
import DiffView from '../../../../modules/git/components/DiffView'
import RevertPanel from '../../../../modules/git/components/RevertPanel'
import type { GitGraphCommit } from '../../../../modules/git/types/repository'
import type {
  GitFailure,
  GitRevertOutcome,
  GitRevertPreview,
} from '../../../../modules/git/types/revert'

interface CommitInspectorProps {
  commit: GitGraphCommit
  diff: string | null
  diffLoading: boolean
  busy: boolean
  error: string | null
  onClose: () => void
  onViewDiff: () => void
  onCloseDiff: () => void
  onCreateBranch: (name: string) => Promise<boolean>
  revertPreview: GitRevertPreview | null
  revertLoading: boolean
  revertFailure: GitFailure | null
  revertOutcome: GitRevertOutcome | null
  onRequestRevertPreview: () => void
  onCancelRevert: () => void
  onConfirmRevert: () => void
}

export default function CommitInspector({
  commit,
  diff,
  diffLoading,
  busy,
  error,
  onClose,
  onViewDiff,
  onCloseDiff,
  onCreateBranch,
  revertPreview,
  revertLoading,
  revertFailure,
  revertOutcome,
  onRequestRevertPreview,
  onCancelRevert,
  onConfirmRevert,
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
            <Button
              onClick={diff === null ? onViewDiff : onCloseDiff}
              disabled={diffLoading}
              busy={diffLoading}
            >
              {diffLoading ? 'Lendo…' : diff === null ? 'Ver diff' : 'Fechar diff'}
            </Button>
          </div>
          {diff !== null && <DiffView source={diff} />}
        </section>

        <RevertPanel
          commit={commit}
          preview={revertPreview}
          loading={revertLoading}
          busy={busy}
          failure={revertFailure}
          outcome={revertOutcome}
          onRequestPreview={onRequestRevertPreview}
          onCancel={onCancelRevert}
          onConfirm={onConfirmRevert}
        />

        <section className="inspector-section">
          <h2>Nova branch a partir deste commit</h2>
          <div className="inspector-form">
            <input
              value={branchName}
              onChange={(event) => setBranchName(event.target.value)}
              placeholder="nome-da-branch"
              disabled={busy}
            />
            <Button
              size="md"
              variant="primary"
              block
              onClick={() => void handleCreateBranch()}
              disabled={busy || branchName.trim().length === 0}
            >
              Criar em {commit.short_hash}
            </Button>
          </div>
        </section>

        {error && <p className="inspector-error">{error}</p>}
      </div>
    </aside>
  )
}
