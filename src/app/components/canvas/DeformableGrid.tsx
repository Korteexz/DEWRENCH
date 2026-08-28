import {
  forwardRef,
  useEffect,
  useImperativeHandle,
  useRef,
} from 'react'
import type { Viewport } from '@xyflow/react'

interface GridPoint {
  restX: number
  restY: number
  x: number
  y: number
  vx: number
  vy: number
}

interface InteractionPoint {
  x: number
  y: number
  vx: number
  vy: number
  active: boolean
}

export interface DeformableGridHandle {
  disturb: (
    clientX: number,
    clientY: number,
    velocityX: number,
    velocityY: number,
  ) => void
  release: () => void
  setViewport: (viewport: Viewport) => void
}

// These are deliberately kept together: changing the fabric response should not
// require searching through rendering code. Values are pixels / animation frame.
const GRID_PHYSICS = {
  spacing: 28,
  influenceRadius: 148,
  spring: 0.052,
  damping: 0.84,
  push: 1.55,
  velocityTransfer: 0.17,
  wireThreshold: 1.6,
  settleThreshold: 0.025,
} as const

/**
 * Canvas owns its animation state so a drag can update hundreds of grid points
 * without causing React renders. React Flow only passes pointer impulses and its
 * viewport transform through the imperative handle above.
 */
