//! Security Core do DEWRENCH.
//!
//! Regra constitucional (ver `docs/DEWRENCH Protection Delegation Manifesto.md`):
//!
//! > MODULES DESCRIBE INTENT. CORE DECIDES AUTHORITY.
//! > CORE EXECUTES THROUGH CONTROLLED BOUNDARIES.
//!
//! Nenhum módulo decide sozinho se uma operação é autorizada, e nenhum módulo
//! cria processo ou resolve caminho por conta própria. O Core é pequeno de
//! propósito: ele existe para ser a ÚNICA porta das fronteiras perigosas —
//! processo, filesystem e autoridade — e não para virar um framework.
//!
//! O que está aqui é enforced hoje. O que é apenas fundação para o futuro está
//! marcado como tal na documentação e no comentário do próprio item.
pub mod approval;
pub mod authority;
pub mod error;
pub mod events;
pub mod path_security;
pub mod policy;
pub mod process;
pub mod state;
