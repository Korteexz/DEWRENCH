import { invoke } from '@tauri-apps/api/core'
import type { ProjectOpenResult } from '../types/project'
import type { GitGraph, GitRepositoryDetails } from '../types/repository'

export async function openProject(
  path: string,
): Promise<ProjectOpenResult> {
  return invoke<ProjectOpenResult>('open_project', { path })
}

export async function createRepository(
  path: string,
  branch: string,
  message: string,
): Promise<ProjectOpenResult> {
  return invoke<ProjectOpenResult>('create_repository', {
    path,
    branch,
    message,
  })
}
export async function getRepositoryDetails(
  path: string,
): Promise<GitRepositoryDetails> {
  return invoke<GitRepositoryDetails>('get_repository_details', { path })
}

export async function stageFile(
  path: string,
  file: string,
): Promise<void> {
  return invoke<void>('stage_file', { path, file })
}

export async function unstageFile(
  path: string,
  file: string,
): Promise<void> {
  return invoke<void>('unstage_file', { path, file })
}

export async function createCommit(
  path: string,
  message: string,
): Promise<string> {
  return invoke<string>('create_commit', { path, message })
}

export async function getRepositoryGraph(
  path: string,
): Promise<GitGraph> {
  return invoke<GitGraph>('get_repository_graph', { path })
}

export async function createBranchFrom(
  path: string,
  startPoint: string,
  branchName: string,
): Promise<void> {
  return invoke<void>('create_branch_from', {
    path,
    startPoint,
    branchName,
  })
}

export async function switchBranch(
  path: string,
  branchName: string,
): Promise<void> {
  return invoke<void>('switch_branch', { path, branchName })
}

export async function getCommitDiff(
  path: string,
  revision: string,
): Promise<string> {
  return invoke<string>('get_commit_diff', { path, revision })
}
