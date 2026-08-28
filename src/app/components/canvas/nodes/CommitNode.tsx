import { Handle, Position, type NodeProps } from '@xyflow/react'

import type { CommitFlowNode } from '../../../graph/types'

export default function CommitNode({ data, selected }: NodeProps<CommitFlowNode>) {
  const { commit } = data
  const isMerge = commit.parents.length > 1

  return (
    <div
      className={[
        'commit-node',
        isMerge ? 'commit-node--merge' : '',
        selected ? 'commit-node--selected' : '',
      ].filter(Boolean).join(' ')}
      title={`${commit.short_hash} — ${commit.message}`}
    >
      <Handle
        id="ancestry-in"
        className="workspace-handle"
        type="target"
        position={Position.Bottom}
        isConnectable={false}
      />
      <Handle
        id="branch-in"
        className="workspace-handle"
        type="target"
        position={Position.Top}
        isConnectable={false}
      />

      <span className="commit-node__visual" aria-hidden="true">
        <span className="commit-node__ring" />
        <span className="commit-node__core" />
      </span>

      <span className="commit-node__label">
        <span className="commit-node__hash">{commit.short_hash}</span>
        <span className="commit-node__message">{commit.message}</span>
        {isMerge && (
          <span
            className="commit-node__merge"
            title={`${commit.parents.length} parent commits`}
          >
            {commit.parents.length}× MERGE
          </span>
        )}
      </span>

      <Handle
        id="ancestry-out"
        className="workspace-handle"
        type="source"
        position={Position.Top}
        isConnectable={false}
      />
    </div>
  )
}
