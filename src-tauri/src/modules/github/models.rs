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

/// Detalhe de um pull request específico.
///
/// Superconjunto de `GithubPullRequest`: acrescenta o que só faz sentido pedir
/// para UM pull request — corpo, contagens e, principalmente, `mergeable` e
/// `merge_state_status`, que são a resposta do servidor à pergunta "isto pode
/// ser mesclado?". Nenhum desses campos é inferido aqui.
#[derive(Serialize, Debug, Clone)]
pub struct GithubPullRequestDetail {
    pub number: u64,
    pub title: String,
    pub body: String,
    pub state: String,
    pub is_draft: bool,
    pub head_branch: String,
    pub base_branch: String,
    /// SHA do topo da branch de origem — a identidade do estado revisado.
    pub head_sha: Option<String>,
    pub author: Option<String>,
    pub url: String,
    pub review_decision: Option<String>,
    /// `MERGEABLE`, `CONFLICTING` ou `UNKNOWN`, como o GitHub reporta.
    pub mergeable: Option<String>,
    /// `CLEAN`, `BLOCKED`, `DIRTY`, `BEHIND`, `UNSTABLE`, `DRAFT`, `HAS_HOOKS`…
    pub merge_state_status: Option<String>,
    pub changed_files: u64,
    pub additions: u64,
    pub deletions: u64,
    pub commit_count: u64,
}

/// Preflight de merge/close: o que o servidor permite AGORA.
///
/// Mesmo contrato de `GitPushPlan`/`GitPullPlan`: `warnings` descreve o que o
/// usuário deveria saber, `blocked` descreve por que a operação não deve
/// acontecer. Enquanto `blocked` for `Some`, nenhuma execução prossegue — e
/// quem decide isso é o backend, nunca a interface.
#[derive(Serialize, Debug, Clone)]
pub struct GithubPullRequestPlan {
    pub number: u64,
    pub title: String,
    pub state: String,
    pub is_draft: bool,
    pub head_branch: String,
    pub base_branch: String,
    pub head_sha: Option<String>,
    pub url: String,
    pub mergeable: Option<String>,
    pub merge_state_status: Option<String>,
    pub review_decision: Option<String>,
    /// Métodos que o repositório aceita: `merge`, `squash`, `rebase`.
    pub available_methods: Vec<String>,
    pub recommended_method: Option<String>,
    pub warnings: Vec<String>,
    /// Motivo pelo qual o merge não deve ser executado como está.
    pub blocked: Option<String>,
}

/// Resultado de um merge efetivamente executado.
#[derive(Serialize, Debug, Clone)]
pub struct GithubMergeOutcome {
    pub number: u64,
    pub method: String,
    pub merged: bool,
    /// Só é verdadeiro quando a interface pediu explicitamente.
    pub deleted_branch: bool,
    pub url: String,
    /// Linhas úteis que a `gh` reportou, já higienizadas.
    pub notes: Vec<String>,
}
