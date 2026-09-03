use std::path::Path;

use crate::core::process::operand;

use super::git_cli;
use super::models::GitBranch;

pub fn get_current(
    path: &Path,
) -> Result<String, String> {
    git_cli::run(
        path,
        &["branch", "--show-current"],
    )
}

/// Branches locais com a relação de rastreamento resolvida.
///
/// `for-each-ref` traz nome, HEAD, hash, upstream e ahead/behind numa chamada
/// só. Consultar `rev-list` por branch multiplicaria processos e daria a mesma
/// resposta.
pub fn get_all(
    path: &Path,
) -> Result<Vec<GitBranch>, String> {
    let output = git_cli::run(
        path,
        &[
            "for-each-ref",
            "--format=%(refname:short)%09%(HEAD)%09%(objectname)%09%(upstream:short)%09%(upstream:track)",
            "refs/heads",
        ],
    )?;

    Ok(parse_local_branches(&output))
}

/// Remote-tracking branches (`refs/remotes`), sem o ponteiro simbólico HEAD.
pub fn get_remote_tracking(
    path: &Path,
) -> Result<Vec<GitBranch>, String> {
    let output = git_cli::run(
        path,
        &[
            "for-each-ref",
            "--format=%(refname)%09%(refname:short)%09%(objectname)",
            "refs/remotes",
        ],
    )?;

    Ok(parse_remote_branches(&output))
}

pub fn parse_local_branches(raw: &str) -> Vec<GitBranch> {
    raw.lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('\t').collect();

            if parts.len() < 3 {
                return None;
            }

            let upstream = parts
                .get(3)
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .map(str::to_string);

            let track = parts.get(4).copied().unwrap_or("");
            let (ahead, behind, gone) = parse_track(track);

            Some(GitBranch {
                name: parts[0].to_string(),
                current: parts[1] == "*",
                head: parts[2].to_string(),
                kind: "local".to_string(),
                remote: None,
                upstream,
                ahead,
                behind,
                gone,
            })
        })
        .collect()
}

pub fn parse_remote_branches(raw: &str) -> Vec<GitBranch> {
    raw.lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('\t').collect();

            if parts.len() < 3 {
                return None;
            }

            // `refs/remotes/origin/HEAD` é ponteiro simbólico, não branch — e o
            // Git o encurta para `origin`, sem sufixo, então o filtro precisa
            // olhar o refname completo.
            if parts[0].ends_with("/HEAD") {
                return None;
            }

            let short = parts[1];
            let remote = short.split_once('/').map(|(remote, _)| remote.to_string());

            Some(GitBranch {
                name: short.to_string(),
                current: false,
                head: parts[2].to_string(),
                kind: "remote".to_string(),
                remote,
                upstream: None,
                ahead: 0,
                behind: 0,
                gone: false,
            })
        })
        .collect()
}

/// Lê `%(upstream:track)`: `[ahead 3, behind 1]`, `[gone]` ou vazio.
fn parse_track(raw: &str) -> (usize, usize, bool) {
    let cleaned = raw.trim().trim_start_matches('[').trim_end_matches(']');

    if cleaned.is_empty() {
        return (0, 0, false);
    }

    if cleaned == "gone" {
        return (0, 0, true);
    }

    let mut ahead = 0;
    let mut behind = 0;

    for piece in cleaned.split(',') {
        let piece = piece.trim();
        if let Some(value) = piece.strip_prefix("ahead ") {
            ahead = value.trim().parse().unwrap_or(0);
        } else if let Some(value) = piece.strip_prefix("behind ") {
            behind = value.trim().parse().unwrap_or(0);
        }
    }

    (ahead, behind, false)
}

pub fn create_from(
    path: &Path,
    start_point: &str,
    branch_name: &str,
) -> Result<(), String> {
    let branch_name = branch_name.trim();
    let start_point = start_point.trim();

    if branch_name.is_empty() {
        return Err(
            "O nome da branch não pode estar vazio.".to_string()
        );
    }

    if start_point.is_empty() {
        return Err(
            "O ponto inicial da branch não pode estar vazio.".to_string()
        );
    }

    // Camada 1: o Core recusa o valor antes que ele vire argv.
    // `check-ref-format` e `rev-parse` NÃO aceitam `--`, então aqui esta é a
    // única barreira — e é por isso que ela vem antes de qualquer execução.
    let branch_name = operand(branch_name).map_err(|error| error.to_string())?;
    let start_point = operand(start_point).map_err(|error| error.to_string())?;

    git_cli::run(
        path,
        &[
            "check-ref-format",
            "--branch",
            branch_name,
        ],
    )?;

    git_cli::run(
        path,
        &[
            "rev-parse",
            "--verify",
            start_point,
        ],
    )?;

    // Camada 2: `--` encerra as opções, então nem um valor que escapasse da
    // camada 1 seria lido como opção por este comando.
    git_cli::run(
        path,
        &[
            "branch",
            "--",
            branch_name,
            start_point,
        ],
    )?;

    Ok(())
}

pub fn switch(
    path: &Path,
    branch_name: &str,
) -> Result<(), String> {
    let branch_name = branch_name.trim();

    if branch_name.is_empty() {
        return Err(
            "O nome da branch não pode estar vazio.".to_string()
        );
    }

    // Duas camadas. Sem elas, `--orphan=<nome>` transformava "trocar de branch"
    // em "criar branch órfã e mover o HEAD" — reproduzido em laboratório.
    let branch_name = operand(branch_name).map_err(|error| error.to_string())?;

    git_cli::run(
        path,
        &[
            "switch",
            "--",
            branch_name,
        ],
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn branch_local_com_upstream_e_divergencia() {
        let raw = "main\t*\tabc123\torigin/main\t[ahead 2, behind 1]";
        let branches = parse_local_branches(raw);

        assert_eq!(branches.len(), 1);
        assert!(branches[0].current);
        assert_eq!(branches[0].upstream.as_deref(), Some("origin/main"));
        assert_eq!(branches[0].ahead, 2);
        assert_eq!(branches[0].behind, 1);
        assert!(!branches[0].gone);
        assert_eq!(branches[0].kind, "local");
    }

    #[test]
    fn branch_local_sem_upstream() {
        let branches = parse_local_branches("experimento\t \tdef456\t\t");
        assert_eq!(branches[0].upstream, None);
        assert_eq!(branches[0].ahead, 0);
        assert!(!branches[0].current);
    }

    #[test]
    fn branch_local_com_upstream_removido() {
        let branches = parse_local_branches("antiga\t \tfff\torigin/antiga\t[gone]");
        assert!(branches[0].gone);
        assert_eq!(branches[0].ahead, 0);
    }

    #[test]
    fn branch_apenas_a_frente() {
        let branches = parse_local_branches("x\t \taaa\torigin/x\t[ahead 5]");
        assert_eq!(branches[0].ahead, 5);
        assert_eq!(branches[0].behind, 0);
    }

    #[test]
    fn remote_tracking_ignora_o_ponteiro_head() {
        let raw = "refs/remotes/origin/HEAD\torigin\taaa\n\
                   refs/remotes/origin/main\torigin/main\tbbb\n\
                   refs/remotes/fork/experimento\tfork/experimento\tccc";
        let branches = parse_remote_branches(raw);

        assert_eq!(branches.len(), 2);
        assert_eq!(branches[0].name, "origin/main");
        assert_eq!(branches[0].remote.as_deref(), Some("origin"));
        assert_eq!(branches[1].remote.as_deref(), Some("fork"));
        assert!(branches.iter().all(|branch| branch.kind == "remote"));
    }
}
