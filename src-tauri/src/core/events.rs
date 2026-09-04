//! Auditoria e redação.
//!
//! Um registro de auditoria descreve AÇÕES. Ele não pode virar um segundo
//! banco de credenciais: qualquer texto que entra aqui passa por `redact`
//! antes de ser guardado.
//!
//! A redação é intencionalmente conservadora — prefere apagar demais a deixar
//! passar. Ela não substitui a regra principal, que é nunca colocar segredo
//! num caminho observável; ela é a última barreira quando uma ferramenta
//! externa imprime algo que não deveria.

use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use super::authority::{ActionId, Actor, ResourceId, Risk, SecurityDecision};

/// Marcador visível: um segredo apagado deve ser percebido, não sumir.
pub const REDACTED: &str = "«REDACTED»";

/// Prefixos de token reconhecíveis por forma.
const TOKEN_PREFIXES: &[&str] = &[
    "ghp_", "gho_", "ghu_", "ghs_", "ghr_", "github_pat_", "glpat-", "xoxb-", "xoxp-", "AKIA",
    "sk-", "AIza",
];

/// Chaves cujo VALOR é sensível em `chave=valor` ou `chave: valor`.
const SENSITIVE_KEYS: &[&str] = &[
    "token",
    "access_token",
    "refresh_token",
    "private_token",
    "password",
    "passwd",
    "secret",
    "client_secret",
    "api_key",
    "apikey",
    "authorization",
    "auth",
];

/// Palavras que aparecem ANTES da credencial e não são a credencial.
/// Preservá-las mantém o log legível; o token seguinte continua redigido.
const AUTH_SCHEMES: &[&str] = &["bearer", "basic", "digest", "token"];

/// Remove material sensível de um texto antes de log, erro ou auditoria.
pub fn redact(raw: &str) -> String {
    let stage = redact_pem_blocks(raw);
    let stage = redact_url_credentials(&stage);
    let stage = redact_prefixed_tokens(&stage);
    redact_key_values(&stage)
}

/// `https://user:token@host/owner/repo` → `https://host/owner/repo`.
fn redact_url_credentials(raw: &str) -> String {
    let mut result = String::with_capacity(raw.len());

    for (index, piece) in raw.split("://").enumerate() {
        if index == 0 {
            result.push_str(piece);
            continue;
        }

        result.push_str("://");

        // A credencial fica entre o esquema e o primeiro '/': só conta o '@'
        // que aparece ANTES do início do caminho.
        let boundary = piece.find('/').unwrap_or(piece.len());
        match piece[..boundary].rfind('@') {
            Some(at) => result.push_str(&piece[at + 1..]),
            None => result.push_str(piece),
        }
    }

    result
}

