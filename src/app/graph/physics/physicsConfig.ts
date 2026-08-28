/**
 * Central tuning surface for the graph's interaction physics.
 *
 * Values are expressed in React Flow graph units and normalized to a 60 Hz
 * frame. The simulation is deliberately overdamped and displacement-limited:
 * it should feel responsive under the pointer without becoming a new layout.
 */
export interface GraphPhysicsConfig {
  repulsionStrength: number
  springStrength: number
  springLength: number
  damping: number
  maxVelocity: number
  interactionRadius: number
  dragInfluence: number
  anchorStrength: number
  maxDisplacement: number
  collisionRadius: number
  springLengthBlend: number
  sleepVelocity: number
  sleepFrames: number
  maxSettleDurationMs: number
}

export const graphPhysicsConfig: Readonly<GraphPhysicsConfig> = {
  /** Strength of the inverse-square push between nearby movable nodes. */
  repulsionStrength: 760,
  /** Hooke's-law stiffness for real graph edges. Higher values feel tighter. */
  springStrength: 0.012,
  /** Nominal edge length, blended gently with each edge's approved rest length. */
  springLength: 104,
  /** Velocity retained per 60 Hz frame. Lower values stop motion sooner. */
  damping: 0.78,
  /** Maximum graph units a node may move during one 60 Hz frame. */
  maxVelocity: 1.85,
  /** Radius in which repulsion and direct drag influence are applied. */
  interactionRadius: 238,
  /** Fraction of a dragged node's movement transferred to connected neighbors. */
  dragInfluence: 0.105,
  /** Weak tether that preserves the deterministic topology after interaction. */
  anchorStrength: 0.016,
  /** Maximum distance an indirectly moved node may travel from its layout anchor. */
  maxDisplacement: 34,
  /** Distance used to strengthen repulsion before nodes visually overlap. */
  collisionRadius: 72,
  /** Blend from each original edge length toward springLength (0 preserves it). */
  springLengthBlend: 0.12,
  /** Speed below which a frame counts toward sleep. */
  sleepVelocity: 0.018,
  /** Consecutive calm frames required before requestAnimationFrame stops. */
  sleepFrames: 18,
  /** Absolute safeguard against an interaction producing endless motion. */
  maxSettleDurationMs: 2800,
}
