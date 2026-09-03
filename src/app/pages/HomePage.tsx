import { useState } from 'react'
import { open } from '@tauri-apps/plugin-dialog'

import { openProject } from '../../modules/git/services/gitServices'
import type { ProjectOpenResult } from '../../modules/git/types/project'

interface HomePageProps {
  onProjectOpened: (project: ProjectOpenResult) => void
}

export default function HomePage({ onProjectOpened }: HomePageProps) {
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  async function handleOpenProject() {
    try {
      setLoading(true)

      const selectedPath = await open({
        directory: true,
        multiple: false,
        title: 'Abrir projeto no DEWRENCH',
      })

      if (selectedPath === null) {
        return
      }

      const result = await openProject(selectedPath)

      setError(null)
      onProjectOpened(result)
    } catch (err) {
      setError(String(err))
    } finally {
      setLoading(false)
    }
  }

  return (
    <main className="home-page">
      <button
        className="home-page__open"
        type="button"
        onClick={handleOpenProject}
        disabled={loading}
      >
        <span className="home-page__eyebrow">DEWRENCH / LOCAL WORKSPACE</span>
        <span className="home-page__mark" aria-hidden="true" />
        <span className="home-page__prompt">
          {loading ? 'abrindo projeto...' : 'clique para abrir um projeto'}
        </span>
        <span className="home-page__hint">Selecione uma pasta local</span>
      </button>

      {error && <p className="feedback feedback--error">Erro: {error}</p>}
    </main>
  )
}
