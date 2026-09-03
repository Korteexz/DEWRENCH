use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GitState {
    NotRepository,
    UnbornRepository,
    Repository,
}

#[derive(Serialize)]
pub struct ProjectOpenResult {
    pub name: String,
    pub path: String,
    pub git_state: GitState,
}


#[derive(Serialize)]
pub struct GitFileStatus {
    pub path: String,
    pub index_status: String,
    pub worktree_status: String,
}

#[derive(Serialize)]
pub struct GitCommit {
    pub hash: String,
    pub message: String,
    pub author: String,
}

#[derive(Serialize)]
pub struct GitRepositoryDetails {
    pub branch: String,
    pub files: Vec<GitFileStatus>,
    pub commits: Vec<GitCommit>,
}
/// Uma branch, local ou remote-tracking.
///
/// Os campos de rastreamento são preenchidos apenas para branches locais;
/// numa remote-tracking, `remote` diz de qual remote ela veio. Manter os dois
/// tipos no mesmo modelo evita que o frontend tenha duas formas de branch.
#[derive(Serialize, Debug, Clone)]
pub struct GitBranch {
    pub name: String,
    pub current: bool,
    pub head: String,
    /// `local` ou `remote`.
    pub kind: String,
    /// Para remote-tracking: o remote de origem.
    pub remote: Option<String>,
    /// Para local: a branch remota rastreada, como `origin/main`.
    pub upstream: Option<String>,
    pub ahead: usize,
    pub behind: usize,
    /// Upstream configurado cuja ref não existe mais.
    pub gone: bool,
}

#[derive(Serialize, Debug, Clone)]
pub struct GitGraphCommit {
    pub hash: String,
    pub short_hash: String,
    pub message: String,
    pub author: String,
    pub parents: Vec<String>,
}

#[derive(Serialize, Debug)]
pub struct GitGraph {
    pub branches: Vec<GitBranch>,
    /// Remote-tracking branches, separadas das locais de propósito: o grafo
    /// decide se as desenha, e nunca as confunde com branches do usuário.
    pub remote_branches: Vec<GitBranch>,
    pub commits: Vec<GitGraphCommit>,
}

/// Um arquivo alterado pelo commit que será revertido.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct GitRevertFileChange {
    pub status: String,
    pub path: String,
    pub original_path: Option<String>,
}

/// Resultado read-only do preflight de Revert.
///
/// O preview nunca muta o repositório. Quando alguma regra bloqueia a operação,
/// o backend devolve `GitOperationError` em vez de um preview inexecutável.
#[derive(Serialize, Debug)]
pub struct GitRevertPreview {
    pub hash: String,
    pub short_hash: String,
    pub subject: String,
    pub author: String,
    pub parent_count: usize,
    pub is_root_commit: bool,
    pub affected_files: Vec<GitRevertFileChange>,
    pub preserved_local_changes: Vec<String>,
    pub warnings: Vec<String>,
    pub creates_new_commit: bool,
    pub preserves_history: bool,
}

/// Resultado de um Revert concluído com sucesso.
///
/// O hash do novo commit é lido do Git após a execução; nunca é fabricado.
#[derive(Serialize, Debug)]
pub struct GitRevertOutcome {
    pub reverted_hash: String,
    pub reverted_short_hash: String,
    pub new_commit_hash: String,
    pub new_commit_short_hash: String,
    pub new_commit_subject: String,
    pub affected_files: Vec<GitRevertFileChange>,
    pub warnings: Vec<String>,
    pub history_preserved: bool,
}

// ============================================================================
// REMOTES
// ============================================================================

/// Identidade extraída da URL de um remote.
///
/// Serve para reconhecer o provider (GitHub e futuros) sem que o módulo Git
/// precise conhecer nenhum deles. Nunca carrega credencial: a URL pode conter
/// usuário e token, e eles são descartados na extração.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct GitRemoteIdentity {
    pub host: Option<String>,
    pub owner: Option<String>,
    pub repository: Option<String>,
    pub provider: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct GitRemote {
    pub name: String,
    pub fetch_url: String,
    pub push_url: String,
    pub is_origin: bool,
    /// Este remote é o do upstream da branch atual.
    pub is_upstream: bool,
    pub identity: GitRemoteIdentity,
}

