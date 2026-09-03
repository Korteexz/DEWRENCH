/**
 * Contratos da operação de Revert.
 *
 * Os modelos de dados usam snake_case, como os demais DTOs do módulo Git.
 * O erro tipado usa camelCase porque é um contrato novo, definido junto com
 * esta operação.
 */

export interface GitRevertFileChange {
  status: string
  path: string
  original_path: string | null
}

export interface GitRevertPreview {
  hash: string
  short_hash: string
  subject: string
  author: string
  parent_count: number
  is_root_commit: boolean
  affected_files: GitRevertFileChange[]
  preserved_local_changes: string[]
  warnings: string[]
  creates_new_commit: boolean
  preserves_history: boolean
}

export interface GitRevertOutcome {
  reverted_hash: string
  reverted_short_hash: string
  new_commit_hash: string
  new_commit_short_hash: string
  new_commit_subject: string
  affected_files: GitRevertFileChange[]
  warnings: string[]
  history_preserved: boolean
}

export interface GitOperationError {
  code: string
  message: string
  details?: string
  affectedFiles: string[]
  recoverable: boolean
  suggestedAction?: string
}

/** Comandos antigos ainda rejeitam com string; os novos, com erro tipado. */
export type GitFailure = GitOperationError | string

export function isGitOperationError(value: unknown): value is GitOperationError {
  if (typeof value !== 'object' || value === null) {
    return false
  }

  const candidate = value as Partial<GitOperationError>

  return typeof candidate.code === 'string'
    && typeof candidate.message === 'string'
    && Array.isArray(candidate.affectedFiles)
    && typeof candidate.recoverable === 'boolean'
}

/** Normaliza qualquer rejeição preservando o erro tipado quando existir. */
export function toGitFailure(value: unknown): GitFailure {
  if (isGitOperationError(value)) {
    return value
  }

  if (value instanceof Error) {
    return value.message
  }

  return String(value)
}

export function describeFailure(failure: GitFailure): string {
  return typeof failure === 'string' ? failure : failure.message
}
