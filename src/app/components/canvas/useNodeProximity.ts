import {
  useEffect,
  useState,
  type RefObject,
} from 'react'

/** Screen-pixel radius around a commit core that reveals its annotation. */
export const NODE_LABEL_PROXIMITY_RADIUS = 78

/**
 * Resolve only the nearest commit once per animation frame. State changes only
 * when that identity changes, keeping pointer tracking inexpensive and calm.
 */
export function useNodeProximity(
  containerRef: RefObject<HTMLElement | null>,
): string | null {
  const [nearbyNodeId, setNearbyNodeId] = useState<string | null>(null)

  useEffect(() => {
    const container = containerRef.current
    if (!container) {
      return
    }

    let frame = 0
    let pointerX = 0
    let pointerY = 0

    const resolveNearestNode = () => {
      frame = 0
      let closestId: string | null = null
      let closestDistance = NODE_LABEL_PROXIMITY_RADIUS

      for (const node of container.querySelectorAll<HTMLElement>(
        '.react-flow__node-commit',
      )) {
        const bounds = node.getBoundingClientRect()
        const distance = Math.hypot(
          pointerX - (bounds.left + bounds.right) / 2,
          pointerY - (bounds.top + bounds.bottom) / 2,
        )

        if (distance < closestDistance) {
          closestDistance = distance
          closestId = node.dataset.id ?? null
        }
      }

      setNearbyNodeId((currentId) => (
        currentId === closestId ? currentId : closestId
      ))
    }

    const handlePointerMove = (event: PointerEvent) => {
      pointerX = event.clientX
      pointerY = event.clientY
      if (frame === 0) {
        frame = window.requestAnimationFrame(resolveNearestNode)
      }
    }
    const handlePointerLeave = () => {
      window.cancelAnimationFrame(frame)
      frame = 0
      setNearbyNodeId(null)
    }

    container.addEventListener('pointermove', handlePointerMove, { passive: true })
    container.addEventListener('pointerleave', handlePointerLeave)

    return () => {
      window.cancelAnimationFrame(frame)
      container.removeEventListener('pointermove', handlePointerMove)
      container.removeEventListener('pointerleave', handlePointerLeave)
    }
  }, [containerRef])

  return nearbyNodeId
}
