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

    // -- Configuração de remotes ------------------------------------------
    pub const REMOTE_NOT_FOUND: &str = "REMOTE_NOT_FOUND";
    pub const REMOTE_ALREADY_EXISTS: &str = "REMOTE_ALREADY_EXISTS";
    pub const INVALID_REMOTE_NAME: &str = "INVALID_REMOTE_NAME";
    pub const INVALID_REMOTE_URL: &str = "INVALID_REMOTE_URL";
    pub const UNSAFE_REMOTE_URL: &str = "UNSAFE_REMOTE_URL";

    // -- Operações de rede -------------------------------------------------
    pub const NO_UPSTREAM: &str = "NO_UPSTREAM";
    pub const UPSTREAM_GONE: &str = "UPSTREAM_GONE";
    pub const NOTHING_TO_PUSH: &str = "NOTHING_TO_PUSH";
    pub const NON_FAST_FORWARD: &str = "NON_FAST_FORWARD";
    pub const PUSH_REJECTED: &str = "PUSH_REJECTED";
    pub const AUTHENTICATION_REQUIRED: &str = "AUTHENTICATION_REQUIRED";
    pub const NETWORK_UNREACHABLE: &str = "NETWORK_UNREACHABLE";
    pub const REMOTE_REPOSITORY_NOT_FOUND: &str = "REMOTE_REPOSITORY_NOT_FOUND";
    pub const UNBORN_BRANCH: &str = "UNBORN_BRANCH";
    pub const DETACHED_HEAD: &str = "DETACHED_HEAD";
    pub const DIVERGED_HISTORY: &str = "DIVERGED_HISTORY";
    pub const LOCAL_CHANGES_WOULD_BE_LOST: &str = "LOCAL_CHANGES_WOULD_BE_LOST";
    pub const MERGE_CONFLICT: &str = "MERGE_CONFLICT";
    pub const STRATEGY_UNAVAILABLE: &str = "STRATEGY_UNAVAILABLE";

    // -- Provider opcional (GitHub) ---------------------------------------
    pub const PROVIDER_UNAVAILABLE: &str = "PROVIDER_UNAVAILABLE";
    pub const PROVIDER_NOT_AUTHENTICATED: &str = "PROVIDER_NOT_AUTHENTICATED";
    pub const PROVIDER_COMMAND_FAILED: &str = "PROVIDER_COMMAND_FAILED";

    // -- Recusas do Security Core -----------------------------------------
    //
    // Os códigos do Core cruzam o IPC com o texto que o próprio Core define
    // (`CoreError::code()`), sem tradução: um código de negação que muda de
    // nome no caminho vira um contrato que ninguém consegue verificar.
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

/// Remove material sensível e limita o tamanho do texto técnico.
///
/// A redação em si pertence ao `core::events`, que é a autoridade única sobre
/// o que conta como segredo; aqui fica apenas o limite de tamanho, que é regra
/// do contrato de IPC e não de segurança. Antes esta função cobria somente
/// credencial embutida em URL — delegar amplia a cobertura (tokens com prefixo
/// conhecido, blocos PEM, pares chave=valor sensíveis) sem alterar assinatura
/// nem formato de saída.
pub fn sanitize(raw: String) -> String {
    let trimmed = crate::core::events::redact(&raw).trim().to_string();

    if trimmed.chars().count() <= MAX_DETAILS_LENGTH {
        return trimmed;
    }

    let mut truncated: String = trimmed.chars().take(MAX_DETAILS_LENGTH).collect();
    truncated.push('…');
    truncated
}

/// Recusa do Security Core apresentada no contrato de erro existente.
///
/// A conversão preserva o código do Core em vez de achatar tudo em
/// `GIT_COMMAND_FAILED`: o frontend precisa conseguir distinguir "o projeto não
/// está aberto" de "o Git recusou o comando", e o red team precisa conseguir
/// afirmar QUAL fronteira bloqueou uma tentativa.
///
/// `recoverable = true` em todas: uma negação de autoridade não deixa o
/// repositório em estado indefinido — ela impede que a operação comece.
impl From<crate::core::error::CoreError> for GitOperationError {
    fn from(error: crate::core::error::CoreError) -> Self {
        let mut mapped = GitOperationError::new(error.code(), error.to_string());

        if let Some(action) = error.suggested_action() {
            mapped = mapped.with_action(action);
        }

        mapped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn erro_do_core_preserva_o_codigo_de_negacao() {
        let denied: GitOperationError = crate::core::error::CoreError::WorkspaceNotRegistered {
            attempted: "/tmp/qualquer".to_string(),
        }
        .into();

        assert_eq!(denied.code, "WORKSPACE_NOT_REGISTERED");
        assert!(denied.suggested_action.is_some());
    }

    #[test]
    fn sanitize_remove_token_com_prefixo_conhecido() {
        let sanitized = sanitize("remote: erro com ghp_aaaabbbbccccddddeeeeffff1234".to_string());
        assert!(!sanitized.contains("ghp_aaaabbbbccccddddeeeeffff1234"));
    }

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
