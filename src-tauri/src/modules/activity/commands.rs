use crate::modules::git::errors::GitOperationError;

use super::models::ActivityStream;
use super::service;

#[tauri::command]
pub fn get_activity_stream(
    path: String,
    limit: Option<usize>,
) -> Result<ActivityStream, GitOperationError> {
    service::collect(&path, limit)
}
