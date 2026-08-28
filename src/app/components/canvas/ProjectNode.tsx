import type { Node, NodeProps } from '@xyflow/react'

import type { GitState } from '../../../modules/git/types/project'

export interface ProjectNodeData extends Record<string, unknown> {
  name: string
  path: string
  gitState: GitState
}

export type ProjectFlowNode =
  Node<ProjectNodeData, 'project'>

export default function ProjectNode({
  data,
  selected,
}: NodeProps<ProjectFlowNode>) {
  return (
    <div
      className={`project-node${
        selected ? ' project-node--selected' : ''
      }`}
    >
      <div className="project-node__orb">
        <span className="project-node__core" />

        {data.gitState === 'repository' && (
          <span
            className="project-node__capability"
            title="Git disponível"
          >
            GIT
          </span>
        )}
      </div>

      <span className="project-node__name">
        {data.name}
      </span>
    </div>
  )
}