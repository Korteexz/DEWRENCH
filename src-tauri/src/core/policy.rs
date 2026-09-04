//! Catálogo de ações e avaliação de política.
//!
//! Capability responde "este ator PODE?". Risk responde "isto DEVE acontecer
//! sem cerimônia?". As duas perguntas são respondidas aqui, separadamente, e
//! a resposta é um `SecurityDecision` — nunca um booleano, porque
//! "precisa de aprovação" não é nem sim nem não.
//!
//! Deny-by-default: ação desconhecida, workspace ausente ou confiança
//! insuficiente resultam em negação. Erro ao avaliar nunca vira permissão.

use super::approval::{validate, ApprovalToken, PreflightSnapshot};
use super::authority::{
    ActionDescriptor, Capability, ExecutionContext, RecoveryKind, Risk, SecurityDecision,
    WorkspaceTrust,
};
use super::error::CoreError;

macro_rules! action {
    ($name:ident, $id:expr, $cap:expr, $risk:expr, $idem:expr, $rec:expr) => {
        pub const $name: ActionDescriptor =
            ActionDescriptor::new($id, $cap, $risk, $idem, $rec);
    };
}

/// Ações que o DEWRENCH sabe executar hoje.
///
/// A lista é fechada em tempo de compilação: o frontend escolhe entre estas,
/// e não descreve uma operação arbitrária.
pub mod actions {
    use super::*;

    action!(OPEN_PROJECT, "git.open_project", Capability::FsProjectRead, Risk::Observe, true, RecoveryKind::Reversible);
    action!(READ_REPOSITORY, "git.read", Capability::GitRead, Risk::Observe, true, RecoveryKind::Reversible);
    action!(READ_ACTIVITY, "activity.read", Capability::GitRead, Risk::Observe, true, RecoveryKind::Reversible);

    action!(CREATE_REPOSITORY, "git.init", Capability::GitLocalWrite, Risk::Medium, false, RecoveryKind::RecoverableWithPrerequisites);
    action!(STAGE, "git.stage", Capability::GitLocalWrite, Risk::Low, true, RecoveryKind::Reversible);
    action!(UNSTAGE, "git.unstage", Capability::GitLocalWrite, Risk::Low, true, RecoveryKind::Reversible);
    action!(COMMIT, "git.commit", Capability::GitLocalWrite, Risk::Medium, false, RecoveryKind::RecoverableWithPrerequisites);
    action!(CREATE_BRANCH, "git.branch.create", Capability::GitLocalWrite, Risk::Low, false, RecoveryKind::Reversible);
    action!(SWITCH_BRANCH, "git.branch.switch", Capability::GitLocalWrite, Risk::Medium, true, RecoveryKind::Reversible);
    action!(REVERT, "git.revert", Capability::GitLocalWrite, Risk::High, false, RecoveryKind::RecoverableWithPrerequisites);

    action!(REMOTE_READ, "git.remote.read", Capability::GitRemoteRead, Risk::Observe, true, RecoveryKind::Reversible);
    action!(REMOTE_CONFIGURE, "git.remote.configure", Capability::GitLocalWrite, Risk::Medium, false, RecoveryKind::Reversible);
    action!(FETCH, "git.fetch", Capability::GitRemoteRead, Risk::Low, true, RecoveryKind::Reversible);
    action!(PULL, "git.pull", Capability::GitRemoteRead, Risk::High, false, RecoveryKind::RecoverableWithPrerequisites);
    // Efeito fora da máquina: desfazer exige autoridade que o DEWRENCH não tem.
    action!(PUSH, "git.push", Capability::GitRemoteWrite, Risk::High, false, RecoveryKind::RecoverableWithPrerequisites);

    action!(GITHUB_READ, "github.read", Capability::NetworkGithub, Risk::Observe, true, RecoveryKind::Reversible);
    action!(GITHUB_PR_CREATE, "github.pr.create", Capability::NetworkGithub, Risk::High, false, RecoveryKind::RecoverableWithPrerequisites);
}

/// Confiança mínima exigida por capability.
fn required_trust(capability: Capability) -> WorkspaceTrust {
    match capability {
        // Leitura exige que o workspace seja conhecido, nada além.
        Capability::GitRead | Capability::FsProjectRead | Capability::GitRemoteRead => {
            WorkspaceTrust::Opened
        }
        Capability::GitLocalWrite
        | Capability::FsProjectWrite
        | Capability::GitRemoteWrite
        | Capability::NetworkGithub
        | Capability::CredentialUse
        | Capability::ProcessSpawn => WorkspaceTrust::Opened,
        // Reescrever histórico é a única capability que já exige confiança
        // além da abertura — e nenhum fluxo a concede hoje.
        Capability::GitHistoryRewrite => WorkspaceTrust::ExecutableContent,
    }
}

/// Avalia a ação no contexto. Não executa nada.
pub fn evaluate(action: &ActionDescriptor, context: &ExecutionContext) -> SecurityDecision {
    if context.workspace.is_none() {
        return SecurityDecision::Deny {
            reason: "nenhum workspace autorizado no contexto".to_string(),
        };
    }

    let minimum = required_trust(action.capability);
    if context.trust < minimum {
        return SecurityDecision::Deny {
            reason: format!(
                "confiança insuficiente: {:?} exige {:?}",
                action.capability, minimum
            ),
        };
    }

    match action.risk {
        Risk::Observe | Risk::Low | Risk::Medium => SecurityDecision::Allow,
        Risk::High | Risk::Critical => SecurityDecision::RequireApproval {
            reason: format!(
                "{} tem risco {:?} e recuperação {:?}",
                action.id, action.risk, action.recovery
            ),
        },
    }
}

