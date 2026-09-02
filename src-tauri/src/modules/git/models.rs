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
#[derive(Serialize)]
pub struct GitBranch {
    pub name: String,
    pub current: bool,
    pub head: String,
}

#[derive(Serialize)]
pub struct GitGraphCommit {
    pub hash: String,
    pub short_hash: String,
    pub message: String,
    pub author: String,
    pub parents: Vec<String>,
}

#[derive(Serialize)]
pub struct GitGraph {
    pub branches: Vec<GitBranch>,
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
