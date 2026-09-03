import type { ReactNode } from 'react'

import {
  Metric,
  MetricCluster,
  Panel,
  SectionHeader,
  TelemetryBar,
} from '../../../design'
import type { ProjectOpenResult } from '../types/project'
import type { GitGraph, GitRepositoryDetails } from '../types/repository'
import { summarizeGraph } from '../view/graphStats'
import { summarizeWorkingTree } from '../view/workingTree'

interface GitInspectorPaneProps {
  project: ProjectOpenResult
  details: GitRepositoryDetails | null
  graph: GitGraph | null
  children?: ReactNode
}

/**
 * Compartimento fixo do inspetor.
 *
 * A geometria é persistente: o inspetor nunca é um card flutuante que aparece
 * e some. Sem seleção, ele mostra o estado agregado do repositório — a tela
 * continua sendo um instrumento ligado, não uma área vazia.
 */
export default function GitInspectorPane({
  project,
  details,
  graph,
  children,
}: GitInspectorPaneProps) {
  const tree = summarizeWorkingTree(details?.files)
  const stats = summarizeGraph(graph)

  return (
    <Panel
      as="aside"
      index="03"
      title="Context inspector"
      aria-label="Inspetor contextual"
      scroll
      className="git-inspector-pane"
    >
      {children ?? (
        <div className="git-inspector-standby">
          <div className="git-inspector-standby__crosshair" aria-hidden="true">
            <span /><span /><i />
          </div>
          <p className="git-inspector-standby__hint">
            Nenhum objeto selecionado. Escolha um commit, branch ou o
            repositório para inspecionar.
          </p>

          <SectionHeader title="Repository" readout={project.name} />
          <MetricCluster>
            <Metric label="Branch" value={details?.branch ?? '—'} />
            <Metric
              label="Branches"
              value={stats.branchCount.toString().padStart(2, '0')}
            />
            <Metric
              label="Commits"
              value={stats.commitCount.toString().padStart(3, '0')}
            />
          </MetricCluster>

          <SectionHeader title="Working tree" />
          <div className="git-inspector-standby__telemetry">
            <TelemetryBar
              label="Staged"
              value={tree.staged}
              total={Math.max(tree.total, 1)}
              tone={tree.staged > 0 ? 'instrument' : 'neutral'}
              readout={`${tree.staged}/${tree.total}`}
            />
            <TelemetryBar
              label="Unstaged"
              value={tree.unstaged}
              total={Math.max(tree.total, 1)}
              tone={tree.unstaged > 0 ? 'warn' : 'neutral'}
              readout={`${tree.unstaged}/${tree.total}`}
            />
          </div>

          <MetricCluster>
            <Metric
              label="Untracked"
              value={tree.untracked.toString().padStart(2, '0')}
            />
            <Metric
              label="Merges"
              value={stats.mergeCount.toString().padStart(2, '0')}
            />
            <Metric
              label="Roots"
              value={stats.rootCount.toString().padStart(2, '0')}
            />
          </MetricCluster>

          <code className="git-inspector-standby__path" title={project.path}>
            {project.path}
          </code>
        </div>
      )}
    </Panel>
  )
}
