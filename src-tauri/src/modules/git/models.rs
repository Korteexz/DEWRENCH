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