/// Evidência de que o usuário revisou o estado exato antes de executar.
pub enum ApprovalEvidence<'a> {
    /// Nenhuma. Só serve para ações que a política libera sozinha.
    None,
    /// Token emitido a partir de um preflight, revalidado contra o estado atual.
    Token {
        token: &'a ApprovalToken,
        current: &'a PreflightSnapshot,
    },
    /// O executor recalculou o plano imediatamente antes de agir e ele
    /// continua satisfazendo a operação pedida.
    ///
    /// É a forma usada hoje por push e pull, que refazem o preflight dentro da
    /// própria execução. Mais fraca que um token — não prova que o usuário viu
    /// ESTE estado — e por isso está registrada como tal na documentação.
    RevalidatedPreflight,
}

/// Decide e, quando exigido, verifica a aprovação. Esta é a função que os
/// módulos chamam: ela devolve `Ok` apenas quando a execução é legítima.
pub fn authorize(
    action: &ActionDescriptor,
    context: &ExecutionContext,
    evidence: ApprovalEvidence<'_>,
) -> Result<SecurityDecision, CoreError> {
    let decision = evaluate(action, context);

    match &decision {
        SecurityDecision::Allow => Ok(decision),
        SecurityDecision::Deny { reason } => Err(CoreError::PolicyDenied {
            action: action.id.to_string(),
            reason: reason.clone(),
        }),
        SecurityDecision::RequireApproval { reason } => match evidence {
            ApprovalEvidence::Token { token, current } => {
                validate(token, current, super::approval::now_secs())?;
                Ok(decision)
            }
            ApprovalEvidence::RevalidatedPreflight => Ok(decision),
            ApprovalEvidence::None => Err(CoreError::ApprovalRequired {
                action: action.id.to_string(),
                reason: reason.clone(),
            }),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::approval::{issue, PreflightSnapshot};
    use crate::core::authority::{ActionId, Actor, ResourceId, WorkspaceId};

    fn context(trust: WorkspaceTrust) -> ExecutionContext {
        ExecutionContext {
            actor: Actor::LocalUser,
            workspace: Some(WorkspaceId("ws_policy".to_string())),
            trust,
        }
    }

    #[test]
    fn sem_workspace_tudo_e_negado() {
        let decision = evaluate(&actions::READ_REPOSITORY, &ExecutionContext::anonymous());
        assert!(matches!(decision, SecurityDecision::Deny { .. }));
    }

    #[test]
    fn confianca_desconhecida_nega_ate_leitura() {
        let decision = evaluate(&actions::READ_REPOSITORY, &context(WorkspaceTrust::Unknown));
        assert!(matches!(decision, SecurityDecision::Deny { .. }));
    }

    #[test]
    fn leitura_em_workspace_aberto_e_liberada() {
        assert!(evaluate(&actions::READ_REPOSITORY, &context(WorkspaceTrust::Opened)).is_allow());
    }

    #[test]
    fn risco_alto_exige_aprovacao_mesmo_com_autoridade() {
        // O usuário PODE dar push; isso não significa que o push acontece sozinho.
        let decision = evaluate(&actions::PUSH, &context(WorkspaceTrust::Opened));
        assert!(matches!(decision, SecurityDecision::RequireApproval { .. }));
    }

    #[test]
    fn reescrita_de_historico_exige_confianca_que_ninguem_concede_hoje() {
        let rewrite = ActionDescriptor::new(
            "git.history.rewrite",
            Capability::GitHistoryRewrite,
            Risk::Critical,
            false,
            RecoveryKind::Irreversible,
        );

        assert!(matches!(
            evaluate(&rewrite, &context(WorkspaceTrust::Opened)),
            SecurityDecision::Deny { .. }
        ));
    }

    #[test]
    fn acao_de_risco_alto_sem_evidencia_e_recusada() {
        let error = authorize(
            &actions::PUSH,
            &context(WorkspaceTrust::Opened),
            ApprovalEvidence::None,
        )
        .unwrap_err();

        assert_eq!(error.code(), "APPROVAL_REQUIRED");
    }

    #[test]
    fn token_de_outro_estado_nao_autoriza() {
        let approved = PreflightSnapshot {
            action: ActionId("git.push"),
            resource: ResourceId::GitRepository(WorkspaceId("ws_policy".to_string())),
            arguments: vec!["origin".to_string()],
            observed_state: vec!["ahead=1".to_string()],
        };
        let token = issue(&approved);

        let mut current = approved.clone();
        current.observed_state = vec!["ahead=9".to_string()];

        let error = authorize(
            &actions::PUSH,
            &context(WorkspaceTrust::Opened),
            ApprovalEvidence::Token {
                token: &token,
                current: &current,
            },
        )
        .unwrap_err();

        assert_eq!(error.code(), "APPROVAL_STALE");
    }

    #[test]
    fn baixo_risco_nao_pede_cerimonia() {
        let decision = authorize(
            &actions::STAGE,
            &context(WorkspaceTrust::Opened),
            ApprovalEvidence::None,
        )
        .unwrap();
        assert!(decision.is_allow());
    }
}
