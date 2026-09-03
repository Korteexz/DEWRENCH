import type { NodeTypes } from '@xyflow/react'

import BranchNode from '../components/canvas/nodes/BranchNode'
import CommitNode from '../components/canvas/nodes/CommitNode'
import ProjectNode from '../components/canvas/nodes/ProjectNode'

export const workspaceNodeTypes: NodeTypes = {
  project: ProjectNode,
  commit: CommitNode,
  branch: BranchNode,
}
