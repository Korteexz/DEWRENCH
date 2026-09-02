import {
  Controls,
  ReactFlow,
  useNodesState,
  type NodeMouseHandler,
  type ReactFlowInstance,
} from '@xyflow/react'
import { useEffect, useMemo, useRef } from 'react'
import '@xyflow/react/dist/style.css'

import ComputationalGrid from './ComputationalGrid'
import DeformableGrid, {
  type DeformableGridHandle,
} from './DeformableGrid'
import { useGraphPhysics } from './useGraphPhysics'
import { useNodeProximity } from './useNodeProximity'
import {
  createGraphFocus,
  getEdgeFocusLevel,
  getNodeFocusLevel,
  withGraphInteractionClasses,
} from '../../graph/focus'
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

const FIT_VIEW_OPTIONS = {
  padding: 0.24,
  minZoom: 0.18,
  maxZoom: 1.05,
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
  const canvasRef = useRef<HTMLDivElement>(null)
  const flowRef = useRef<ReactFlowInstance<WorkspaceFlowNode, WorkspaceFlowEdge>>(null)
  const gridRef = useRef<DeformableGridHandle>(null)
  const previousPointerRef = useRef<{ x: number; y: number } | null>(null)
  const graphPhysics = useGraphPhysics(initialNodes, edges, setNodes)
  const proximityNodeId = useNodeProximity(canvasRef)
  const graphFocus = useMemo(
    () => createGraphFocus(selectedNodeId, edges),
    [edges, selectedNodeId],
  )
  const renderedEdges = useMemo(() => edges.map((edge) => ({
    ...edge,
    className: withGraphInteractionClasses(
      edge.className,
      getEdgeFocusLevel(edge.id, graphFocus),
    ),
  })), [edges, graphFocus])

  useEffect(() => {
    // Mirror sidebar selection plus derived one-hop focus into React Flow state.
    setNodes((currentNodes) => currentNodes.map((node) => ({
      ...node,
      selected: node.id === selectedNodeId,
      className: withGraphInteractionClasses(
        node.className,
        getNodeFocusLevel(node.id, graphFocus),
        node.id === proximityNodeId,
      ),
    })))
  }, [graphFocus, proximityNodeId, selectedNodeId, setNodes])

  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas) {
      return
    }

    let frame = 0
    const fitGraph = () => {
      window.cancelAnimationFrame(frame)
      frame = window.requestAnimationFrame(() => {
        void flowRef.current?.fitView(FIT_VIEW_OPTIONS)
      })
    }
    const resizeObserver = new ResizeObserver(fitGraph)
    resizeObserver.observe(canvas)

    return () => {
      window.cancelAnimationFrame(frame)
      resizeObserver.disconnect()
    }
  }, [])

  return (
    <div ref={canvasRef} className="workspace-canvas">
      <ComputationalGrid />
      <DeformableGrid ref={gridRef} />
      <ReactFlow<WorkspaceFlowNode, WorkspaceFlowEdge>
        nodes={nodes}
        edges={renderedEdges}
        nodeTypes={workspaceNodeTypes}
        onInit={(instance) => {
          flowRef.current = instance
          void instance.fitView(FIT_VIEW_OPTIONS)
        }}
        onNodesChange={onNodesChange}
        onNodeClick={onNodeClick}
        onNodeContextMenu={onNodeContextMenu}
        onNodeDragStart={(event, node) => {
          graphPhysics.beginDrag(node.id, node.position)
          const pointer = getPointerPosition(event)
          if (!pointer) {
            return
          }
          previousPointerRef.current = pointer
          gridRef.current?.disturb(pointer.x, pointer.y, 0, 0)
        }}
        onNodeDrag={(event, node) => {
          graphPhysics.updateDrag(node.id, node.position)
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
        onNodeDragStop={(_event, node) => {
          graphPhysics.endDrag(node.id, node.position)
          previousPointerRef.current = null
          gridRef.current?.release()
        }}
        onPaneClick={onPaneClick}
        onMoveStart={onMoveStart}
        onPaneContextMenu={(event) => event.preventDefault()}
        nodesConnectable={false}
        edgesReconnectable={false}
        deleteKeyCode={null}
        fitView
        fitViewOptions={FIT_VIEW_OPTIONS}
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
