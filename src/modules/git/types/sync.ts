/**
 * Contratos de push, fetch e pull.
 *
 * Planos são read-only e sempre precedem a execução: nenhuma operação de rede
 * acontece sem o usuário ter visto origem, destino e conteúdo.
 */
import type { GitGraphCommit } from './repository'
import type { GitUpstream } from './remote'

export type PullStrategy = 'fast-forward' | 'merge' | 'rebase'

export interface GitPushPlan {
  remote: string
  remote_exists: boolean
  source_branch: string
  target_branch: string
  upstream: GitUpstream | null
  will_create_upstream: boolean
  remote_branch_exists: boolean
  ahead: number
  behind: number
  diverged: boolean
  commits: GitGraphCommit[]
  warnings: string[]
  /** Motivo pelo qual o push não deve ser executado como está. */
  blocked: string | null
}

export interface GitPushOutcome {
  remote: string
  source_branch: string
  target_branch: string
  pushed_commits: number
  created_upstream: boolean
  created_remote_branch: boolean
  new_remote_hash: string
  details: string[]
}

export interface GitRefUpdate {
  ref_name: string
  old_hash: string | null
  new_hash: string | null
  /** `new`, `updated`, `pruned` ou `forced`. */
  kind: string
  received_commits: number
}

export interface GitFetchOutcome {
  remote: string
  updated_refs: GitRefUpdate[]
  new_branches: string[]
  pruned_branches: string[]
  received_commits: number
  had_changes: boolean
  upstream: GitUpstream | null
}

export interface GitPullPlan {
  remote: string
  branch: string
  upstream: GitUpstream | null
  incoming: GitGraphCommit[]
  outgoing: GitGraphCommit[]
  available_strategies: PullStrategy[]
  recommended_strategy: PullStrategy | ''
  can_fast_forward: boolean
  diverged: boolean
  local_changes: string[]
  /** Arquivos sujos localmente E tocados pelos commits que vão entrar. */
  conflict_risk: string[]
  warnings: string[]
  blocked: string | null
}

export interface GitPullOutcome {
  remote: string
  branch: string
  strategy: PullStrategy
  applied_commits: number
  files_changed: string[]
  previous_head: string
  new_head: string
  fetch: GitFetchOutcome
}

export const STRATEGY_LABEL: Record<PullStrategy, string> = {
  'fast-forward': 'Fast-forward',
  merge: 'Merge',
  rebase: 'Rebase',
}

export const STRATEGY_EXPLANATION: Record<PullStrategy, string> = {
  'fast-forward':
    'Move a branch local para o commit remoto. Só é possível quando você não tem commits próprios ainda não enviados.',
  merge:
    'Cria um commit de junção preservando os dois históricos. Seus commits locais permanecem como estão.',
  rebase:
    'Reaplica seus commits locais sobre os remotos. O histórico fica linear, mas seus commits são reescritos com novos hashes.',
}
