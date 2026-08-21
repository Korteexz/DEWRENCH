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
      <div className="project-node__orb">
        <span className="project-node__core" />

        {data.project.git_state === 'repository' && (
          <span
            className="project-node__capability"
            title="Git disponível"
          >
            GIT
          </span>
        )}
      </div>

      <span className="project-node__name">
        {data.project.name}
      </span>
    </div>
  )
}
