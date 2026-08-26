import {
  PROJECT_NODE_ID,
  branchNodeId,
  commitNodeId,
  type WorkspaceGraph,
  type WorkspaceGraphEdge,
  type WorkspaceGraphNode,
} from '../../../app/graph/types'
import type { ProjectOpenResult } from '../types/project'
import type { GitGraph } from '../types/repository'

export function adaptGitGraph(
  project: ProjectOpenResult,
  gitGraph: GitGraph | null,
): WorkspaceGraph {
  const nodes: WorkspaceGraphNode[] = [
    {
      id: PROJECT_NODE_ID,
      type: 'project',
      data: { kind: 'project', project },
    },
  ]
  const edges: WorkspaceGraphEdge[] = []

  if (!gitGraph) {
    return { nodes, edges }
  }

  const commitsByHash = new Map(
    gitGraph.commits.map((commit) => [commit.hash, commit]),
  )

  for (const commit of commitsByHash.values()) {
    nodes.push({
      id: commitNodeId(commit.hash),
      type: 'commit',
      data: { kind: 'commit', commit },
    })

    for (const parentHash of commit.parents) {
      if (!commitsByHash.has(parentHash)) {
        continue
      }

      edges.push({
        id: `parent:${parentHash}:${commit.hash}`,
        source: commitNodeId(parentHash),
        target: commitNodeId(commit.hash),
        kind: 'commit-parent',
      })
    }
  }

  const sortedBranches = [...gitGraph.branches].sort((left, right) => {
    if (left.current !== right.current) {
      return left.current ? -1 : 1
    }

    return left.name.localeCompare(right.name)
  })

  for (const branch of sortedBranches) {
    nodes.push({
      id: branchNodeId(branch.name),
      type: 'branch',
      data: { kind: 'branch', branch },
    })

    if (commitsByHash.has(branch.head)) {
      edges.push({
        id: `branch-head:${branch.name}:${branch.head}`,
        source: branchNodeId(branch.name),
        target: commitNodeId(branch.head),
        kind: 'branch-head',
      })
    }
  }

  return { nodes, edges }
}
