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