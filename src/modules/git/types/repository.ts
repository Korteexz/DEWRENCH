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
/**
 * Uma branch, local ou remote-tracking.
 *
 * Os campos de rastreamento só são preenchidos para branches locais; numa
 * remote-tracking, `remote` diz de qual remote ela veio.
 */
export interface GitBranch {
  name: string
  current: boolean
  head: string
  /** `local` ou `remote`. */
  kind: string
  /** Para remote-tracking: o remote de origem. */
  remote: string | null
  /** Para local: a branch remota rastreada, como `origin/main`. */
  upstream: string | null
  ahead: number
  behind: number
  /** Upstream configurado cuja ref não existe mais. */
  gone: boolean
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
  /** Remote-tracking branches, separadas das locais de propósito. */
  remote_branches: GitBranch[]
  commits: GitGraphCommit[]
}