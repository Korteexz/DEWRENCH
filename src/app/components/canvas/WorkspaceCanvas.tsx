import {
  Controls,
  ReactFlow,
  useNodesState,
  type NodeMouseHandler,
} from '@xyflow/react'
import { useEffect, useRef } from 'react'
import '@xyflow/react/dist/style.css'

import ComputationalGrid from './ComputationalGrid'
import DeformableGrid, {
  type DeformableGridHandle,
} from './DeformableGrid'
import { workspaceNodeTypes } from '../../graph/nodeTypes'
import type {
  WorkspaceFlowEdge,
  WorkspaceFlowNode,
} from '../../graph/types'

interface WorkspaceCanvasProps {
  initialNodes: WorkspaceFlowNode[]
  edges: WorkspaceFlowEdge[]
  selectedNodeId: string | null
  onNodeClick: NodeMouseHandler<WorkspaceFlowNode>
  onNodeContextMenu: NodeMouseHandler<WorkspaceFlowNode>
  onPaneClick: () => void
  onMoveStart: () => void
}

function getPointerPosition(event: MouseEvent | TouchEvent) {
  if ('touches' in event) {
    const touch = event.touches[0] ?? event.changedTouches[0]
    return touch ? { x: touch.clientX, y: touch.clientY } : null
  }

  return { x: event.clientX, y: event.clientY }
}

export default function WorkspaceCanvas({
  initialNodes,
  edges,
  selectedNodeId,
  onNodeClick,
  onNodeContextMenu,
  onPaneClick,
  onMoveStart,
}: WorkspaceCanvasProps) {
  const [nodes, setNodes, onNodesChange] = useNodesState<WorkspaceFlowNode>(initialNodes)
  const gridRef = useRef<DeformableGridHandle>(null)
  const previousPointerRef = useRef<{ x: number; y: number } | null>(null)

  useEffect(() => {
    // Sidebar selection is external to React Flow, so mirror it into node state.
    setNodes((currentNodes) => currentNodes.map((node) => ({
      ...node,
      selected: node.id === selectedNodeId,
    })))
  }, [selectedNodeId, setNodes])

  return (
    <div className="workspace-canvas">
      <ComputationalGrid />
      <DeformableGrid ref={gridRef} />
      <ReactFlow<WorkspaceFlowNode, WorkspaceFlowEdge>
        nodes={nodes}
        edges={edges}
        nodeTypes={workspaceNodeTypes}
        onNodesChange={onNodesChange}
        onNodeClick={onNodeClick}
        onNodeContextMenu={onNodeContextMenu}
        onNodeDragStart={(event) => {
          const pointer = getPointerPosition(event)
          if (!pointer) {
            return
          }
          previousPointerRef.current = pointer
          gridRef.current?.disturb(pointer.x, pointer.y, 0, 0)
        }}
        onNodeDrag={(event) => {
          const pointer = getPointerPosition(event)
          if (!pointer) {
            return
          }
          const previous = previousPointerRef.current
          const velocityX = previous ? pointer.x - previous.x : 0
          const velocityY = previous ? pointer.y - previous.y : 0
          gridRef.current?.disturb(
            pointer.x,
            pointer.y,
            velocityX,
            velocityY,
          )
          previousPointerRef.current = pointer
        }}
        onNodeDragStop={() => {
          previousPointerRef.current = null
          gridRef.current?.release()
        }}
        onPaneClick={onPaneClick}
        onMoveStart={onMoveStart}
        onMove={(_event, viewport) => gridRef.current?.setViewport(viewport)}
        onPaneContextMenu={(event) => event.preventDefault()}
        nodesConnectable={false}
        edgesReconnectable={false}
        deleteKeyCode={null}
        fitView
        fitViewOptions={{ padding: 0.24, minZoom: 0.18, maxZoom: 1.05 }}
        minZoom={0.1}
        maxZoom={2.2}
        onlyRenderVisibleElements
        zoomOnDoubleClick={false}
      >
        <Controls showInteractive={false} />
      </ReactFlow>
    </div>
  )
}
