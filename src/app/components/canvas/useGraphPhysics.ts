import type { Edge, Node, XYPosition } from '@xyflow/react'
import {
  useCallback,
  useEffect,
  useRef,
  type Dispatch,
  type SetStateAction,
} from 'react'

import {
  GraphPhysicsEngine,
  type PhysicsPoint,
} from '../../graph/physics/GraphPhysicsEngine'

interface GraphPhysicsControls {
  beginDrag: (nodeId: string, position: XYPosition) => void
  updateDrag: (nodeId: string, position: XYPosition) => void
  endDrag: (nodeId: string, position: XYPosition) => void
}

/** Bridge the generic physics engine to React Flow's controlled node state. */
export function useGraphPhysics<GraphNode extends Node>(
  initialNodes: GraphNode[],
  edges: Edge[],
  setNodes: Dispatch<SetStateAction<GraphNode[]>>,
): GraphPhysicsControls {
  const engineRef = useRef(new GraphPhysicsEngine())
  const animationFrameRef = useRef(0)
  const previousFrameTimeRef = useRef<number | null>(null)

  const runFrame = useCallback(function animate(timestamp: number) {
    const previousTimestamp = previousFrameTimeRef.current ?? timestamp
    previousFrameTimeRef.current = timestamp
    const result = engineRef.current.step(timestamp - previousTimestamp)

    setNodes((currentNodes) => currentNodes.map((node) => {
      const position = result.positions.get(node.id)
      if (!position) {
        return node
      }

      const moved = Math.abs(node.position.x - position.x) > 0.001
        || Math.abs(node.position.y - position.y) > 0.001
      return moved ? { ...node, position } : node
    }))

    if (result.active) {
      animationFrameRef.current = window.requestAnimationFrame(animate)
    } else {
      animationFrameRef.current = 0
      previousFrameTimeRef.current = null
    }
  }, [setNodes])

  const startAnimation = useCallback(() => {
    if (animationFrameRef.current !== 0) {
      return
    }

    previousFrameTimeRef.current = null
    animationFrameRef.current = window.requestAnimationFrame(runFrame)
  }, [runFrame])

  useEffect(() => {
    const engine = engineRef.current
    engine.reset(
      initialNodes.map((node) => ({
        id: node.id,
        x: node.position.x,
        y: node.position.y,
        movable: true,
      })),
      edges.map((edge) => ({ source: edge.source, target: edge.target })),
    )

    return () => {
      window.cancelAnimationFrame(animationFrameRef.current)
      animationFrameRef.current = 0
      previousFrameTimeRef.current = null
    }
  }, [edges, initialNodes])

  const beginDrag = useCallback((nodeId: string, position: PhysicsPoint) => {
    engineRef.current.beginDrag(nodeId, position)
    startAnimation()
  }, [startAnimation])

  const updateDrag = useCallback((nodeId: string, position: PhysicsPoint) => {
    engineRef.current.updateDrag(nodeId, position)
    startAnimation()
  }, [startAnimation])

  const endDrag = useCallback((nodeId: string, position: PhysicsPoint) => {
    engineRef.current.endDrag(nodeId, position)
    startAnimation()
  }, [startAnimation])

  return { beginDrag, updateDrag, endDrag }
}
