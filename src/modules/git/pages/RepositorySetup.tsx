import { useState } from 'react'
import { createRepository } from '../services/gitServices'
import type { ProjectOpenResult } from '../types/project'

interface RepositorySetupProps {
  project: ProjectOpenResult
  onCreated: (project: ProjectOpenResult) => void
  onCancel: () => void
}

export default function RepositorySetup({
  project,
  onCreated,
  onCancel,
}: RepositorySetupProps) {
  const [branch, setBranch] = useState('main')
  const [message, setMessage] = useState('Initial commit')
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  async function handleCreateRepository() {
    try {
      setLoading(true)
      setError(null)

      const result = await createRepository(
        project.path,
        branch,
        message
      )

      onCreated(result)
    } catch (err) {
      setError(String(err))
    } finally {
      setLoading(false)
    }
  }

  return (
    <main className="status-page">
      <section className="status-panel">
        <span className="status-panel__tag">GIT / SETUP</span>
        <h1>Criar repositório Git</h1>
        <p>
          <strong>{project.name}</strong> ainda não possui um repositório Git.
        </p>
        <code className="status-panel__path">{project.path}</code>

        <div className="setup-form">
          <label>
            <span>Branch inicial</span>
            <input
              value={branch}
              onChange={(event) => setBranch(event.target.value)}
              disabled={loading}
            />
          </label>

          <label>
            <span>Primeiro commit</span>
            <input
              value={message}
              onChange={(event) => setMessage(event.target.value)}
              disabled={loading}
            />
          </label>
        </div>

        <div className="status-panel__actions">
          <button className="button button--secondary" type="button" onClick={onCancel}>
            Cancelar
          </button>
          <button
            className="button button--primary"
            type="button"
            onClick={handleCreateRepository}
            disabled={loading}
          >
            {loading ? 'Criando...' : 'Criar repositório'}
          </button>
        </div>

        {error && <p className="feedback feedback--error">Erro: {error}</p>}
      </section>
    </main>
  )
}
