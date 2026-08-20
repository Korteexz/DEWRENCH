use super::service;

#[tauri::command]
pub fn check_git_repository(path: String) -> bool {
    service::is_git_repository(&path)
}