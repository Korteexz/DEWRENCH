/**
 * Contratos de remote.
 *
 * Espelham os modelos Rust em snake_case, como os demais DTOs do módulo Git.
 */

export interface GitRemoteIdentity {
  host: string | null
  owner: string | null
  repository: string | null
  /** `github`, `gitlab`, `bitbucket`, `other` ou `unknown`. */
  provider: string
}

export interface GitRemote {
  name: string
  fetch_url: string
  push_url: string
  is_origin: boolean
  /** Este remote é o do upstream da branch atual. */
  is_upstream: boolean
  identity: GitRemoteIdentity
}

export interface GitUpstream {
  remote: string
  branch: string
  /** Nome completo como o Git escreve: `origin/main`. */
  ref_name: string
  ahead: number
  behind: number
  /** Upstream configurado cuja ref não existe mais. */
  gone: boolean
}

export interface GitRemotesView {
  remotes: GitRemote[]
  default_remote: string | null
  current_branch: string | null
  upstream: GitUpstream | null
}

/** Rótulo curto do estado de rastreamento, para leitura de instrumento. */
export function describeTracking(upstream: GitUpstream | null): string {
  if (!upstream) {
    return 'SEM UPSTREAM'
  }

  if (upstream.gone) {
    return 'UPSTREAM AUSENTE'
  }

  if (upstream.ahead > 0 && upstream.behind > 0) {
    return 'DIVERGENTE'
  }

  if (upstream.ahead > 0) {
    return 'À FRENTE'
  }

  if (upstream.behind > 0) {
    return 'ATRÁS'
  }

  return 'SINCRONIZADO'
}
