import type { ReactNode } from 'react'

import type { ProjectOpenResult } from '../../../modules/git/types/project'
import CrtOverlay from '../effects/CrtOverlay'
import ModuleNavigation from '../navigation/ModuleNavigation'

interface AppShellProps {
  project: ProjectOpenResult
  branch: string | null
  connected: boolean
  onOpenAnotherProject: () => void
  children: ReactNode
}

/**
 * The shell owns application-wide chrome only. Module workspaces remain children,
 * so Docker or database tooling can be added later without importing Git code here.
 */
export default function AppShell({
  project,
  branch,
  connected,
  onOpenAnotherProject,
  children,
}: AppShellProps) {
  return (
    <main className="app-shell">
      <header className="system-bar">
        <div className="system-bar__identity">
          <span className="system-bar__mark" aria-hidden="true">DW</span>
          <div>
            <strong>DEWRENCH</strong>
            <span>LOCAL SYSTEMS WORKBENCH</span>
          </div>
        </div>

        <ModuleNavigation />

        <div className="system-bar__repository">
          <span className={`system-led${connected ? ' system-led--online' : ''}`} />
          <div title={project.path}>
            <span>{connected ? 'REPOSITORY LINKED' : 'CONNECTING'}</span>
            <strong>{project.name}{branch ? ` / ${branch}` : ''}</strong>
          </div>
          <button type="button" onClick={onOpenAnotherProject}>
            EJECT
          </button>
        </div>
      </header>

      <div className="app-shell__workspace">{children}</div>
      <CrtOverlay />
    </main>
  )
}
