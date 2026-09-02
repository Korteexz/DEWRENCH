import type { NodeProps } from '@xyflow/react'

import type { ProjectFlowNode } from '../../../graph/types'

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
      <span className="project-node__visual" aria-hidden="true">
        <span className="project-node__hex" />
        <span className="project-node__halo" />
      </span>

      {data.project.git_state === 'repository' && (
        <span className="project-node__capability" title="Git disponível">
          GIT
        </span>
      )}

      <span className="project-node__name">
        {data.project.name}
      </span>
    </div>
  )
}
