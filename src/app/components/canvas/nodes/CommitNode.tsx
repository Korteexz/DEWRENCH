import { Handle, Position, type NodeProps } from '@xyflow/react'

import type { CommitFlowNode } from '../../../graph/types'

export default function CommitNode({ data, selected }: NodeProps<CommitFlowNode>) {
  const { commit } = data

  return (
    <div
      className={`commit-node${selected ? ' commit-node--selected' : ''}`}
      title={`${commit.short_hash} — ${commit.message}`}
    >
      <Handle
        id="ancestry-in"
        className="workspace-handle"
        type="target"
        position={Position.Left}
        isConnectable={false}
      />
      <Handle
        id="branch-in"
        className="workspace-handle"
        type="target"
        position={Position.Top}
        isConnectable={false}
      />

      <span className="commit-node__point" />
      <span className="commit-node__hash">{commit.short_hash}</span>
      <span className="commit-node__message">{commit.message}</span>

      <Handle
        id="ancestry-out"
        className="workspace-handle"
        type="source"
        position={Position.Right}
        isConnectable={false}
      />
    </div>
  )
}
