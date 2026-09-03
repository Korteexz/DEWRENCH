use std::path::Path;

use super::branches;
use super::commits;
use super::history;
use super::graph;
use super::repository;
use super::working_tree;

use super::remote;
use super::sync;

use super::errors::GitOperationError;
use super::models::{
    GitFetchOutcome,
    GitGraph,
    GitPullOutcome,
    GitPullPlan,
    GitPushOutcome,
    GitPushPlan,
    GitRemotesView,
    GitRepositoryDetails,
    GitRevertOutcome,
    GitRevertPreview,
    ProjectOpenResult,
};

pub fn open_project(
    path: &str,
) -> Result<ProjectOpenResult, String> {
    repository::open(
        Path::new(path),
    )
}


pub fn create_repository(
    path: &str,
    branch: &str,
    message: &str,
) -> Result<ProjectOpenResult, String> {
    repository::create(
        Path::new(path),
        branch,
        message,
    )
}





pub fn get_repository_details(
    path: &str,
) -> Result<GitRepositoryDetails, String> {
    let repository_path = Path::new(path);

    if !repository_path.join(".git").exists() {
        return Err(
            "Este projeto não possui repositório Git."
                .to_string()
        );
    }

    Ok(GitRepositoryDetails {
    branch: branches::get_current(repository_path)?,
    files: working_tree::get_status(repository_path)?,
    commits: commits::get_recent(repository_path, 10)?,
})
}
pub fn stage_file(
    path: &str,
    file: &str,
) -> Result<(), String> {
    let repository_path = Path::new(path);

    working_tree::stage_file(
        repository_path,
        file,
    )
}
/// Faz stage de todas as alterações do repositório.
///
/// A camada service recebe strings vindas da interface
/// e converte o caminho para Path antes de chamar
/// a implementação Git propriamente dita.
pub fn stage_all(
    path: &str,
) -> Result<(), String> {
    let repository_path = Path::new(path);

    working_tree::stage_all(
        repository_path,
    )
}
pub fn unstage_file(
    path: &str,
    file: &str,
) -> Result<(), String> {
    let repository_path = Path::new(path);

    working_tree::unstage_file(
        repository_path,
        file,
    )
}
pub fn create_commit(
    path: &str,
    message: &str,
) -> Result<String, String> {
    let repository_path = Path::new(path);

    commits::create(
        repository_path,
        message,
    )
}

pub fn get_repository_graph(
    path: &str,
) -> Result<GitGraph, String> {
    let repository_path = Path::new(path);

    if !repository_path.join(".git").exists() {
        return Err(
            "Este projeto não possui repositório Git."
                .to_string(),
        );
    }

    graph::get(repository_path)
}
pub fn create_branch_from(
    path: &str,
    start_point: &str,
    branch_name: &str,
) -> Result<(), String> {
    let repository_path = Path::new(path);

    branches::create_from(
        repository_path,
        start_point,
        branch_name,
    )
}
pub fn get_commit_diff(
    path: &str,
    revision: &str,
) -> Result<String, String> {
    let repository_path = Path::new(path);

    commits::get_diff(
        repository_path,
        revision,
    )
}


pub fn switch_branch(
    path: &str,
    branch_name: &str,
) -> Result<(), String> {
    let repository_path = Path::new(path);

    branches::switch(
        repository_path,
        branch_name,
    )
}

/// Preview read-only do Revert.
///
/// Não altera o repositório; existe para que a confirmação da interface
/// descreva consequências reais em vez de perguntar apenas "tem certeza?".
pub fn get_revert_preview(
    path: &str,
    revision: &str,
) -> Result<GitRevertPreview, GitOperationError> {
    let repository_path = Path::new(path);

    history::get_revert_preview(
        repository_path,
        revision,
    )
}

/// Executa o Revert revalidando todo o preflight antes da mutação.
pub fn revert_commit(
    path: &str,
    revision: &str,
) -> Result<GitRevertOutcome, GitOperationError> {
    let repository_path = Path::new(path);

    history::revert_commit(
        repository_path,
        revision,
    )
}


// ============================================================================
// REMOTES
// ============================================================================

pub fn get_remotes(path: &str) -> Result<GitRemotesView, GitOperationError> {
    remote::get_view(Path::new(path))
}

pub fn add_remote(path: &str, name: &str, url: &str) -> Result<(), GitOperationError> {
    remote::add(Path::new(path), name, url)
}

pub fn remove_remote(path: &str, name: &str) -> Result<(), GitOperationError> {
    remote::remove(Path::new(path), name)
}

pub fn rename_remote(path: &str, from: &str, to: &str) -> Result<(), GitOperationError> {
    remote::rename(Path::new(path), from, to)
}

pub fn set_remote_url(
    path: &str,
    name: &str,
    url: &str,
    push_only: bool,
) -> Result<(), GitOperationError> {
    remote::set_url(Path::new(path), name, url, push_only)
}

// ============================================================================
// PUSH / FETCH / PULL
// ============================================================================

pub fn get_push_plan(
    path: &str,
    remote_name: Option<String>,
    source_branch: Option<String>,
    target_branch: Option<String>,
) -> Result<GitPushPlan, GitOperationError> {
    sync::plan_push(
        Path::new(path),
        remote_name.as_deref(),
        source_branch.as_deref(),
        target_branch.as_deref(),
    )
}

pub fn push_branch(
    path: &str,
    remote_name: Option<String>,
    source_branch: Option<String>,
    target_branch: Option<String>,
    set_upstream: bool,
) -> Result<GitPushOutcome, GitOperationError> {
    sync::push(
        Path::new(path),
        remote_name.as_deref(),
        source_branch.as_deref(),
        target_branch.as_deref(),
        set_upstream,
    )
}

pub fn fetch_remote(
    path: &str,
    remote_name: Option<String>,
    prune: bool,
) -> Result<GitFetchOutcome, GitOperationError> {
    sync::fetch(Path::new(path), remote_name.as_deref(), prune)
}

pub fn get_pull_plan(
    path: &str,
    remote_name: Option<String>,
    remote_branch: Option<String>,
) -> Result<GitPullPlan, GitOperationError> {
    sync::plan_pull(
        Path::new(path),
        remote_name.as_deref(),
        remote_branch.as_deref(),
    )
}

pub fn pull_branch(
    path: &str,
    remote_name: Option<String>,
    remote_branch: Option<String>,
    strategy: &str,
) -> Result<GitPullOutcome, GitOperationError> {
    sync::pull(
        Path::new(path),
        remote_name.as_deref(),
        remote_branch.as_deref(),
        strategy,
    )
}
