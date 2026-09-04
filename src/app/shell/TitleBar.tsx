import { useCallback, useEffect, useState } from 'react'

import { getCurrentWindow } from '@tauri-apps/api/window'

/**
 * Janela atual, quando existe.
 *
 * Fora do runtime do Tauri (preview no navegador, teste) não há janela: a
 * barra continua desenhando, apenas sem ação. Falhar aqui derrubaria a
 * aplicação inteira por causa da moldura.
 */
function currentWindow() {
  try {
    return getCurrentWindow()
  } catch {
    return null
  }
}

/**
 * Moldura própria da janela.
 *
 * A decoração nativa do Windows foi desligada em `tauri.conf.json`; o que
 * substitui minimizar, maximizar e fechar é esta barra. Arrastar e o duplo
 * clique continuam sendo do sistema, via `data-tauri-drag-region` — os
 * controles ficam deliberadamente FORA dessa região, senão clicar neles
 * moveria a janela em vez de acioná-los.
 */
export function TitleBar() {
  const [maximized, setMaximized] = useState(false)

  const sync = useCallback(() => {
    const win = currentWindow()

    if (!win) {
      return
    }

    void win
      .isMaximized()
      .then(setMaximized)
      .catch(() => undefined)
  }, [])

  useEffect(() => {
    sync()

    const win = currentWindow()

    if (!win) {
      return
    }

    let unlisten: (() => void) | undefined

    void win
      .onResized(() => sync())
      .then((stop) => {
        unlisten = stop
      })
      .catch(() => undefined)

    return () => unlisten?.()
  }, [sync])

  const minimize = () => {
    void currentWindow()?.minimize().catch(() => undefined)
  }

  const toggleMaximize = () => {
    void currentWindow()
      ?.toggleMaximize()
      .then(sync)
      .catch(() => undefined)
  }

  const close = () => {
    void currentWindow()?.close().catch(() => undefined)
  }

  return (
    <header className="dw-titlebar" data-tauri-drag-region>
      <div className="dw-titlebar__identity" data-tauri-drag-region>
        <img
          className="dw-titlebar__logo"
          src="/dewrench_logo_transparent.svg"
          alt=""
          aria-hidden="true"
          width={13}
          height={13}
          draggable={false}
        />
        <span className="dw-titlebar__wordmark" data-tauri-drag-region>
          DEWRENCH
        </span>
      </div>

      <div className="dw-titlebar__controls">
        <button
          type="button"
          className="dw-titlebar__control"
          data-control="minimize"
          aria-label="Minimizar janela"
          title="Minimizar"
          onClick={minimize}
        >
          <span className="dw-titlebar__glyph" aria-hidden="true" />
        </button>

        <button
          type="button"
          className="dw-titlebar__control"
          data-control="maximize"
          aria-label={maximized ? 'Restaurar janela' : 'Maximizar janela'}
          title={maximized ? 'Restaurar' : 'Maximizar'}
          onClick={toggleMaximize}
        >
          <span className="dw-titlebar__glyph" aria-hidden="true" />
        </button>

        <button
          type="button"
          className="dw-titlebar__control"
          data-control="close"
          aria-label="Fechar janela"
          title="Fechar"
          onClick={close}
        >
          <span className="dw-titlebar__glyph" aria-hidden="true" />
        </button>
      </div>
    </header>
  )
}
