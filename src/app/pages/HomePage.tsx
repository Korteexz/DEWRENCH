import { useState } from 'react'
import { open } from '@tauri-apps/plugin-dialog'
import { openProject } from '../../modules/git/services/gitServices'
import type { ProjectOpenResult } from '../../modules/git/types/project'

export default function HomePage() {
  const [project, setProject] = useState<ProjectOpenResult | null>(null)
  const [error, setError] = useState<string | null>(null)

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

      console.log(result)
    } catch (err) {
      setError(String(err))
    }
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
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        cursor: 'pointer',
        fontFamily: 'Roboto, sans-serif',
      }}
    >
      {!project && !error && (
        <p>[ clique aqui para abrir um projeto ]</p>
      )}

      {project && (
        <>
          <h1>{project.name}</h1>

          <p>{project.path}</p>

          <p>
            Estado Git: {project.git_state}
          </p>
        </>
      )}

      {error && (
        <p>Erro: {error}</p>
      )}
    </main>
  )
}