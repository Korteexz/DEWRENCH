import { useCallback, useEffect, useRef, useState } from 'react'

import { getActivityStream } from './activityService'
import type { ActivityStream } from './types'

interface UseActivityResult {
  stream: ActivityStream | null
  loading: boolean
  error: string | null
  reload: () => Promise<void>
}

/**
 * Carrega a atividade do projeto.
 *
 * Mesma disciplina de concorrência do `useGitGraph`: um contador de requisição
 * descarta resposta obsoleta, para que trocar de projeto rápido não pinte a
 * tela com dados do projeto anterior.
 */
export function useActivity(projectPath: string, enabled: boolean): UseActivityResult {
  const [stream, setStream] = useState<ActivityStream | null>(null)
  const [loading, setLoading] = useState(enabled)
  const [error, setError] = useState<string | null>(null)
  const requestIdRef = useRef(0)

  const reload = useCallback(async () => {
    if (!enabled) {
      return
    }

    const requestId = ++requestIdRef.current
    setLoading(true)

    try {
      const next = await getActivityStream(projectPath)
      if (requestId === requestIdRef.current) {
        setStream(next)
        setError(null)
      }
    } catch (loadError) {
      if (requestId === requestIdRef.current) {
        setError(
          loadError instanceof Error ? loadError.message : String(loadError),
        )
      }
    } finally {
      if (requestId === requestIdRef.current) {
        setLoading(false)
      }
    }
  }, [enabled, projectPath])

  useEffect(() => {
    void reload()
    return () => {
      requestIdRef.current += 1
    }
  }, [reload])

  return { stream, loading, error, reload }
}
