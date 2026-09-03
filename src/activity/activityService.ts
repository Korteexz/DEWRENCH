/**
 * Coleta de atividade.
 *
 * Fica fora de `modules/git` porque a atividade não pertence ao Git: hoje o
 * Git é a única fonte, amanhã haverá outras, e o consumidor não deve mudar.
 */
import { invoke } from '@tauri-apps/api/core'

import type { ActivityStream } from './types'

export async function getActivityStream(
  path: string,
  limit?: number,
): Promise<ActivityStream> {
  return invoke<ActivityStream>('get_activity_stream', { path, limit })
}
