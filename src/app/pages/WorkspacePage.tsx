import { useEffect, useMemo, useState } from 'react'
import {
  Background,
  BackgroundVariant,
  Controls,
  ReactFlow,
  useNodesState,
  type NodeMouseHandler,
  type NodeTypes,
} from '@xyflow/react'
import '@xyflow/react/dist/style.css'

import ProjectNode, {
  type ProjectNodeData,
  type ProjectFlowNode,
} from '../components/canvas/ProjectNode'

import type { ProjectOpenResult } from '../../modules/git/types/project'
import type { GitRepositoryDetails, GitGraph } from '../../modules/git/types/Repository'

import { getRepositoryDetails, getRepositoryGraph } from '../../modules/git/services/gitServices'


interface WorkspacePageProps {
  project: ProjectOpenResult
  onOpenAnotherProject: () => void
}

const nodeTypes: NodeTypes = {
  project: ProjectNode,
}

export default function WorkspacePage({
  project,
  onOpenAnotherProject,
}: WorkspacePageProps) {

  // Estado com os detalhes reais do Git
  const [repositoryDetails, setRepositoryDetails] =
    useState<GitRepositoryDetails | null>(null)

  const [gitGraph, setGitGraph] =
    useState<GitGraph | null>(null)

  // Caso Rust/Git devolva algum erro
  const [gitError, setGitError] =
    useState<string | null>(null)

  // Node inicial do canvas
  const initialNodes = useMemo<ProjectFlowNode[]>(
    () => [
      {
        id: 'current-project',
        type: 'project',
        position: { x: 0, y: 0 },
        data: {
          name: project.name,
          path: project.path,
          gitState: project.git_state,
        },
      },
    ],
    [project],
  )

  const [nodes, , onNodesChange] =
    useNodesState(initialNodes)

  const [selectedProject, setSelectedProject] =
    useState<ProjectNodeData | null>(null)


  // Quando o Workspace abrir, consulta Rust para
  // buscar branch, alterações e commits.
  useEffect(() => {
    async function loadGitData() {
      try {
        const [details, graph] = await Promise.all([
          getRepositoryDetails(project.path),
          getRepositoryGraph(project.path),
        ])

        setRepositoryDetails(details)
        setGitGraph(graph)
        setGitError(null)
      } catch (error) {
        setGitError(String(error))
      }
    }

    if (project.git_state === 'repository') {
      loadGitData()
    }
  }, [project.path, project.git_state])


  const handleNodeClick: NodeMouseHandler<ProjectFlowNode> =
    (_event, node) => {
      setSelectedProject(node.data)
    }


  return (
    <main className="workspace-page">

      <header className="workspace-header">

        <div className="workspace-header__brand">
          <span className="workspace-header__signal" />
          <span>DEWRENCH</span>
        </div>

        <div
          className="workspace-header__project"
          title={project.path}
        >
          <span>PROJETO</span>
          <strong>{project.name}</strong>
        </div>

        <button
          className="workspace-header__action nodrag nopan"
          type="button"
          onClick={onOpenAnotherProject}
        >
          Voltar / Abrir outro projeto
        </button>

      </header>


      <section
        className="workspace-canvas"
        aria-label={`Workspace de ${project.name}`}
      >

        <ReactFlow<ProjectFlowNode>
          nodes={nodes}
          edges={[]}
          nodeTypes={nodeTypes}
          onNodesChange={onNodesChange}
          onNodeClick={handleNodeClick}
          onPaneClick={() => setSelectedProject(null)}
          nodesConnectable={false}
          deleteKeyCode={null}
          fitView
          fitViewOptions={{
            padding: 0.45,
            maxZoom: 1.15,
          }}
          minZoom={0.2}
          maxZoom={2.4}
        >

          <Background
            variant={BackgroundVariant.Dots}
            color="var(--canvas-dot)"
            gap={26}
            size={1}
          />

          <Controls showInteractive={false} />

        </ReactFlow>

      </section>


      <div
        className="workspace-legend"
        aria-hidden="true"
      >
        <span /> PROJECT
      </div>


      {selectedProject && (

        <aside
          className="project-inspector nodrag nopan"
          aria-label="Detalhes do projeto"
        >

          <div className="project-inspector__heading">

            <span>PROJECT / SELECTED</span>

            <button
              type="button"
              aria-label="Fechar detalhes"
              onClick={() =>
                setSelectedProject(null)
              }
            >
              ×
            </button>

          </div>


          <dl>

            <div>
              <dt>Nome</dt>
              <dd>{selectedProject.name}</dd>
            </div>

            <div>
              <dt>Path</dt>
              <dd title={selectedProject.path}>
                {selectedProject.path}
              </dd>
            </div>

            <div>
              <dt>Git</dt>
              <dd>{selectedProject.gitState}</dd>
            </div>


            {repositoryDetails && (
              <>
                <div>
                  <dt>Branch</dt>
                  <dd>{repositoryDetails.branch}</dd>
                </div>

                <div>
                  <dt>Changes</dt>
                  <dd>
                    {repositoryDetails.files.length}
                  </dd>
                </div>

                <div>
                  <dt>Commits</dt>
                  <dd>
                    {repositoryDetails.commits.length}
                  </dd>
                </div>
              </>
            )}
            {gitGraph && (
              <>
                <div>
                  <dt>Branches</dt>
                  <dd>{gitGraph.branches.length}</dd>
                </div>

                <div>
                  <dt>Graph commits</dt>
                  <dd>{gitGraph.commits.length}</dd>
                </div>
              </>
            )}
          </dl>


          {gitError && (
            <p>
              Git error: {gitError}
            </p>
          )}

        </aside>

      )}

    </main>
  )
}