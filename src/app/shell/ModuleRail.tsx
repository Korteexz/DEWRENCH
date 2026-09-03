import { useState } from 'react'

export type ModuleId = 'git' | 'docker' | 'database' | 'rrf'

interface ModuleDefinition {
  id: ModuleId
  label: string
  shortLabel: string
  available: boolean
}

/**
 * Registro dos instrumentos montáveis no chassi.
 *
 * Adicionar Docker/Kubernetes/Terraform começa aqui: o shell renderiza este
 * registro genericamente e não conhece nenhum módulo em particular.
 */
const MODULES: ModuleDefinition[] = [
  { id: 'git', label: 'Git workspace', shortLabel: 'GIT', available: true },
  { id: 'docker', label: 'Docker', shortLabel: 'DOCKER', available: false },
  { id: 'database', label: 'Database viewer', shortLabel: 'DB', available: false },
  { id: 'rrf', label: 'Request/response flow', shortLabel: 'RRF', available: false },
]

interface ModuleRailProps {
  activeModule: ModuleId
}

export function ModuleRail({ activeModule }: ModuleRailProps) {
  const [touchedModule, setTouchedModule] = useState<ModuleId | null>(null)

  return (
    <nav className="dw-module-rail" aria-label="Instrumentos DEWRENCH">
      {MODULES.map((module) => {
        const isActive = module.id === activeModule

        return (
          <button
            key={module.id}
            className="dw-module-rail__slot"
            type="button"
            data-active={isActive}
            data-available={module.available}
            data-touched={touchedModule === module.id}
            data-module={module.id}
            aria-current={isActive ? 'page' : undefined}
            aria-disabled={!module.available}
            title={module.label}
            onClick={() => {
              if (!module.available) {
                setTouchedModule(module.id)
              }
            }}
            onAnimationEnd={() => setTouchedModule(null)}
          >
            <span className="dw-module-rail__mark" aria-hidden="true" />
            <span className="dw-module-rail__name">{module.shortLabel}</span>
            {!module.available && (
              <span className="dw-module-rail__state">OFFLINE</span>
            )}
          </button>
        )
      })}
      <span className="sr-only" aria-live="polite">
        {touchedModule ? `${touchedModule} ainda não está disponível` : ''}
      </span>
    </nav>
  )
}
