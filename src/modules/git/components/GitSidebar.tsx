import { branchNodeId, commitNodeId, PROJECT_NODE_ID } from '../../../app/graph/types'
import type { ProjectOpenResult } from '../types/project'
import type { GitGraph, GitRepositoryDetails } from '../types/repository'

interface GitSidebarProps {
  project: ProjectOpenResult
  details: GitRepositoryDetails | null
  graph: GitGraph | null
  selectedNodeId: string | null
  loading: boolean
  onSelectNode: (nodeId: string) => void
  onRefresh: () => void
}

/** Compact repository navigation mirrors Git hierarchy without duplicating data fetching. */
export default function GitSidebar({
  project,
  details,
  graph,
  selectedNodeId,
  loading,
  onSelectNode,
  onRefresh,
}: GitSidebarProps) {
  const currentBranch = graph?.branches.find((branch) => branch.current)
  const branches = graph?.branches ?? []
  const commits = graph?.commits ?? []
  const dirtyCount = details?.files.length ?? 0

  return (
    <aside className="git-sidebar" aria-label="Histórico Git">
      <div className="panel-terminal-label">
        <span>01</span>
        <strong>REPOSITORY INDEX</strong>
        <button type="button" onClick={onRefresh} disabled={loading}>
          {loading ? 'SYNCING' : 'SYNC'}
        </button>
      </div>

      <button
        className={`repository-readout${selectedNodeId === PROJECT_NODE_ID ? ' is-selected' : ''}`}
        type="button"
        onClick={() => onSelectNode(PROJECT_NODE_ID)}
      >
        <span className="repository-readout__status" aria-hidden="true" />
        <span>
          <strong>{project.name}</strong>
          <small>{currentBranch?.name ?? details?.branch ?? 'resolving branch'}</small>
        </span>
        <code>{dirtyCount.toString().padStart(2, '0')} Δ</code>
      </button>

      <section className="git-tree-section">
        <header>
          <span>BRANCHES</span>
          <code>{branches.length.toString().padStart(2, '0')}</code>
        </header>
        <ul className="git-tree-list git-tree-list--branches">
          {branches.map((branch) => {
            const nodeId = branchNodeId(branch.name)
            return (
              <li key={branch.name}>
                <button
                  className={selectedNodeId === nodeId ? 'is-selected' : ''}
                  type="button"
                  onClick={() => onSelectNode(nodeId)}
                >
                  <span className="git-tree-list__fork" aria-hidden="true" />
                  <span title={branch.name}>{branch.name}</span>
                  {branch.current && <small>HEAD</small>}
                </button>
              </li>
            )
          })}
        </ul>
      </section>

      <section className="git-tree-section git-tree-section--commits">
        <header>
          <span>RECENT COMMITS</span>
          <code>{commits.length.toString().padStart(2, '0')}</code>
        </header>
        <ul className="git-tree-list git-tree-list--commits">
          {commits.slice(0, 18).map((commit, index) => {
            const nodeId = commitNodeId(commit.hash)
            return (
              <li key={commit.hash}>
                <button
                  className={selectedNodeId === nodeId ? 'is-selected' : ''}
                  type="button"
                  onClick={() => onSelectNode(nodeId)}
                >
                  <span className="git-tree-list__line" aria-hidden="true">
                    <i />
                  </span>
                  <span>
                    <code>{commit.short_hash}</code>
                    <small title={commit.message}>{commit.message}</small>
                  </span>
                  <em>{String(index + 1).padStart(2, '0')}</em>
                </button>
              </li>
            )
          })}
        </ul>
      </section>

      <footer className="git-sidebar__footer">
        <span><i className="status-dot status-dot--green" />GIT ONLINE</span>
        <span>LOCAL</span>
      </footer>
    </aside>
  )
}
