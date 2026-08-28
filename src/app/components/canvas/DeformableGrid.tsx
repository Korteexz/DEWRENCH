import {
  forwardRef,
  useEffect,
  useImperativeHandle,
  useRef,
} from 'react'
import type { Viewport } from '@xyflow/react'

import { deformableGridConfig as gridConfig } from './deformableGridConfig'

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
  const releasedAtRef = useRef<number | null>(null)
  const reducedMotionRef = useRef(false)
  const drawFrameRef = useRef<(timestamp: number) => void>(() => undefined)
  const rebuildRef = useRef<() => void>(() => undefined)

  function wake() {
    if (frameRef.current === null) {
      frameRef.current = window.requestAnimationFrame((timestamp) => (
        drawFrameRef.current(timestamp)
      ))
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
      releasedAtRef.current = null
      wake()
    },
    release() {
      interactionRef.current.active = false
      releasedAtRef.current = performance.now()
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
    const resizeTarget = surface.parentElement ?? surface

    const motionQuery = window.matchMedia('(prefers-reduced-motion: reduce)')
    reducedMotionRef.current = motionQuery.matches

    function rebuild() {
      const bounds = resizeTarget.getBoundingClientRect()
      const width = Math.max(1, bounds.width)
      const height = Math.max(1, bounds.height)
      const dpr = Math.min(window.devicePixelRatio || 1, 2)
      surface.width = Math.round(width * dpr)
      surface.height = Math.round(height * dpr)
      drawingContext.setTransform(dpr, 0, 0, dpr, 0, 0)

      const scaledSpacing = Math.max(
        20,
        Math.min(42, gridConfig.gridSpacing * viewportRef.current.zoom),
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
      releasedAtRef.current = null
      wake()
    }

    function pointDisplacement(point: GridPoint) {
      return Math.hypot(point.x - point.restX, point.y - point.restY)
    }

    function drawLocalOcclusion(
      interaction: InteractionPoint,
      maximumDisplacement: number,
    ) {
      const radius = gridConfig.influenceRadius * 1.15
      const strength = interaction.active
        ? 1
        : Math.min(1, maximumDisplacement / 4)
      const occlusion = drawingContext.createRadialGradient(
        interaction.x,
        interaction.y,
        radius * 0.16,
        interaction.x,
        interaction.y,
        radius,
      )

      // Locally soften the static tile before drawing displaced points. The
      // gradient reaches zero opacity, so it can never limit base grid coverage.
      occlusion.addColorStop(0, `rgba(9, 12, 10, ${0.9 * strength})`)
      occlusion.addColorStop(0.62, `rgba(9, 12, 10, ${0.52 * strength})`)
      occlusion.addColorStop(1, 'rgba(9, 12, 10, 0)')
      drawingContext.fillStyle = occlusion
      drawingContext.fillRect(
        interaction.x - radius,
        interaction.y - radius,
        radius * 2,
        radius * 2,
      )
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
            ) / (gridConfig.influenceRadius * 0.92))
            : 0
          const tension = Math.max(
            0,
            localDisplacement - gridConfig.wireframeRevealThreshold,
          )
          const reveal = Math.min(
            0.42,
            proximity * 0.11 + tension / gridConfig.maxDisplacement * 0.42,
          )

          if (
            reveal < 0.055
            || localDisplacement < gridConfig.wireframeRevealThreshold
          ) {
            continue
          }

          drawingContext.strokeStyle = `rgba(111, 223, 174, ${reveal})`
          const neighbours = [
            column + 1 < columns ? points[index + 1] : undefined,
            row + 1 < rows ? points[index + columns] : undefined,
          ]
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

    function drawFrame(timestamp: number) {
      frameRef.current = null
      const { width, height } = dimensionsRef.current
      const interaction = interactionRef.current
      const forceRest = !interaction.active
        && releasedAtRef.current !== null
        && timestamp - releasedAtRef.current >= gridConfig.maxSettleDurationMs
      let moving = interaction.active
      let maximumDisplacement = 0

      drawingContext.clearRect(0, 0, width, height)

      for (const point of pointsRef.current) {
        if (forceRest) {
          point.x = point.restX
          point.y = point.restY
          point.vx = 0
          point.vy = 0
          continue
        }

        if (interaction.active && !reducedMotionRef.current) {
          const dx = point.x - interaction.x
          const dy = point.y - interaction.y
          const distance = Math.max(1, Math.hypot(dx, dy))

          if (distance < gridConfig.influenceRadius) {
            const falloff = (1 - distance / gridConfig.influenceRadius) ** 2
            point.vx += (
              (dx / distance) * gridConfig.displacementStrength
              + interaction.vx * gridConfig.pointerVelocityTransfer
            ) * falloff
            point.vy += (
              (dy / distance) * gridConfig.displacementStrength
              + interaction.vy * gridConfig.pointerVelocityTransfer
            ) * falloff
          }
        }

        point.vx += (point.restX - point.x) * gridConfig.springStrength
        point.vy += (point.restY - point.y) * gridConfig.springStrength
        point.vx *= gridConfig.damping
        point.vy *= gridConfig.damping

        const speed = Math.hypot(point.vx, point.vy)
        if (speed > gridConfig.maxVelocity) {
          const scale = gridConfig.maxVelocity / speed
          point.vx *= scale
          point.vy *= scale
        }

        point.x += point.vx
        point.y += point.vy

        let displacement = pointDisplacement(point)
        if (displacement > gridConfig.maxDisplacement) {
          const scale = gridConfig.maxDisplacement / displacement
          point.x = point.restX + (point.x - point.restX) * scale
          point.y = point.restY + (point.y - point.restY) * scale
          point.vx *= 0.42
          point.vy *= 0.42
          displacement = gridConfig.maxDisplacement
        }

        maximumDisplacement = Math.max(maximumDisplacement, displacement)

        if (
          Math.abs(point.vx) > gridConfig.settleVelocity
          || Math.abs(point.vy) > gridConfig.settleVelocity
          || displacement > gridConfig.settleDisplacement
        ) {
          moving = true
        }
      }

      if (moving) {
        drawLocalOcclusion(interaction, maximumDisplacement)
        drawWireframe(interaction)
      }

      for (const point of pointsRef.current) {
        const displacement = pointDisplacement(point)
        if (displacement < gridConfig.settleDisplacement) {
          continue
        }

        const radius = 0.95 + Math.min(0.8, displacement * 0.035)
        const opacity = 0.22 + Math.min(0.58, displacement * 0.032)
        drawingContext.fillStyle = `rgba(144, 158, 148, ${opacity})`
        drawingContext.beginPath()
        drawingContext.arc(point.x, point.y, radius, 0, Math.PI * 2)
        drawingContext.fill()
      }

      if (moving && !document.hidden) {
        frameRef.current = window.requestAnimationFrame(drawFrame)
      } else if (!interaction.active) {
        releasedAtRef.current = null
      }
    }

    function handleMotionPreference(event: MediaQueryListEvent) {
      reducedMotionRef.current = event.matches
      if (event.matches) {
        interactionRef.current.active = false
        releasedAtRef.current = null
        rebuild()
      }
    }

    function handleVisibilityChange() {
      if (document.hidden) {
        interactionRef.current.active = false
        rebuild()
      }
    }

    rebuildRef.current = rebuild
    drawFrameRef.current = drawFrame
    const resizeObserver = new ResizeObserver(rebuild)
    resizeObserver.observe(resizeTarget)
    motionQuery.addEventListener('change', handleMotionPreference)
    document.addEventListener('visibilitychange', handleVisibilityChange)
    rebuild()

    return () => {
      resizeObserver.disconnect()
      motionQuery.removeEventListener('change', handleMotionPreference)
      document.removeEventListener('visibilitychange', handleVisibilityChange)
      if (frameRef.current !== null) {
        window.cancelAnimationFrame(frameRef.current)
      }
      frameRef.current = null
    }
  }, [])

  return <canvas ref={canvasRef} className="deformable-grid" aria-hidden="true" />
})

export default DeformableGrid
