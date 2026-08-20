use std::path::Path;
use std::process::Command;

use super::models::{GitState, ProjectOpenResult};

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