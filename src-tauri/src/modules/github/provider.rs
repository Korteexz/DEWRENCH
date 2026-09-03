//! Adapter da CLI `gh`.
//!
//! Mesma disciplina do `git_cli`: argumentos separados, nunca string de shell,
//! e o resultado do processo devolvido inteiro para quem decide como
//! classificá-lo. O adapter não sabe o que é um pull request.

use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct GhOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

/// Executa `gh` no diretório do repositório.
///
/// `Err` significa que o processo não pôde ser iniciado — quase sempre porque
/// a `gh` não está instalada. Exit code diferente de zero é sucesso do ponto de
/// vista do adapter e falha do ponto de vista do domínio.
pub fn run(path: &Path, args: &[&str]) -> Result<GhOutput, std::io::Error> {
    let output = Command::new("gh")
        .args(args)
        .current_dir(path)
        // Impede que a gh tente abrir prompt interativo dentro do app.
        .env("GH_PROMPT_DISABLED", "1")
        .env("GH_NO_UPDATE_NOTIFIER", "1")
        .output()?;

    Ok(GhOutput {
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

/// A `gh` está instalada nesta máquina?
pub fn is_installed(path: &Path) -> bool {
    run(path, &["--version"]).map(|out| out.success).unwrap_or(false)
}

/// Há sessão autenticada? Não lê nem devolve o token — apenas o estado.
pub fn is_authenticated(path: &Path) -> bool {
    run(path, &["auth", "status"]).map(|out| out.success).unwrap_or(false)
}
