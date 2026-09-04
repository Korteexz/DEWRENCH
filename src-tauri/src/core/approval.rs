//! Preflight e aprovação vinculada a estado.
//!
//! "Tem certeza?" não é modelo de segurança. Uma aprovação precisa apontar
//! para o ESTADO EXATO que foi revisado: mesma ação, mesmo recurso, mesmos
//! argumentos, mesma fotografia do repositório.
//!
//! Se qualquer um desses elementos mudar entre a revisão e a execução, a
//! aprovação deixa de valer. Isso impede que uma aprovação de "enviar 3
//! commits para origin/main" autorize silenciosamente "enviar 40 commits para
//! outro remote" porque a tela ficou aberta.

use std::time::{SystemTime, UNIX_EPOCH};

use super::authority::{ActionId, ResourceId};
use super::error::CoreError;

/// Impressão digital do estado aprovado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreflightDigest(u64);

impl PreflightDigest {
    pub fn value(&self) -> u64 {
        self.0
    }
}

/// Fotografia do que o usuário está revisando.
#[derive(Debug, Clone)]
pub struct PreflightSnapshot {
    pub action: ActionId,
    pub resource: ResourceId,
    /// Argumentos efetivos da operação, na ordem em que serão usados.
    pub arguments: Vec<String>,
    /// Estado do mundo que a decisão levou em conta (hashes, contagens,
    /// destino remoto). É o que torna a aprovação frágil de propósito.
    pub observed_state: Vec<String>,
}

impl PreflightSnapshot {
    pub fn digest(&self) -> PreflightDigest {
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;

        let mut absorb = |value: &str| {
            for byte in value.as_bytes() {
                hash ^= u64::from(*byte);
                hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
            }
            // Separador explícito: sem ele, ["ab","c"] e ["a","bc"] teriam a
            // mesma digestão, e trocar a fronteira entre argumentos passaria
            // despercebido.
            hash ^= 0xff;
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        };

        absorb(self.action.0);
        absorb(&format!("{:?}", self.resource));
        for argument in &self.arguments {
            absorb(argument);
        }
        for observation in &self.observed_state {
            absorb(observation);
        }

        PreflightDigest(hash)
    }
}

/// Autorização emitida para UM estado específico.
#[derive(Debug, Clone)]
pub struct ApprovalToken {
    pub action: ActionId,
    pub digest: PreflightDigest,
    pub issued_at_secs: u64,
    pub ttl_secs: u64,
}

/// Validade padrão. Curta de propósito: aprovação antiga é aprovação suspeita.
pub const DEFAULT_TTL_SECS: u64 = 300;

pub fn issue(snapshot: &PreflightSnapshot) -> ApprovalToken {
    ApprovalToken {
        action: snapshot.action,
        digest: snapshot.digest(),
        issued_at_secs: now_secs(),
        ttl_secs: DEFAULT_TTL_SECS,
    }
}

/// Revalida a aprovação contra o estado ATUAL, no momento da execução.
pub fn validate(
    token: &ApprovalToken,
    current: &PreflightSnapshot,
    now_secs_value: u64,
) -> Result<(), CoreError> {
    if token.action != current.action {
        return Err(CoreError::ApprovalStale {
            action: current.action.to_string(),
        });
    }

    if now_secs_value.saturating_sub(token.issued_at_secs) > token.ttl_secs {
        return Err(CoreError::ApprovalExpired {
            action: token.action.to_string(),
        });
    }

    if token.digest != current.digest() {
        return Err(CoreError::ApprovalStale {
            action: token.action.to_string(),
        });
    }

    Ok(())
}

pub fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::authority::WorkspaceId;

    fn snapshot() -> PreflightSnapshot {
        PreflightSnapshot {
            action: ActionId("git.push"),
            resource: ResourceId::GitRepository(WorkspaceId("ws_1".to_string())),
            arguments: vec!["origin".to_string(), "main:refs/heads/main".to_string()],
            observed_state: vec![
                "head=abc123".to_string(),
                "ahead=3".to_string(),
                "remote=origin/main@def456".to_string(),
            ],
        }
    }

    #[test]
    fn aprovacao_do_mesmo_estado_e_aceita() {
        let current = snapshot();
        let token = issue(&current);
        assert!(validate(&token, &current, now_secs()).is_ok());
    }

    #[test]
    fn mudanca_de_argumento_invalida_a_aprovacao() {
        let approved = snapshot();
        let token = issue(&approved);

        let mut modified = snapshot();
        modified.arguments[0] = "fork".to_string();

        let error = validate(&token, &modified, now_secs()).unwrap_err();
        assert_eq!(error.code(), "APPROVAL_STALE");
    }

    #[test]
    fn mudanca_de_estado_observado_invalida_a_aprovacao() {
        let approved = snapshot();
        let token = issue(&approved);

        // O repositório avançou entre a revisão e a execução.
        let mut modified = snapshot();
        modified.observed_state[1] = "ahead=40".to_string();

        let error = validate(&token, &modified, now_secs()).unwrap_err();
        assert_eq!(error.code(), "APPROVAL_STALE");
    }

    #[test]
    fn mudanca_de_recurso_invalida_a_aprovacao() {
        let approved = snapshot();
        let token = issue(&approved);

        let mut modified = snapshot();
        modified.resource = ResourceId::GitRepository(WorkspaceId("ws_2".to_string()));

        let error = validate(&token, &modified, now_secs()).unwrap_err();
        assert_eq!(error.code(), "APPROVAL_STALE");
    }

    #[test]
    fn aprovacao_de_uma_acao_nao_autoriza_outra() {
        let approved = snapshot();
        let token = issue(&approved);

        let mut other = snapshot();
        other.action = ActionId("git.revert");

        let error = validate(&token, &other, now_secs()).unwrap_err();
        assert_eq!(error.code(), "APPROVAL_STALE");
    }

    #[test]
    fn aprovacao_expira() {
        let current = snapshot();
        let token = issue(&current);
        let futuro = token.issued_at_secs + DEFAULT_TTL_SECS + 1;

        let error = validate(&token, &current, futuro).unwrap_err();
        assert_eq!(error.code(), "APPROVAL_EXPIRED");
    }

    /// Fronteira entre argumentos não pode ser ambígua.
    #[test]
    fn concatenacao_diferente_produz_digestao_diferente() {
        let mut a = snapshot();
        a.arguments = vec!["ab".to_string(), "c".to_string()];

        let mut b = snapshot();
        b.arguments = vec!["a".to_string(), "bc".to_string()];

        assert_ne!(a.digest(), b.digest());
    }
}
