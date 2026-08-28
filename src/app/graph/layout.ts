import {
  PROJECT_NODE_ID,
  commitNodeId,
  type BranchNodeData,
  type CommitNodeData,
  type PositionedWorkspaceGraph,
  type WorkspaceFlowEdge,
  type WorkspaceFlowNode,
  type WorkspaceGraph,
  type WorkspaceGraphNode,
} from './types'
import { layoutConstellation } from './constellationLayout'

type CommitGraphNode = WorkspaceGraphNode & {
  type: 'commit'
  data: CommitNodeData
}

type BranchGraphNode = WorkspaceGraphNode & {
  type: 'branch'
  data: BranchNodeData
}

function isCommitNode(node: WorkspaceGraphNode): node is CommitGraphNode {
  return node.type === 'commit' && node.data.kind === 'commit'
}

function isBranchNode(node: WorkspaceGraphNode): node is BranchGraphNode {
  return node.type === 'branch' && node.data.kind === 'branch'
}

function asFlowNode(
  node: WorkspaceGraphNode,
  position: { x: number; y: number },
): WorkspaceFlowNode {
  return { ...node, position } as WorkspaceFlowNode
}

/**
 * Translate semantic Git graph data into visual positions and React Flow edges.
 * The adapter remains the only place relationships are created; this function
 * reorganizes geometry but maps every input edge exactly once.
 */
export function layoutWorkspaceGraph(
  graph: WorkspaceGraph,
): PositionedWorkspaceGraph {
  const commitNodes = graph.nodes.filter(isCommitNode)
  const branchNodes = graph.nodes.filter(isBranchNode)
  const constellation = layoutConstellation(
    commitNodes.map((node) => ({
      id: node.id,
      parentIds: node.data.commit.parents.map(commitNodeId),
    })),
    branchNodes.map((node) => ({
      id: node.id,
      headId: commitNodeId(node.data.branch.head),
      current: node.data.branch.current,
    })),
  )

  const nodes = graph.nodes.map((node) => {
    if (node.id === PROJECT_NODE_ID) {
      return asFlowNode(node, constellation.projectPosition)
    }

    if (node.type === 'commit') {
      return asFlowNode(
        node,
        constellation.commitPositions.get(node.id) ?? { x: 0, y: 0 },
      )
    }

    return asFlowNode(
      node,
      constellation.branchPositions.get(node.id) ?? { x: 0, y: 0 },
    )
  })

  const nodeById = new Map(graph.nodes.map((node) => [node.id, node]))
  const edges: WorkspaceFlowEdge[] = graph.edges.map((edge) => {
    const isBranchHead = edge.kind === 'branch-head'
    const targetNode = nodeById.get(edge.target)
    const isMerge = !isBranchHead
      && targetNode?.data.kind === 'commit'
      && targetNode.data.commit.parents.length > 1

    return {
      id: edge.id,
      source: edge.source,
      target: edge.target,
      sourceHandle: isBranchHead ? 'branch-out' : 'ancestry-out',
      targetHandle: isBranchHead ? 'branch-in' : 'ancestry-in',
      // Straight segments keep nearby relationships legible and prevent SVG
      // control points from producing viewport-scale decorative sweeps.
      type: 'straight',
      className: [
        `workspace-edge--${edge.kind}`,
        isMerge ? 'workspace-edge--merge' : '',
      ].filter(Boolean).join(' '),
      data: { kind: edge.kind },
      deletable: false,
      selectable: false,
      focusable: false,
      interactionWidth: 8,
      zIndex: isBranchHead ? 2 : 1,
    }
  })

  return { nodes, edges }
}
