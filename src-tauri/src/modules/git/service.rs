use std::path::Path;
use std::process::Command;



pub fn open_project(path: &str) -> Result<ProjectOpenResult, String> {
    let project_path = Path::new(path);

    if !project_path.exists() {
        return Err("O caminho informado não existe.".to_string());
    }

    if !project_path.is_dir() {
        return Err("O caminho informado não é um diretório.".to_string());
    }

    let canonical_path = project_path
        .canonicalize()
        .map_err(|error| error.to_string())?;

    let project_name = canonical_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Projeto")
        .to_string();

    let git_directory = canonical_path.join(".git");

    let git_state = if !git_directory.exists() {
        GitState::NotRepository
    } else {
        detect_existing_repository_state(&canonical_path)?
    };

    Ok(ProjectOpenResult {
        name: project_name,
        path: canonical_path.to_string_lossy().to_string(),
        git_state,
    })
}

fn detect_existing_repository_state(path: &Path) -> Result<GitState, String> {
    let output = Command::new("git")
        .args(["rev-parse", "--verify", "HEAD"])
        .current_dir(path)
        .output()
        .map_err(|error| format!("Não foi possível executar o Git: {error}"))?;

    if output.status.success() {
        Ok(GitState::Repository)
    } else {
        Ok(GitState::UnbornRepository)
    }
}

fn run_git(path: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(path)
        .output()
        .map_err(|error| format!("Não foi possível executar Git: {error}"))?;

    if !output.status.success() {
        return Err(
            String::from_utf8_lossy(&output.stderr)
                .trim()
                .to_string()
        );
    }

    Ok(
        String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string()
    )
}

fn validate_branch_name(
    path: &Path,
    branch: &str,
) -> Result<(), String> {
    run_git(
        path,
        &["check-ref-format", "--branch", branch],
    )?;

    Ok(())
}

pub fn create_repository(
    path: &str,
    branch: &str,
    message: &str,
) -> Result<ProjectOpenResult, String> {
    let project_path = Path::new(path);

    if !project_path.exists() {
        return Err("O projeto não existe.".to_string());
    }

    if !project_path.is_dir() {
        return Err("O caminho não é um diretório.".to_string());
    }

    if project_path.join(".git").exists() {
        return Err(
            "Este projeto já possui um repositório Git.".to_string()
        );
    }

    if branch.trim().is_empty() {
        return Err(
            "O nome da branch não pode estar vazio.".to_string()
        );
    }

    if message.trim().is_empty() {
        return Err(
            "A mensagem do commit não pode estar vazia.".to_string()
        );
    }

    validate_branch_name(project_path, branch)?;

    run_git(
        project_path,
        &["init", "-b", branch],
    )?;

    run_git(
        project_path,
        &["add", "."],
    )?;

    run_git(
        project_path,
        &["commit", "-m", message],
    )?;

    open_project(path)
}

use super::git_cli;
use super::models::{
    GitBranch,
    GitCommit,
    GitFileStatus,
    GitGraph,
    GitGraphCommit,
    GitRepositoryDetails,
    GitState,
    ProjectOpenResult,
};
fn get_current_branch(
    path: &Path,
) -> Result<String, String> {
    git_cli::run(
        path,
        &["branch", "--show-current"],
    )
}
fn get_file_status(
    path: &Path,
) -> Result<Vec<GitFileStatus>, String> {
    let output = git_cli::run(
        path,
        &["status", "--porcelain"],
    )?;

    let files = output
        .lines()
        .filter(|line| line.len() >= 3)
        .map(|line| {
            let index_status =
                line.chars().nth(0).unwrap_or(' ');

            let worktree_status =
                line.chars().nth(1).unwrap_or(' ');

            let file_path =
                line[3..].to_string();

            GitFileStatus {
                path: file_path,
                index_status: index_status.to_string(),
                worktree_status: worktree_status.to_string(),
            }
        })
        .collect();

    Ok(files)
}
fn get_recent_commits(
    path: &Path,
) -> Result<Vec<GitCommit>, String> {
    let output = git_cli::run(
        path,
        &[
            "log",
            "-10",
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
        branch: get_current_branch(repository_path)?,
        files: get_file_status(repository_path)?,
        commits: get_recent_commits(repository_path)?,
    })
}
pub fn stage_file(
    path: &str,
    file: &str,
) -> Result<(), String> {
    let repository_path = Path::new(path);

    git_cli::run(
        repository_path,
        &["add", "--", file],
    )?;

    Ok(())
}
pub fn create_commit(
    path: &str,
    message: &str,
) -> Result<String, String> {
    if message.trim().is_empty() {
        return Err(
            "A mensagem do commit não pode estar vazia."
                .to_string()
        );
    }

    let repository_path = Path::new(path);

    git_cli::run(
        repository_path,
        &[
            "commit",
            "-m",
            message.trim(),
        ],
    )
}
fn get_branches(
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
            let parts: Vec<&str> =
                line.split('\t').collect();

            if parts.len() != 3 {
                return None;
            }

            Some(GitBranch {
                name: parts[0].to_string(),
                current: parts[1].trim() == "*",
                head: parts[2].to_string(),
            })
        })
        .collect();

    Ok(branches)
}
fn get_graph_commits(
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
                line.split('\x1f').collect();

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

    Ok(GitGraph {
        branches: get_branches(repository_path)?,
        commits: get_graph_commits(repository_path)?,
    })
}
pub fn create_branch_from(
    path: &str,
    from_branch: &str,
    new_branch: &str,
) -> Result<(), String> {
    let repository_path = Path::new(path);

    if new_branch.trim().is_empty() {
        return Err(
            "O nome da nova branch não pode estar vazio."
                .to_string(),
        );
    }

    git_cli::run(
        repository_path,
        &[
            "check-ref-format",
            "--branch",
            new_branch,
        ],
    )?;

    git_cli::run(
        repository_path,
        &[
            "branch",
            new_branch,
            from_branch,
        ],
    )?;

    Ok(())
}