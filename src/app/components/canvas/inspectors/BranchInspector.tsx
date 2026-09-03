import { useState } from 'react'

import { Button } from '../../../../design'
import type { GitBranch } from '../../../../modules/git/types/repository'

interface BranchInspectorProps {
  branch: GitBranch
  busy: boolean
  error: string | null
  onClose: () => void
  onSwitch: () => Promise<boolean>
  onCreateBranch: (name: string) => Promise<boolean>
}

export default function BranchInspector({
  branch,
  busy,
  error,
  onClose,
  onSwitch,
  onCreateBranch,
}: BranchInspectorProps) {
  const [branchName, setBranchName] = useState('')

  async function handleCreateBranch() {
    const created = await onCreateBranch(branchName)
    if (created) {
      setBranchName('')
    }
  }

  return (
    <aside className="canvas-inspector nodrag nopan" aria-label="Detalhes da branch">
      <header className="canvas-inspector__header">
        <div>
          <span>BRANCH</span>
          <strong>{branch.name}</strong>
        </div>
        <button type="button" aria-label="Fechar detalhes" onClick={onClose}>×</button>
      </header>

      <div className="canvas-inspector__content">
        <dl className="inspector-details">
          <div><dt>Estado</dt><dd>{branch.current ? 'current' : 'available'}</dd></div>
          <div><dt>Head</dt><dd title={branch.head}>{branch.head}</dd></div>
        </dl>

        {!branch.current && (
          <div className="inspector-action">
            <Button size="md" block onClick={() => void onSwitch()} disabled={busy}>
              Switch branch
            </Button>
          </div>
        )}

        <section className="inspector-section">
          <h2>Nova branch a partir daqui</h2>
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
              Criar em {branch.name}
            </Button>
          </div>
        </section>

        {error && <p className="inspector-error">{error}</p>}
      </div>
    </aside>
  )
}