/// Relação de rastreamento entre a branch local e uma branch remota.
#[derive(Serialize, Debug, Clone)]
pub struct GitUpstream {
    pub remote: String,
    pub branch: String,
    /// Nome completo como o Git escreve: `origin/main`.
    pub ref_name: String,
    pub ahead: usize,
    pub behind: usize,
    /// Upstream configurado cuja ref não existe mais localmente.
    pub gone: bool,
}

#[derive(Serialize, Debug, Clone)]
pub struct GitRemotesView {
    pub remotes: Vec<GitRemote>,
    pub default_remote: Option<String>,
    pub current_branch: Option<String>,
    pub upstream: Option<GitUpstream>,
}

// ============================================================================
// OPERAÇÕES DE REDE
// ============================================================================

/// Plano de push: tudo que o usuário precisa saber ANTES de enviar.
///
/// O plano é read-only e sempre calculado a partir do estado real; ele existe
/// para que nenhuma operação de rede aconteça sem o usuário ter visto origem,
/// destino e conteúdo.
#[derive(Serialize, Debug)]
pub struct GitPushPlan {
    pub remote: String,
    pub remote_exists: bool,
    pub source_branch: String,
    pub target_branch: String,
    pub upstream: Option<GitUpstream>,
    pub will_create_upstream: bool,
    pub remote_branch_exists: bool,
    pub ahead: usize,
    pub behind: usize,
    pub diverged: bool,
    pub commits: Vec<GitGraphCommit>,
    pub warnings: Vec<String>,
    /// Motivo pelo qual o push não deve ser executado como está.
    pub blocked: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct GitPushOutcome {
    pub remote: String,
    pub source_branch: String,
    pub target_branch: String,
    pub pushed_commits: usize,
    pub created_upstream: bool,
    pub created_remote_branch: bool,
    pub new_remote_hash: String,
    pub details: Vec<String>,
}

/// Uma ref remota que mudou durante o fetch.
#[derive(Serialize, Debug, Clone)]
pub struct GitRefUpdate {
    pub ref_name: String,
    pub old_hash: Option<String>,
    pub new_hash: Option<String>,
    /// `new`, `updated`, `pruned` ou `forced`.
    pub kind: String,
    pub received_commits: usize,
}

#[derive(Serialize, Debug)]
pub struct GitFetchOutcome {
    pub remote: String,
    pub updated_refs: Vec<GitRefUpdate>,
    pub new_branches: Vec<String>,
    pub pruned_branches: Vec<String>,
    pub received_commits: usize,
    pub had_changes: bool,
    pub upstream: Option<GitUpstream>,
}

/// Plano de pull, incluindo o risco de conflito calculado com dados reais.
#[derive(Serialize, Debug)]
pub struct GitPullPlan {
    pub remote: String,
    pub branch: String,
    pub upstream: Option<GitUpstream>,
    pub incoming: Vec<GitGraphCommit>,
    pub outgoing: Vec<GitGraphCommit>,
    /// Estratégias que o estado atual permite.
    pub available_strategies: Vec<String>,
    pub recommended_strategy: String,
    pub can_fast_forward: bool,
    pub diverged: bool,
    pub local_changes: Vec<String>,
    /// Arquivos alterados localmente E tocados pelos commits que vão entrar.
    pub conflict_risk: Vec<String>,
    pub warnings: Vec<String>,
    pub blocked: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct GitPullOutcome {
    pub remote: String,
    pub branch: String,
    pub strategy: String,
    pub applied_commits: usize,
    pub files_changed: Vec<String>,
    pub previous_head: String,
    pub new_head: String,
    pub fetch: GitFetchOutcome,
}
