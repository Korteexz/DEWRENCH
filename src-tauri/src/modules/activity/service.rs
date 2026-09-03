//! Coleta de atividade a partir das fontes disponíveis.
//!
//! Hoje a lista de fontes tem um item. O ponto do módulo é que ela SEJA uma
//! lista: acrescentar Docker ou CI/CD significa acrescentar uma função que
//! devolve `Vec<ActivityEvent>` e registrá-la aqui, sem tocar em quem desenha.

use std::path::Path;

use crate::modules::git::activity as git_activity;
use crate::modules::git::errors::GitOperationError;

use super::models::ActivityStream;

/// Teto de eventos por coleta.
///
/// Alto o bastante para cobrir anos de histórico de um projeto e baixo o
/// bastante para a serialização não pesar no IPC.
pub const EVENT_LIMIT: usize = 5000;

pub fn collect(path: &str, limit: Option<usize>) -> Result<ActivityStream, GitOperationError> {
    let limit = limit.unwrap_or(EVENT_LIMIT).min(EVENT_LIMIT);
    let events = git_activity::collect(Path::new(path), limit)?;
    let truncated = events.len() >= limit;

    Ok(ActivityStream {
        events,
        sources: vec!["git".to_string()],
        truncated,
    })
}
