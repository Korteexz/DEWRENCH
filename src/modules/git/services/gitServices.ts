import { invoke } from '@tauri-apps/api/core'
import type { ProjectOpenResult } from '../types/project'
import type { GitGraph, GitRepositoryDetails } from '../types/repository'
import type { GitRevertOutcome, GitRevertPreview } from '../types/revert'
import type { GitRemotesView } from '../types/remote'
import type { GitBranchComparison } from '../types/compare'
import type {
  GitFetchOutcome,
  GitPullOutcome,
  GitPullPlan,
  GitPushOutcome,
  GitPushPlan,
  PullStrategy,
} from '../types/sync'

export async function openProject(
  path: string,
): Promise<ProjectOpenResult> {
  return invoke<ProjectOpenResult>('open_project', { path })
}

export async function createRepository(
  path: string,
  branch: string,
  message: string,
): Promise<ProjectOpenResult> {
  return invoke<ProjectOpenResult>('create_repository', {
    path,
    branch,
    message,
  })
}
export async function getRepositoryDetails(
  path: string,
): Promise<GitRepositoryDetails> {
  return invoke<GitRepositoryDetails>('get_repository_details', { path })
}

export async function stageFile(
  path: string,
  file: string,
): Promise<void> {
  return invoke<void>('stage_file', { path, file })
}
/**
 * Envia todas as mudanças do working tree para o staging area.
 */
export async function stageAll(
  path: string,
): Promise<void> {
  return invoke<void>('stage_all', { path })
}


export async function unstageFile(
  path: string,
  file: string,
): Promise<void> {
  return invoke<void>('unstage_file', { path, file })
}

export async function createCommit(
  path: string,
  message: string,
): Promise<string> {
  return invoke<string>('create_commit', { path, message })
}

export async function getRepositoryGraph(
  path: string,
): Promise<GitGraph> {
  return invoke<GitGraph>('get_repository_graph', { path })
}

export async function createBranchFrom(
  path: string,
  startPoint: string,
  branchName: string,
): Promise<void> {
  return invoke<void>('create_branch_from', {
    path,
    startPoint,
    branchName,
  })
}

export async function switchBranch(
  path: string,
  branchName: string,
): Promise<void> {
  return invoke<void>('switch_branch', { path, branchName })
}

export async function getCommitDiff(
  path: string,
  revision: string,
): Promise<string> {
  return invoke<string>('get_commit_diff', { path, revision })
}

/**
 * Preview read-only do Revert.
 *
 * Rejeita com `GitOperationError` quando alguma regra bloqueia a operação.
 */
export async function getRevertPreview(
  path: string,
  revision: string,
): Promise<GitRevertPreview> {
  return invoke<GitRevertPreview>('get_revert_preview', { path, revision })
}

/**
 * Cria o commit inverso. O backend revalida todo o preflight antes de mutar.
 */
export async function revertCommit(
  path: string,
  revision: string,
): Promise<GitRevertOutcome> {
  return invoke<GitRevertOutcome>('revert_commit', { path, revision })
}


// ============================================================================
// REMOTES
// ============================================================================

export async function getRemotes(path: string): Promise<GitRemotesView> {
  return invoke<GitRemotesView>('get_remotes', { path })
}

export async function addRemote(
  path: string,
  name: string,
  url: string,
): Promise<void> {
  return invoke<void>('add_remote', { path, name, url })
}

export async function removeRemote(path: string, name: string): Promise<void> {
  return invoke<void>('remove_remote', { path, name })
}

export async function renameRemote(
  path: string,
  from: string,
  to: string,
): Promise<void> {
  return invoke<void>('rename_remote', { path, from, to })
}

/** Troca a URL de um remote. `pushOnly` altera apenas o destino de push. */
export async function setRemoteUrl(
  path: string,
  name: string,
  url: string,
  pushOnly: boolean,
): Promise<void> {
  return invoke<void>('set_remote_url', { path, name, url, pushOnly })
}

// ============================================================================
// PUSH / FETCH / PULL
// ============================================================================

/** Preflight read-only: não toca a rede e não altera nada. */
export async function getPushPlan(
  path: string,
  remoteName?: string,
  sourceBranch?: string,
  targetBranch?: string,
): Promise<GitPushPlan> {
  return invoke<GitPushPlan>('get_push_plan', {
    path,
    remoteName,
    sourceBranch,
    targetBranch,
  })
}

export async function pushBranch(
  path: string,
  remoteName: string | undefined,
  sourceBranch: string | undefined,
  targetBranch: string | undefined,
  setUpstream: boolean,
): Promise<GitPushOutcome> {
  return invoke<GitPushOutcome>('push_branch', {
    path,
    remoteName,
    sourceBranch,
    targetBranch,
    setUpstream,
  })
}

/** Fetch nunca altera o working tree; só atualiza as refs remotas locais. */
export async function fetchRemote(
  path: string,
  remoteName?: string,
  prune = true,
): Promise<GitFetchOutcome> {
  return invoke<GitFetchOutcome>('fetch_remote', { path, remoteName, prune })
}

export async function getPullPlan(
  path: string,
  remoteName?: string,
  remoteBranch?: string,
): Promise<GitPullPlan> {
  return invoke<GitPullPlan>('get_pull_plan', { path, remoteName, remoteBranch })
}

/** A estratégia é sempre explícita: o backend nunca escolhe sozinho. */
export async function pullBranch(
  path: string,
  remoteName: string | undefined,
  remoteBranch: string | undefined,
  strategy: PullStrategy,
): Promise<GitPullOutcome> {
  return invoke<GitPullOutcome>('pull_branch', {
    path,
    remoteName,
    remoteBranch,
    strategy,
  })
}

// ============================================================================
// COMPARE
// ============================================================================

/** Comparação read-only entre duas referências locais. Não fala com a rede. */
export async function getBranchComparison(
  path: string,
  base: string,
  head: string,
): Promise<GitBranchComparison> {
  return invoke<GitBranchComparison>('get_branch_comparison', { path, base, head })
}

/** Diff da mesma comparação, buscado só quando alguém pede para ver. */
export async function getComparisonDiff(
  path: string,
  base: string,
  head: string,
): Promise<string> {
  return invoke<string>('get_comparison_diff', { path, base, head })
}
