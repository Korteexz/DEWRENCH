import type { ReactNode } from 'react'

import { Button } from '../../design'
import { ModuleRail, type ModuleId } from './ModuleRail'

interface SystemBarProps {
  projectName: string
  projectPath: string
  activeModule: ModuleId
  /**
   * Leitura fornecida pelo módulo ativo. O shell não sabe o que é uma branch:
   * ele só reserva o espaço de instrumentação e deixa o módulo preenchê-lo.
   */
  readout?: ReactNode
  onEject: () => void
}

export function SystemBar({
  projectName,
  projectPath,
  activeModule,
  readout,
  onEject,
}: SystemBarProps) {
  return (
    <header className="dw-systembar">
      <div className="dw-systembar__identity">
        <span className="dw-systembar__mark" aria-hidden="true" />
        <span className="dw-systembar__wordmark">DEWRENCH</span>
        <span className="dw-systembar__subject" title={projectPath}>
          / {projectName}
        </span>
      </div>

      <ModuleRail activeModule={activeModule} />

      <div className="dw-systembar__readout">
        {readout}
        <Button onClick={onEject} title="Fechar projeto e voltar à Home">
          EJECT
        </Button>
      </div>
    </header>
  )
}
