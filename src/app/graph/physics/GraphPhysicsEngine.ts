import {
  graphPhysicsConfig,
  type GraphPhysicsConfig,
} from './physicsConfig'

export interface PhysicsPoint {
  x: number
  y: number
}

export interface PhysicsNode extends PhysicsPoint {
  id: string
  movable: boolean
}

export interface PhysicsLink {
  source: string
  target: string
}

interface PhysicsBody extends PhysicsNode {
  anchorX: number
  anchorY: number
  vx: number
  vy: number
}

interface PhysicsSpring extends PhysicsLink {
  restLength: number
}

export interface PhysicsStep {
  active: boolean
  positions: ReadonlyMap<string, PhysicsPoint>
}

const FRAME_DURATION_MS = 1000 / 60

function clamp(value: number, minimum: number, maximum: number): number {
  return Math.max(minimum, Math.min(maximum, value))
}

/**
 * Small, deterministic interaction engine independent of Git and React Flow.
 *
 * The approved layout becomes a set of anchors. Physics sleeps until a drag,
 * then combines inverse-square repulsion, Hooke springs, anchor restoration,
 * and velocity damping. It never creates relationships; only supplied links
 * participate in spring forces.
 */
export class GraphPhysicsEngine {
  private readonly config: Readonly<GraphPhysicsConfig>

  private bodies = new Map<string, PhysicsBody>()

  private springs: PhysicsSpring[] = []

  private neighbors = new Map<string, Set<string>>()

  private draggedId: string | null = null

  private previousDragPosition: PhysicsPoint | null = null

  private active = false

  private calmFrameCount = 0

  private settlingDurationMs = 0

  constructor(config: Readonly<GraphPhysicsConfig> = graphPhysicsConfig) {
    this.config = config
  }

  reset(nodes: PhysicsNode[], links: PhysicsLink[]): void {
    this.bodies = new Map(nodes.map((node) => [node.id, {
      ...node,
      anchorX: node.x,
      anchorY: node.y,
      vx: 0,
      vy: 0,
    }]))
    this.neighbors = new Map(nodes.map((node) => [node.id, new Set<string>()]))

    this.springs = links.flatMap((link) => {
      const source = this.bodies.get(link.source)
      const target = this.bodies.get(link.target)
      if (!source || !target) {
        return []
      }

      this.neighbors.get(link.source)?.add(link.target)
      this.neighbors.get(link.target)?.add(link.source)

      const approvedLength = Math.max(1, Math.hypot(
        target.x - source.x,
        target.y - source.y,
      ))
      const restLength = approvedLength
        + (this.config.springLength - approvedLength)
          * this.config.springLengthBlend

      return [{ ...link, restLength }]
    })

    this.draggedId = null
    this.previousDragPosition = null
    this.stop()
  }

  beginDrag(nodeId: string, position: PhysicsPoint): void {
    const body = this.bodies.get(nodeId)
    if (!body?.movable) {
      return
    }

    body.x = position.x
    body.y = position.y
    body.vx = 0
    body.vy = 0
    this.draggedId = nodeId
    this.previousDragPosition = position
    this.wake()
  }

  updateDrag(nodeId: string, position: PhysicsPoint): void {
    const body = this.bodies.get(nodeId)
    if (!body?.movable || this.draggedId !== nodeId) {
      return
    }

    const previous = this.previousDragPosition ?? position
    const deltaX = position.x - previous.x
    const deltaY = position.y - previous.y
    body.x = position.x
    body.y = position.y
    body.vx = 0
    body.vy = 0
    this.previousDragPosition = position

    // Only real direct neighbors receive the pointer impulse. The spatial
    // falloff prevents a long edge from pulling a distant cluster abruptly;
    // regular spring forces propagate a much softer response afterward.
    for (const neighborId of this.neighbors.get(nodeId) ?? []) {
      const neighbor = this.bodies.get(neighborId)
      if (!neighbor?.movable) {
        continue
      }

      const distance = Math.hypot(
        neighbor.x - position.x,
        neighbor.y - position.y,
      )
      const falloff = clamp(
        1 - distance / this.config.interactionRadius,
        0,
        1,
      )
      neighbor.vx += deltaX * this.config.dragInfluence * falloff
      neighbor.vy += deltaY * this.config.dragInfluence * falloff
    }

    this.wake()
  }

  endDrag(nodeId: string, position: PhysicsPoint): void {
    const body = this.bodies.get(nodeId)
    if (!body?.movable || this.draggedId !== nodeId) {
      return
    }

    body.x = position.x
    body.y = position.y
    // A deliberate drop becomes this node's new anchor; surrounding nodes
    // remain tethered to the approved layout and can only move locally.
    body.anchorX = position.x
    body.anchorY = position.y
    body.vx = 0
    body.vy = 0
    this.draggedId = null
    this.previousDragPosition = null
    this.wake()
  }

