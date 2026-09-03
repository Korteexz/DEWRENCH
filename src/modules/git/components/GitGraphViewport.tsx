import { useCallback, useRef, type ReactNode } from 'react'
import type { NodeMouseHandler, Viewport } from '@xyflow/react'

import WorkspaceCanvas from '../../../app/components/canvas/WorkspaceCanvas'
import type { WorkspaceFlowEdge, WorkspaceFlowNode } from '../../../app/graph/types'
import {
  InstrumentFrame,
  StatusIndicator,
  TechnicalLabel,
} from '../../../design'
import GraphViewportReadout, {
  type GraphViewportReadoutHandle,
} from './GraphViewportReadout'

interface GitGraphViewportProps {
  projectName: string
  branchName: string | null
  initialNodes: WorkspaceFlowNode[]
  edges: WorkspaceFlowEdge[]
  selectedNodeId: string | null
  loading: boolean
  activity: string | null
  error: string | null
  onNodeClick: NodeMouseHandler<WorkspaceFlowNode>
  onNodeContextMenu: NodeMouseHandler<WorkspaceFlowNode>
  onPaneClick: () => void
  onMoveStart: () => void
  /** Seletor de instrumento do compartimento central. */
  surfaceSwitch?: ReactNode
}

/**
 * Campo observado do módulo Git.
 *
 * Este componente é a fronteira do XYFlow: nenhuma outra parte da aplicação
 * conhece a biblioteca de grafo. Trocá-la deve significar reescrever apenas
 * `WorkspaceCanvas` e este arquivo, sem tocar na instrumentação em volta.
 */
export default function GitGraphViewport({
  projectName,
  branchName,
  initialNodes,
  edges,
  selectedNodeId,
  loading,
  activity,
  error,
  onNodeClick,
  onNodeContextMenu,
  onPaneClick,
  onMoveStart,
  surfaceSwitch,
}: GitGraphViewportProps) {
  const readoutRef = useRef<GraphViewportReadoutHandle>(null)

  const handleViewportChange = useCallback((viewport: Viewport) => {
    readoutRef.current?.update(viewport)
  }, [])

  return (
    <section
      className="git-graph-viewport"
      aria-label={`Topologia Git de ${projectName}`}
    >
      <header className="git-graph-viewport__bar">
        <span className="dw-panel__index">02</span>
        <TechnicalLabel tone="mid">Topology surface</TechnicalLabel>
        {surfaceSwitch}
        <span className="git-graph-viewport__bar-rule" aria-hidden="true" />
        <span className="git-graph-viewport__ref">
          {branchName ?? 'DETACHED HEAD'}
        </span>
        <span className="dw-coord">
          <span><b>N</b>{initialNodes.length.toString().padStart(3, '0')}</span>
          <span><b>E</b>{edges.length.toString().padStart(3, '0')}</span>
        </span>
      </header>

      <div className="git-graph-viewport__field">
        <WorkspaceCanvas
          initialNodes={initialNodes}
          edges={edges}
          selectedNodeId={selectedNodeId}
          onNodeClick={onNodeClick}
          onNodeContextMenu={onNodeContextMenu}
          onPaneClick={onPaneClick}
          onMoveStart={onMoveStart}
          onViewportChange={handleViewportChange}
        />
        <InstrumentFrame />
      </div>

      <footer className="git-graph-viewport__foot">
        <span className="git-graph-viewport__legend" aria-hidden="true">
          <span><i data-mark="project" />PROJECT</span>
          <span><i data-mark="commit" />COMMIT</span>
          <span><i data-mark="merge" />MERGE</span>
          <span><i data-mark="branch" />BRANCH</span>
        </span>

        <GraphViewportReadout ref={readoutRef} />

        <StatusIndicator
          tone={activity ? 'info' : loading ? 'info' : 'nominal'}
          label={activity ?? (loading ? 'READING REPOSITORY' : 'IDLE')}
          live={Boolean(activity) || loading}
        />
      </footer>

      {error && (
        <p className="git-graph-viewport__error" role="status">
          <TechnicalLabel tone="fault" size="micro">FAULT</TechnicalLabel>
          {error}
        </p>
      )}
    </section>
  )
}
