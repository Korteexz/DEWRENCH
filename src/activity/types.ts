/**
 * Modelo de atividade — deliberadamente agnóstico de ferramenta.
 *
 * Esta pasta fica FORA de `modules/git` de propósito. A Temporal Matrix
 * consome daqui, e nada aqui sabe o que é um commit, um container ou um
 * pipeline: quando Docker, CI/CD ou colaboração entre máquinas existirem, eles
 * passam a produzir os mesmos eventos, e a visualização não muda.
 */

export interface ActivityEvent {
  id: string
  /** Epoch em segundos, UTC. */
  timestamp: number
  /** Fuso de quem gerou o evento, em minutos. */
  utc_offset_minutes: number
  /** `git` hoje; `docker`, `ci`, `agent` depois. */
  source: string
  /** Máquina de origem; nulo quando local. */
  machine: string | null
  actor: string | null
  module: string
  /** Tipo dentro da fonte: `commit`, `merge`, `revert`, `root`… */
  kind: string
  repository: string
  branch: string | null
  metadata: Record<string, string>
}

export interface ActivityStream {
  events: ActivityEvent[]
  sources: string[]
  truncated: boolean
}
