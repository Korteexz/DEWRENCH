import { useState } from 'react'

type ModuleId = 'git' | 'docker' | 'database' | 'rrf'

interface ModuleDefinition {
  id: ModuleId
  label: string
  shortLabel: string
  available: boolean
  icon: string
}

// Adding a future workspace starts here; the shell renders this registry generically.
const MODULES: ModuleDefinition[] = [
  { id: 'git', label: 'Git workspace', shortLabel: 'GIT', available: true, icon: 'branch' },
  { id: 'docker', label: 'Docker', shortLabel: 'DOCKER', available: false, icon: 'container' },
  { id: 'database', label: 'Database viewer', shortLabel: 'DB VIEWER', available: false, icon: 'database' },
  { id: 'rrf', label: 'Request/response flow', shortLabel: 'RRF', available: false, icon: 'flow' },
]

function ModuleIcon({ type }: { type: string }) {
  if (type === 'branch') {
    return <><circle cx="6" cy="5" r="2" /><circle cx="18" cy="5" r="2" /><circle cx="6" cy="19" r="2" /><path d="M6 7v10M8 11h4a6 6 0 0 0 6-6" /></>
  }

  if (type === 'container') {
    return <><path d="M4 8h16v10H4zM7 5h4v3M13 5h4v3" /><path d="M8 13h8" /></>
  }

  if (type === 'database') {
    return <><ellipse cx="12" cy="5" rx="7" ry="3" /><path d="M5 5v7c0 1.7 3.1 3 7 3s7-1.3 7-3V5M5 12v7c0 1.7 3.1 3 7 3s7-1.3 7-3v-7" /></>
  }

  return <><circle cx="5" cy="12" r="2" /><circle cx="19" cy="6" r="2" /><circle cx="19" cy="18" r="2" /><path d="M7 12h3c4 0 4-6 7-6M10 12c4 0 4 6 7 6" /></>
}

export default function ModuleNavigation() {
  const [touchedModule, setTouchedModule] = useState<ModuleId | null>(null)

  return (
    <nav className="module-navigation" aria-label="Módulos DEWRENCH">
      <div className="module-navigation__rail">
        {MODULES.map((module) => {
          const isActive = module.id === 'git'
          const wasTouched = touchedModule === module.id && !module.available

          return (
            <button
              key={module.id}
              className={[
                'module-navigation__item',
                isActive ? 'module-navigation__item--active' : '',
                wasTouched ? 'module-navigation__item--touched' : '',
              ].filter(Boolean).join(' ')}
              type="button"
              aria-current={isActive ? 'page' : undefined}
              aria-disabled={!module.available}
              onClick={() => {
                if (!module.available) {
                  setTouchedModule(module.id)
                }
              }}
              onAnimationEnd={() => setTouchedModule(null)}
            >
              <span className="module-navigation__icon" aria-hidden="true">
                <svg viewBox="0 0 24 24"><ModuleIcon type={module.icon} /></svg>
              </span>
              <span>{module.shortLabel}</span>
              {!module.available && <small>SOON</small>}
            </button>
          )
        })}
      </div>
      <span className="sr-only" aria-live="polite">
        {touchedModule ? `${touchedModule} ainda não está disponível` : ''}
      </span>
    </nav>
  )
}
