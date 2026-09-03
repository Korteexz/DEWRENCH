import type { GitFileStatus } from '../types/repository'

/**
 * View-models do working tree.
 *
 * Regra Git NÃO pertence a componente de apresentação. Estas funções traduzem
 * os códigos de status porcelain vindos do backend em conceitos que a interface
 * consegue exibir, e existem separadas para poderem ser testadas sem React.
 *
 * Códigos: primeiro caractere = index (staging area), segundo = working tree.
 * '?' representa arquivo não rastreado; '!' representa ignorado.
 */

export function isStaged(file: GitFileStatus): boolean {
  return ![' ', '?', '!'].includes(file.index_status)
}

export function hasUnstagedChanges(file: GitFileStatus): boolean {
  return ![' ', '!'].includes(file.worktree_status)
}

export function isUntracked(file: GitFileStatus): boolean {
  return file.index_status === '?' || file.worktree_status === '?'
}

/** Rótulo de duas colunas; espaço vira '·' para a coluna nunca colapsar. */
export function statusLabel(file: GitFileStatus): string {
  return `${file.index_status}${file.worktree_status}`.replaceAll(' ', '·')
}

export interface WorkingTreeSummary {
  total: number
  staged: number
  unstaged: number
  untracked: number
  clean: boolean
}

export function summarizeWorkingTree(
  files: GitFileStatus[] | undefined,
): WorkingTreeSummary {
  const list = files ?? []

  return {
    total: list.length,
    staged: list.filter(isStaged).length,
    unstaged: list.filter(hasUnstagedChanges).length,
    untracked: list.filter(isUntracked).length,
    clean: list.length === 0,
  }
}
