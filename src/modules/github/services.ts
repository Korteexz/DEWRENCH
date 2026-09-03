/**
 * Chamadas IPC do provider GitHub.
 *
 * Todas rejeitam com `GitOperationError`. Ausência da CLI `gh` chega como
 * `PROVIDER_UNAVAILABLE` — é um estado esperado, não uma falha do produto.
 */
import { invoke } from '@tauri-apps/api/core'

import type { GithubContext, GithubPullRequest } from './types'

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
