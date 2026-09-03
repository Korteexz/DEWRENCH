//! Adapter da CLI `gh`.
//!
//! Mesma disciplina do `git_cli`: argumentos separados, nunca string de shell,
//! e o resultado do processo devolvido inteiro para quem decide como
//! classificá-lo. O adapter não sabe o que é um pull request.
//!
//! A criação de processo é do `core::process`. O DEWRENCH continua NÃO lendo,
//! extraindo ou armazenando token: a autenticação vive inteiramente dentro da
//! `gh`, e o que este módulo observa é apenas o exit code de `gh auth status`.

use std::path::Path;

use crate::core::error::CoreError;
use crate::core::process::{self, ProcessRequest, ProgramId, NETWORK_TIMEOUT};

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
    let owned: Vec<String> = args.iter().map(|arg| (*arg).to_string()).collect();

    let request = ProcessRequest::new(ProgramId::Gh, owned, path)
        // Toda chamada útil da `gh` fala com a rede.
        .with_timeout(NETWORK_TIMEOUT)
        // Impede que a gh tente abrir prompt interativo dentro do app.
        .with_env("GH_PROMPT_DISABLED", "1")
        .with_env("GH_NO_UPDATE_NOTIFIER", "1");

    let outcome = process::run(request).map_err(core_error_to_io)?;

    Ok(GhOutput {
        success: outcome.success,
        stdout: outcome.stdout,
        stderr: outcome.stderr,
    })
}

fn core_error_to_io(error: CoreError) -> std::io::Error {
    let kind = match &error {
        CoreError::ExecutionFailed { io_kind, .. } => io_kind.unwrap_or(std::io::ErrorKind::Other),
        CoreError::ExecutionTimeout { .. } => std::io::ErrorKind::TimedOut,
        CoreError::ArgumentRejected { .. } => std::io::ErrorKind::InvalidInput,
        _ => std::io::ErrorKind::PermissionDenied,
    };

    std::io::Error::new(kind, error.to_string())
}

/// A `gh` está instalada nesta máquina?
pub fn is_installed(path: &Path) -> bool {
    run(path, &["--version"]).map(|out| out.success).unwrap_or(false)
}

/// Há sessão autenticada? Não lê nem devolve o token — apenas o estado.
pub fn is_authenticated(path: &Path) -> bool {
    run(path, &["auth", "status"]).map(|out| out.success).unwrap_or(false)
}
