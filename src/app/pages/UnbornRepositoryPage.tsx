import type { ProjectOpenResult } from '../../modules/git/types/project'

interface UnbornRepositoryPageProps {
  project: ProjectOpenResult
  onBack: () => void
}

export default function UnbornRepositoryPage({
  project,
  onBack,
}: UnbornRepositoryPageProps) {
  return (
    <main className="status-page">
      <section className="status-panel">
        <span className="status-panel__tag">GIT / UNBORN REPOSITORY</span>
        <h1>Repositório sem commit</h1>
        <p>
          <strong>{project.name}</strong> já foi inicializado como repositório Git,
          mas ainda não possui um commit.
        </p>
        <code className="status-panel__path">{project.path}</code>
        <button className="button button--primary" type="button" onClick={onBack}>
          Voltar para a Home
        </button>
      </section>
    </main>
  )
}
