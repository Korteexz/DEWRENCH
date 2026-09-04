use std::path::Path;

use crate::modules::git::errors::GitOperationError;

use super::models::{
    GithubContext, GithubMergeOutcome, GithubPullRequest, GithubPullRequestDetail,
    GithubPullRequestPlan,
};
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

#[tauri::command]
pub fn get_pull_request(
    path: String,
    number: u64,
) -> Result<GithubPullRequestDetail, GitOperationError> {
    service::get_pull_request(Path::new(&path), number)
}

#[tauri::command]
pub fn get_pull_request_diff(path: String, number: u64) -> Result<String, GitOperationError> {
    service::get_pull_request_diff(Path::new(&path), number)
}

/// Preflight read-only de merge/close.
#[tauri::command]
pub fn get_pull_request_plan(
    path: String,
    number: u64,
) -> Result<GithubPullRequestPlan, GitOperationError> {
    service::get_pull_request_plan(Path::new(&path), number)
}

/// Executa o merge. O backend revalida o plano antes de mutar.
#[tauri::command]
pub fn merge_pull_request(
    path: String,
    number: u64,
    method: String,
    delete_branch: bool,
    expected_head_sha: Option<String>,
) -> Result<GithubMergeOutcome, GitOperationError> {
    service::merge_pull_request(
        Path::new(&path),
        number,
        &method,
        delete_branch,
        expected_head_sha.as_deref(),
    )
}

#[tauri::command]
pub fn close_pull_request(
    path: String,
    number: u64,
    delete_branch: bool,
    expected_head_sha: Option<String>,
) -> Result<GithubPullRequestDetail, GitOperationError> {
    service::close_pull_request(
        Path::new(&path),
        number,
        delete_branch,
        expected_head_sha.as_deref(),
    )
}
