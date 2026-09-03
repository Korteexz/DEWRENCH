export type GitState =
  | 'not_repository'
  | 'unborn_repository'
  | 'repository'

export interface ProjectOpenResult {
  name: string
  path: string
  git_state: GitState
}