import {
  useCallback,
  useEffect,
  useRef,
  useState,
  type CSSProperties,
  type PointerEvent as ReactPointerEvent,
  type KeyboardEvent as ReactKeyboardEvent,
  type ReactNode,
} from 'react'

interface SplitDeckProps {
  /** Identidade da persistência. Cada deck guarda suas larguras separadamente. */
  id: string
  left: ReactNode
  center: ReactNode
  right: ReactNode
  defaultLeft?: number
  defaultRight?: number
  minLeft?: number
  minRight?: number
  /** Fração da largura total que um painel lateral não pode ultrapassar. */
  maxSideRatio?: number
  leftLabel?: string
  rightLabel?: string
  className?: string
}

interface PaneSizes {
  left: number
  right: number
}

const KEYBOARD_STEP = 8
const KEYBOARD_STEP_COARSE = 32

function storageKey(id: string): string {
  return `dewrench:deck:${id}`
}

/**
 * Preferência de layout é local e descartável: se o storage falhar (modo
 * restrito, primeiro boot), o deck volta para os padrões em vez de quebrar.
 */
function readStoredSizes(id: string): PaneSizes | null {
  try {
    const raw = window.localStorage.getItem(storageKey(id))
    if (!raw) {
      return null
    }

    const parsed: unknown = JSON.parse(raw)
    if (typeof parsed !== 'object' || parsed === null) {
      return null
    }

    const candidate = parsed as Partial<PaneSizes>
    if (typeof candidate.left !== 'number' || typeof candidate.right !== 'number') {
      return null
    }

    return { left: candidate.left, right: candidate.right }
  } catch {
    return null
  }
}

function writeStoredSizes(id: string, sizes: PaneSizes): void {
  try {
    window.localStorage.setItem(storageKey(id), JSON.stringify(sizes))
  } catch {
    // Layout preference is not load-bearing; losing it must never break the UI.
  }
}

/**
 * Deck de três compartimentos com juntas arrastáveis.
 *
 * Duas decisões estruturais:
 *
 * 1. As larguras vivem em custom properties (`--pane-left`/`--pane-right`) e,
 *    durante o arrasto, são escritas DIRETO no DOM. Passar por estado React a
 *    cada pointermove re-renderizaria a viewport do grafo dezenas de vezes por
 *    segundo. O estado só é atualizado ao soltar.
 * 2. O template de grid fica no CSS, não inline, para que media queries possam
 *    reorganizar o deck em telas estreitas sem competir com estilo inline.
 */
