import {
  Background,
  BackgroundVariant,
  Controls,
  ReactFlow,
  useNodesState,
  type NodeMouseHandler,
} from '@xyflow/react'
import '@xyflow/react/dist/style.css'

import { workspaceNodeTypes } from '../../graph/nodeTypes'
import type {
  WorkspaceFlowEdge,
  WorkspaceFlowNode,
} from '../../graph/types'

interface WorkspaceCanvasProps {
  initialNodes: WorkspaceFlowNode[]
  edges: WorkspaceFlowEdge[]
  onNodeClick: NodeMouseHandler<WorkspaceFlowNode>
  onNodeContextMenu: NodeMouseHandler<WorkspaceFlowNode>
  onPaneClick: () => void
  onMoveStart: () => void
}

export default function WorkspaceCanvas({
  initialNodes,
  edges,
  onNodeClick,
  onNodeContextMenu,
  onPaneClick,
  onMoveStart,
}: WorkspaceCanvasProps) {
  const [nodes, , onNodesChange] = useNodesState<WorkspaceFlowNode>(initialNodes)

  return (
    <ReactFlow<WorkspaceFlowNode, WorkspaceFlowEdge>
      nodes={nodes}
      edges={edges}
      nodeTypes={workspaceNodeTypes}
      onNodesChange={onNodesChange}
      onNodeClick={onNodeClick}
      onNodeContextMenu={onNodeContextMenu}
      onPaneClick={onPaneClick}
      onMoveStart={onMoveStart}
      onPaneContextMenu={(event) => event.preventDefault()}
      nodesConnectable={false}
      edgesReconnectable={false}
      deleteKeyCode={null}
      fitView
      fitViewOptions={{ padding: 0.22, minZoom: 0.12, maxZoom: 1.1 }}
      minZoom={0.08}
      maxZoom={2.4}
      onlyRenderVisibleElements
    >
      <Background
        variant={BackgroundVariant.Dots}
        color="var(--canvas-dot)"
        gap={26}
        size={1}
      />
      <Controls showInteractive={false} />
    </ReactFlow>
  )
}
