use std::path::Path;

use super::branches;
use super::commits;
use super::graph;
use super::repository;
use super::working_tree;

use super::models::{
    GitGraph,
    GitRepositoryDetails,
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