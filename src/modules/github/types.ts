/**
 * Contratos do provider GitHub.
 *
 * Tudo aqui é opcional por natureza: o Git funciona inteiro sem GitHub, e a
 * ausência da CLI `gh` é um estado normal, descrito em `limitation`.
 */

export interface GithubContext {
  /** Algum remote aponta para o GitHub. */
  detected: boolean
  /** A CLI `gh` está instalada. */
  cli_available: boolean
  authenticated: boolean
  owner: string | null
  repository: string | null
  remote_name: string | null
  remote_url: string | null
  default_branch: string | null
  current_branch: string | null
  web_url: string | null
  /** O que impede a integração de funcionar agora, em uma linha. */
  limitation: string | null
}

export interface GithubPullRequest {
  number: number
  title: string
  state: string
  is_draft: boolean
  head_branch: string
  base_branch: string
  author: string | null
  url: string
  review_decision: string | null
}

/** Métodos de merge que o backend aceita. Lista fechada, espelhando o Rust. */
export type MergeMethod = 'merge' | 'squash' | 'rebase'

/**
 * Detalhe de um pull request.
 *
 * `mergeable` e `merge_state_status` são a resposta do GitHub, repassada sem
 * interpretação: quem decide se o merge pode acontecer é o preflight do
 * backend, nunca esta interface.
 */
export interface GithubPullRequestDetail {
  number: number
  title: string
  body: string
  state: string
  is_draft: boolean
  head_branch: string
  base_branch: string
  head_sha: string | null
  author: string | null
  url: string
  review_decision: string | null
  mergeable: string | null
  merge_state_status: string | null
  changed_files: number
  additions: number
  deletions: number
  commit_count: number
}

/**
 * Preflight de merge/close.
 *
 * Mesmo contrato de `GitPushPlan`/`GitPullPlan`: enquanto `blocked` não for
 * nulo, a confirmação fica indisponível — e o backend recusa de novo, mesmo
 * que a interface deixasse passar.
 */
export interface GithubPullRequestPlan {
  number: number
  title: string
  state: string
  is_draft: boolean
  head_branch: string
  base_branch: string
  head_sha: string | null
  url: string
  mergeable: string | null
  merge_state_status: string | null
  review_decision: string | null
  available_methods: MergeMethod[]
  recommended_method: MergeMethod | null
  warnings: string[]
  blocked: string | null
}

export interface GithubMergeOutcome {
  number: number
  method: string
  merged: boolean
  deleted_branch: boolean
  url: string
  notes: string[]
}
