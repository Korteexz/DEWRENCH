use super::models::{
    GitGraph,
    GitRepositoryDetails,
    ProjectOpenResult,
};

use super::service;

#[tauri::command]
pub fn open_project(
    path: String,
) -> Result<ProjectOpenResult, String> {
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
#[tauri::command]
pub fn get_repository_details(
    path: String,
) -> Result<GitRepositoryDetails, String> {
    service::get_repository_details(&path)
}


#[tauri::command]
pub fn create_commit(
    path: String,
    message: String,
) -> Result<String, String> {
    service::create_commit(
        &path,
        &message,
    )
}
#[tauri::command]
pub fn get_repository_graph(
    path: String,
) -> Result<GitGraph, String> {
    service::get_repository_graph(&path)
}
#[tauri::command]
pub fn create_branch_from(
    path: String,
    start_point: String,
    branch_name: String,
) -> Result<(), String> {
    service::create_branch_from(
        &path,
        &start_point,
        &branch_name,
    )
}
#[tauri::command]
pub fn stage_file(
    path: String,
    file: String,
) -> Result<(), String> {
    service::stage_file(
        &path,
        &file,
    )
}
#[tauri::command]
pub fn unstage_file(
    path: String,
    file: String,
) -> Result<(), String> {
    service::unstage_file(
        &path,
        &file,
    )
}
#[tauri::command]
pub fn get_commit_diff(
    path: String,
    revision: String,
) -> Result<String, String> {
    service::get_commit_diff(
        &path,
        &revision,
    )
}


#[tauri::command]
pub fn switch_branch(
    path: String,
    branch_name: String,
) -> Result<(), String> {
    service::switch_branch(
        &path,
        &branch_name,
    )
}