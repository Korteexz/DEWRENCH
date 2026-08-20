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
    <main>
      <h1>Criar repositório Git</h1>

      <p>{project.name}</p>
      <p>{project.path}</p>

      <label>
        Branch inicial
        <input
          value={branch}
          onChange={(event) => setBranch(event.target.value)}
        />
      </label>

      <label>
        Primeiro commit
        <input
          value={message}
          onChange={(event) => setMessage(event.target.value)}
        />
      </label>

      <button onClick={onCancel}>
        Cancelar
      </button>

      <button
        onClick={handleCreateRepository}
        disabled={loading}
      >
        {loading ? 'Criando...' : 'Criar repositório'}
      </button>

      {error && <p>{error}</p>}
    </main>
  )
}