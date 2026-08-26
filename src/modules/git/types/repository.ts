export interface GitFileStatus {
  path: string
  index_status: string
  worktree_status: string
}

export interface GitCommit {
  hash: string
  message: string
  author: string
}

export interface GitRepositoryDetails {
  branch: string
  files: GitFileStatus[]
  commits: GitCommit[]
}
export interface GitBranch {
  name: string
  current: boolean
  head: string
}

export interface GitGraphCommit {
  hash: string
  short_hash: string
  message: string
  author: string
  parents: string[]
}

export interface GitGraph {
  branches: GitBranch[]
  commits: GitGraphCommit[]
}