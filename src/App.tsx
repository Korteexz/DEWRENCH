import { useState } from 'react'

import HomePage from './app/pages/HomePage'
import UnbornRepositoryPage from './app/pages/UnbornRepositoryPage'
import WorkspacePage from './app/pages/WorkspacePage'
import { TitleBar } from './app/shell/TitleBar'
import RepositorySetup from './modules/git/pages/RepositorySetup'
import type { ProjectOpenResult } from './modules/git/types/project'

import './App.css'

function App() {
  const [project, setProject] = useState<ProjectOpenResult | null>(null)

  function page() {
    if (project?.git_state === 'not_repository') {
      return (
        <RepositorySetup
          project={project}
          onCreated={setProject}
          onCancel={() => setProject(null)}
        />
      )
    }

    if (project?.git_state === 'unborn_repository') {
      return (
        <UnbornRepositoryPage
          project={project}
          onBack={() => setProject(null)}
        />
      )
    }

    if (project?.git_state === 'repository') {
      return (
        <WorkspacePage
          project={project}
          onOpenAnotherProject={() => setProject(null)}
        />
      )
    }

    return <HomePage onProjectOpened={setProject} />
  }

  // A moldura nativa está desligada: a title bar acompanha TODAS as telas,
  // inclusive Home e setup, que não passam pelo chassi.
  return (
    <>
      <TitleBar />
      {page()}
    </>
  )
}

export default App
