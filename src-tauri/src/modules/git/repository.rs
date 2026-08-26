use std::path::Path;

use super::git_cli;
use super::models::{
    GitState,
    ProjectOpenResult,
};

pub fn open(
    path: &Path,
) -> Result<ProjectOpenResult, String> {
    if !path.exists() {
        return Err(
            "O caminho informado não existe.".to_string()
        );
    }

    if !path.is_dir() {
        return Err(
            "O caminho informado não é um diretório.".to_string()
        );
    }

    let canonical_path = path
        .canonicalize()
        .map_err(|error| {
            format!(
                "Não foi possível resolver o caminho do projeto: {error}"
            )
        })?;

    let name = canonical_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            "Não foi possível determinar o nome do projeto."
                .to_string()
        })?
        .to_string();

    let git_state = detect_git_state(
        &canonical_path,
    );

    Ok(ProjectOpenResult {
        name,
        path: canonical_path
            .to_string_lossy()
            .into_owned(),
        git_state,
    })
}

pub fn create(
    path: &Path,
    branch: &str,
    message: &str,
) -> Result<ProjectOpenResult, String> {
    if !path.exists() {
        return Err(
            "O caminho informado não existe.".to_string()
        );
    }

    if !path.is_dir() {
        return Err(
            "O caminho informado não é um diretório.".to_string()
        );
    }

    if path.join(".git").exists() {
        return Err(
            "Este diretório já possui um repositório Git."
                .to_string()
        );
    }

    let branch = branch.trim();
    let message = message.trim();

    if branch.is_empty() {
        return Err(
            "O nome da branch inicial não pode estar vazio."
                .to_string()
        );
    }

    if message.is_empty() {
        return Err(
            "A mensagem do commit inicial não pode estar vazia."
                .to_string()
        );
    }

    git_cli::run(
        path,
        &[
            "check-ref-format",
            "--branch",
            branch,
        ],
    )?;

    git_cli::run(
        path,
        &[
            "init",
            "-b",
            branch,
        ],
    )?;

    git_cli::run(
        path,
        &[
            "add",
            ".",
        ],
    )?;

    git_cli::run(
        path,
        &[
            "commit",
            "-m",
            message,
        ],
    )?;

    open(path)
}

fn detect_git_state(
    path: &Path,
) -> GitState {
    if !path.join(".git").exists() {
        return GitState::NotRepository;
    }

    match git_cli::run(
        path,
        &[
            "rev-parse",
            "--verify",
            "HEAD",
        ],
    ) {
        Ok(_) => GitState::Repository,
        Err(_) => GitState::UnbornRepository,
    }
}