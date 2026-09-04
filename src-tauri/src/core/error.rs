//! Erros do Security Core.
//!
//! Deny-by-default é implementado no tipo: falha ao AVALIAR segurança nunca
//! vira permissão. Toda variante aqui representa uma negação ou uma falha de
//! fronteira, e nenhuma delas é conversível em "siga em frente".

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreError {
    /// O caminho resolvido cai fora da autoridade concedida.
    PathOutsideScope { attempted: String, scope: String },
    /// O caminho não pôde ser resolvido para um objeto real.
    PathUnresolvable { attempted: String, reason: String },
    /// Nenhum workspace registrado corresponde a este caminho.
    WorkspaceNotRegistered { attempted: String },
    /// O workspace existe, mas não tem confiança suficiente para a operação.
    WorkspaceNotTrusted { workspace: String, required: &'static str },
    /// O executável pedido não está na allowlist do broker.
    ExecutableNotAllowed { program: String },
    /// Um argumento foi recusado antes de chegar ao processo.
    ArgumentRejected { reason: String, argument: String },
    /// O processo excedeu o tempo permitido e foi encerrado.
    ExecutionTimeout { program: String, seconds: u64 },
    /// Falha ao iniciar o processo (ausente, permissão negada).
    ///
    /// `io_kind` é preservado porque a diferença entre "Git não instalado" e
    /// "permissão negada" muda a instrução dada ao usuário, e essa informação
    /// se perde se o erro virar apenas texto.
    ExecutionFailed {
        program: String,
        reason: String,
        io_kind: Option<std::io::ErrorKind>,
    },
    /// Outro fluxo detém a autoridade sobre este recurso.
    ResourceLocked { resource: String },
    /// A aprovação não corresponde mais ao estado revisado.
    ApprovalStale { action: String },
    /// A aprovação expirou.
    ApprovalExpired { action: String },
    /// A política negou a ação.
    PolicyDenied { action: String, reason: String },
    /// A ação exige aprovação explícita que ainda não foi dada.
    ApprovalRequired { action: String, reason: String },
}

impl CoreError {
    /// Código estável para o contrato de erro do frontend.
    pub fn code(&self) -> &'static str {
        match self {
            CoreError::PathOutsideScope { .. } => "PATH_OUTSIDE_SCOPE",
            CoreError::PathUnresolvable { .. } => "PATH_UNRESOLVABLE",
            CoreError::WorkspaceNotRegistered { .. } => "WORKSPACE_NOT_REGISTERED",
            CoreError::WorkspaceNotTrusted { .. } => "WORKSPACE_NOT_TRUSTED",
            CoreError::ExecutableNotAllowed { .. } => "EXECUTABLE_NOT_ALLOWED",
            CoreError::ArgumentRejected { .. } => "ARGUMENT_REJECTED",
            CoreError::ExecutionTimeout { .. } => "EXECUTION_TIMEOUT",
            CoreError::ExecutionFailed { .. } => "EXECUTION_FAILED",
            CoreError::ResourceLocked { .. } => "RESOURCE_LOCKED",
            CoreError::ApprovalStale { .. } => "APPROVAL_STALE",
            CoreError::ApprovalExpired { .. } => "APPROVAL_EXPIRED",
            CoreError::PolicyDenied { .. } => "POLICY_DENIED",
            CoreError::ApprovalRequired { .. } => "APPROVAL_REQUIRED",
        }
    }

    /// Sugestão acionável, quando existir uma que não revele estado interno.
    pub fn suggested_action(&self) -> Option<&'static str> {
        match self {
            CoreError::WorkspaceNotRegistered { .. } => {
                Some("Abra o projeto pelo DEWRENCH antes de operar sobre ele.")
            }
            CoreError::ResourceLocked { .. } => {
                Some("Aguarde a operação em andamento terminar neste repositório.")
            }
            CoreError::ApprovalStale { .. } | CoreError::ApprovalExpired { .. } => {
                Some("O estado mudou desde a revisão. Refaça o preflight.")
            }
            CoreError::ExecutionTimeout { .. } => {
                Some("A operação demorou além do limite e foi interrompida.")
            }
            _ => None,
        }
    }
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreError::PathOutsideScope { attempted, scope } => write!(
                f,
                "O caminho '{attempted}' está fora da autoridade concedida a '{scope}'."
            ),
            CoreError::PathUnresolvable { attempted, reason } => {
                write!(f, "Não foi possível resolver '{attempted}': {reason}.")
            }
            CoreError::WorkspaceNotRegistered { attempted } => write!(
                f,
                "Nenhum projeto aberto corresponde a '{attempted}'."
            ),
            CoreError::WorkspaceNotTrusted { workspace, required } => write!(
                f,
                "O workspace '{workspace}' não tem confiança suficiente ({required}) para esta operação."
            ),
            CoreError::ExecutableNotAllowed { program } => {
                write!(f, "O executável '{program}' não é permitido.")
            }
            CoreError::ArgumentRejected { reason, .. } => {
                write!(f, "Argumento recusado: {reason}.")
            }
            CoreError::ExecutionTimeout { program, seconds } => {
                write!(f, "'{program}' excedeu {seconds}s e foi encerrado.")
            }
            CoreError::ExecutionFailed { program, reason, .. } => {
                write!(f, "Não foi possível executar '{program}': {reason}.")
            }
            CoreError::ResourceLocked { resource } => {
                write!(f, "Já existe uma operação em andamento sobre '{resource}'.")
            }
            CoreError::ApprovalStale { action } => {
                write!(f, "A aprovação de '{action}' não corresponde ao estado atual.")
            }
            CoreError::ApprovalExpired { action } => {
                write!(f, "A aprovação de '{action}' expirou.")
            }
            CoreError::PolicyDenied { action, reason } => {
                write!(f, "'{action}' foi negada: {reason}.")
            }
            CoreError::ApprovalRequired { action, reason } => {
                write!(f, "'{action}' exige aprovação explícita: {reason}.")
            }
        }
    }
}

impl std::error::Error for CoreError {}
