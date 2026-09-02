/**
 * Central tuning surface for the background computational fabric.
 * Values are CSS pixels or per-60 Hz-frame coefficients unless noted.
 */
export const deformableGridConfig = {
  /** Resting distance between grid points; keep aligned with --grid-spacing. */
  gridSpacing: 32,
  /** Resting radial pressure; kept low so a stationary node makes no crater. */
  displacementStrength: 0.72,
  /** Base pointer distance used to build the velocity-shaped influence field. */
  influenceRadius: 200,
  /** Extra undisturbed ring used to cross-fade back into the CSS base grid. */
  edgeBlendWidth: 42,
  /** Influence radius multiplier when pointer velocity is zero. */
  stationaryRadiusScale: 0.7,
  /** Added radius along the movement axis at maximum directional velocity. */
  directionalRadiusStretch: 0.7,
  /** Added radius across the movement axis, keeping the fast field elliptical. */
  perpendicularRadiusStretch: 0.08,
  /** Pointer delta that produces the maximum directional field shape. */
  velocityForFullDirection: 12,
  /** Maximum distance the influence center trails behind a moving node. */
  maxWakeOffset: 48,
  /** How much radial pressure gives way to directional pull during movement. */
  radialMotionReduction: 0.58,
  /** Minimum share of directional force applied ahead of the moving node. */
  forwardForceFloor: 0.22,
  /** Pull toward each point's rest position. Higher values return faster. */
  springStrength: 0.056,
  /** Velocity retained each frame. Lower values remove wobble sooner. */
  damping: 0.82,
  /** Hard speed limit that prevents fast drags from creating unstable motion. */
  maxVelocity: 4.8,
  /** Portion of pointer velocity transferred into nearby points. */
  pointerVelocityTransfer: 0.68,
  /** Hard local displacement limit, keeping deformation restrained. */
  maxDisplacement: 42,
  /** Displacement at which faint point-to-point wire lines begin to appear. */
  wireframeRevealThreshold: 4,
  /** Force multiplier used instead of disabling the fabric for reduced motion. */
  reducedMotionScale: 0.62,
  /** Velocity below which a point is considered settled. */
  settleVelocity: 0.024,
  /** Distance from rest below which a point no longer needs drawing. */
  settleDisplacement: 0.075,
  /** Safety cutoff after release so animation can never drift indefinitely. */
  maxSettleDurationMs: 2600,
  /** Diagnostic radius, coordinates and displacement overlay. Disable after QA. */
  debugMode: false,
} as const
