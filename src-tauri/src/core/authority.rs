//! Vocabulário tipado de autoridade.
//!
//! O manifesto proíbe segurança stringly-typed: `String action`, `String
//! permission`, `String risk` não conseguem ser exaustivamente verificados
//! pelo compilador nem comparados sem ambiguidade. Aqui cada conceito é um
//! tipo, e o `match` obriga a decidir sobre casos novos.
//!
//! Separação central do modelo: CAPABILITY responde "este ator PODE?" e RISK
//! responde "isto DEVE acontecer automaticamente?". As duas perguntas são
//! independentes — uma ação pode ser autorizada e ainda assim perigosa.

use std::fmt;

/// Identidade estável de uma ação. `&'static str` porque a lista é fechada em
/// tempo de compilação: o frontend não inventa ações.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ActionId(pub &'static str);

impl fmt::Display for ActionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Quem pede. Agentes existem no modelo desde já porque o manifesto proíbe
/// que uma IA vire atalho privilegiado: ela passa pelo mesmo Core.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Actor {
    /// Pessoa operando a interface local.
    LocalUser,
    /// Agente de código. Nunca recebe autoridade adicional por ser agente.
    Agent { name: String },
    /// Rotina interna do próprio DEWRENCH.
    System,
}

/// Identificador opaco de workspace. O frontend recebe e devolve isto; o Core
/// é quem sabe a qual caminho real corresponde.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WorkspaceId(pub String);

impl fmt::Display for WorkspaceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Recurso sobre o qual a autoridade é concedida.
///
/// Preferir isto a "path cru vindo do frontend" é o que impede o frontend de
/// redefinir o significado de um recurso já autorizado.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceId {
    Workspace(WorkspaceId),
    /// Arquivo dentro do workspace, sempre relativo à raiz canônica.
    WorkspaceFile { workspace: WorkspaceId, relative: String },
    GitRepository(WorkspaceId),
    GitBranch { workspace: WorkspaceId, name: String },
    GitRemote { workspace: WorkspaceId, name: String },
}

impl ResourceId {
    /// Chave de lock: identifica o recurso mutável disputado.
    pub fn lock_key(&self) -> String {
        match self {
            ResourceId::Workspace(id)
            | ResourceId::GitRepository(id) => format!("repo:{id}"),
            ResourceId::WorkspaceFile { workspace, .. }
            | ResourceId::GitBranch { workspace, .. }
            | ResourceId::GitRemote { workspace, .. } => format!("repo:{workspace}"),
        }
    }

    pub fn workspace(&self) -> &WorkspaceId {
        match self {
            ResourceId::Workspace(id)
            | ResourceId::GitRepository(id)
            | ResourceId::WorkspaceFile { workspace: id, .. }
            | ResourceId::GitBranch { workspace: id, .. }
            | ResourceId::GitRemote { workspace: id, .. } => id,
        }
    }
}

/// Autoridade. Descreve o que pode ser feito, nunca o quanto é perigoso.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Capability {
    GitRead,
    GitLocalWrite,
    GitRemoteRead,
    GitRemoteWrite,
    GitHistoryRewrite,
    FsProjectRead,
    FsProjectWrite,
    ProcessSpawn,
    NetworkGithub,
    CredentialUse,
}

/// Risco. Descreve quanta cerimônia a ação exige, nunca se ela é permitida.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Risk {
    Observe,
    Low,
    Medium,
    High,
    Critical,
}

/// O que o sistema consegue prometer sobre desfazer a operação.
///
/// O manifesto proíbe inventar recuperação: `Unknown` é uma resposta honesta e
/// deliberadamente distinta de `Irreversible`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryKind {
    Reversible,
    RecoverableWithPrerequisites,
    Irreversible,
    Unknown,
}

/// Confiança no workspace. Abrir um projeto NÃO é confiar no conteúdo dele.
///
/// `Unknown` é o default e nega tudo que não seja leitura: deny-by-default
/// aplicado ao próprio conceito de confiança.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WorkspaceTrust {
    /// Ainda não avaliado. Nega escrita e execução de conteúdo do projeto.
    Unknown,
    /// O usuário abriu deliberadamente: leitura e mutação do Git são
    /// permitidas; conteúdo executável do repositório continua não confiável.
    Opened,
    /// Reservado para quando o usuário confiar explicitamente em conteúdo
    /// executável do projeto (hooks, scripts, ferramentas externas).
    /// AINDA NÃO CONCEDIDO POR NENHUM FLUXO.
    ExecutableContent,
}

/// Contexto de uma execução: quem, sobre o quê, com qual confiança.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub actor: Actor,
    pub workspace: Option<WorkspaceId>,
    pub trust: WorkspaceTrust,
}

impl ExecutionContext {
    /// Contexto local sem workspace resolvido: nega qualquer coisa que exija
    /// autoridade sobre recurso.
    pub fn anonymous() -> Self {
        ExecutionContext {
            actor: Actor::LocalUser,
            workspace: None,
            trust: WorkspaceTrust::Unknown,
        }
    }

    pub fn for_workspace(workspace: WorkspaceId, trust: WorkspaceTrust) -> Self {
        ExecutionContext {
            actor: Actor::LocalUser,
            workspace: Some(workspace),
            trust,
        }
    }
}

/// Descrição completa de uma ação: o que ela exige e o que ela custa.
#[derive(Debug, Clone)]
pub struct ActionDescriptor {
    pub id: ActionId,
    pub capability: Capability,
    pub risk: Risk,
    /// Repetir a operação produz o mesmo efeito?
    pub idempotent: bool,
    pub recovery: RecoveryKind,
}

impl ActionDescriptor {
    pub const fn new(
        id: &'static str,
        capability: Capability,
        risk: Risk,
        idempotent: bool,
        recovery: RecoveryKind,
    ) -> Self {
        ActionDescriptor {
            id: ActionId(id),
            capability,
            risk,
            idempotent,
            recovery,
        }
    }
}

/// Resultado de uma avaliação de segurança.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityDecision {
    Allow,
    RequireApproval { reason: String },
    Deny { reason: String },
}

impl SecurityDecision {
    pub fn is_allow(&self) -> bool {
        matches!(self, SecurityDecision::Allow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn risco_e_ordenavel_para_comparacao_de_cerimonia() {
        assert!(Risk::Critical > Risk::High);
        assert!(Risk::Observe < Risk::Low);
    }

    #[test]
    fn confianca_desconhecida_e_o_menor_nivel() {
        assert!(WorkspaceTrust::Unknown < WorkspaceTrust::Opened);
        assert!(WorkspaceTrust::Opened < WorkspaceTrust::ExecutableContent);
    }

    #[test]
    fn recursos_do_mesmo_workspace_compartilham_a_chave_de_lock() {
        let workspace = WorkspaceId("w1".to_string());
        let repo = ResourceId::GitRepository(workspace.clone());
        let branch = ResourceId::GitBranch {
            workspace: workspace.clone(),
            name: "main".to_string(),
        };

        // Duas mutações Git no mesmo repositório disputam a MESMA autoridade.
        assert_eq!(repo.lock_key(), branch.lock_key());
    }

    #[test]
    fn contexto_anonimo_nao_carrega_confianca() {
        let context = ExecutionContext::anonymous();
        assert_eq!(context.trust, WorkspaceTrust::Unknown);
        assert!(context.workspace.is_none());
    }
}
