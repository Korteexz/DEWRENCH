use std::path::Path;

use super::git_cli;
use super::models::GitCommit;

pub fn get_recent(
    path: &Path,
    limit: usize,
) -> Result<Vec<GitCommit>, String> {
    let limit_argument = format!("-{limit}");

    let output = git_cli::run(
        path,
        &[
            "log",
            limit_argument.as_str(),
            "--pretty=format:%h%x1f%an%x1f%s",
        ],
    )?;

    let commits = output
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> =
                line.split('\x1f').collect();

            if parts.len() != 3 {
                return None;
            }

            Some(GitCommit {
                hash: parts[0].to_string(),
                author: parts[1].to_string(),
                message: parts[2].to_string(),
            })
        })
        .collect();

    Ok(commits)
}

pub fn create(
    path: &Path,
    message: &str,
) -> Result<String, String> {
    let message = message.trim();

    if message.is_empty() {
        return Err(
            "A mensagem do commit não pode estar vazia."
                .to_string(),
        );
    }

    git_cli::run(
        path,
        &[
            "commit",
            "-m",
            message,
        ],
    )
}
pub fn get_diff(
    path: &Path,
    revision: &str,
) -> Result<String, String> {
    verify_commit(path, revision)?;

    git_cli::run(
        path,
        &[
            "show",
            "--format=",
            "--no-ext-diff",
            "--unified=3",
            revision,
            "--",
        ],
    )
}


fn verify_commit(
    path: &Path,
    revision: &str,
) -> Result<(), String> {
    let revision_expression =
        format!("{revision}^{{commit}}");

    git_cli::run(
        path,
        &[
            "rev-parse",
            "--verify",
            revision_expression.as_str(),
        ],
    )?;

    Ok(())
}