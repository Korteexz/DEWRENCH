use super::models::ProjectOpenResult;
use super::service;

#[tauri::command]
pub fn open_project(path: String) -> Result<ProjectOpenResult, String> {
    service::open_project(&path)
}