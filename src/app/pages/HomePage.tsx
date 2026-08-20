import { useState } from 'react'
import { open } from '@tauri-apps/plugin-dialog'

import { openProject } from '../../modules/git/services/gitServices'
import RepositorySetup from '../../modules/git/pages/RepositorySetup'

import type { ProjectOpenResult } from '../../modules/git/types/project'

export default function HomePage() {
  const [project, setProject] =
    useState<ProjectOpenResult | null>(null)

  const [error, setError] =
    useState<string | null>(null)

  async function handleOpenProject() {
    try {
      const selectedPath = await open({
        directory: true,
        multiple: false,
        title: 'Abrir projeto no DEWRENCH',
      })

      if (selectedPath === null) {
        return
      }

      const result = await openProject(selectedPath)

      setProject(result)
      setError(null)
    } catch (err) {
      setError(String(err))
    }
  }

  // REGRA 1:
  // pasta não possui Git
  if (project?.git_state === 'not_repository') {
    return (
      <RepositorySetup
        project={project}
        onCreated={setProject}
        onCancel={() => setProject(null)}
      />
    )
  }

  // REGRA 2:
  // já é um repositório
  if (project?.git_state === 'repository') {
    return (
      <main>
        <h1>{project.name}</h1>
        <p>{project.path}</p>
        <p>Repository detected ✅</p>
      </main>
    )
  }

  return (
    <main
      onClick={handleOpenProject}
      style={{
        width: '100vw',
        height: '100vh',
        background: '#000',
        color: '#fff',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        cursor: 'pointer',
        fontFamily: 'Roboto, sans-serif',
      }}
    >
      {!error && (
        <p>[ clique aqui para abrir um projeto ]</p>
      )}

      {error && (
        <p>Erro: {error}</p>
      )}
    </main>
  )
}