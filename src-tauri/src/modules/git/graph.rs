use std::path::Path;

use super::branches;
use super::git_cli;
use super::models::{
    GitGraph,
    GitGraphCommit,
};

pub fn get(
    path: &Path,
) -> Result<GitGraph, String> {
    Ok(GitGraph {
        branches: branches::get_all(path)?,
        commits: get_commits(path)?,
    })
}

fn get_commits(
    path: &Path,
) -> Result<Vec<GitGraphCommit>, String> {
    let output = git_cli::run(
        path,
        &[
            "log",
            "--all",
            "--topo-order",
            "-80",
            "--pretty=format:%H%x1f%h%x1f%P%x1f%an%x1f%s",
        ],
    )?;

    let commits = output
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> =
                line.splitn(5, '\x1f').collect();

            if parts.len() != 5 {
                return None;
            }

            let parents = if parts[2].is_empty() {
                Vec::new()
            } else {
                parts[2]
                    .split_whitespace()
                    .map(str::to_string)
                    .collect()
            };

            Some(GitGraphCommit {
                hash: parts[0].to_string(),
                short_hash: parts[1].to_string(),
                parents,
                author: parts[3].to_string(),
                message: parts[4].to_string(),
            })
        })
        .collect();

    Ok(commits)
}