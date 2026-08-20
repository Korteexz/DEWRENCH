import { invoke } from '@tauri-apps/api/core'
import type { ProjectOpenResult } from '../types/project'

export async function openProject(
  path: string
): Promise<ProjectOpenResult> {
  return invoke<ProjectOpenResult>('open_project', { path })
}

export async function createRepository(
  path: string,
  branch: string,
  message: string
): Promise<ProjectOpenResult> {
  return invoke<ProjectOpenResult>('create_repository', {
    path,
    branch,
    message,
  })
}