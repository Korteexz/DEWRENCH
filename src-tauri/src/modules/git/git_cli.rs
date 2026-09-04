//! Adapter da CLI do Git.
//!
//! Este arquivo NÃO cria processo. Ele descreve a intenção — "git com estes
//! argumentos, neste diretório" — e delega a criação ao `core::process`, que é
//! a única porta de execução do DEWRENCH.
//!
//! As assinaturas públicas (`run`, `run_raw`, `run_structured`) são idênticas
//! às anteriores de propósito: nenhum dos chamadores muda, e a fronteira de
//! segurança passa a valer para todos eles de uma vez.

use std::path::Path;

use crate::core::error::CoreError;
use crate::core::process::{self, ProcessRequest, ProgramId, NETWORK_TIMEOUT};

/// Subcomandos que falam com a rede e merecem o tempo limite maior.
///
/// A escolha é do adapter, não do chamador: um módulo não deveria precisar
/// saber quanto tempo o broker concede.
const NETWORK_SUBCOMMANDS: &[&str] = &["fetch", "pull", "push", "clone", "ls-remote", "submodule"];

/// Opções globais do git que consomem o argumento seguinte.
///
/// Sem isto, `git -c x=y fetch` seria lido como subcomando `x=y`.
const GLOBAL_OPTIONS_WITH_VALUE: &[&str] = &["-c", "-C", "--git-dir", "--work-tree", "--namespace"];

/// Primeiro argumento que é de fato o subcomando.
fn subcommand_of<'a>(args: &'a [&'a str]) -> Option<&'a str> {
    let mut index = 0;

    while index < args.len() {
        let arg = args[index];

        if GLOBAL_OPTIONS_WITH_VALUE.contains(&arg) {
            index += 2;
            continue;
        }

        if arg.starts_with('-') {
            index += 1;
            continue;
        }

        return Some(arg);
    }

    None
}

fn build_request(path: &Path, args: &[&str]) -> ProcessRequest {
    let owned: Vec<String> = args.iter().map(|arg| (*arg).to_string()).collect();
    let network = subcommand_of(args)
        .map(|sub| NETWORK_SUBCOMMANDS.contains(&sub))
        .unwrap_or(false);

    let request = ProcessRequest::new(ProgramId::Git, owned, path);

    if network {
        request.with_timeout(NETWORK_TIMEOUT)
    } else {
        request
    }
}

pub fn run(path: &Path, args: &[&str]) -> Result<String, String> {
    Ok(run_raw(path, args)?.trim().to_string())
}

pub fn run_raw(path: &Path, args: &[&str]) -> Result<String, String> {
    let outcome = process::run(build_request(path, args))
        .map_err(|error| format!("Não foi possível executar Git: {error}"))?;

    if !outcome.success {
        return Err(outcome.stderr.trim().to_string());
    }

    Ok(outcome.stdout)
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
/// permissão negada) ou recusa do broker (argumento inválido, tempo limite). Um
/// comando que executou e terminou com exit code diferente de zero é sucesso do
/// ponto de vista do adapter e falha do ponto de vista do domínio, que decide
/// como classificá-la.
///
/// O tipo de erro continua sendo `std::io::Error` para não quebrar os
/// consumidores que classificam por `error.kind()`.
pub fn run_structured(path: &Path, args: &[&str]) -> Result<GitCommandOutput, std::io::Error> {
    let outcome = process::run(build_request(path, args)).map_err(core_error_to_io)?;

    Ok(GitCommandOutput {
        success: outcome.success,
        exit_code: outcome.exit_code,
        stdout: outcome.stdout,
        stderr: outcome.stderr,
    })
}

/// Converte a recusa do Core em `io::Error` preservando a distinção que muda a
/// instrução dada ao usuário (Git ausente vs. permissão negada).
fn core_error_to_io(error: CoreError) -> std::io::Error {
    let kind = match &error {
        CoreError::ExecutionFailed { io_kind, .. } => io_kind.unwrap_or(std::io::ErrorKind::Other),
        CoreError::ExecutionTimeout { .. } => std::io::ErrorKind::TimedOut,
        CoreError::ArgumentRejected { .. } => std::io::ErrorKind::InvalidInput,
        _ => std::io::ErrorKind::PermissionDenied,
    };

    std::io::Error::new(kind, error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn lab(nome: &str) -> std::path::PathBuf {
        let base = std::env::temp_dir().join(format!("dw_gitcli_{nome}"));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).expect("criar laboratório");
        run(&base, &["init", "-q"]).expect("git init");
        base
    }

    #[test]
    fn separadores_de_shell_chegam_como_argumento_literal() {
        let path = lab("shell");

        // Se houvesse shell no caminho, isto criaria um arquivo. Não há.
        let saida = run(&path, &["rev-parse", "--verify", "HEAD && touch invadido"]);

        assert!(saida.is_err(), "o git deveria recusar a revisão inválida");
        assert!(
            !path.join("invadido").exists(),
            "o separador de shell foi interpretado — houve execução de comando"
        );
    }

    #[test]
    fn timeout_de_rede_e_maior_que_o_local() {
        let local = build_request(Path::new("."), &["status"]);
        let rede = build_request(Path::new("."), &["push", "origin", "main"]);

        assert!(rede.timeout > local.timeout);
    }

    #[test]
    fn flag_antes_do_subcomando_nao_confunde_a_escolha_de_timeout() {
        let rede = build_request(Path::new("."), &["-c", "x=y", "fetch", "--prune"]);

        assert_eq!(rede.timeout, NETWORK_TIMEOUT);
    }

    #[test]
    fn diretorio_inexistente_e_recusado_antes_do_processo() {
        let erro = run_structured(
            Path::new("/caminho/que/nao/existe/em/lugar/nenhum"),
            &["status"],
        )
        .expect_err("deveria recusar");

        assert_eq!(erro.kind(), std::io::ErrorKind::NotFound);
    }
}
