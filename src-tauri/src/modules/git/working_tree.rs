use std::path::Path;

use super::git_cli;
use super::models::GitFileStatus;

pub fn get_status(path: &Path) -> Result<Vec<GitFileStatus>, String> {
    let output = git_cli::run(
        path,
        &["status", "--porcelain=v1"],
    )?;

    let files = output
        .lines()
        .filter(|line| line.len() >= 3)
        .map(|line| {
            let index_status = line
                .chars()
                .nth(0)
                .unwrap_or(' ');

            let worktree_status = line
                .chars()
                .nth(1)
                .unwrap_or(' ');

            let file_path = line[3..].to_string();

            GitFileStatus {
                path: file_path,
                index_status: index_status.to_string(),
                worktree_status: worktree_status.to_string(),
            }
        })
        .collect();

    Ok(files)
}

pub fn stage_file(
    path: &Path,
    file: &str,
) -> Result<(), String> {
    git_cli::run(
        path,
        &["add", "--", file],
    )?;

    Ok(())
}

pub fn unstage_file(
    path: &Path,
    file: &str,
) -> Result<(), String> {
    git_cli::run(
        path,
        &["restore", "--staged", "--", file],
    )?;

    Ok(())
}