fn redact_prefixed_tokens(raw: &str) -> String {
    let mut result = raw.to_string();

    for prefix in TOKEN_PREFIXES {
        while let Some(start) = result.find(prefix) {
            let tail = &result[start + prefix.len()..];
            let length = tail
                .char_indices()
                .take_while(|(_, c)| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
                .count();

            // Prefixo solto (sem corpo) não é token; evita laço infinito.
            if length < 8 {
                let replacement = format!("{REDACTED}{}", &result[start + prefix.len()..]);
                result = format!("{}{}", &result[..start], replacement);
                break;
            }

            let end = start + prefix.len() + length;
            result = format!("{}{REDACTED}{}", &result[..start], &result[end..]);
        }
    }

    result
}

fn redact_key_values(raw: &str) -> String {
    let mut result = String::with_capacity(raw.len());
    let lowered = raw.to_ascii_lowercase();
    let bytes = raw.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        let mut matched = None;

        for key in SENSITIVE_KEYS {
            if lowered[index..].starts_with(key) {
                let after = index + key.len();
                let separator = lowered[after..]
                    .chars()
                    .take_while(|c| *c == ' ')
                    .count();
                let marker = after + separator;

                if marker < bytes.len() && (bytes[marker] == b'=' || bytes[marker] == b':') {
                    matched = Some((key.len(), marker + 1));
                    break;
                }
            }
        }

        let Some((key_len, value_start)) = matched else {
            let char_len = raw[index..].chars().next().map(char::len_utf8).unwrap_or(1);
            result.push_str(&raw[index..index + char_len]);
            index += char_len;
            continue;
        };

        result.push_str(&raw[index..index + key_len]);
        result.push_str(&raw[index + key_len..value_start]);

        let value_offset = raw[value_start..]
            .chars()
            .take_while(|c| *c == ' ')
            .count();
        result.push_str(&raw[value_start..value_start + value_offset]);

        // O valor pode vir precedido de um esquema de autenticação
        // (`Authorization: Bearer <segredo>`). O esquema em si não é segredo,
        // mas o que vem depois dele é — redigir só o primeiro token deixaria a
        // credencial inteira no texto.
        let mut cursor = value_start + value_offset;
        loop {
            let token_len = raw[cursor..]
                .chars()
                .take_while(|c| !c.is_whitespace() && *c != '&' && *c != '"' && *c != '\'')
                .map(char::len_utf8)
                .sum::<usize>();

            if token_len == 0 {
                break;
            }

            let token = &raw[cursor..cursor + token_len];
            if !AUTH_SCHEMES
                .iter()
                .any(|scheme| token.eq_ignore_ascii_case(scheme))
            {
                result.push_str(REDACTED);
                cursor += token_len;
                break;
            }

            result.push_str(token);
            cursor += token_len;

            let gap = raw[cursor..].chars().take_while(|c| *c == ' ').count();
            if gap == 0 {
                break;
            }
            result.push_str(&raw[cursor..cursor + gap]);
            cursor += gap;
        }

        index = cursor;
    }

    result
}

fn redact_pem_blocks(raw: &str) -> String {
    const BEGIN: &str = "-----BEGIN";
    const END: &str = "-----END";

    let mut result = String::with_capacity(raw.len());
    let mut rest = raw;

    while let Some(start) = rest.find(BEGIN) {
        result.push_str(&rest[..start]);
        result.push_str(REDACTED);

        match rest[start..].find(END) {
            Some(end) => {
                let after = start + end;
                let line_end = rest[after..].find('\n').map(|i| after + i).unwrap_or(rest.len());
                rest = &rest[line_end..];
            }
            None => return result,
        }
    }

    result.push_str(rest);
    result
}

/// Fatos de segurança registráveis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditKind {
    ActionRequested,
    ActionDenied,
    ActionPrepared,
    ActionApproved,
    ActionStarted,
    ActionSucceeded,
    ActionFailed,
}

#[derive(Debug, Clone)]
pub struct AuditEvent {
    pub kind: AuditKind,
    pub action: ActionId,
    pub actor: Actor,
    pub resource: Option<ResourceId>,
    pub risk: Risk,
    pub decision: Option<String>,
    /// Já redigido na construção; nunca guardar texto cru.
    pub detail: Option<String>,
    pub timestamp_secs: u64,
}

impl AuditEvent {
    pub fn new(kind: AuditKind, action: ActionId, actor: Actor, risk: Risk) -> Self {
        AuditEvent {
            kind,
            action,
            actor,
            resource: None,
            risk,
            decision: None,
            detail: None,
            timestamp_secs: now_secs(),
        }
    }

    pub fn with_resource(mut self, resource: ResourceId) -> Self {
        self.resource = Some(resource);
        self
    }

    pub fn with_decision(mut self, decision: &SecurityDecision) -> Self {
        self.decision = Some(match decision {
            SecurityDecision::Allow => "allow".to_string(),
            SecurityDecision::RequireApproval { reason } => {
                format!("require_approval: {}", redact(reason))
            }
            SecurityDecision::Deny { reason } => format!("deny: {}", redact(reason)),
        });
        self
    }

    /// Detalhe SEMPRE redigido na entrada: não existe caminho para gravar cru.
    pub fn with_detail(mut self, detail: &str) -> Self {
        self.detail = Some(redact(detail));
        self
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs())
        .unwrap_or(0)
}

