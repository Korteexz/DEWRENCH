use serde::Serialize;

/// Estado da integração com o GitHub para o projeto aberto.
///
/// Todos os campos são observações verificadas: se a `gh` não está instalada,
/// `available` é falso e o resto fica vazio — nada é presumido.
#[derive(Serialize, Debug, Clone)]
pub struct GithubContext {
    /// O repositório tem algum remote apontando para o GitHub.
    pub detected: bool,
    /// A CLI `gh` está instalada nesta máquina.
    pub cli_available: bool,
    /// Existe sessão autenticada na `gh`.
    pub authenticated: bool,
    pub owner: Option<String>,
    pub repository: Option<String>,
    pub remote_name: Option<String>,
    pub remote_url: Option<String>,
    pub default_branch: Option<String>,
    pub current_branch: Option<String>,
    pub web_url: Option<String>,
    /// O que impede a integração de funcionar agora, em uma linha.
    pub limitation: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct GithubPullRequest {
    pub number: u64,
    pub title: String,
    pub state: String,
    pub is_draft: bool,
    pub head_branch: String,
    pub base_branch: String,
    pub author: Option<String>,
    pub url: String,
    /// Estado de revisão como a `gh` reporta, quando disponível.
    pub review_decision: Option<String>,
}
