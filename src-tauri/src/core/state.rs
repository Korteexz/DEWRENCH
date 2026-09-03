//! Registro de workspaces e locks de recurso.
//!
//! Antes deste módulo, todo command recebia um `path: String` do frontend e o
//! usava como autoridade. Isso significa que qualquer chamada IPC podia operar
//! sobre QUALQUER diretório da máquina — o frontend definia o alvo, não o
//! Core.
//!
//! Aqui a autoridade passa a ser concedida uma vez, quando o usuário abre o
//! projeto, e verificada em toda operação seguinte. O caminho continua
//! cruzando o IPC (o contrato existente foi preservado), mas ele deixou de
//! ser CREDENCIAL: agora é apenas uma referência que precisa corresponder a um
//! workspace já registrado.

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use super::authority::{ResourceId, WorkspaceId, WorkspaceTrust};
use super::error::CoreError;
use super::path_security::{display_path, Scope};

#[derive(Debug, Clone)]
pub struct WorkspaceRecord {
    pub id: WorkspaceId,
    pub scope: Scope,
    pub trust: WorkspaceTrust,
}

fn registry() -> &'static Mutex<HashMap<WorkspaceId, WorkspaceRecord>> {
    static REGISTRY: OnceLock<Mutex<HashMap<WorkspaceId, WorkspaceRecord>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn locks() -> &'static Mutex<HashSet<String>> {
    static LOCKS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    LOCKS.get_or_init(|| Mutex::new(HashSet::new()))
}

/// Identidade derivada do caminho canônico.
///
/// Determinística de propósito: reabrir o mesmo projeto devolve o mesmo id, e
/// dois caminhos textualmente diferentes que resolvem para o mesmo diretório
/// (symlink, maiúsculas no Windows) recebem a MESMA identidade — caso
/// contrário seria possível registrar o mesmo recurso duas vezes e escapar do
/// lock.
fn derive_id(scope: &Scope) -> WorkspaceId {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let canonical = scope.root().to_string_lossy();
    let normalized = if cfg!(windows) {
        canonical.to_lowercase()
    } else {
        canonical.into_owned()
    };

    for byte in normalized.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }

    WorkspaceId(format!("ws_{hash:016x}"))
}

/// Concede autoridade sobre um diretório que o usuário abriu deliberadamente.
///
/// Confiança inicial é `Opened`: leitura e mutação do Git são permitidas,
/// conteúdo executável do repositório continua NÃO confiável.
pub fn register_workspace(root: &Path) -> Result<WorkspaceRecord, CoreError> {
    let scope = Scope::create(root)?;
    let id = derive_id(&scope);

    let record = WorkspaceRecord {
        id: id.clone(),
        scope,
        trust: WorkspaceTrust::Opened,
    };

    if let Ok(mut map) = registry().lock() {
        map.insert(id, record.clone());
    }

    Ok(record)
}

/// Resolve um caminho vindo do IPC para um workspace JÁ registrado.
///
/// Deny-by-default: caminho desconhecido é negado, e não promovido a
/// workspace por conveniência.
pub fn authorize_workspace(path: &str) -> Result<WorkspaceRecord, CoreError> {
    let scope = Scope::create(Path::new(path)).map_err(|_| CoreError::WorkspaceNotRegistered {
        attempted: path.to_string(),
    })?;
    let id = derive_id(&scope);

    let map = registry().lock().map_err(|_| CoreError::WorkspaceNotRegistered {
        attempted: path.to_string(),
    })?;

    map.get(&id)
        .cloned()
        .ok_or_else(|| CoreError::WorkspaceNotRegistered {
            attempted: display_path(scope.root()),
        })
}

/// Exige um nível mínimo de confiança para a operação.
pub fn require_trust(
    record: &WorkspaceRecord,
    minimum: WorkspaceTrust,
    label: &'static str,
) -> Result<(), CoreError> {
    if record.trust >= minimum {
        return Ok(());
    }

    Err(CoreError::WorkspaceNotTrusted {
        workspace: record.scope.display_root(),
        required: label,
    })
}

/// Autoridade exclusiva sobre um recurso mutável.
///
/// Liberada no `Drop`, o que cobre retorno antecipado, `?` e panic — os três
/// caminhos por onde um lock manual vaza.
#[derive(Debug)]
pub struct ResourceGuard {
    key: String,
}

impl Drop for ResourceGuard {
    fn drop(&mut self) {
        if let Ok(mut held) = locks().lock() {
            held.remove(&self.key);
        }
    }
}

/// Tenta adquirir o lock; não bloqueia.
///
/// Não bloquear é deliberado: uma segunda mutação simultânea sobre o mesmo
/// repositório quase sempre é clique duplo ou IPC duplicado, e enfileirá-la
/// executaria a operação duas vezes em vez de recusar a segunda.
pub fn acquire(resource: &ResourceId) -> Result<ResourceGuard, CoreError> {
    let key = resource.lock_key();

    let mut held = locks().lock().map_err(|_| CoreError::ResourceLocked {
        resource: key.clone(),
    })?;

    if !held.insert(key.clone()) {
        return Err(CoreError::ResourceLocked { resource: key });
    }

    Ok(ResourceGuard { key })
}

