/**
 * Contratos da comparação entre duas referências.
 *
 * O cálculo é do Git local: funciona offline, sem `gh` e sem consumir API.
 * O provider GitHub apenas consome este resultado.
 */
import type { GitGraphCommit } from './repository'

export interface GitComparisonFile {
  path: string
  /** Letra de status do Git: `A`, `M`, `D`, `R100`… */
  status: string
  /** Nulo em arquivo binário — o Git não conta linhas nesse caso. */
  additions: number | null
  deletions: number | null
}

export interface GitBranchComparison {
  base: string
  head: string
  /** Ancestral comum. Nulo quando as histórias não se tocam. */
  merge_base: string | null
  ahead: number
  behind: number
  commits: GitGraphCommit[]
  files: GitComparisonFile[]
  warnings: string[]
  /** Motivo pelo qual a comparação não produz resultado útil. */
  blocked: string | null
}
