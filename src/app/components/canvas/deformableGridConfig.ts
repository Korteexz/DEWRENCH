/**
 * Central tuning surface for the background computational fabric.
 * Values are CSS pixels or per-60 Hz-frame coefficients unless noted.
 */
export const deformableGridConfig = {
  /** Resting distance between grid points; keep aligned with --grid-spacing. */
  gridSpacing: 28,
  /** Radial impulse applied at the drag position. Higher values bend farther. */
  displacementStrength: 1.42,
  /** Maximum pointer distance that can disturb a point. */
  influenceRadius: 148,
  /** Pull toward each point's rest position. Higher values return faster. */
  springStrength: 0.056,
  /** Velocity retained each frame. Lower values remove wobble sooner. */
  damping: 0.82,
  /** Hard speed limit that prevents fast drags from creating unstable motion. */
  maxVelocity: 3.15,
  /** Portion of pointer velocity transferred into nearby points. */
  pointerVelocityTransfer: 0.145,
  /** Hard local displacement limit, keeping deformation restrained. */
  maxDisplacement: 25,
  /** Displacement at which faint point-to-point wire lines begin to appear. */
  wireframeRevealThreshold: 2.1,
  /** Velocity below which a point is considered settled. */
  settleVelocity: 0.024,
  /** Distance from rest below which a point no longer needs drawing. */
  settleDisplacement: 0.075,
  /** Safety cutoff after release so animation can never drift indefinitely. */
  maxSettleDurationMs: 2600,
} as const