#[cfg(test)]
pub fn forget_workspace(id: &WorkspaceId) {
    if let Ok(mut map) = registry().lock() {
        map.remove(id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn lab(name: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("dw-state-{name}-{nanos}"));
        fs::create_dir_all(root.join("projeto")).unwrap();
        fs::create_dir_all(root.join("outro")).unwrap();
        root
    }

    #[test]
    fn caminho_nao_registrado_e_negado() {
        let root = lab("negado");
        let error = authorize_workspace(root.join("projeto").to_str().unwrap()).unwrap_err();
        assert_eq!(error.code(), "WORKSPACE_NOT_REGISTERED");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn registro_concede_autoridade_apenas_ao_proprio_caminho() {
        let root = lab("autoridade");
        let record = register_workspace(&root.join("projeto")).unwrap();

        assert!(authorize_workspace(root.join("projeto").to_str().unwrap()).is_ok());

        // Registrar A não pode autorizar B.
        let error = authorize_workspace(root.join("outro").to_str().unwrap()).unwrap_err();
        assert_eq!(error.code(), "WORKSPACE_NOT_REGISTERED");

        forget_workspace(&record.id);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn caminho_inexistente_nao_vira_workspace() {
        let error = authorize_workspace("/dw-nunca-existiu-42").unwrap_err();
        assert_eq!(error.code(), "WORKSPACE_NOT_REGISTERED");
    }

    #[cfg(unix)]
    #[test]
    fn symlink_para_o_mesmo_diretorio_recebe_a_mesma_identidade() {
        let root = lab("identidade");
        let record = register_workspace(&root.join("projeto")).unwrap();

        std::os::unix::fs::symlink(root.join("projeto"), root.join("atalho")).unwrap();

        // Autoridade é sobre o OBJETO, não sobre a string.
        let via_symlink = authorize_workspace(root.join("atalho").to_str().unwrap()).unwrap();
        assert_eq!(via_symlink.id, record.id);

        forget_workspace(&record.id);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn confianca_inicial_nao_alcanca_conteudo_executavel() {
        let root = lab("confianca");
        let record = register_workspace(&root.join("projeto")).unwrap();

        assert_eq!(record.trust, WorkspaceTrust::Opened);
        assert!(require_trust(&record, WorkspaceTrust::Opened, "abertura").is_ok());

        let error = require_trust(&record, WorkspaceTrust::ExecutableContent, "execução")
            .unwrap_err();
        assert_eq!(error.code(), "WORKSPACE_NOT_TRUSTED");

        forget_workspace(&record.id);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn segunda_aquisicao_do_mesmo_recurso_e_recusada() {
        let workspace = WorkspaceId("ws_lock_teste_a".to_string());
        let resource = ResourceId::GitRepository(workspace.clone());

        let first = acquire(&resource).unwrap();
        let error = acquire(&resource).unwrap_err();
        assert_eq!(error.code(), "RESOURCE_LOCKED");

        drop(first);
        assert!(acquire(&resource).is_ok());
    }

    #[test]
    fn lock_e_liberado_quando_a_operacao_falha() {
        let workspace = WorkspaceId("ws_lock_teste_b".to_string());
        let resource = ResourceId::GitRepository(workspace.clone());

        fn operacao_que_falha(resource: &ResourceId) -> Result<(), &'static str> {
            let _guard = acquire(resource).unwrap();
            Err("a operação falhou no meio")
        }

        let outcome = operacao_que_falha(&resource);

        assert!(outcome.is_err());
        // Se o guard não tivesse liberado, esta linha falharia.
        assert!(acquire(&resource).is_ok());
    }

    #[test]
    fn lock_e_liberado_apos_panic() {
        let workspace = WorkspaceId("ws_lock_teste_c".to_string());
        let resource = ResourceId::GitRepository(workspace.clone());

        let result = std::panic::catch_unwind(|| {
            let _guard = acquire(&resource).unwrap();
            panic!("falha catastrófica no meio da operação");
        });

        assert!(result.is_err());
        assert!(
            acquire(&resource).is_ok(),
            "lock ficou preso depois de um panic"
        );
    }

    #[test]
    fn recursos_de_workspaces_diferentes_nao_disputam_lock() {
        let a = ResourceId::GitRepository(WorkspaceId("ws_lock_teste_d".to_string()));
        let b = ResourceId::GitRepository(WorkspaceId("ws_lock_teste_e".to_string()));

        let _first = acquire(&a).unwrap();
        assert!(acquire(&b).is_ok());
    }

    #[test]
    fn branch_e_repositorio_do_mesmo_workspace_disputam_o_mesmo_lock() {
        let workspace = WorkspaceId("ws_lock_teste_f".to_string());
        let repo = ResourceId::GitRepository(workspace.clone());
        let branch = ResourceId::GitBranch {
            workspace,
            name: "main".to_string(),
        };

        let _guard = acquire(&repo).unwrap();
        let error = acquire(&branch).unwrap_err();
        assert_eq!(error.code(), "RESOURCE_LOCKED");
    }
}
