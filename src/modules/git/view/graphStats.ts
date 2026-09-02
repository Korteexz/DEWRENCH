import type { GitGraph } from '../types/repository'

/**
 * Estatísticas derivadas do grafo Git carregado.
 *
 * Toda leitura exibida na instrumentação precisa vir daqui — se um número não
 * puder ser derivado do estado real, ele não é exibido.
 */
export interface GraphStats {
  commitCount: number
  branchCount: number
  mergeCount: number
  rootCount: number
  currentBranch: string | null
  headHash: string | null
}

export function summarizeGraph(graph: GitGraph | null): GraphStats {
  if (!graph) {
    return {
      commitCount: 0,
      branchCount: 0,
      mergeCount: 0,
      rootCount: 0,
      currentBranch: null,
      headHash: null,
    }
  }

  const current = graph.branches.find((branch) => branch.current) ?? null

  return {
    commitCount: graph.commits.length,
    branchCount: graph.branches.length,
    mergeCount: graph.commits.filter((c) => c.parents.length > 1).length,
    rootCount: graph.commits.filter((c) => c.parents.length === 0).length,
    currentBranch: current?.name ?? null,
    headHash: current?.head ?? null,
  }
}
