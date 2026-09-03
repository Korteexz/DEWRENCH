use std::path::Path;

use crate::modules::git::errors::GitOperationError;

use super::models::{GithubContext, GithubPullRequest};
use super::service;

#[tauri::command]
pub fn get_github_context(path: String) -> Result<GithubContext, GitOperationError> {
    service::get_context(Path::new(&path))
}

#[tauri::command]
pub fn list_pull_requests(
    path: String,
    head_branch: Option<String>,
) -> Result<Vec<GithubPullRequest>, GitOperationError> {
    service::list_pull_requests(Path::new(&path), head_branch.as_deref())
}

#[tauri::command]
pub fn create_pull_request(
    path: String,
    title: String,
    body: String,
    base: Option<String>,
    head: String,
    draft: bool,
) -> Result<String, GitOperationError> {
    service::create_pull_request(
        Path::new(&path),
        &title,
        &body,
        base.as_deref(),
        &head,
        draft,
    )
}

#[tauri::command]
pub fn open_github_in_browser(
    path: String,
    branch: Option<String>,
) -> Result<String, GitOperationError> {
    service::open_in_browser(Path::new(&path), branch.as_deref())
}
