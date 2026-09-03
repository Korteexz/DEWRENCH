import { forwardRef, useImperativeHandle, useRef } from 'react'
import type { Viewport } from '@xyflow/react'

export interface GraphViewportReadoutHandle {
  update: (viewport: Viewport) => void
}

/**
 * Leitura de câmera do campo observado.
 *
 * Pan e zoom emitem eventos a cada frame. Se isso virasse estado React, todo
 * arrasto re-renderizaria a viewport inteira e o grafo junto. Por isso a
 * leitura é escrita direto no DOM através de um handle imperativo: o número é
 * real e contínuo, e o custo é uma escrita de texto por frame.
 */
const GraphViewportReadout = forwardRef<GraphViewportReadoutHandle>(
  function GraphViewportReadout(_props, forwardedRef) {
    const xRef = useRef<HTMLSpanElement>(null)
    const yRef = useRef<HTMLSpanElement>(null)
    const zoomRef = useRef<HTMLSpanElement>(null)

    useImperativeHandle(forwardedRef, () => ({
      update(viewport: Viewport) {
        if (xRef.current) {
          xRef.current.textContent = Math.round(viewport.x).toString()
        }
        if (yRef.current) {
          yRef.current.textContent = Math.round(viewport.y).toString()
        }
        if (zoomRef.current) {
          zoomRef.current.textContent = `${viewport.zoom.toFixed(2)}×`
        }
      },
    }), [])

    return (
      <span className="dw-coord graph-viewport-readout" aria-hidden="true">
        <span><b>X</b><span ref={xRef}>0</span></span>
        <span><b>Y</b><span ref={yRef}>0</span></span>
        <span><b>Z</b><span ref={zoomRef}>1.00×</span></span>
      </span>
    )
  },
)

export default GraphViewportReadout
