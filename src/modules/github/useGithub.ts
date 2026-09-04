import { useCallback, useEffect, useState } from 'react'

import { toGitFailure, type GitFailure } from '../git/types/revert'
import {
  createPullRequest,
  getGithubContext,
  listPullRequests,
} from './services'
import type { GithubContext, GithubPullRequest } from './types'

export interface UseGithubResult {
  context: GithubContext | null
  pullRequests: GithubPullRequest[]
  loading: boolean
  busy: boolean
  failure: GitFailure | null
  /** URL do último pull request criado, para confirmação visual. */
  createdUrl: string | null
  reload: () => Promise<void>
  /** Devolve `true` quando o PR foi criado; a lista é recarregada em seguida. */
  create: (
    title: string,
    body: string,
    head: string,
    base: string | null,
    draft: boolean,
  ) => Promise<boolean>
  dismiss: () => void
}

/**
 * Estado do provider GitHub para o projeto aberto.
 *
 * Mesma divisão de trabalho do `useGitSync`: o hook concentra as chamadas IPC e
 * o estado, e os componentes só apresentam. Nenhuma decisão de permissão é
 * tomada aqui — o que o backend devolve é o que vale.
 *
 * A lista de pull requests é lida sem filtro de branch: a interface passou a
 * ter uma tela de PR e comparar branches, então esconder os PRs das outras
 * branches deixaria a lista mentindo sobre o repositório.
 */
export function useGithub(projectPath: string): UseGithubResult {
  const [context, setContext] = useState<GithubContext | null>(null)
  const [pullRequests, setPullRequests] = useState<GithubPullRequest[]>([])
  const [loading, setLoading] = useState(false)
  const [busy, setBusy] = useState(false)
  const [failure, setFailure] = useState<GitFailure | null>(null)
  const [createdUrl, setCreatedUrl] = useState<string | null>(null)

  const reload = useCallback(async () => {
    setLoading(true)
    setFailure(null)

    try {
      const next = await getGithubContext(projectPath)
      setContext(next)

      if (next.authenticated) {
        try {
          setPullRequests(await listPullRequests(projectPath))
        } catch (error) {
          // Falha ao listar não invalida o contexto já lido.
          setFailure(toGitFailure(error))
        }
      } else {
        setPullRequests([])
      }
    } catch (error) {
      setContext(null)
      setPullRequests([])
      setFailure(toGitFailure(error))
    } finally {
      setLoading(false)
    }
  }, [projectPath])

  useEffect(() => {
    void reload()
  }, [reload])

  const create = useCallback(
    async (
      title: string,
      body: string,
      head: string,
      base: string | null,
      draft: boolean,
    ): Promise<boolean> => {
      setBusy(true)
      setFailure(null)
      setCreatedUrl(null)

      try {
        const url = await createPullRequest(projectPath, title, body, head, base, draft)
        setCreatedUrl(url)
        await reload()
        return true
      } catch (error) {
        setFailure(toGitFailure(error))
        return false
      } finally {
        setBusy(false)
      }
    },
    [projectPath, reload],
  )

  const dismiss = useCallback(() => {
    setFailure(null)
    setCreatedUrl(null)
  }, [])

  return {
    context,
    pullRequests,
    loading,
    busy,
    failure,
    createdUrl,
    reload,
    create,
    dismiss,
  }
}
