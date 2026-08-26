import type { Edge, Node } from '@xyflow/react'

import type { ProjectOpenResult } from '../../modules/git/types/project'
import type {
  GitBranch,
  GitGraphCommit,
} from '../../modules/git/types/repository'

export interface ProjectNodeData extends Record<string, unknown> {
  kind: 'project'
  project: ProjectOpenResult
}

export interface CommitNodeData extends Record<string, unknown> {
  kind: 'commit'
  commit: GitGraphCommit
}

export interface BranchNodeData extends Record<string, unknown> {
  kind: 'branch'
  branch: GitBranch
}

export type WorkspaceNodeData =
  | ProjectNodeData
  | CommitNodeData
  | BranchNodeData

export type ProjectFlowNode = Node<ProjectNodeData, 'project'>
export type CommitFlowNode = Node<CommitNodeData, 'commit'>
export type BranchFlowNode = Node<BranchNodeData, 'branch'>
export type WorkspaceFlowNode =
  | ProjectFlowNode
  | CommitFlowNode
  | BranchFlowNode

export type WorkspaceNodeKind = WorkspaceFlowNode['type']
export type WorkspaceEdgeKind = 'commit-parent' | 'branch-head'

export interface WorkspaceGraphNode {
  id: string
  type: WorkspaceNodeKind
  data: WorkspaceNodeData
}

export interface WorkspaceGraphEdge {
  id: string
  source: string
  target: string
  kind: WorkspaceEdgeKind
}

export interface WorkspaceGraph {
  nodes: WorkspaceGraphNode[]
  edges: WorkspaceGraphEdge[]
}

export interface WorkspaceEdgeData extends Record<string, unknown> {
  kind: WorkspaceEdgeKind
}

export type WorkspaceFlowEdge = Edge<WorkspaceEdgeData>

export interface PositionedWorkspaceGraph {
  nodes: WorkspaceFlowNode[]
  edges: WorkspaceFlowEdge[]
}

export const PROJECT_NODE_ID = 'project:current'

export function commitNodeId(hash: string): string {
  return `commit:${hash}`
}

export function branchNodeId(name: string): string {
  return `branch:${name}`
}
