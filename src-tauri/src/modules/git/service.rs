use std::path::{Path, PathBuf};

use crate::core::state;

use super::branches;
use super::commits;
use super::compare;
use super::history;
use super::graph;
use super::repository;
use super::working_tree;

use super::remote;
use super::sync;

use super::errors::GitOperationError;
use super::models::{
    GitBranchComparison,
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


// ============================================================================
// FRONTEIRA DE AUTORIDADE
// ============================================================================
//
// Todo command recebe `path: &str` do frontend. Antes deste ponto, esse path
// ERA a autoridade: qualquer chamada IPC podia apontar para qualquer diretório
// da máquina. Aqui ele deixa de ser credencial e passa a ser apenas uma
// referência que precisa corresponder a um workspace já registrado — o que só
// acontece quando o usuário abre o projeto deliberadamente.
//
// O caminho devolvido é o do REGISTRO, não o recebido: mesmo um path que passe
// na verificação executa contra a raiz canônica conhecida.

fn authority(path: &str) -> Result<PathBuf, String> {
    state::authorize_workspace(path)
        .map(|record| record.scope.root().to_path_buf())
        .map_err(|error| error.to_string())
}

fn authority_typed(path: &str) -> Result<PathBuf, GitOperationError> {
    state::authorize_workspace(path)
        .map(|record| record.scope.root().to_path_buf())
        .map_err(GitOperationError::from)
}

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
    let repository_path = &authority(path)?;

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
    let repository_path = &authority(path)?;

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
    let repository_path = &authority(path)?;

    working_tree::stage_all(
        repository_path,
    )
}
pub fn unstage_file(
    path: &str,
    file: &str,
) -> Result<(), String> {
    let repository_path = &authority(path)?;

    working_tree::unstage_file(
        repository_path,
        file,
    )
}
pub fn create_commit(
    path: &str,
    message: &str,
) -> Result<String, String> {
    let repository_path = &authority(path)?;

    commits::create(
        repository_path,
        message,
    )
}

pub fn get_repository_graph(
    path: &str,
) -> Result<GitGraph, String> {
    let repository_path = &authority(path)?;

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
    let repository_path = &authority(path)?;

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
    let repository_path = &authority(path)?;

    commits::get_diff(
        repository_path,
        revision,
    )
}


pub fn switch_branch(
    path: &str,
    branch_name: &str,
) -> Result<(), String> {
    let repository_path = &authority(path)?;

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
    let repository_path = &authority_typed(path)?;

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
    let repository_path = &authority_typed(path)?;

    history::revert_commit(
        repository_path,
        revision,
    )
}


// ============================================================================
// REMOTES
// ============================================================================

pub fn get_remotes(path: &str) -> Result<GitRemotesView, GitOperationError> {
    remote::get_view(&authority_typed(path)?)
}

pub fn add_remote(path: &str, name: &str, url: &str) -> Result<(), GitOperationError> {
    remote::add(&authority_typed(path)?, name, url)
}

pub fn remove_remote(path: &str, name: &str) -> Result<(), GitOperationError> {
    remote::remove(&authority_typed(path)?, name)
}

pub fn rename_remote(path: &str, from: &str, to: &str) -> Result<(), GitOperationError> {
    remote::rename(&authority_typed(path)?, from, to)
}

pub fn set_remote_url(
    path: &str,
    name: &str,
    url: &str,
    push_only: bool,
) -> Result<(), GitOperationError> {
    remote::set_url(&authority_typed(path)?, name, url, push_only)
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
        &authority_typed(path)?,
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
        &authority_typed(path)?,
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
    sync::fetch(&authority_typed(path)?, remote_name.as_deref(), prune)
}

pub fn get_pull_plan(
    path: &str,
    remote_name: Option<String>,
    remote_branch: Option<String>,
) -> Result<GitPullPlan, GitOperationError> {
    sync::plan_pull(
        &authority_typed(path)?,
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
        &authority_typed(path)?,
        remote_name.as_deref(),
        remote_branch.as_deref(),
        strategy,
    )
}

// ============================================================================
// COMPARE
// ============================================================================

pub fn get_branch_comparison(
    path: &str,
    base: &str,
    head: &str,
) -> Result<GitBranchComparison, GitOperationError> {
    compare::compare(&authority_typed(path)?, base, head)
}

pub fn get_comparison_diff(
    path: &str,
    base: &str,
    head: &str,
) -> Result<String, GitOperationError> {
    compare::diff(&authority_typed(path)?, base, head)
}