const DeformableGrid = forwardRef<DeformableGridHandle>(function DeformableGrid(
  _props,
  forwardedRef,
) {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const pointsRef = useRef<GridPoint[]>([])
  const dimensionsRef = useRef({ width: 0, height: 0, columns: 0, rows: 0 })
  const viewportRef = useRef<Viewport>({ x: 0, y: 0, zoom: 1 })
  const interactionRef = useRef<InteractionPoint>({
    x: 0,
    y: 0,
    vx: 0,
    vy: 0,
    active: false,
  })
  const frameRef = useRef<number | null>(null)
  const reducedMotionRef = useRef(false)
  const drawFrameRef = useRef<() => void>(() => undefined)
  const rebuildRef = useRef<() => void>(() => undefined)

  function wake() {
    if (frameRef.current === null) {
      frameRef.current = window.requestAnimationFrame(() => drawFrameRef.current())
    }
  }

  useImperativeHandle(forwardedRef, () => ({
    disturb(clientX, clientY, velocityX, velocityY) {
      if (reducedMotionRef.current) {
        return
      }

      const canvas = canvasRef.current
      if (!canvas) {
        return
      }

      const bounds = canvas.getBoundingClientRect()
      interactionRef.current = {
        x: clientX - bounds.left,
        y: clientY - bounds.top,
        vx: velocityX,
        vy: velocityY,
        active: true,
      }
      wake()
    },
    release() {
      interactionRef.current.active = false
      wake()
    },
    setViewport(viewport) {
      const previous = viewportRef.current
      viewportRef.current = viewport

      // Rebuild only after a visible transform change; doing this outside React
      // keeps panning smooth while the background remains spatially anchored.
      if (
        Math.abs(previous.x - viewport.x) > 0.5
        || Math.abs(previous.y - viewport.y) > 0.5
        || Math.abs(previous.zoom - viewport.zoom) > 0.005
      ) {
        rebuildRef.current()
      }
    },
  }))

  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas) {
      return
    }

    const context = canvas.getContext('2d')
    if (!context) {
      return
    }

    // Explicit non-null aliases remain narrowed inside the animation closures.
    const surface: HTMLCanvasElement = canvas
    const drawingContext: CanvasRenderingContext2D = context

    const motionQuery = window.matchMedia('(prefers-reduced-motion: reduce)')
    reducedMotionRef.current = motionQuery.matches

    function rebuild() {
      const bounds = surface.getBoundingClientRect()
      const width = Math.max(1, bounds.width)
      const height = Math.max(1, bounds.height)
      const dpr = Math.min(window.devicePixelRatio || 1, 2)
      surface.width = Math.round(width * dpr)
      surface.height = Math.round(height * dpr)
      surface.style.width = `${width}px`
      surface.style.height = `${height}px`
      drawingContext.setTransform(dpr, 0, 0, dpr, 0, 0)

      const scaledSpacing = Math.max(
        20,
        Math.min(42, GRID_PHYSICS.spacing * viewportRef.current.zoom),
      )
      const xOffset = ((viewportRef.current.x % scaledSpacing) + scaledSpacing)
        % scaledSpacing
      const yOffset = ((viewportRef.current.y % scaledSpacing) + scaledSpacing)
        % scaledSpacing
      const columns = Math.ceil((width - xOffset) / scaledSpacing) + 2
      const rows = Math.ceil((height - yOffset) / scaledSpacing) + 2
      const points: GridPoint[] = []

      for (let row = -1; row < rows - 1; row += 1) {
        for (let column = -1; column < columns - 1; column += 1) {
          const restX = xOffset + column * scaledSpacing
          const restY = yOffset + row * scaledSpacing
          points.push({ restX, restY, x: restX, y: restY, vx: 0, vy: 0 })
        }
      }

      dimensionsRef.current = { width, height, columns, rows }
      pointsRef.current = points
      wake()
    }

    function pointDisplacement(point: GridPoint) {
      return Math.hypot(point.x - point.restX, point.y - point.restY)
    }

    function drawWireframe(interaction: InteractionPoint) {
      const { columns, rows } = dimensionsRef.current
      const points = pointsRef.current

      drawingContext.lineWidth = 0.7
      for (let row = 0; row < rows; row += 1) {
        for (let column = 0; column < columns; column += 1) {
          const index = row * columns + column
          const point = points[index]
          if (!point) {
            continue
          }

          const localDisplacement = pointDisplacement(point)
          const proximity = interaction.active
            ? Math.max(0, 1 - Math.hypot(
              point.x - interaction.x,
              point.y - interaction.y,
            ) / (GRID_PHYSICS.influenceRadius * 0.92))
            : 0
          const reveal = Math.max(
            proximity * 0.34,
            Math.min(0.46, localDisplacement / 28),
          )

          if (reveal < 0.07 || localDisplacement < GRID_PHYSICS.wireThreshold) {
            continue
          }

          drawingContext.strokeStyle = `rgba(111, 223, 174, ${reveal})`
          const neighbours = [points[index + 1], points[index + columns]]
          for (const neighbour of neighbours) {
            if (!neighbour) {
              continue
            }
            drawingContext.beginPath()
            drawingContext.moveTo(point.x, point.y)
            drawingContext.lineTo(neighbour.x, neighbour.y)
            drawingContext.stroke()
          }
        }
      }
    }

    function drawFrame() {
      frameRef.current = null
      const { width, height } = dimensionsRef.current
      const interaction = interactionRef.current
      let moving = interaction.active

      drawingContext.clearRect(0, 0, width, height)

      for (const point of pointsRef.current) {
        if (interaction.active && !reducedMotionRef.current) {
          const dx = point.x - interaction.x
          const dy = point.y - interaction.y
          const distance = Math.max(1, Math.hypot(dx, dy))

          if (distance < GRID_PHYSICS.influenceRadius) {
            const falloff = (1 - distance / GRID_PHYSICS.influenceRadius) ** 2
            point.vx += (
              (dx / distance) * GRID_PHYSICS.push
              + interaction.vx * GRID_PHYSICS.velocityTransfer
            ) * falloff
            point.vy += (
              (dy / distance) * GRID_PHYSICS.push
              + interaction.vy * GRID_PHYSICS.velocityTransfer
            ) * falloff
          }
        }

        point.vx += (point.restX - point.x) * GRID_PHYSICS.spring
        point.vy += (point.restY - point.y) * GRID_PHYSICS.spring
        point.vx *= GRID_PHYSICS.damping
        point.vy *= GRID_PHYSICS.damping
        point.x += point.vx
        point.y += point.vy

        if (
          Math.abs(point.vx) > GRID_PHYSICS.settleThreshold
          || Math.abs(point.vy) > GRID_PHYSICS.settleThreshold
          || pointDisplacement(point) > 0.08
        ) {
          moving = true
        }
      }

      drawWireframe(interaction)

      for (const point of pointsRef.current) {
        const displacement = pointDisplacement(point)
        const radius = 0.95 + Math.min(0.8, displacement * 0.035)
        const opacity = 0.34 + Math.min(0.46, displacement * 0.024)
        drawingContext.fillStyle = `rgba(144, 158, 148, ${opacity})`
        drawingContext.beginPath()
        drawingContext.arc(point.x, point.y, radius, 0, Math.PI * 2)
        drawingContext.fill()
      }

      if (moving && !document.hidden) {
        frameRef.current = window.requestAnimationFrame(drawFrame)
      }
    }

    function handleMotionPreference(event: MediaQueryListEvent) {
      reducedMotionRef.current = event.matches
      if (event.matches) {
        interactionRef.current.active = false
        rebuild()
      }
    }

    rebuildRef.current = rebuild
    drawFrameRef.current = drawFrame
    const resizeObserver = new ResizeObserver(rebuild)
    resizeObserver.observe(surface)
    motionQuery.addEventListener('change', handleMotionPreference)
    rebuild()

    return () => {
      resizeObserver.disconnect()
      motionQuery.removeEventListener('change', handleMotionPreference)
      if (frameRef.current !== null) {
        window.cancelAnimationFrame(frameRef.current)
      }
      frameRef.current = null
    }
  }, [])

  return <canvas ref={canvasRef} className="deformable-grid" aria-hidden="true" />
})

export default DeformableGrid
