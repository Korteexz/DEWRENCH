use std::path::Path;

use crate::core::process::operand;

use super::git_cli;
use super::models::{GitCommit, GitGraphCommit};

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
    // Explícito de propósito: `verify_commit` já recusaria, mas depender disso
    // faz a segurança deste comando morar em outra função — e quem reordenar
    // as chamadas amanhã não teria como saber.
    let revision = operand(revision).map_err(|error| error.to_string())?;

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
    // `rev-parse` e `show` não separam opções de revisão com `--` (em `show`,
    // `--` já significa "daqui em diante são caminhos"). Para revisão, a
    // recusa de valores iniciados por `-` é a barreira, e ela precisa
    // acontecer antes da primeira execução.
    let revision = operand(revision).map_err(|error| error.to_string())?;

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

/// Commits de um intervalo (`a..b`), no formato usado pelo grafo.
///
/// Push e pull precisam mostrar QUAIS commits entram ou saem; reusar o mesmo
/// formato do grafo mantém uma única forma de commit no frontend.
pub fn list_range(
    path: &Path,
    range: &str,
    limit: usize,
) -> Result<Vec<GitGraphCommit>, String> {
    let range = operand(range).map_err(|error| error.to_string())?;
    let limit_arg = format!("-{limit}");

    let output = git_cli::run(
        path,
        &[
            "log",
            &limit_arg,
            "--pretty=format:%H%x1f%h%x1f%P%x1f%an%x1f%s",
            range,
        ],
    )?;

    Ok(parse_commit_lines(&output))
}

/// Converte a saída de `git log --pretty` no modelo de commit do grafo.
pub fn parse_commit_lines(raw: &str) -> Vec<GitGraphCommit> {
    raw.lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(5, '\x1f').collect();

            if parts.len() != 5 {
                return None;
            }

            let parents = if parts[2].is_empty() {
                Vec::new()
            } else {
                parts[2].split_whitespace().map(str::to_string).collect()
            };

            Some(GitGraphCommit {
                hash: parts[0].to_string(),
                short_hash: parts[1].to_string(),
                parents,
                author: parts[3].to_string(),
                message: parts[4].to_string(),
            })
        })
        .collect()
}
