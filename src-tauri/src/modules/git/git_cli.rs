use std::path::Path;
use std::process::Command;

pub fn run(
    path: &Path,
    args: &[&str],
) -> Result<String, String> {
    Ok(
        run_raw(path, args)?
            .trim()
            .to_string()
    )
}

pub fn run_raw(
    path: &Path,
    args: &[&str],
) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(path)
        .output()
        .map_err(|error| {
            format!("Não foi possível executar Git: {error}")
        })?;

    if !output.status.success() {
        return Err(
            String::from_utf8_lossy(&output.stderr)
                .trim()
                .to_string()
        );
    }

    Ok(
        String::from_utf8_lossy(&output.stdout)
            .into_owned()
    )
}

/// Resultado estruturado de uma execução do Git.
///
/// `run` e `run_raw` descartam informação necessária para classificar falhas:
/// o exit code, o stdout de execuções malsucedidas e o stderr de execuções bem
/// sucedidas. Operações que precisam distinguir conflito de erro — como o
/// Revert — usam `run_structured`. A extensão é aditiva: os consumidores
/// existentes continuam usando `run`/`run_raw` sem alteração.
#[derive(Debug, Clone)]
pub struct GitCommandOutput {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

/// Executa o Git preservando todo o resultado do processo.
///
/// O erro devolvido representa apenas falha ao iniciar o processo (Git ausente,
/// permissão negada). Um comando que executou e terminou com exit code
/// diferente de zero é sucesso do ponto de vista do adapter e falha do ponto de
/// vista do domínio, que decide como classificá-la.
pub fn run_structured(
    path: &Path,
    args: &[&str],
) -> Result<GitCommandOutput, std::io::Error> {
    let output = Command::new("git")
        .args(args)
        .current_dir(path)
        .output()?;

    Ok(GitCommandOutput {
        success: output.status.success(),
        exit_code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}
