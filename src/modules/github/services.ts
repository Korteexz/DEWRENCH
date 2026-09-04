/**
 * Chamadas IPC do provider GitHub.
 *
 * Todas rejeitam com `GitOperationError`. Ausência da CLI `gh` chega como
 * `PROVIDER_UNAVAILABLE` — é um estado esperado, não uma falha do produto.
 */
import { invoke } from '@tauri-apps/api/core'

import type {
  GithubContext,
  GithubMergeOutcome,
  GithubPullRequest,
  GithubPullRequestDetail,
  GithubPullRequestPlan,
  MergeMethod,
} from './types'

export async function getGithubContext(path: string): Promise<GithubContext> {
  return invoke<GithubContext>('get_github_context', { path })
}

export async function listPullRequests(
  path: string,
  headBranch?: string,
): Promise<GithubPullRequest[]> {
  return invoke<GithubPullRequest[]>('list_pull_requests', { path, headBranch })
}

export async function createPullRequest(
  path: string,
  title: string,
  body: string,
  head: string,
  base: string | null,
  draft: boolean,
): Promise<string> {
  return invoke<string>('create_pull_request', {
    path,
    title,
    body,
    head,
    base,
    draft,
  })
}

/** Abre o repositório no navegador e devolve a URL usada. */
export async function openGithubInBrowser(
  path: string,
  branch?: string,
): Promise<string> {
  return invoke<string>('open_github_in_browser', { path, branch })
}

/** Detalhe de um pull request específico. */
export async function getPullRequest(
  path: string,
  number: number,
): Promise<GithubPullRequestDetail> {
  return invoke<GithubPullRequestDetail>('get_pull_request', { path, number })
}

/** Diff unificado do PR, no formato que `view/diff.ts` já sabe ler. */
export async function getPullRequestDiff(
  path: string,
  number: number,
): Promise<string> {
  return invoke<string>('get_pull_request_diff', { path, number })
}

/** Preflight read-only: não altera nada no GitHub. */
export async function getPullRequestPlan(
  path: string,
  number: number,
): Promise<GithubPullRequestPlan> {
  return invoke<GithubPullRequestPlan>('get_pull_request_plan', { path, number })
}

/**
 * Executa o merge.
 *
 * `expectedHeadSha` é o commit que o usuário revisou: o backend recusa a
 * operação se a branch andou desde então, e repassa a mesma exigência ao
 * GitHub. `deleteBranch` é destrutivo e só é verdadeiro quando pedido.
 */
export async function mergePullRequest(
  path: string,
  number: number,
  method: MergeMethod,
  deleteBranch: boolean,
  expectedHeadSha: string | null,
): Promise<GithubMergeOutcome> {
  return invoke<GithubMergeOutcome>('merge_pull_request', {
    path,
    number,
    method,
    deleteBranch,
    expectedHeadSha,
  })
}

export async function closePullRequest(
  path: string,
  number: number,
  deleteBranch: boolean,
  expectedHeadSha: string | null,
): Promise<GithubPullRequestDetail> {
  return invoke<GithubPullRequestDetail>('close_pull_request', {
    path,
    number,
    deleteBranch,
    expectedHeadSha,
  })
}
