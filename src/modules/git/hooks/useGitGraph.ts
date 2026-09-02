import { useCallback, useEffect, useRef, useState } from 'react'

import { getRepositoryDetails, getRepositoryGraph } from '../services/gitServices'
import type { GitGraph, GitRepositoryDetails } from '../types/repository'

interface UseGitGraphResult {
  repositoryDetails: GitRepositoryDetails | null
  gitGraph: GitGraph | null
  loading: boolean
  error: string | null
  refresh: () => Promise<void>
}

function getErrorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

export function useGitGraph(
  projectPath: string,
  enabled = true,
): UseGitGraphResult {
  const [repositoryDetails, setRepositoryDetails] =
    useState<GitRepositoryDetails | null>(null)
  const [gitGraph, setGitGraph] = useState<GitGraph | null>(null)
  const [loading, setLoading] = useState(enabled)
  const [error, setError] = useState<string | null>(null)
  const requestIdRef = useRef(0)

  const refresh = useCallback(async () => {
    if (!enabled) {
      return
    }

    const requestId = ++requestIdRef.current
    setLoading(true)

    try {
      const [details, graph] = await Promise.all([
        getRepositoryDetails(projectPath),
        getRepositoryGraph(projectPath),
      ])

      if (requestId === requestIdRef.current) {
        setRepositoryDetails(details)
        setGitGraph(graph)
        setError(null)
      }
    } catch (refreshError) {
      if (requestId === requestIdRef.current) {
        setError(getErrorMessage(refreshError))
      }

      throw refreshError
    } finally {
      if (requestId === requestIdRef.current) {
        setLoading(false)
      }
    }
  }, [enabled, projectPath])

  useEffect(() => {
    const timeoutId = window.setTimeout(() => {
      void refresh().catch(() => undefined)
    }, 0)

    return () => {
      window.clearTimeout(timeoutId)
      requestIdRef.current += 1
    }
  }, [refresh])

  return {
    repositoryDetails,
    gitGraph,
    loading,
    error,
    refresh,
  }
}
