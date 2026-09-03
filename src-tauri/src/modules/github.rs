//! GitHub como PROVIDER OPCIONAL.
//!
//! O motor do DEWRENCH é o Git. Este módulo acrescenta contexto quando o
//! repositório aponta para o GitHub e a CLI `gh` está instalada e autenticada —
//! e desaparece silenciosamente quando não está. Nenhuma função do Git depende
//! dele, e nenhum token é lido, gravado ou transportado por aqui: a
//! autenticação é inteiramente responsabilidade da `gh`.
pub mod commands;
pub mod models;
pub mod provider;
pub mod service;
