import {
  forwardRef,
  useEffect,
  useImperativeHandle,
  useRef,
} from 'react'

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

interface InfluenceField {
  centerX: number
  centerY: number
  directionX: number
  directionY: number
  speed: number
  speedRatio: number
  majorRadius: number
  minorRadius: number
}

export interface DeformableGridHandle {
  disturb: (
    clientX: number,
    clientY: number,
    velocityX: number,
    velocityY: number,
  ) => void
  release: () => void
}

/**
 * Canvas owns its animation state so a drag can update hundreds of grid points
 * without causing React renders. React Flow only passes pointer impulses and its
 * drag coordinates through the imperative handle above.
 */
const DeformableGrid = forwardRef<DeformableGridHandle>(function DeformableGrid(
  _props,
  forwardedRef,
) {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const pointsRef = useRef<GridPoint[]>([])
  const dimensionsRef = useRef({ width: 0, height: 0, columns: 0, rows: 0 })
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

  function wake() {
    if (frameRef.current === null) {
      frameRef.current = window.requestAnimationFrame((timestamp) => (
        drawFrameRef.current(timestamp)
      ))
    }
  }

  useImperativeHandle(forwardedRef, () => ({
    disturb(clientX, clientY, velocityX, velocityY) {
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
    surface.dataset.reducedMotion = String(motionQuery.matches)

    function rebuild() {
      const bounds = resizeTarget.getBoundingClientRect()
      const width = Math.max(1, bounds.width)
      const height = Math.max(1, bounds.height)
      const dpr = Math.min(window.devicePixelRatio || 1, 2)
      surface.width = Math.round(width * dpr)
      surface.height = Math.round(height * dpr)
      drawingContext.setTransform(dpr, 0, 0, dpr, 0, 0)

      // CSS radial-gradient dots sit at each tile's center. Matching the
      // half-spacing origin lets this Canvas replace those exact dots locally.
      const spacing = gridConfig.gridSpacing
      const origin = spacing / 2
      const columns = Math.ceil(width / spacing) + 2
      const rows = Math.ceil(height / spacing) + 2
      const points: GridPoint[] = []

      for (let row = -1; row < rows - 1; row += 1) {
        for (let column = -1; column < columns - 1; column += 1) {
          const restX = origin + column * spacing
          const restY = origin + row * spacing
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

    function createInfluenceField(interaction: InteractionPoint): InfluenceField {
      const speed = Math.hypot(interaction.vx, interaction.vy)
      const speedRatio = Math.min(1, speed / gridConfig.velocityForFullDirection)
      const directionX = speed > 0.01 ? interaction.vx / speed : 1
      const directionY = speed > 0.01 ? interaction.vy / speed : 0
      const wakeOffset = gridConfig.maxWakeOffset * speedRatio
      const restingRadius = gridConfig.influenceRadius
        * gridConfig.stationaryRadiusScale

      return {
        centerX: interaction.x - directionX * wakeOffset,
        centerY: interaction.y - directionY * wakeOffset,
        directionX,
        directionY,
        speed,
        speedRatio,
        majorRadius: restingRadius
          + gridConfig.influenceRadius
          * gridConfig.directionalRadiusStretch
          * speedRatio,
        minorRadius: restingRadius
          + gridConfig.influenceRadius
          * gridConfig.perpendicularRadiusStretch
          * speedRatio,
      }
    }

    function fieldCoordinates(
      x: number,
      y: number,
      field: InfluenceField,
    ) {
      const dx = x - field.centerX
      const dy = y - field.centerY
      return {
        along: dx * field.directionX + dy * field.directionY,
        across: -dx * field.directionY + dy * field.directionX,
      }
    }

    function normalizedFieldDistance(
      x: number,
      y: number,
      field: InfluenceField,
      expansion = 0,
    ) {
      const { along, across } = fieldCoordinates(x, y, field)
      return Math.hypot(
        along / (field.majorRadius + expansion),
        across / (field.minorRadius + expansion),
      )
    }

    function patchCoverage(point: GridPoint, field: InfluenceField) {
      const innerDistance = normalizedFieldDistance(
        point.restX,
        point.restY,
        field,
      )
      if (innerDistance <= 1) {
        return 1
      }

      const outerDistance = normalizedFieldDistance(
        point.restX,
        point.restY,
        field,
        gridConfig.edgeBlendWidth,
      )
      if (outerDistance >= 1) {
        return 0
      }

      // Convert the two ellipse distances into a direction-independent blend
      // from the force boundary to the expanded rendering boundary.
      const value = (1 - outerDistance) / (innerDistance - outerDistance)
      return value * value * (3 - 2 * value)
    }

    function drawLocalOcclusion(field: InfluenceField) {
      for (const point of pointsRef.current) {
        const coverage = patchCoverage(point, field)
        if (coverage < 0.01) {
          continue
        }

        // Mask only each original CSS dot instead of darkening the surrounding
        // surface. This preserves exact dot replacement without a radial void.
        drawingContext.fillStyle = `rgba(9, 12, 10, ${coverage})`
        drawingContext.beginPath()
        drawingContext.arc(point.restX, point.restY, 1.65, 0, Math.PI * 2)
        drawingContext.fill()
      }
    }

    function drawWireframe(field: InfluenceField) {
      const { columns, rows } = dimensionsRef.current
      const points = pointsRef.current

      drawingContext.lineWidth = 0.6
      for (let row = 0; row < rows; row += 1) {
        for (let column = 0; column < columns; column += 1) {
          const index = row * columns + column
          const point = points[index]
          if (!point) {
            continue
          }

          const localDisplacement = pointDisplacement(point)
          const coverage = patchCoverage(point, field)
          const tension = Math.max(
            0,
            localDisplacement - gridConfig.wireframeRevealThreshold,
          )
          const reveal = Math.min(
            0.18,
            tension / gridConfig.maxDisplacement * 0.3,
          ) * coverage

          if (
            reveal < 0.032
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

    function drawDeformedPoints(field: InfluenceField) {
      for (const point of pointsRef.current) {
        const coverage = patchCoverage(point, field)
        if (coverage < 0.01) {
          continue
        }

        const displacement = pointDisplacement(point)
        const tension = Math.min(1, displacement / gridConfig.maxDisplacement)
        const radius = 1 + tension * 0.9
        // This resting alpha matches --grid-dot-color. At the fade boundary,
        // Canvas and the partially occluded CSS dot sum to one visual point.
        const opacity = (0.34 + tension * 0.5) * coverage
        drawingContext.fillStyle = `rgba(144, 158, 148, ${opacity})`
        drawingContext.beginPath()
        drawingContext.arc(point.x, point.y, radius, 0, Math.PI * 2)
        drawingContext.fill()
      }
    }

    function drawDebugOverlay(
      interaction: InteractionPoint,
      field: InfluenceField,
      maximumDisplacement: number,
    ) {
      if (!gridConfig.debugMode) {
        return
      }

      drawingContext.save()
      drawingContext.strokeStyle = 'rgba(255, 128, 72, 0.9)'
      drawingContext.lineWidth = 1
      drawingContext.setLineDash([6, 5])
      drawingContext.beginPath()
      drawingContext.ellipse(
        field.centerX,
        field.centerY,
        field.majorRadius,
        field.minorRadius,
        Math.atan2(field.directionY, field.directionX),
        0,
        Math.PI * 2,
      )
      drawingContext.stroke()
      drawingContext.setLineDash([])
      drawingContext.fillStyle = 'rgba(255, 176, 106, 0.96)'
      drawingContext.font = '10px monospace'
      drawingContext.fillText(
        `GRID DEBUG x:${interaction.x.toFixed(1)} y:${interaction.y.toFixed(1)}`,
        interaction.x + 14,
        interaction.y - 28,
      )
      drawingContext.fillText(
        `max:${maximumDisplacement.toFixed(2)} reduced:${reducedMotionRef.current ? 'ON' : 'OFF'}`,
        interaction.x + 14,
        interaction.y - 14,
      )
      drawingContext.restore()
    }

    function drawFrame(timestamp: number) {
      frameRef.current = null
      const { width, height } = dimensionsRef.current
      const interaction = interactionRef.current
      const influenceField = createInfluenceField(interaction)
      const forceRest = !interaction.active
        && releasedAtRef.current !== null
        && timestamp - releasedAtRef.current >= gridConfig.maxSettleDurationMs
      let moving = interaction.active
      let maximumDisplacement = 0
      const motionScale = reducedMotionRef.current
        ? gridConfig.reducedMotionScale
        : 1

      drawingContext.clearRect(0, 0, width, height)

      for (const point of pointsRef.current) {
        if (forceRest) {
          point.x = point.restX
          point.y = point.restY
          point.vx = 0
          point.vy = 0
          continue
        }

        if (interaction.active) {
          const fieldDistance = normalizedFieldDistance(
            point.restX,
            point.restY,
            influenceField,
          )

          if (fieldDistance < 1) {
            const dx = point.x - interaction.x
            const dy = point.y - interaction.y
            const radialDistance = Math.max(1, Math.hypot(dx, dy))
            const { along } = fieldCoordinates(
              point.restX,
              point.restY,
              influenceField,
            )
            const trailingWeight = Math.max(
              0,
              Math.min(1, -along / influenceField.majorRadius),
            )
            const directionalWeight = gridConfig.forwardForceFloor
              + (1 - gridConfig.forwardForceFloor) * trailingWeight
            const radialStrength = gridConfig.displacementStrength
              * (1 - influenceField.speedRatio
                * gridConfig.radialMotionReduction)
            const directionalStrength = Math.min(
              influenceField.speed,
              gridConfig.velocityForFullDirection,
            ) * gridConfig.pointerVelocityTransfer * directionalWeight
            const falloff = (1 - fieldDistance) ** 2

            point.vx += (
              (dx / radialDistance) * radialStrength
              + influenceField.directionX * directionalStrength
            ) * falloff * motionScale
            point.vy += (
              (dy / radialDistance) * radialStrength
              + influenceField.directionY * directionalStrength
            ) * falloff * motionScale
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
        drawLocalOcclusion(influenceField)
        drawWireframe(influenceField)
        drawDeformedPoints(influenceField)
        drawDebugOverlay(interaction, influenceField, maximumDisplacement)
      }

      // Expose live diagnostics for visual automation and field debugging.
      // They remain harmless when the rendered debug overlay is disabled.
      surface.dataset.deforming = String(moving)
      surface.dataset.interactionX = interaction.x.toFixed(1)
      surface.dataset.interactionY = interaction.y.toFixed(1)
      surface.dataset.maximumDisplacement = maximumDisplacement.toFixed(2)
      surface.dataset.interactionSpeed = influenceField.speed.toFixed(2)

      if (moving && !document.hidden) {
        frameRef.current = window.requestAnimationFrame(drawFrame)
      } else if (!interaction.active) {
        releasedAtRef.current = null
      }
    }

    function handleMotionPreference(event: MediaQueryListEvent) {
      reducedMotionRef.current = event.matches
      surface.dataset.reducedMotion = String(event.matches)
    }

    function handleVisibilityChange() {
      if (document.hidden) {
        interactionRef.current.active = false
        rebuild()
      }
    }

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
