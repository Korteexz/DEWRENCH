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

const COMMIT_COLUMN_GAP = 138
const BRANCH_LANE_GAP = 112
const PROJECT_GAP = 220
const BRANCH_LABEL_GAP = 70

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

function laneNumber(index: number): number {
  const distance = Math.floor(index / 2) + 1
  return index % 2 === 0 ? distance : -distance
}

function asFlowNode(
  node: WorkspaceGraphNode,
  position: { x: number; y: number },
): WorkspaceFlowNode {
  return { ...node, position } as WorkspaceFlowNode
}

export function layoutWorkspaceGraph(
  graph: WorkspaceGraph,
): PositionedWorkspaceGraph {
  const commitNodes = graph.nodes.filter(isCommitNode)
  const branchNodes = graph.nodes.filter(isBranchNode)
  const commitById = new Map(commitNodes.map((node) => [node.id, node]))
  const columnByCommit = new Map<string, number>()

  function getColumn(commitId: string, visiting = new Set<string>()): number {
    const cachedColumn = columnByCommit.get(commitId)
    if (cachedColumn !== undefined) {
      return cachedColumn
    }

    if (visiting.has(commitId)) {
      return 0
    }

    const commitNode = commitById.get(commitId)
    if (!commitNode) {
      return 0
    }

    const nextVisiting = new Set(visiting).add(commitId)
    const parentColumns = commitNode.data.commit.parents
      .map(commitNodeId)
      .filter((parentId) => commitById.has(parentId))
      .map((parentId) => getColumn(parentId, nextVisiting))
    const column = parentColumns.length === 0
      ? 0
      : Math.max(...parentColumns) + 1

    columnByCommit.set(commitId, column)
    return column
  }

  for (const commitNode of commitNodes) {
    getColumn(commitNode.id)
  }

  const laneByCommit = new Map<string, number>()
  let allocatedLaneCount = 0

  function assignFirstParentRoute(startId: string, lane: number): void {
    let currentId: string | undefined = startId

    while (currentId && commitById.has(currentId) && !laneByCommit.has(currentId)) {
      laneByCommit.set(currentId, lane)
      const currentCommit = commitById.get(currentId)
      const firstParentHash = currentCommit?.data.commit.parents[0]
      currentId = firstParentHash ? commitNodeId(firstParentHash) : undefined
    }
  }

  const currentBranch = branchNodes.find((node) => node.data.branch.current)
  const principalHeadId = currentBranch
    ? commitNodeId(currentBranch.data.branch.head)
    : commitNodes[0]?.id

  if (principalHeadId) {
    assignFirstParentRoute(principalHeadId, 0)
  }

  const orderedBranches = [...branchNodes].sort((left, right) => {
    if (left.data.branch.current !== right.data.branch.current) {
      return left.data.branch.current ? -1 : 1
    }

    return left.data.branch.name.localeCompare(right.data.branch.name)
  })

  for (const branchNode of orderedBranches) {
    const headId = commitNodeId(branchNode.data.branch.head)
    if (!commitById.has(headId) || laneByCommit.has(headId)) {
      continue
    }

    assignFirstParentRoute(headId, laneNumber(allocatedLaneCount++))
  }

  const commitsNewestFirst = [...commitNodes].sort((left, right) => {
    const columnDifference = getColumn(right.id) - getColumn(left.id)
    return columnDifference || left.data.commit.hash.localeCompare(right.data.commit.hash)
  })

  for (const commitNode of commitsNewestFirst) {
    for (const secondaryParent of commitNode.data.commit.parents.slice(1)) {
      const parentId = commitNodeId(secondaryParent)
      if (commitById.has(parentId) && !laneByCommit.has(parentId)) {
        assignFirstParentRoute(parentId, laneNumber(allocatedLaneCount++))
      }
    }
  }

  for (const commitNode of commitsNewestFirst) {
    if (!laneByCommit.has(commitNode.id)) {
      assignFirstParentRoute(commitNode.id, laneNumber(allocatedLaneCount++))
    }
  }

  const commitPositions = new Map<string, { x: number; y: number }>()
  for (const commitNode of commitNodes) {
    commitPositions.set(commitNode.id, {
      x: getColumn(commitNode.id) * COMMIT_COLUMN_GAP,
      y: (laneByCommit.get(commitNode.id) ?? 0) * BRANCH_LANE_GAP,
    })
  }

  const branchesByHead = new Map<string, BranchGraphNode[]>()
  for (const branchNode of orderedBranches) {
    const existing = branchesByHead.get(branchNode.data.branch.head) ?? []
    existing.push(branchNode)
    branchesByHead.set(branchNode.data.branch.head, existing)
  }

  const branchPositions = new Map<string, { x: number; y: number }>()
  for (const [headHash, branches] of branchesByHead) {
    const headPosition = commitPositions.get(commitNodeId(headHash)) ?? { x: 0, y: 0 }

    branches.forEach((branchNode, index) => {
      branchPositions.set(branchNode.id, {
        x: headPosition.x + index * 24,
        y: headPosition.y - BRANCH_LABEL_GAP - index * 42,
      })
    })
  }

  const minimumCommitX = commitPositions.size > 0
    ? Math.min(...[...commitPositions.values()].map((position) => position.x))
    : 0

  const nodes = graph.nodes.map((node) => {
    if (node.id === PROJECT_NODE_ID) {
      return asFlowNode(node, { x: minimumCommitX - PROJECT_GAP, y: 0 })
    }

    if (node.type === 'commit') {
      return asFlowNode(node, commitPositions.get(node.id) ?? { x: 0, y: 0 })
    }

    return asFlowNode(node, branchPositions.get(node.id) ?? { x: 0, y: 0 })
  })

  const nodeById = new Map(graph.nodes.map((node) => [node.id, node]))
  const edges: WorkspaceFlowEdge[] = graph.edges.map((edge) => {
    const isBranchHead = edge.kind === 'branch-head'
    const targetNode = nodeById.get(edge.target)
    const isMerge = targetNode?.data.kind === 'commit'
      && targetNode.data.commit.parents.length > 1

    return {
      id: edge.id,
      source: edge.source,
      target: edge.target,
      sourceHandle: isBranchHead ? 'branch-out' : 'ancestry-out',
      targetHandle: isBranchHead ? 'branch-in' : 'ancestry-in',
      // React Flow names its built-in Bezier renderer "default". The curve reads
      // as a flexible signal cable and follows nodes automatically while dragging.
      type: 'default',
      className: [
        `workspace-edge--${edge.kind}`,
        isMerge ? 'workspace-edge--merge' : '',
      ].filter(Boolean).join(' '),
      data: { kind: edge.kind },
      deletable: false,
      selectable: false,
      focusable: false,
      interactionWidth: 10,
      zIndex: isBranchHead ? 2 : 1,
    }
  })

  return { nodes, edges }
}
