/**
 * The resting grid is a viewport-sized CSS tile rather than a bitmap with a
 * guessed extent. It therefore covers every resize immediately, while Canvas
 * effects can deform a local patch above it without owning base coverage.
 */
export default function ComputationalGrid() {
  return (
    <div className="computational-grid" aria-hidden="true">
      <span className="computational-grid__lighting" />
    </div>
  )
}