/// Teto do buffer em memória. Auditoria durável ainda não existe.
const AUDIT_CAPACITY: usize = 500;

fn journal() -> &'static Mutex<VecDeque<AuditEvent>> {
    static JOURNAL: OnceLock<Mutex<VecDeque<AuditEvent>>> = OnceLock::new();
    JOURNAL.get_or_init(|| Mutex::new(VecDeque::with_capacity(AUDIT_CAPACITY)))
}

pub fn record(event: AuditEvent) {
    let Ok(mut log) = journal().lock() else {
        return;
    };

    if log.len() == AUDIT_CAPACITY {
        log.pop_front();
    }

    log.push_back(event);
}

pub fn recent(limit: usize) -> Vec<AuditEvent> {
    let Ok(log) = journal().lock() else {
        return Vec::new();
    };

    log.iter().rev().take(limit).cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credencial_em_url_e_removida() {
        let out = redact("fatal: unable to access 'https://kortexo:ghp_abcdefghijklmnopqrstuvwxyz012345@github.com/o/r.git/'");
        assert!(!out.contains("ghp_abcdefghijklmnopqrstuvwxyz012345"));
        assert!(!out.contains("kortexo:"));
        assert!(out.contains("github.com/o/r.git"));
    }

    #[test]
    fn token_solto_no_texto_e_removido() {
        for token in [
            "ghp_abcdefghijklmnopqrstuvwxyz0123456789",
            "github_pat_11ABCDEFG0aBcDeFgHiJkLmNoPqRsTuVwXyZ",
            "glpat-abcdefghijklmnopqrst",
            "AKIAIOSFODNN7EXAMPLE",
        ] {
            let out = redact(&format!("erro do servidor: {token} rejeitado"));
            assert!(!out.contains(token), "token vazou: {token}");
            assert!(out.contains(REDACTED));
        }
    }

    #[test]
    fn par_chave_valor_sensivel_e_removido() {
        let out = redact("curl -H 'Authorization: Bearer abc123def456' https://api.github.com");
        assert!(!out.contains("abc123def456"));

        let out = redact("https://host/callback?access_token=xyz789secret&state=1");
        assert!(!out.contains("xyz789secret"));
        assert!(out.contains("state=1"), "campo inofensivo foi apagado junto");
    }

    #[test]
    fn bloco_de_chave_privada_e_removido() {
        let pem = "antes\n-----BEGIN OPENSSH PRIVATE KEY-----\nb3BlbnNzaC1rZXktdjE\nAAAA\n-----END OPENSSH PRIVATE KEY-----\ndepois";
        let out = redact(pem);
        assert!(!out.contains("b3BlbnNzaC1rZXktdjE"));
        assert!(out.contains("antes"));
        assert!(out.contains("depois"));
    }

    #[test]
    fn texto_inofensivo_atravessa_sem_mutilacao() {
        let text = "fatal: couldn't find remote ref refs/heads/main";
        assert_eq!(redact(text), text);
    }

    #[test]
    fn detalhe_de_auditoria_ja_entra_redigido() {
        let event = AuditEvent::new(
            AuditKind::ActionFailed,
            ActionId("git.push"),
            Actor::LocalUser,
            Risk::High,
        )
        .with_detail("token=ghp_abcdefghijklmnopqrstuvwxyz0123456789 recusado");

        let detail = event.detail.unwrap();
        assert!(!detail.contains("ghp_"));
    }

    #[test]
    fn diario_respeita_o_teto_e_devolve_do_mais_recente() {
        for index in 0..(AUDIT_CAPACITY + 20) {
            record(
                AuditEvent::new(
                    AuditKind::ActionRequested,
                    ActionId("teste.evento"),
                    Actor::System,
                    Risk::Observe,
                )
                .with_detail(&format!("evento {index}")),
            );
        }

        let recent = recent(5);
        assert_eq!(recent.len(), 5);
        assert!(recent[0].detail.as_deref().unwrap().contains(&format!(
            "evento {}",
            AUDIT_CAPACITY + 19
        )));
    }
}
