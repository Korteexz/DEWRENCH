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
