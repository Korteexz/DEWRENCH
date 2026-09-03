use super::errors::GitOperationError;
use super::models::{
    GitGraph,
    GitRepositoryDetails,
    GitRevertOutcome,
    GitRevertPreview,
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
pub fn stage_all(
    path: String,
) -> Result<(), String> {
    service::stage_all(
        &path,
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

#[tauri::command]
pub fn get_revert_preview(
    path: String,
    revision: String,
) -> Result<GitRevertPreview, GitOperationError> {
    service::get_revert_preview(
        &path,
        &revision,
    )
}

#[tauri::command]
pub fn revert_commit(
    path: String,
    revision: String,
) -> Result<GitRevertOutcome, GitOperationError> {
    service::revert_commit(
        &path,
        &revision,
    )
}


// ============================================================================
// REMOTES
// ============================================================================

#[tauri::command]
pub fn get_remotes(
    path: String,
) -> Result<super::models::GitRemotesView, GitOperationError> {
    service::get_remotes(&path)
}

#[tauri::command]
pub fn add_remote(
    path: String,
    name: String,
    url: String,
) -> Result<(), GitOperationError> {
    service::add_remote(&path, &name, &url)
}

#[tauri::command]
pub fn remove_remote(
    path: String,
    name: String,
) -> Result<(), GitOperationError> {
    service::remove_remote(&path, &name)
}

#[tauri::command]
pub fn rename_remote(
    path: String,
    from: String,
    to: String,
) -> Result<(), GitOperationError> {
    service::rename_remote(&path, &from, &to)
}

#[tauri::command]
pub fn set_remote_url(
    path: String,
    name: String,
    url: String,
    push_only: bool,
) -> Result<(), GitOperationError> {
    service::set_remote_url(&path, &name, &url, push_only)
}

// ============================================================================
// PUSH / FETCH / PULL
// ============================================================================

#[tauri::command]
pub fn get_push_plan(
    path: String,
    remote_name: Option<String>,
    source_branch: Option<String>,
    target_branch: Option<String>,
) -> Result<super::models::GitPushPlan, GitOperationError> {
    service::get_push_plan(&path, remote_name, source_branch, target_branch)
}

#[tauri::command]
pub fn push_branch(
    path: String,
    remote_name: Option<String>,
    source_branch: Option<String>,
    target_branch: Option<String>,
    set_upstream: bool,
) -> Result<super::models::GitPushOutcome, GitOperationError> {
    service::push_branch(&path, remote_name, source_branch, target_branch, set_upstream)
}

#[tauri::command]
pub fn fetch_remote(
    path: String,
    remote_name: Option<String>,
    prune: bool,
) -> Result<super::models::GitFetchOutcome, GitOperationError> {
    service::fetch_remote(&path, remote_name, prune)
}

#[tauri::command]
pub fn get_pull_plan(
    path: String,
    remote_name: Option<String>,
    remote_branch: Option<String>,
) -> Result<super::models::GitPullPlan, GitOperationError> {
    service::get_pull_plan(&path, remote_name, remote_branch)
}

#[tauri::command]
pub fn pull_branch(
    path: String,
    remote_name: Option<String>,
    remote_branch: Option<String>,
    strategy: String,
) -> Result<super::models::GitPullOutcome, GitOperationError> {
    service::pull_branch(&path, remote_name, remote_branch, &strategy)
}
