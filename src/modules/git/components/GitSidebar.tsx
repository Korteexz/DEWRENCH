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
import type { GitBranch, GitGraph, GitRepositoryDetails } from '../types/repository'
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
 * Leitura de rastreamento de uma branch local.
 *
 * Setas com número em vez de rótulo por extenso: o índice é uma coluna
 * estreita, e ↑/↓ com contagem é a convenção que qualquer usuário de Git já lê
 * sem legenda.
 */
function TrackingReadout({ branch }: { branch: GitBranch }) {
  if (branch.gone) {
    return <span className="git-tracking" data-tone="warn">upstream ausente</span>
  }

  if (!branch.upstream) {
    return <span className="git-tracking" data-tone="idle">local</span>
  }

  if (branch.ahead === 0 && branch.behind === 0) {
    return <span className="git-tracking" data-tone="ok">sync</span>
  }

  return (
    <span className="git-tracking" data-tone={branch.ahead > 0 && branch.behind > 0 ? 'warn' : 'active'}>
      {branch.ahead > 0 && <b>↑{branch.ahead}</b>}
      {branch.behind > 0 && <i>↓{branch.behind}</i>}
    </span>
  )
}

function describeBranch(branch: GitBranch): string {
  const tracking = branch.upstream
    ? `rastreando ${branch.upstream}${branch.gone ? ' (ausente)' : ''}`
    : 'sem upstream'
  return `${branch.name} → ${branch.head} · ${tracking}`
}

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
  const remoteBranches = graph?.remote_branches ?? []
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
                secondary={branch.upstream ?? undefined}
                lead={<span className="git-sidebar__branch-mark" />}
                tag={branch.current ? 'HEAD' : undefined}
                trail={<TrackingReadout branch={branch} />}
                selected={selectedNodeId === nodeId}
                onSelect={() => onSelectNode(nodeId)}
                title={describeBranch(branch)}
              />
            )
          })}
        </div>
      </section>

      {remoteBranches.length > 0 && (
        <section className="git-sidebar__section">
          <SectionHeader
            title="Remote tracking"
            readout={String(remoteBranches.length).padStart(2, '0')}
          />
          <div className="git-sidebar__list">
            {remoteBranches.map((branch) => (
              <DataRow
                key={branch.name}
                primary={branch.name}
                lead={<span className="git-sidebar__remote-mark" />}
                trail={branch.head.slice(0, 7)}
                selected={false}
                onSelect={() => onSelectNode(branchNodeId(branch.name))}
                title={`${branch.name} → ${branch.head}`}
              />
            ))}
          </div>
        </section>
      )}

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
