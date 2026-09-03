//! Modelo de atividade do DEWRENCH.
//!
//! Existe para que uma visualização temporal nunca precise conhecer Git.
//! O fluxo é:
//!
//! ```text
//! estado/eventos de uma ferramenta  ->  ActivityEvent  ->  visualização
//! ```
//!
//! Hoje há uma única fonte (o histórico Git local). Quando Docker, CI/CD,
//! deploy ou colaboração entre máquinas existirem, cada um vira apenas mais
//! uma fonte que produz `ActivityEvent` — sem tocar em quem desenha.
pub mod models;
pub mod service;
pub mod commands;
