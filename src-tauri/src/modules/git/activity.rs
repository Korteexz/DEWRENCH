//! Fonte de atividade do módulo Git.
//!
//! Traduz o histórico real em `ActivityEvent`. É o único lugar onde conceitos
//! de Git (hash, parents, autor) viram conceitos de atividade — a partir daqui,
//! quem consome não sabe que Git existe.

use std::collections::BTreeMap;
use std::path::Path;

use crate::modules::activity::models::ActivityEvent;

use super::errors::{codes, sanitize, GitOperationError};
use super::git_cli;

/// Separador de campo. `\x1f` (unit separator) não aparece em mensagem de
/// commit, ao contrário de tabulação e barra vertical.
const FIELD: char = '\x1f';

pub fn collect(path: &Path, limit: usize) -> Result<Vec<ActivityEvent>, GitOperationError> {
    if !path.join(".git").exists() {
        return Err(GitOperationError::new(
            codes::NOT_REPOSITORY,
            "Este projeto não possui repositório Git.",
        ));
    }

    let repository = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .to_string();

    let limit_arg = format!("-{limit}");

    let raw = git_cli::run(
        path,
        &[
            "log",
            "--all",
            &limit_arg,
            // %at = data do AUTOR em epoch; %az = fuso do autor.
            "--pretty=format:%H\x1f%at\x1f%az\x1f%an\x1f%ae\x1f%P\x1f%s",
        ],
    )
    .map_err(|error| {
        GitOperationError::new(
            codes::GIT_COMMAND_FAILED,
            "Não foi possível ler o histórico do repositório.",
        )
        .with_details(sanitize(error))
    })?;

    Ok(parse_events(&raw, &repository))
}

pub fn parse_events(raw: &str, repository: &str) -> Vec<ActivityEvent> {
    raw.lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(7, FIELD).collect();

            if parts.len() != 7 {
                return None;
            }

            let timestamp: i64 = parts[1].trim().parse().ok()?;
            let parents: Vec<&str> = parts[5].split_whitespace().collect();
            let subject = parts[6];

            let mut metadata = BTreeMap::new();
            metadata.insert("hash".to_string(), parts[0].to_string());
            metadata.insert("short_hash".to_string(), parts[0][..7.min(parts[0].len())].to_string());
            metadata.insert("subject".to_string(), subject.to_string());
            metadata.insert("parents".to_string(), parents.len().to_string());

            if let Some(email) = non_empty(parts[4]) {
                metadata.insert("email".to_string(), email);
            }

            Some(ActivityEvent {
                id: format!("git:{}", parts[0]),
                timestamp,
                utc_offset_minutes: parse_offset(parts[2]),
                source: "git".to_string(),
                // Colaboração entre máquinas ainda não existe: todo evento é local.
                machine: None,
                actor: non_empty(parts[3]),
                module: "git".to_string(),
                kind: classify(subject, parents.len()).to_string(),
                repository: repository.to_string(),
                // O commit não guarda em qual branch foi feito; a relação é
                // derivada do grafo, e inventá-la aqui seria dado falso.
                branch: None,
                metadata,
            })
        })
        .collect()
}

/// Classifica o evento pelo que o commit realmente é.
///
/// Merge é estrutural (número de pais). Revert é convenção de mensagem que o
/// próprio Git escreve com `git revert`; qualquer outra coisa é commit.
fn classify(subject: &str, parent_count: usize) -> &'static str {
    if parent_count > 1 {
        return "merge";
    }

    if subject.starts_with("Revert \"") || subject.starts_with("Revert: ") {
        return "revert";
    }

    if parent_count == 0 {
        return "root";
    }

    "commit"
}

/// `%az` vem como `+0200` / `-0300`.
fn parse_offset(raw: &str) -> i32 {
    let raw = raw.trim();

    if raw.len() < 5 {
        return 0;
    }

    let sign = if raw.starts_with('-') { -1 } else { 1 };
    let digits = &raw[1..];

    let hours: i32 = digits[..2].parse().unwrap_or(0);
    let minutes: i32 = digits[2..4].parse().unwrap_or(0);

    sign * (hours * 60 + minutes)
}

fn non_empty(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(hash: &str, ts: &str, tz: &str, author: &str, parents: &str, subject: &str) -> String {
        format!("{hash}\u{1f}{ts}\u{1f}{tz}\u{1f}{author}\u{1f}a@b.c\u{1f}{parents}\u{1f}{subject}")
    }

    #[test]
    fn commit_simples_vira_evento_de_atividade() {
        let raw = line("abc1234def", "1788300000", "-0300", "Kortexo", "parent1", "faz coisa");
        let events = parse_events(&raw, "DEWRENCH");

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, "commit");
        assert_eq!(events[0].source, "git");
        assert_eq!(events[0].module, "git");
        assert_eq!(events[0].timestamp, 1788300000);
        assert_eq!(events[0].utc_offset_minutes, -180);
        assert_eq!(events[0].actor.as_deref(), Some("Kortexo"));
        assert_eq!(events[0].repository, "DEWRENCH");
        assert_eq!(events[0].id, "git:abc1234def");
        assert_eq!(events[0].metadata.get("short_hash").unwrap(), "abc1234");
    }

    #[test]
    fn merge_e_reconhecido_pelo_numero_de_pais() {
        let raw = line("m1", "1", "+0000", "A", "p1 p2", "Merge branch 'x'");
        assert_eq!(parse_events(&raw, "r")[0].kind, "merge");
    }

    #[test]
    fn revert_e_reconhecido_pela_convencao_do_git() {
        let raw = line("r1", "1", "+0000", "A", "p1", "Revert \"faz coisa\"");
        assert_eq!(parse_events(&raw, "r")[0].kind, "revert");
    }

    #[test]
    fn commit_raiz_e_marcado() {
        let raw = line("r0", "1", "+0000", "A", "", "primeiro");
        assert_eq!(parse_events(&raw, "r")[0].kind, "root");
    }

    #[test]
    fn fuso_positivo_e_lido() {
        let raw = line("a", "1", "+0530", "A", "p", "s");
        assert_eq!(parse_events(&raw, "r")[0].utc_offset_minutes, 330);
    }

    #[test]
    fn linha_malformada_e_descartada_sem_derrubar() {
        let raw = "lixo sem separadores\n".to_string()
            + &line("a", "10", "+0000", "A", "p", "ok");
        let events = parse_events(&raw, "r");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].timestamp, 10);
    }

    #[test]
    fn evento_nao_inventa_branch() {
        let raw = line("a", "1", "+0000", "A", "p", "s");
        assert!(parse_events(&raw, "r")[0].branch.is_none());
    }
}
