use serde::Serialize;

/// Códigos de erro tipados desta operação.
///
/// Este modelo é intencionalmente mínimo: ele cobre o Revert e a infraestrutura
/// reutilizável mais próxima. Os commands antigos continuam retornando `String`
/// e não foram migrados.
pub mod codes {
    pub const NOT_REPOSITORY: &str = "NOT_REPOSITORY";
    pub const INVALID_COMMIT: &str = "INVALID_COMMIT";
    pub const MERGE_COMMIT_UNSUPPORTED: &str = "MERGE_COMMIT_UNSUPPORTED";
    pub const OPERATION_IN_PROGRESS: &str = "OPERATION_IN_PROGRESS";
    pub const STAGED_CHANGES: &str = "STAGED_CHANGES";
    pub const OVERLAPPING_WORKTREE_CHANGES: &str = "OVERLAPPING_WORKTREE_CHANGES";
    pub const IDENTITY_NOT_CONFIGURED: &str = "IDENTITY_NOT_CONFIGURED";
    pub const REVERT_CONFLICT_ABORTED: &str = "REVERT_CONFLICT_ABORTED";
    pub const REVERT_CONFLICT_ABORT_FAILED: &str = "REVERT_CONFLICT_ABORT_FAILED";
    pub const GIT_NOT_FOUND: &str = "GIT_NOT_FOUND";
    pub const PERMISSION_DENIED: &str = "PERMISSION_DENIED";
    pub const GIT_COMMAND_FAILED: &str = "GIT_COMMAND_FAILED";
}

/// Limite de tamanho para texto técnico devolvido ao frontend.
const MAX_DETAILS_LENGTH: usize = 2000;

/// Erro serializável para o frontend.
///
/// Os nomes de campo cruzam o IPC em camelCase porque este é um contrato novo,
/// definido junto com a operação de Revert.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitOperationError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    pub affected_files: Vec<String>,
    pub recoverable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_action: Option<String>,
}

impl GitOperationError {
    /// Erro do qual o usuário consegue se recuperar sozinho.
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        GitOperationError {
            code: code.to_string(),
            message: message.into(),
            details: None,
            affected_files: Vec::new(),
            recoverable: true,
            suggested_action: None,
        }
    }

    /// Erro que deixa o repositório em estado não confirmado.
    pub fn critical(code: &str, message: impl Into<String>) -> Self {
        let mut error = GitOperationError::new(code, message);
        error.recoverable = false;
        error
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        let sanitized = sanitize(details.into());
        if !sanitized.is_empty() {
            self.details = Some(sanitized);
        }
        self
    }

    pub fn with_files(mut self, files: Vec<String>) -> Self {
        self.affected_files = files;
        self
    }

    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.suggested_action = Some(action.into());
        self
    }
}

/// Remove credenciais embutidas em URLs e limita o tamanho do texto técnico.
///
/// O Revert não acessa a rede, mas a saída do Git pode citar remotes
/// configurados; o saneamento evita que credenciais cheguem à interface.
pub fn sanitize(raw: String) -> String {
    let mut result = String::with_capacity(raw.len());

    for piece in raw.split("://") {
        if result.is_empty() {
            result.push_str(piece);
            continue;
        }

        result.push_str("://");

        match (piece.find('@'), piece.find('/')) {
            (Some(at), Some(slash)) if at < slash => result.push_str(&piece[at..]),
            (Some(at), None) => result.push_str(&piece[at..]),
            _ => result.push_str(piece),
        }
    }

    let trimmed = result.trim().to_string();

    if trimmed.chars().count() <= MAX_DETAILS_LENGTH {
        return trimmed;
    }

    let mut truncated: String = trimmed.chars().take(MAX_DETAILS_LENGTH).collect();
    truncated.push('…');
    truncated
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_remove_credencial_de_url() {
        let sanitized =
            sanitize("fatal: https://user:token@host/owner/repo.git recusou".to_string());
        assert!(!sanitized.contains("token"));
        assert!(!sanitized.contains("user:"));
        assert!(sanitized.contains("host/owner/repo.git"));
    }

    #[test]
    fn sanitize_preserva_texto_comum() {
        assert_eq!(sanitize("  erro simples  ".to_string()), "erro simples");
    }

    #[test]
    fn erro_critico_nao_e_recuperavel() {
        let error = GitOperationError::critical(codes::REVERT_CONFLICT_ABORT_FAILED, "x");
        assert!(!error.recoverable);
    }
}
