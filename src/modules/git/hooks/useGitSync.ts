import { useCallback, useEffect, useRef, useState } from 'react'

import {
  fetchRemote,
  getPullPlan,
  getPushPlan,
  getRemotes,
  pullBranch,
  pushBranch,
} from '../services/gitServices'
import type { GitRemotesView } from '../types/remote'
import type {
  GitFetchOutcome,
  GitPullOutcome,
  GitPullPlan,
  GitPushOutcome,
  GitPushPlan,
  PullStrategy,
} from '../types/sync'
import { toGitFailure, type GitFailure } from '../types/revert'

/**
 * Estado das operações de rede do módulo Git.
 *
 * A máquina é deliberadamente explícita: `idle` → `plan` → `running` → `done`.
 * Nenhuma operação sai de `idle` direto para `running`, porque nenhuma
 * operação de rede deve acontecer sem o usuário ter visto o plano.
 *
 * Toda leitura vem do backend; nada aqui deduz sucesso. Depois de uma mutação
 * bem-sucedida, `onMutated` relê o repositório inteiro.
 */
export type SyncOperation = 'idle' | 'push' | 'pull' | 'fetch'

export interface UseGitSyncResult {
  remotes: GitRemotesView | null
  remotesLoading: boolean
  selectedRemote: string | null
  selectRemote: (name: string) => void
  operation: SyncOperation
  busy: boolean
  failure: GitFailure | null
  pushPlan: GitPushPlan | null
  pullPlan: GitPullPlan | null
  pushOutcome: GitPushOutcome | null
  pullOutcome: GitPullOutcome | null
  fetchOutcome: GitFetchOutcome | null
  reloadRemotes: () => Promise<void>
  preparePush: (targetBranch?: string) => Promise<void>
  confirmPush: (setUpstream: boolean, targetBranch?: string) => Promise<void>
  preparePull: () => Promise<void>
  confirmPull: (strategy: PullStrategy) => Promise<void>
  runFetch: () => Promise<void>
  dismiss: () => void
}

export function useGitSync(
  projectPath: string,
  onMutated: () => void,
): UseGitSyncResult {
  const [remotes, setRemotes] = useState<GitRemotesView | null>(null)
  const [remotesLoading, setRemotesLoading] = useState(false)
  const [selectedRemote, setSelectedRemote] = useState<string | null>(null)
  const [operation, setOperation] = useState<SyncOperation>('idle')
  const [busy, setBusy] = useState(false)
  const [failure, setFailure] = useState<GitFailure | null>(null)
  const [pushPlan, setPushPlan] = useState<GitPushPlan | null>(null)
  const [pullPlan, setPullPlan] = useState<GitPullPlan | null>(null)
  const [pushOutcome, setPushOutcome] = useState<GitPushOutcome | null>(null)
  const [pullOutcome, setPullOutcome] = useState<GitPullOutcome | null>(null)
  const [fetchOutcome, setFetchOutcome] = useState<GitFetchOutcome | null>(null)

  // Descarta resposta de leitura obsoleta, como o useGitGraph faz.
  const requestIdRef = useRef(0)

  const reloadRemotes = useCallback(async () => {
    const requestId = ++requestIdRef.current
    setRemotesLoading(true)

    try {
      const view = await getRemotes(projectPath)
      if (requestId !== requestIdRef.current) {
        return
      }

      setRemotes(view)
      setSelectedRemote((current) => {
        if (current && view.remotes.some((remote) => remote.name === current)) {
          return current
        }
        return view.default_remote ?? view.remotes[0]?.name ?? null
      })
    } catch (error) {
      if (requestId === requestIdRef.current) {
        setFailure(toGitFailure(error))
      }
    } finally {
      if (requestId === requestIdRef.current) {
        setRemotesLoading(false)
      }
    }
  }, [projectPath])

  useEffect(() => {
    void reloadRemotes()
    return () => {
      requestIdRef.current += 1
    }
  }, [reloadRemotes])

  const dismiss = useCallback(() => {
    setOperation('idle')
    setPushPlan(null)
    setPullPlan(null)
    setPushOutcome(null)
    setPullOutcome(null)
    setFetchOutcome(null)
    setFailure(null)
  }, [])

  const remoteArgument = useCallback(
    () => selectedRemote ?? undefined,
    [selectedRemote],
  )

  const preparePush = useCallback(async (targetBranch?: string) => {
    if (busy) {
      return
    }

    setBusy(true)
    setFailure(null)
    setPushOutcome(null)
    setOperation('push')

    try {
      setPushPlan(await getPushPlan(projectPath, remoteArgument(), undefined, targetBranch))
    } catch (error) {
      setPushPlan(null)
      setFailure(toGitFailure(error))
    } finally {
      setBusy(false)
    }
  }, [busy, projectPath, remoteArgument])

  const confirmPush = useCallback(async (
    setUpstream: boolean,
    targetBranch?: string,
  ) => {
    if (busy) {
      return
    }

    setBusy(true)
    setFailure(null)

    try {
      const outcome = await pushBranch(
        projectPath,
        remoteArgument(),
        undefined,
        targetBranch,
        setUpstream,
      )
      setPushOutcome(outcome)
      setPushPlan(null)
      await reloadRemotes()
      onMutated()
    } catch (error) {
      setFailure(toGitFailure(error))
    } finally {
      setBusy(false)
    }
  }, [busy, onMutated, projectPath, reloadRemotes, remoteArgument])

  const preparePull = useCallback(async () => {
    if (busy) {
      return
    }

    setBusy(true)
    setFailure(null)
    setPullOutcome(null)
    setOperation('pull')

    try {
      setPullPlan(await getPullPlan(projectPath, remoteArgument()))
    } catch (error) {
      setPullPlan(null)
      setFailure(toGitFailure(error))
    } finally {
      setBusy(false)
    }
  }, [busy, projectPath, remoteArgument])

  const confirmPull = useCallback(async (strategy: PullStrategy) => {
    if (busy) {
      return
    }

    setBusy(true)
    setFailure(null)

    try {
      const outcome = await pullBranch(projectPath, remoteArgument(), undefined, strategy)
      setPullOutcome(outcome)
      setPullPlan(null)
      await reloadRemotes()
      onMutated()
    } catch (error) {
      setFailure(toGitFailure(error))
    } finally {
      setBusy(false)
    }
  }, [busy, onMutated, projectPath, reloadRemotes, remoteArgument])

  /**
   * Fetch não tem plano porque não altera nada: ele só atualiza as refs
   * remotas locais. O relatório vem depois, com o que realmente chegou.
   */
  const runFetch = useCallback(async () => {
    if (busy) {
      return
    }

    setBusy(true)
    setFailure(null)
    setOperation('fetch')

    try {
      setFetchOutcome(await fetchRemote(projectPath, remoteArgument(), true))
      await reloadRemotes()
      onMutated()
    } catch (error) {
      setFetchOutcome(null)
      setFailure(toGitFailure(error))
    } finally {
      setBusy(false)
    }
  }, [busy, onMutated, projectPath, reloadRemotes, remoteArgument])

  return {
    remotes,
    remotesLoading,
    selectedRemote,
    selectRemote: setSelectedRemote,
    operation,
    busy,
    failure,
    pushPlan,
    pullPlan,
    pushOutcome,
    pullOutcome,
    fetchOutcome,
    reloadRemotes,
    preparePush,
    confirmPush,
    preparePull,
    confirmPull,
    runFetch,
    dismiss,
  }
}