  step(deltaMs: number): PhysicsStep {
    if (!this.active) {
      return { active: false, positions: this.getPositions() }
    }

    const timeStep = clamp(deltaMs / FRAME_DURATION_MS, 0.35, 1.5)
    const forces = new Map(
      [...this.bodies.keys()].map((id) => [id, { x: 0, y: 0 }]),
    )
    const movableBodies = [...this.bodies.values()].filter((body) => body.movable)

    // Nearby nodes repel with an inverse-square force. collisionRadius keeps
    // the force strong but finite at close range, avoiding explosive motion.
    for (let leftIndex = 0; leftIndex < movableBodies.length; leftIndex += 1) {
      for (
        let rightIndex = leftIndex + 1;
        rightIndex < movableBodies.length;
        rightIndex += 1
      ) {
        const left = movableBodies[leftIndex]
        const right = movableBodies[rightIndex]
        const leftForce = forces.get(left.id)
        const rightForce = forces.get(right.id)
        if (!leftForce || !rightForce) {
          continue
        }

        let dx = right.x - left.x
        let dy = right.y - left.y
        let distance = Math.hypot(dx, dy)
        if (distance >= this.config.interactionRadius) {
          continue
        }

        if (distance < 0.001) {
          dx = left.id.localeCompare(right.id) < 0 ? 0.5 : -0.5
          dy = 0.5
          distance = Math.hypot(dx, dy)
        }

        const effectiveDistance = Math.max(
          distance,
          this.config.collisionRadius * 0.34,
        )
        const falloff = 1 - distance / this.config.interactionRadius
        const magnitude = this.config.repulsionStrength
          * falloff
          / (effectiveDistance * effectiveDistance)
        const forceX = dx / distance * magnitude
        const forceY = dy / distance * magnitude
        leftForce.x -= forceX
        leftForce.y -= forceY
        rightForce.x += forceX
        rightForce.y += forceY
      }
    }

    // Hooke's law: F = stiffness × extension. Only supplied graph edges are
    // springs, so the physics layer cannot invent Git relationships.
    for (const spring of this.springs) {
      const source = this.bodies.get(spring.source)
      const target = this.bodies.get(spring.target)
      const sourceForce = forces.get(spring.source)
      const targetForce = forces.get(spring.target)
      if (!source || !target || !sourceForce || !targetForce) {
        continue
      }

      const dx = target.x - source.x
      const dy = target.y - source.y
      const distance = Math.max(0.001, Math.hypot(dx, dy))
      const magnitude = (distance - spring.restLength)
        * this.config.springStrength
      const forceX = dx / distance * magnitude
      const forceY = dy / distance * magnitude

      if (source.movable) {
        sourceForce.x += forceX
        sourceForce.y += forceY
      }
      if (target.movable) {
        targetForce.x -= forceX
        targetForce.y -= forceY
      }
    }

    let maximumSpeed = 0
    for (const body of movableBodies) {
      if (body.id === this.draggedId) {
        continue
      }

      const force = forces.get(body.id)
      if (!force) {
        continue
      }

      // The weak anchor is what makes physics an interaction layer rather than
      // a second layout algorithm. Neighbors return toward readable positions.
      force.x += (body.anchorX - body.x) * this.config.anchorStrength
      force.y += (body.anchorY - body.y) * this.config.anchorStrength

      body.vx = (body.vx + force.x * timeStep)
        * Math.pow(this.config.damping, timeStep)
      body.vy = (body.vy + force.y * timeStep)
        * Math.pow(this.config.damping, timeStep)

      const speed = Math.hypot(body.vx, body.vy)
      if (speed > this.config.maxVelocity) {
        body.vx = body.vx / speed * this.config.maxVelocity
        body.vy = body.vy / speed * this.config.maxVelocity
      }

      body.x += body.vx * timeStep
      body.y += body.vy * timeStep

      const displacementX = body.x - body.anchorX
      const displacementY = body.y - body.anchorY
      const displacement = Math.hypot(displacementX, displacementY)
      if (displacement > this.config.maxDisplacement) {
        const scale = this.config.maxDisplacement / displacement
        body.x = body.anchorX + displacementX * scale
        body.y = body.anchorY + displacementY * scale
        const outwardVelocity = body.vx * displacementX
          + body.vy * displacementY
        if (outwardVelocity > 0) {
          body.vx *= 0.35
          body.vy *= 0.35
        }
      }

      maximumSpeed = Math.max(maximumSpeed, Math.hypot(body.vx, body.vy))
    }

    if (this.draggedId) {
      this.calmFrameCount = 0
      this.settlingDurationMs = 0
    } else {
      this.settlingDurationMs += deltaMs
      this.calmFrameCount = maximumSpeed < this.config.sleepVelocity
        ? this.calmFrameCount + 1
        : 0

      if (
        this.calmFrameCount >= this.config.sleepFrames
        || this.settlingDurationMs >= this.config.maxSettleDurationMs
      ) {
        this.stop()
      }
    }

    return { active: this.active, positions: this.getPositions() }
  }

  isActive(): boolean {
    return this.active
  }

  private getPositions(): ReadonlyMap<string, PhysicsPoint> {
    return new Map([...this.bodies].map(([id, body]) => [id, {
      x: body.x,
      y: body.y,
    }]))
  }

  private wake(): void {
    this.active = true
    this.calmFrameCount = 0
    this.settlingDurationMs = 0
  }

  private stop(): void {
    this.active = false
    this.calmFrameCount = 0
    this.settlingDurationMs = 0
    for (const body of this.bodies.values()) {
      body.vx = 0
      body.vy = 0
    }
  }
}
