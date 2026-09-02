import { Handle, Position, type NodeProps } from '@xyflow/react'

import type { BranchFlowNode } from '../../../graph/types'

export default function BranchNode({ data, selected }: NodeProps<BranchFlowNode>) {
  const { branch } = data

  return (
    <div
      className={[
        'branch-node',
        branch.current ? 'branch-node--current' : '',
        selected ? 'branch-node--selected' : '',
      ].filter(Boolean).join(' ')}
      title={`${branch.name} → ${branch.head}`}
    >
      <span className="branch-node__diamond" />
      <span className="branch-node__name">{branch.name}</span>
      {branch.current && <span className="branch-node__current">CURRENT</span>}

      <Handle
        id="branch-out"
        className="workspace-handle"
        type="source"
        position={Position.Bottom}
        isConnectable={false}
      />
    </div>
  )
}
