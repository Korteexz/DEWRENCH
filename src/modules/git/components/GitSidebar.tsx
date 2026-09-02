import {
  Button,
  DataRow,
  Panel,
  SectionHeader,
  StatusIndicator,
  TelemetryBar,
} from '../../../design'
import { branchNodeId, commitNodeId, PROJECT_NODE_ID } from '../../../app/graph/types'
import type { ProjectOpenResult } from '../types/project'
import type { GitGraph, GitRepositoryDetails } from '../types/repository'
import { summarizeGraph } from '../view/graphStats'
import { summarizeWorkingTree } from '../view/workingTree'

interface GitSidebarProps {
  project: ProjectOpenResult
  details: GitRepositoryDetails | null
  graph: GitGraph | null
  selectedNodeId: string | null
  loading: boolean
  onSelectNode: (nodeId: string) => void
  onRefresh: () => void
}

/** Quantidade de commits listados no índice; o grafo continua mostrando todos. */
const COMMIT_INDEX_LIMIT = 24

/**
 * Índice do repositório.
 *
 * Não busca dados: recebe o mesmo snapshot que o grafo usa, o que garante que
 * lista e topologia nunca discordem sobre o estado do repositório.
 */
export default function GitSidebar({
  project,
  details,
  graph,
  selectedNodeId,
  loading,
  onSelectNode,
  onRefresh,
}: GitSidebarProps) {
  const stats = summarizeGraph(graph)
  const tree = summarizeWorkingTree(details?.files)
  const branches = graph?.branches ?? []
  const commits = graph?.commits ?? []

  return (
    <Panel
      as="aside"
      index="01"
      title="Repository index"
      aria-label="Índice do repositório"
      flush
      scroll
      className="git-sidebar"
      actions={(
        <Button onClick={onRefresh} disabled={loading} busy={loading}>
          {loading ? 'SYNC…' : 'SYNC'}
        </Button>
      )}
      footer={(
        <>
          <StatusIndicator
            tone={graph ? 'nominal' : 'idle'}
            label={graph ? 'GIT CLI' : 'NO DATA'}
            live={loading}
          />
          <StatusIndicator tone="idle" label="LOCAL" filled={false} />
        </>
      )}
    >
      <div className="git-sidebar__subject">
        <DataRow
          primary={project.name}
          secondary={stats.currentBranch ?? details?.branch ?? 'resolvendo branch'}
          lead={<span className="git-sidebar__subject-mark" />}
          trail={tree.clean ? 'CLEAN' : `${tree.total}Δ`}
          selected={selectedNodeId === PROJECT_NODE_ID}
          onSelect={() => onSelectNode(PROJECT_NODE_ID)}
          title={project.path}
        />

        <div className="git-sidebar__telemetry">
          <TelemetryBar
            label="Staged"
            value={tree.staged}
            total={Math.max(tree.total, 1)}
            segments={18}
            tone={tree.staged > 0 ? 'instrument' : 'neutral'}
            readout={`${tree.staged}/${tree.total}`}
          />
        </div>
      </div>

      <section className="git-sidebar__section">
        <SectionHeader
          title="Branches"
          readout={branches.length.toString().padStart(2, '0')}
        />
        <div className="git-sidebar__list">
          {branches.map((branch) => {
            const nodeId = branchNodeId(branch.name)
            return (
              <DataRow
                key={branch.name}
                primary={branch.name}
                lead={<span className="git-sidebar__branch-mark" />}
                tag={branch.current ? 'HEAD' : undefined}
                trail={branch.head.slice(0, 7)}
                selected={selectedNodeId === nodeId}
                onSelect={() => onSelectNode(nodeId)}
                title={`${branch.name} → ${branch.head}`}
              />
            )
          })}
        </div>
      </section>

      <section className="git-sidebar__section">
        <SectionHeader
          title="History"
          readout={`${Math.min(commits.length, COMMIT_INDEX_LIMIT)}/${stats.commitCount}`}
        />
        <div className="git-sidebar__list">
          {commits.slice(0, COMMIT_INDEX_LIMIT).map((commit, index) => {
            const nodeId = commitNodeId(commit.hash)
            return (
              <DataRow
                key={commit.hash}
                primary={commit.short_hash}
                secondary={commit.message}
                lead={(
                  <span
                    className="git-sidebar__commit-mark"
                    data-merge={commit.parents.length > 1}
                    data-root={commit.parents.length === 0}
                  />
                )}
                trail={String(index + 1).padStart(2, '0')}
                selected={selectedNodeId === nodeId}
                onSelect={() => onSelectNode(nodeId)}
                title={commit.message}
              />
            )
          })}
        </div>
      </section>
    </Panel>
  )
}
