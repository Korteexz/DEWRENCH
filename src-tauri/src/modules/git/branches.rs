use std::path::Path;

use super::git_cli;
use super::models::GitBranch;

pub fn get_current(
    path: &Path,
) -> Result<String, String> {
    git_cli::run(
        path,
        &["branch", "--show-current"],
    )
}

pub fn get_all(
    path: &Path,
) -> Result<Vec<GitBranch>, String> {
    let output = git_cli::run(
        path,
        &[
            "branch",
            "--format=%(refname:short)%09%(HEAD)%09%(objectname)",
        ],
    )?;

    let branches = output
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('\t').collect();

            if parts.len() != 3 {
                return None;
            }

            Some(GitBranch {
                name: parts[0].to_string(),
                current: parts[1] == "*",
                head: parts[2].to_string(),
            })
        })
        .collect();

    Ok(branches)
}

pub fn create_from(
    path: &Path,
    start_point: &str,
    branch_name: &str,
) -> Result<(), String> {
    let branch_name = branch_name.trim();
    let start_point = start_point.trim();

    if branch_name.is_empty() {
        return Err(
            "O nome da branch não pode estar vazio.".to_string()
        );
    }

    if start_point.is_empty() {
        return Err(
            "O ponto inicial da branch não pode estar vazio.".to_string()
        );
    }

    git_cli::run(
        path,
        &[
            "check-ref-format",
            "--branch",
            branch_name,
        ],
    )?;

    git_cli::run(
        path,
        &[
            "rev-parse",
            "--verify",
            start_point,
        ],
    )?;

    git_cli::run(
        path,
        &[
            "branch",
            branch_name,
            start_point,
        ],
    )?;

    Ok(())
}

pub fn switch(
    path: &Path,
    branch_name: &str,
) -> Result<(), String> {
    let branch_name = branch_name.trim();

    if branch_name.is_empty() {
        return Err(
            "O nome da branch não pode estar vazio.".to_string()
        );
    }

    git_cli::run(
        path,
        &[
            "switch",
            branch_name,
        ],
    )?;

    Ok(())
}