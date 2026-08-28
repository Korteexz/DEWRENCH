import type { NodeMouseHandler } from '@xyflow/react'

import WorkspaceCanvas from '../../../app/components/canvas/WorkspaceCanvas'
import type { WorkspaceFlowEdge, WorkspaceFlowNode } from '../../../app/graph/types'

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
}

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
}: GitGraphViewportProps) {
  return (
    <section className="git-graph-viewport" aria-label={`Grafo Git de ${projectName}`}>
      <div className="graph-instrument graph-instrument--top">
        <span>02 / TOPOLOGY SURFACE</span>
        <strong>{branchName ?? 'DETACHED'}</strong>
        <code>{initialNodes.length.toString().padStart(3, '0')} NODES</code>
      </div>

      <WorkspaceCanvas
        initialNodes={initialNodes}
        edges={edges}
        selectedNodeId={selectedNodeId}
        onNodeClick={onNodeClick}
        onNodeContextMenu={onNodeContextMenu}
        onPaneClick={onPaneClick}
        onMoveStart={onMoveStart}
      />

      <div className="graph-axis graph-axis--x" aria-hidden="true" />
      <div className="graph-axis graph-axis--y" aria-hidden="true" />

      <div className="graph-legend" aria-hidden="true">
        <span><i className="graph-legend__project" />PROJECT</span>
        <span><i className="graph-legend__commit" />COMMIT</span>
        <span><i className="graph-legend__branch" />BRANCH</span>
      </div>

      {(loading || activity) && (
        <div className="graph-activity" role="status">
          <span />{activity ?? 'READING REPOSITORY'}
        </div>
      )}

      {error && <p className="graph-error">ERR / {error}</p>}
    </section>
  )
}
