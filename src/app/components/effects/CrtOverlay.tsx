/**
 * CRT texture is isolated from functional UI so its intensity can be tuned or
 * disabled without touching graph, navigation, or inspector components.
 */
export default function CrtOverlay() {
  return (
    <div className="crt-overlay" aria-hidden="true">
      <span className="crt-overlay__scanlines" />
      <span className="crt-overlay__vignette" />
    </div>
  )
}
