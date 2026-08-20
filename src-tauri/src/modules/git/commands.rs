use super::models::ProjectOpenResult;
use super::service;

#[tauri::command]
pub fn open_project(path: String) -> Result<ProjectOpenResult, String> {
    service::open_project(&path)
}
#[tauri::command]
pub fn create_repository(
    path: String,
    branch: String,
    message: String,
) -> Result<ProjectOpenResult, String> {
    service::create_repository(
        &path,
        &branch,
        &message,
    )
}