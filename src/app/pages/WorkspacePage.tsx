import { useCallback, useMemo, useState } from 'react'
import type { NodeMouseHandler } from '@xyflow/react'

import AppShell from '../components/shell/AppShell'
import BranchInspector from '../components/canvas/inspectors/BranchInspector'
import CommitInspector from '../components/canvas/inspectors/CommitInspector'
import ProjectInspector from '../components/canvas/inspectors/ProjectInspector'
import NodeContextMenu, {
  type NodeContextMenuItem,
} from '../components/canvas/menus/NodeContextMenu'
import { layoutWorkspaceGraph } from '../graph/layout'
import { commitNodeId, type WorkspaceFlowNode } from '../graph/types'
import { adaptGitGraph } from '../../modules/git/adapters/gitGraphAdapter'
import GitGraphViewport from '../../modules/git/components/GitGraphViewport'
import GitInspectorPane from '../../modules/git/components/GitInspectorPane'
import GitSidebar from '../../modules/git/components/GitSidebar'
import { useGitGraph } from '../../modules/git/hooks/useGitGraph'
import {
  createBranchFrom,
  createCommit,
  getCommitDiff,
  stageAll,
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
  function handleStageAll(): Promise<boolean> {
  return executeMutation(
    'Stage all',
    () => stageAll(project.path),
  )
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

  const inspector = selectedNode?.data.kind === 'project' ? (
    <ProjectInspector
      project={selectedNode.data.project}
      details={repositoryDetails}
      loading={loading}
      busy={busyAction !== null}
      error={visibleError}
      onClose={() => setSelectedNodeId(null)}
      onRefresh={() => void handleRefresh()}
      onStage={handleUnstage}
      onStageAll={handleStageAll}
      onUnstage={handleUnstage}
      onCommit={handleCommit}
    />
  ) : selectedBranch ? (
    <BranchInspector
      branch={selectedBranch}
      busy={busyAction !== null}
      error={visibleError}
      onClose={() => setSelectedNodeId(null)}
      onSwitch={() => handleSwitchBranch(selectedBranch)}
      onCreateBranch={(name) => handleCreateBranch(selectedBranch.name, name)}
    />
  ) : selectedCommit ? (
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
      onCreateBranch={(name) => handleCreateBranch(selectedCommit.hash, name)}
    />
  ) : null

  return (
    <AppShell
      project={project}
      branch={repositoryDetails?.branch ?? null}
      connected={gitGraph !== null}
      onOpenAnotherProject={onOpenAnotherProject}
    >
      <div className="git-workspace">
        <GitSidebar
          project={project}
          details={repositoryDetails}
          graph={gitGraph}
          selectedNodeId={selectedNodeId}
          loading={loading}
          onSelectNode={(nodeId) => {
            setSelectedNodeId(nodeId)
            closeContextMenu()
            setActionError(null)
          }}
          onRefresh={() => void handleRefresh()}
        />

        <GitGraphViewport
          key={layoutVersion}
          projectName={project.name}
          branchName={repositoryDetails?.branch ?? null}
          initialNodes={positionedGraph.nodes}
          edges={positionedGraph.edges}
          selectedNodeId={selectedNodeId}
          loading={loading}
          activity={busyAction}
          error={!selectedNode ? visibleError : null}
          onNodeClick={handleNodeClick}
          onNodeContextMenu={handleNodeContextMenu}
          onPaneClick={() => {
            setSelectedNodeId(null)
            closeContextMenu()
          }}
          onMoveStart={closeContextMenu}
        />

        <GitInspectorPane project={project} details={repositoryDetails}>
          {inspector}
        </GitInspectorPane>
      </div>

      {contextMenu && contextMenuItems.length > 0 && (
        <NodeContextMenu
          x={contextMenu.x}
          y={contextMenu.y}
          items={contextMenuItems}
          onClose={closeContextMenu}
        />
      )}
    </AppShell>
  )
}
