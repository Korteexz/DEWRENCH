import { useCallback, useMemo, useState } from 'react'
import type { NodeMouseHandler } from '@xyflow/react'

import AppShell from '../shell/AppShell'
import { SplitDeck } from '../../design'
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
import GitSystemReadout from '../../modules/git/components/GitSystemReadout'
import { useGitGraph } from '../../modules/git/hooks/useGitGraph'
import {
  createBranchFrom,
  createCommit,
  getCommitDiff,
  getRevertPreview,
  revertCommit,
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
import type {
  GitFailure,
  GitRevertOutcome,
  GitRevertPreview,
} from '../../modules/git/types/revert'

// Folha do deck do Git. Fica aqui enquanto esta página for o container do
// módulo; ela acompanha o container quando ele migrar para modules/git.
import '../../modules/git/git-workspace.css'
import {
  describeFailure,
  toGitFailure,
} from '../../modules/git/types/revert'

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

/** Estado do fluxo de Revert, sempre associado a um commit específico. */
interface RevertState {
  commitHash: string
  preview: GitRevertPreview | null
  loading: boolean
  failure: GitFailure | null
  outcome: GitRevertOutcome | null
}

function getErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message
  }

  // Commands antigos rejeitam com string; os novos, com erro tipado.
  return describeFailure(toGitFailure(error))
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
  const [revert, setRevert] = useState<RevertState | null>(null)

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

  /** Preview read-only: não muta o repositório e não confirma nada sozinho. */
  async function handleRequestRevertPreview(commit: GitGraphCommit): Promise<void> {
    if (busyAction || revert?.loading) {
      return
    }

    setRevert({
      commitHash: commit.hash,
      preview: null,
      loading: true,
      failure: null,
      outcome: null,
    })

    try {
      const preview = await getRevertPreview(project.path, commit.hash)
      setRevert({
        commitHash: commit.hash,
        preview,
        loading: false,
        failure: null,
        outcome: null,
      })
    } catch (error) {
      setRevert({
        commitHash: commit.hash,
        preview: null,
        loading: false,
        failure: toGitFailure(error),
        outcome: null,
      })
    }
  }

  function handleCancelRevert(): void {
    setRevert(null)
  }

  /**
   * Executa o Revert reutilizando executeMutation: busyAction impede clique
   * duplo e o refresh relê detalhes e grafo a partir do backend.
   *
   * A rejeição não é propagada porque o erro tipado é apresentado pelo
   * RevertPanel; propagá-la duplicaria a mensagem no erro genérico da tela.
   */
  async function handleConfirmRevert(commit: GitGraphCommit): Promise<boolean> {
    let succeeded = false

    await executeMutation('Revert commit', async () => {
      try {
        const outcome = await revertCommit(project.path, commit.hash)
        setRevert({
          commitHash: commit.hash,
          preview: null,
          loading: false,
          failure: null,
          outcome,
        })
        succeeded = true
      } catch (error) {
        setRevert({
          commitHash: commit.hash,
          preview: null,
          loading: false,
          failure: toGitFailure(error),
          outcome: null,
        })
      }
    })

    return succeeded
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

  function revertFor(commitHash: string): RevertState | null {
    return revert?.commitHash === commitHash ? revert : null
  }

  const inspector = selectedNode?.data.kind === 'project' ? (
    <ProjectInspector
      project={selectedNode.data.project}
      details={repositoryDetails}
      loading={loading}
      busy={busyAction !== null}
      error={visibleError}
      onClose={() => setSelectedNodeId(null)}
      onRefresh={() => void handleRefresh()}
      onStage={handleStage}
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
      revertPreview={revertFor(selectedCommit.hash)?.preview ?? null}
      revertLoading={revertFor(selectedCommit.hash)?.loading ?? false}
      revertFailure={revertFor(selectedCommit.hash)?.failure ?? null}
      revertOutcome={revertFor(selectedCommit.hash)?.outcome ?? null}
      onRequestRevertPreview={() => void handleRequestRevertPreview(selectedCommit)}
      onCancelRevert={handleCancelRevert}
      onConfirmRevert={() => void handleConfirmRevert(selectedCommit)}
    />
  ) : null

  return (
    <AppShell
      projectName={project.name}
      projectPath={project.path}
      activeModule="git"
      systemReadout={(
        <GitSystemReadout
          details={repositoryDetails}
          linked={gitGraph !== null}
          loading={loading}
          activity={busyAction}
        />
      )}
      onOpenAnotherProject={onOpenAnotherProject}
    >
      <SplitDeck
        id="git-workspace"
        className="git-workspace"
        leftLabel="Redimensionar índice do repositório"
        rightLabel="Redimensionar inspetor"
        left={(
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
        )}
        center={(
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
        )}
        right={(
          <GitInspectorPane
            project={project}
            details={repositoryDetails}
            graph={gitGraph}
          >
            {inspector}
          </GitInspectorPane>
        )}
      />

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
