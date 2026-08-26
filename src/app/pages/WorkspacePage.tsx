import { useCallback, useMemo, useState } from 'react'
import type { NodeMouseHandler } from '@xyflow/react'

import BranchInspector from '../components/canvas/inspectors/BranchInspector'
import CommitInspector from '../components/canvas/inspectors/CommitInspector'
import ProjectInspector from '../components/canvas/inspectors/ProjectInspector'
import NodeContextMenu, {
  type NodeContextMenuItem,
} from '../components/canvas/menus/NodeContextMenu'
import WorkspaceCanvas from '../components/canvas/WorkspaceCanvas'
import { layoutWorkspaceGraph } from '../graph/layout'
import { commitNodeId, type WorkspaceFlowNode } from '../graph/types'
import { adaptGitGraph } from '../../modules/git/adapters/gitGraphAdapter'
import { useGitGraph } from '../../modules/git/hooks/useGitGraph'
import {
  createBranchFrom,
  createCommit,
  getCommitDiff,
  stageFile,
  switchBranch,
  unstageFile,
} from '../../modules/git/services/gitServices'
import type { ProjectOpenResult } from '../../modules/git/types/project'
import type {
  GitBranch,
  GitGraphCommit,
} from '../../modules/git/types/repository'

interface WorkspacePageProps {
  project: ProjectOpenResult
  onOpenAnotherProject: () => void
}

interface ContextMenuState {
  nodeId: string
  x: number
  y: number
}

interface CommitDiffState {
  commitHash: string
  value: string
}

function getErrorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

