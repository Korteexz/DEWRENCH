import { useEffect } from 'react'

export interface NodeContextMenuItem {
  label: string
  onSelect: () => void
  disabled?: boolean
}

interface NodeContextMenuProps {
  x: number
  y: number
  items: NodeContextMenuItem[]
  onClose: () => void
}

export default function NodeContextMenu({
  x,
  y,
  items,
  onClose,
}: NodeContextMenuProps) {
  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        onClose()
      }
    }

    function handlePointerDown() {
      onClose()
    }

    window.addEventListener('keydown', handleKeyDown)
    window.addEventListener('pointerdown', handlePointerDown)

    return () => {
      window.removeEventListener('keydown', handleKeyDown)
      window.removeEventListener('pointerdown', handlePointerDown)
    }
  }, [onClose])

  return (
    <div
      className="node-context-menu nodrag nopan"
      style={{ left: x, top: y }}
      role="menu"
      onPointerDown={(event) => event.stopPropagation()}
    >
      {items.map((item) => (
        <button
          key={item.label}
          type="button"
          role="menuitem"
          onClick={item.onSelect}
          disabled={item.disabled}
        >
          {item.label}
        </button>
      ))}
    </div>
  )
}
