import { invoke } from '@tauri-apps/api/core'

export async function checkGitRepository(path: string): Promise<boolean> {
  return await invoke<boolean>('check_git_repository', { path })
}