export function SplitDeck({
  id,
  left,
  center,
  right,
  defaultLeft = 268,
  defaultRight = 324,
  minLeft = 208,
  minRight = 248,
  maxSideRatio = 0.34,
  leftLabel = 'Redimensionar painel esquerdo',
  rightLabel = 'Redimensionar painel direito',
  className,
}: SplitDeckProps) {
  const rootRef = useRef<HTMLDivElement>(null)
  const [sizes, setSizes] = useState<PaneSizes>(() => ({
    left: defaultLeft,
    right: defaultRight,
  }))
  const sizesRef = useRef(sizes)
  const dragRef = useRef<{
    side: 'left' | 'right'
    originX: number
    originWidth: number
  } | null>(null)

  const clamp = useCallback((side: 'left' | 'right', value: number): number => {
    const total = rootRef.current?.clientWidth ?? window.innerWidth
    const min = side === 'left' ? minLeft : minRight
    const max = Math.max(min, Math.round(total * maxSideRatio))
    return Math.min(Math.max(Math.round(value), min), max)
  }, [maxSideRatio, minLeft, minRight])

  const applyToDom = useCallback((next: PaneSizes) => {
    const root = rootRef.current
    if (!root) {
      return
    }
    root.style.setProperty('--pane-left', `${next.left}px`)
    root.style.setProperty('--pane-right', `${next.right}px`)
  }, [])

  const commit = useCallback((next: PaneSizes) => {
    sizesRef.current = next
    setSizes(next)
    applyToDom(next)
    writeStoredSizes(id, next)
  }, [applyToDom, id])

  // Restaura a preferência depois da montagem, já reancorada na largura atual.
  useEffect(() => {
    const stored = readStoredSizes(id)
    const next = {
      left: clamp('left', stored?.left ?? defaultLeft),
      right: clamp('right', stored?.right ?? defaultRight),
    }
    sizesRef.current = next
    setSizes(next)
    applyToDom(next)
  }, [applyToDom, clamp, defaultLeft, defaultRight, id])

  // A janela encolher não pode deixar um painel maior do que o deck permite.
  useEffect(() => {
    const root = rootRef.current
    if (!root) {
      return
    }

    const observer = new ResizeObserver(() => {
      const current = sizesRef.current
      const next = {
        left: clamp('left', current.left),
        right: clamp('right', current.right),
      }
      if (next.left !== current.left || next.right !== current.right) {
        sizesRef.current = next
        setSizes(next)
        applyToDom(next)
      }
    })

    observer.observe(root)
    return () => observer.disconnect()
  }, [applyToDom, clamp])

  function handlePointerDown(
    side: 'left' | 'right',
    event: ReactPointerEvent<HTMLDivElement>,
  ): void {
    event.preventDefault()
    event.currentTarget.setPointerCapture(event.pointerId)
    dragRef.current = {
      side,
      originX: event.clientX,
      originWidth: sizesRef.current[side],
    }
  }

  function handlePointerMove(event: ReactPointerEvent<HTMLDivElement>): void {
    const drag = dragRef.current
    if (!drag) {
      return
    }

    const delta = event.clientX - drag.originX
    // O painel direito cresce para a esquerda: o sinal do delta se inverte.
    const raw = drag.side === 'left'
      ? drag.originWidth + delta
      : drag.originWidth - delta
    const width = clamp(drag.side, raw)

    sizesRef.current = { ...sizesRef.current, [drag.side]: width }
    applyToDom(sizesRef.current)
  }

  function handlePointerUp(
    event: ReactPointerEvent<HTMLDivElement>,
  ): void {
    if (!dragRef.current) {
      return
    }
    event.currentTarget.releasePointerCapture(event.pointerId)
    dragRef.current = null
    commit(sizesRef.current)
  }

  function handleKeyDown(
    side: 'left' | 'right',
    event: ReactKeyboardEvent<HTMLDivElement>,
  ): void {
    const step = event.shiftKey ? KEYBOARD_STEP_COARSE : KEYBOARD_STEP
    const current = sizesRef.current[side]
    const towardsWider = side === 'left' ? 'ArrowRight' : 'ArrowLeft'
    const towardsNarrower = side === 'left' ? 'ArrowLeft' : 'ArrowRight'

    if (event.key === towardsWider) {
      event.preventDefault()
      commit({ ...sizesRef.current, [side]: clamp(side, current + step) })
      return
    }

    if (event.key === towardsNarrower) {
      event.preventDefault()
      commit({ ...sizesRef.current, [side]: clamp(side, current - step) })
      return
    }

    if (event.key === 'Home') {
      event.preventDefault()
      const fallback = side === 'left' ? defaultLeft : defaultRight
      commit({ ...sizesRef.current, [side]: clamp(side, fallback) })
    }
  }

  function resetSide(side: 'left' | 'right'): void {
    const fallback = side === 'left' ? defaultLeft : defaultRight
    commit({ ...sizesRef.current, [side]: clamp(side, fallback) })
  }

  function renderDivider(side: 'left' | 'right', label: string) {
    return (
      <div
        className="dw-split-deck__divider"
        role="separator"
        aria-orientation="vertical"
        aria-label={label}
        aria-valuenow={sizes[side]}
        aria-valuemin={side === 'left' ? minLeft : minRight}
        tabIndex={0}
        onPointerDown={(event) => handlePointerDown(side, event)}
        onPointerMove={handlePointerMove}
        onPointerUp={handlePointerUp}
        onPointerCancel={handlePointerUp}
        onDoubleClick={() => resetSide(side)}
        onKeyDown={(event) => handleKeyDown(side, event)}
      >
        <span className="dw-split-deck__grip" aria-hidden="true" />
      </div>
    )
  }

  return (
    <div
      ref={rootRef}
      className={['dw-split-deck', className].filter(Boolean).join(' ')}
      style={{
        '--pane-left': `${sizes.left}px`,
        '--pane-right': `${sizes.right}px`,
      } as CSSProperties}
    >
      {left}
      {renderDivider('left', leftLabel)}
      {center}
      {renderDivider('right', rightLabel)}
      {right}
    </div>
  )
}