export default function WorkspacePage({
  project,
  onOpenAnotherProject,
}: WorkspacePageProps) {
  const {
    repositoryDetails,
    gitGraph,
    loading,
    error: loadingError,
    refresh,
  } = useGitGraph(project.path, project.git_state === 'repository')
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null)
  const [contextMenu, setContextMenu] = useState<ContextMenuState | null>(null)
  const [busyAction, setBusyAction] = useState<string | null>(null)
  const [actionError, setActionError] = useState<string | null>(null)
  const [commitDiff, setCommitDiff] = useState<CommitDiffState | null>(null)
  const [diffLoading, setDiffLoading] = useState(false)

  const workspaceGraph = useMemo(
    () => adaptGitGraph(project, gitGraph),
    [gitGraph, project],
  )
  const positionedGraph = useMemo(
    () => layoutWorkspaceGraph(workspaceGraph),
    [workspaceGraph],
  )
  const layoutVersion = useMemo(() => {
    if (!gitGraph) {
      return `project:${project.path}`
    }

    const branchVersion = gitGraph.branches
      .map((branch) => `${branch.name}:${branch.head}:${branch.current}`)
      .join('|')
    const commitVersion = gitGraph.commits
      .map((commit) => `${commit.hash}:${commit.parents.join(',')}`)
      .join('|')
    return `${branchVersion}::${commitVersion}`
  }, [gitGraph, project.path])

  const selectedNode = positionedGraph.nodes.find(
    (node) => node.id === selectedNodeId,
  )
  const contextNode = positionedGraph.nodes.find(
    (node) => node.id === contextMenu?.nodeId,
  )

  const closeContextMenu = useCallback(() => setContextMenu(null), [])

  async function executeMutation(
    label: string,
    operation: () => Promise<unknown>,
  ): Promise<boolean> {
    if (busyAction) {
      return false
    }

    setBusyAction(label)
    setActionError(null)

    try {
      await operation()
      await refresh()
      return true
    } catch (error) {
      setActionError(getErrorMessage(error))
      return false
    } finally {
      setBusyAction(null)
    }
  }

  async function handleRefresh(): Promise<void> {
    setActionError(null)
    try {
      await refresh()
    } catch {
      // useGitGraph exposes the backend error through loadingError.
    }
  }

  function handleStage(file: string): Promise<boolean> {
    return executeMutation('Stage file', () => stageFile(project.path, file))
  }

  function handleUnstage(file: string): Promise<boolean> {
    return executeMutation('Unstage file', () => unstageFile(project.path, file))
  }

  function handleCommit(message: string): Promise<boolean> {
    return executeMutation('Create commit', () => createCommit(project.path, message))
  }

  function handleSwitchBranch(branch: GitBranch): Promise<boolean> {
    return executeMutation('Switch branch', () => switchBranch(project.path, branch.name))
  }

  function handleCreateBranch(
    startPoint: string,
    branchName: string,
  ): Promise<boolean> {
    return executeMutation('Create branch', () => (
      createBranchFrom(project.path, startPoint, branchName)
    ))
  }

  async function handleViewDiff(commit: GitGraphCommit): Promise<void> {
    setSelectedNodeId(commitNodeId(commit.hash))
    setContextMenu(null)
    setDiffLoading(true)
    setActionError(null)

    try {
      const value = await getCommitDiff(project.path, commit.hash)
      setCommitDiff({ commitHash: commit.hash, value })
    } catch (error) {
      setActionError(getErrorMessage(error))
    } finally {
      setDiffLoading(false)
    }
  }

  const handleNodeClick: NodeMouseHandler<WorkspaceFlowNode> = (_event, node) => {
    setSelectedNodeId(node.id)
    setContextMenu(null)
    setActionError(null)
  }

  const handleNodeContextMenu: NodeMouseHandler<WorkspaceFlowNode> = (
    event,
    node,
  ) => {
    event.preventDefault()
    setSelectedNodeId(node.id)
    setActionError(null)
    setContextMenu({
      nodeId: node.id,
      x: Math.min(event.clientX, window.innerWidth - 230),
      y: Math.min(event.clientY, window.innerHeight - 150),
    })
  }

  function getContextMenuItems(node: WorkspaceFlowNode): NodeContextMenuItem[] {
    if (node.data.kind === 'project') {
      return [
        {
          label: 'Atualizar dados Git',
          onSelect: () => {
            closeContextMenu()
            void handleRefresh()
          },
          disabled: loading,
        },
      ]
    }

    if (node.data.kind === 'branch') {
      const { branch } = node.data
      return [
        ...(!branch.current ? [{
          label: 'Switch branch',
          onSelect: () => {
            closeContextMenu()
            void handleSwitchBranch(branch)
          },
          disabled: busyAction !== null,
        }] : []),
        {
          label: 'Nova branch a partir daqui…',
          onSelect: closeContextMenu,
        },
      ]
    }

    const { commit } = node.data
    return [
      {
        label: 'View diff',
        onSelect: () => void handleViewDiff(commit),
        disabled: diffLoading,
      },
      {
        label: 'Nova branch a partir daqui…',
        onSelect: closeContextMenu,
      },
    ]
  }

  const contextMenuItems = contextNode ? getContextMenuItems(contextNode) : []
  const visibleError = actionError ?? loadingError
  const selectedBranch = selectedNode?.data.kind === 'branch'
    ? selectedNode.data.branch
    : null
  const selectedCommit = selectedNode?.data.kind === 'commit'
    ? selectedNode.data.commit
    : null

  return (
    <main className="workspace-page">
      <header className="workspace-header">
        <div className="workspace-header__brand">
          <span className="workspace-header__signal" />
          <span>DEWRENCH</span>
        </div>
        <div className="workspace-header__project" title={project.path}>
          <span>PROJETO</span>
          <strong>
            {project.name}
            {repositoryDetails ? ` / ${repositoryDetails.branch}` : ''}
          </strong>
        </div>
        <button
          className="workspace-header__action nodrag nopan"
          type="button"
          onClick={onOpenAnotherProject}
        >
          Voltar / Abrir outro projeto
        </button>
      </header>

      <section className="workspace-canvas" aria-label={`Workspace de ${project.name}`}>
        <WorkspaceCanvas
          key={layoutVersion}
          initialNodes={positionedGraph.nodes}
          edges={positionedGraph.edges}
          onNodeClick={handleNodeClick}
          onNodeContextMenu={handleNodeContextMenu}
          onPaneClick={() => {
            setSelectedNodeId(null)
            closeContextMenu()
          }}
          onMoveStart={closeContextMenu}
        />
      </section>

      <div className="workspace-legend" aria-hidden="true">
        <span className="workspace-legend__project" /> PROJECT
        <span className="workspace-legend__commit" /> COMMIT
        <span className="workspace-legend__branch" /> BRANCH
      </div>

      {(loading || busyAction) && (
        <div className="workspace-activity" role="status">
          {busyAction ?? 'Carregando Git…'}
        </div>
      )}

      {visibleError && !selectedNode && (
        <p className="workspace-error">{visibleError}</p>
      )}

      {selectedNode?.data.kind === 'project' && (
        <ProjectInspector
          project={selectedNode.data.project}
          details={repositoryDetails}
          loading={loading}
          busy={busyAction !== null}
          error={visibleError}
          onClose={() => setSelectedNodeId(null)}
          onRefresh={() => void handleRefresh()}
          onStage={handleStage}
          onUnstage={handleUnstage}
          onCommit={handleCommit}
        />
      )}

      {selectedBranch && (
        <BranchInspector
          branch={selectedBranch}
          busy={busyAction !== null}
          error={visibleError}
          onClose={() => setSelectedNodeId(null)}
          onSwitch={() => handleSwitchBranch(selectedBranch)}
          onCreateBranch={(name) => (
            handleCreateBranch(selectedBranch.name, name)
          )}
        />
      )}

      {selectedCommit && (
        <CommitInspector
          commit={selectedCommit}
          diff={commitDiff?.commitHash === selectedCommit.hash
            ? commitDiff.value
            : null}
          diffLoading={diffLoading}
          busy={busyAction !== null}
          error={visibleError}
          onClose={() => setSelectedNodeId(null)}
          onViewDiff={() => void handleViewDiff(selectedCommit)}
          onCreateBranch={(name) => (
            handleCreateBranch(selectedCommit.hash, name)
          )}
        />
      )}

      {contextMenu && contextMenuItems.length > 0 && (
        <NodeContextMenu
          x={contextMenu.x}
          y={contextMenu.y}
          items={contextMenuItems}
          onClose={closeContextMenu}
        />
      )}
    </main>
  )
}
