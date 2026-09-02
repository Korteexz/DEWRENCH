import type { ReactNode } from 'react'

import CrtOverlay from '../components/effects/CrtOverlay'
import { SystemBar } from './SystemBar'
import type { ModuleId } from './ModuleRail'

import './shell.css'

interface AppShellProps {
  projectName: string
  projectPath: string
  activeModule: ModuleId
  systemReadout?: ReactNode
  onOpenAnotherProject: () => void
  children: ReactNode
}

/**
 * Chassi da aplicação.
 *
 * O shell só conhece: qual instrumento está montado, o projeto aberto e um
 * espaço de leitura que o módulo preenche. Ele NÃO importa tipos de Git — foi
 * essa dependência que antes fez o shell carregar internals de um módulo.
 *
 * `data-module` propaga a identidade do instrumento ativo por CSS
 * (`--instrument-current`), de modo que trocar de módulo troca o acento da
 * interface inteira sem passar cor por props.
 */
export default function AppShell({
  projectName,
  projectPath,
  activeModule,
  systemReadout,
  onOpenAnotherProject,
  children,
}: AppShellProps) {
  return (
    <main className="dw-shell" data-module={activeModule}>
      <SystemBar
        projectName={projectName}
        projectPath={projectPath}
        activeModule={activeModule}
        readout={systemReadout}
        onEject={onOpenAnotherProject}
      />

      <div className="dw-shell__deck">{children}</div>

      <CrtOverlay />
    </main>
  )
}
