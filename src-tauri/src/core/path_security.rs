//! Autoridade de filesystem.
//!
//! A pergunta que este módulo responde NÃO é "esta string parece segura?" e
//! sim "que objeto esta operação vai realmente alcançar?".
//!
//! Por isso a validação nunca é textual: o caminho é resolvido contra o
//! filesystem real (seguindo symlinks, junctions e reparse points) e só então
//! comparado com a raiz canônica do escopo. Comparar strings antes de resolver
//! é exatamente o erro que `../`, symlink e normalização exploram.
//!
//! LIMITE CONHECIDO (residual): a validação e o uso do caminho acontecem em
//! momentos diferentes. Um atacante com escrita no diretório pode trocar um
//! componente por symlink ENTRE a checagem e a operação (TOCTOU). Fechar isso
//! exige `openat`/handles por componente, que não estão implementados aqui.

use std::path::{Component, Path, PathBuf};

use super::error::CoreError;

/// Raiz canônica dentro da qual uma operação pode agir.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scope {
    root: PathBuf,
}

impl Scope {
    /// Cria um escopo a partir de um diretório existente.
    ///
    /// Canonicaliza na criação: a raiz precisa ser um objeto real, não um
    /// texto. Um escopo cuja raiz não existe seria autoridade sobre nada.
    pub fn create(root: &Path) -> Result<Scope, CoreError> {
        let canonical = root.canonicalize().map_err(|error| CoreError::PathUnresolvable {
            attempted: root.to_string_lossy().into_owned(),
            reason: error.to_string(),
        })?;

        if !canonical.is_dir() {
            return Err(CoreError::PathUnresolvable {
                attempted: display_path(&canonical),
                reason: "não é um diretório".to_string(),
            });
        }

        Ok(Scope { root: canonical })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Caminho da raiz pronto para exibição.
    pub fn display_root(&self) -> String {
        display_path(&self.root)
    }

    /// Resolve um caminho candidato e garante que ele permanece no escopo.
    ///
    /// Aceita candidato relativo (interpretado a partir da raiz) ou absoluto.
    /// Alvos inexistentes são permitidos — criar arquivo é legítimo — mas o
    /// ANCESTRAL existente mais profundo é canonicalizado, de modo que um
    /// symlink no meio do caminho não escapa da checagem.
    pub fn resolve(&self, candidate: &Path) -> Result<PathBuf, CoreError> {
        let attempted = candidate.to_string_lossy().into_owned();

        if attempted.is_empty() {
            return Err(CoreError::PathUnresolvable {
                attempted,
                reason: "caminho vazio".to_string(),
            });
        }

        if attempted.contains('\0') {
            return Err(CoreError::PathUnresolvable {
                attempted: attempted.replace('\0', "\\0"),
                reason: "caminho contém byte nulo".to_string(),
            });
        }

        // Defesa em profundidade: `..` nunca é necessário nos caminhos que o
        // DEWRENCH manipula (vêm do próprio Git, relativos à raiz). Recusar
        // antes de resolver elimina uma classe inteira sem depender só da
        // canonicalização.
        if candidate
            .components()
            .any(|component| matches!(component, Component::ParentDir))
        {
            return Err(CoreError::PathOutsideScope {
                attempted,
                scope: self.display_root(),
            });
        }

        let joined = if candidate.is_absolute() {
            candidate.to_path_buf()
        } else {
            self.root.join(candidate)
        };

        let resolved = resolve_deepest_existing(&joined)?;

        if !is_within(&self.root, &resolved) {
            return Err(CoreError::PathOutsideScope {
                attempted,
                scope: self.display_root(),
            });
        }

        Ok(resolved)
    }

    /// Resolve garantindo que o alvo EXISTE. Usado onde operar sobre um
    /// caminho inexistente seria erro, não criação.
    pub fn resolve_existing(&self, candidate: &Path) -> Result<PathBuf, CoreError> {
        let resolved = self.resolve(candidate)?;

        if !resolved.exists() {
            return Err(CoreError::PathUnresolvable {
                attempted: display_path(&resolved),
                reason: "o alvo não existe".to_string(),
            });
        }

        Ok(resolved)
    }

    /// O caminho relativo do alvo dentro do escopo, para virar `ResourceId`.
    pub fn relative_of(&self, resolved: &Path) -> Result<String, CoreError> {
        resolved
            .strip_prefix(&self.root)
            .map(|relative| relative.to_string_lossy().replace('\\', "/"))
            .map_err(|_| CoreError::PathOutsideScope {
                attempted: display_path(resolved),
                scope: self.display_root(),
            })
    }
}

/// Canonicaliza o ancestral existente mais profundo e reanexa o resto.
///
/// `canonicalize` falha em caminho inexistente; sem isto, criar um arquivo
/// novo seria indistinguível de um erro, e a tentação seria pular a checagem
/// justamente no caso de escrita.
fn resolve_deepest_existing(path: &Path) -> Result<PathBuf, CoreError> {
    let mut existing = path.to_path_buf();
    let mut tail: Vec<std::ffi::OsString> = Vec::new();

    loop {
        if existing.exists() {
            break;
        }

        let Some(name) = existing.file_name().map(|name| name.to_os_string()) else {
            return Err(CoreError::PathUnresolvable {
                attempted: path.to_string_lossy().into_owned(),
                reason: "nenhum ancestral existente".to_string(),
            });
        };

        tail.push(name);

        if !existing.pop() {
            return Err(CoreError::PathUnresolvable {
                attempted: path.to_string_lossy().into_owned(),
                reason: "nenhum ancestral existente".to_string(),
            });
        }
    }

    let mut resolved = existing.canonicalize().map_err(|error| CoreError::PathUnresolvable {
        attempted: path.to_string_lossy().into_owned(),
        reason: error.to_string(),
    })?;

    for name in tail.into_iter().rev() {
        resolved.push(name);
    }

    Ok(resolved)
}

/// Contenção por COMPONENTE, nunca por prefixo de string.
///
/// `"/repo-malicioso"` tem o prefixo textual `"/repo"`, mas não está dentro
/// dele. `Path::starts_with` compara componentes e não cai nessa; a função
/// existe para deixar a intenção explícita e para tratar a insensibilidade a
/// maiúsculas do Windows.
fn is_within(root: &Path, candidate: &Path) -> bool {
    if candidate.starts_with(root) {
        return true;
    }

    if cfg!(windows) {
        let root_lower = root.to_string_lossy().to_lowercase();
        let candidate_lower = candidate.to_string_lossy().to_lowercase();
        return Path::new(&candidate_lower).starts_with(Path::new(&root_lower));
    }

    false
}

/// Remove o prefixo verbatim do Windows (`\\?\`) para exibição.
///
/// `canonicalize` no Windows devolve `\\?\C:\...`; mostrar isso na interface
/// vaza detalhe de implementação e confunde o usuário.
pub fn display_path(path: &Path) -> String {
    let text = path.to_string_lossy().into_owned();

    if let Some(stripped) = text.strip_prefix(r"\\?\UNC\") {
        return format!(r"\\{stripped}");
    }

    if let Some(stripped) = text.strip_prefix(r"\\?\") {
        return stripped.to_string();
    }

    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    struct Lab {
        root: PathBuf,
    }

    impl Lab {
        fn new(name: &str) -> Lab {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root = std::env::temp_dir().join(format!("dw-path-{name}-{nanos}"));
            fs::create_dir_all(root.join("inside")).unwrap();
            fs::write(root.join("inside/file.txt"), "conteudo").unwrap();
            fs::create_dir_all(root.join("outside")).unwrap();
            fs::write(root.join("outside/secret.txt"), "segredo").unwrap();
            Lab { root }
        }

        fn scope(&self) -> Scope {
            Scope::create(&self.root.join("inside")).unwrap()
        }
    }

    impl Drop for Lab {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn caminho_relativo_dentro_do_escopo_e_aceito() {
        let lab = Lab::new("ok");
        let scope = lab.scope();
        let resolved = scope.resolve(Path::new("file.txt")).unwrap();
        assert!(resolved.ends_with("file.txt"));
    }

    #[test]
    fn traversal_com_pontos_e_recusado() {
        let lab = Lab::new("traversal");
        let scope = lab.scope();

        for payload in [
            "../outside/secret.txt",
            "../../etc/passwd",
            "sub/../../outside/secret.txt",
            "./../outside/secret.txt",
        ] {
            let error = scope.resolve(Path::new(payload)).unwrap_err();
            assert_eq!(error.code(), "PATH_OUTSIDE_SCOPE", "payload: {payload}");
        }
    }

    #[test]
    fn caminho_absoluto_fora_do_escopo_e_recusado() {
        let lab = Lab::new("absolute");
        let scope = lab.scope();
        let outside = lab.root.join("outside/secret.txt");

        let error = scope.resolve(&outside).unwrap_err();
        assert_eq!(error.code(), "PATH_OUTSIDE_SCOPE");
    }

    #[test]
    fn caminho_absoluto_dentro_do_escopo_e_aceito() {
        let lab = Lab::new("absolute-ok");
        let scope = lab.scope();
        let inside = lab.root.join("inside/file.txt");

        assert!(scope.resolve(&inside).is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn symlink_apontando_para_fora_e_recusado() {
        let lab = Lab::new("symlink");
        let scope = lab.scope();

        std::os::unix::fs::symlink(
            lab.root.join("outside/secret.txt"),
            lab.root.join("inside/escape.txt"),
        )
        .unwrap();

        let error = scope.resolve(Path::new("escape.txt")).unwrap_err();
        assert_eq!(error.code(), "PATH_OUTSIDE_SCOPE");
    }

    #[cfg(unix)]
    #[test]
    fn symlink_de_diretorio_no_meio_do_caminho_e_recusado() {
        let lab = Lab::new("symlink-dir");
        let scope = lab.scope();

        std::os::unix::fs::symlink(lab.root.join("outside"), lab.root.join("inside/link")).unwrap();

        // O componente do meio é o symlink; o alvo final nem precisa existir.
        let error = scope.resolve(Path::new("link/secret.txt")).unwrap_err();
        assert_eq!(error.code(), "PATH_OUTSIDE_SCOPE");
    }

    #[cfg(unix)]
    #[test]
    fn symlink_encadeado_para_fora_e_recusado() {
        let lab = Lab::new("symlink-chain");
        let scope = lab.scope();

        std::os::unix::fs::symlink(lab.root.join("outside"), lab.root.join("hop")).unwrap();
        std::os::unix::fs::symlink(lab.root.join("hop"), lab.root.join("inside/chain")).unwrap();

        let error = scope.resolve(Path::new("chain/secret.txt")).unwrap_err();
        assert_eq!(error.code(), "PATH_OUTSIDE_SCOPE");
    }

    #[cfg(unix)]
    #[test]
    fn symlink_interno_continua_permitido() {
        let lab = Lab::new("symlink-inside");
        let scope = lab.scope();

        std::os::unix::fs::symlink(
            lab.root.join("inside/file.txt"),
            lab.root.join("inside/alias.txt"),
        )
        .unwrap();

        assert!(scope.resolve(Path::new("alias.txt")).is_ok());
    }

    #[test]
    fn alvo_inexistente_dentro_do_escopo_e_aceito() {
        let lab = Lab::new("novo");
        let scope = lab.scope();
        assert!(scope.resolve(Path::new("sub/dir/novo.txt")).is_ok());
    }

    #[test]
    fn alvo_inexistente_fora_do_escopo_e_recusado() {
        let lab = Lab::new("novo-fora");
        let scope = lab.scope();
        let error = scope
            .resolve(&lab.root.join("outside/novo.txt"))
            .unwrap_err();
        assert_eq!(error.code(), "PATH_OUTSIDE_SCOPE");
    }

    #[test]
    fn resolve_existing_recusa_alvo_inexistente() {
        let lab = Lab::new("existing");
        let scope = lab.scope();
        let error = scope.resolve_existing(Path::new("nao-existe.txt")).unwrap_err();
        assert_eq!(error.code(), "PATH_UNRESOLVABLE");
    }

    #[test]
    fn byte_nulo_e_recusado() {
        let lab = Lab::new("nul");
        let scope = lab.scope();
        let error = scope.resolve(Path::new("file\0.txt")).unwrap_err();
        assert_eq!(error.code(), "PATH_UNRESOLVABLE");
    }

    #[test]
    fn caminho_vazio_e_recusado() {
        let lab = Lab::new("vazio");
        let scope = lab.scope();
        assert!(scope.resolve(Path::new("")).is_err());
    }

    /// Diretório irmão com o mesmo prefixo textual da raiz.
    ///
    /// `/tmp/x/inside-malicioso` começa com a string `/tmp/x/inside`, e uma
    /// checagem por prefixo de texto aceitaria. A checagem por componente não.
    #[test]
    fn irmao_com_prefixo_textual_nao_e_considerado_dentro() {
        let lab = Lab::new("prefixo");
        let scope = lab.scope();
        let sibling = lab.root.join("inside-malicioso");
        fs::create_dir_all(&sibling).unwrap();
        fs::write(sibling.join("alvo.txt"), "x").unwrap();

        let error = scope.resolve(&sibling.join("alvo.txt")).unwrap_err();
        assert_eq!(error.code(), "PATH_OUTSIDE_SCOPE");
    }

    #[test]
    fn escopo_precisa_ser_diretorio_existente() {
        let lab = Lab::new("escopo");
        assert!(Scope::create(&lab.root.join("inside/file.txt")).is_err());
        assert!(Scope::create(&lab.root.join("nao-existe")).is_err());
    }

    #[test]
    fn relative_of_devolve_caminho_do_recurso() {
        let lab = Lab::new("relative");
        let scope = lab.scope();
        let resolved = scope.resolve(Path::new("file.txt")).unwrap();
        assert_eq!(scope.relative_of(&resolved).unwrap(), "file.txt");
    }

    #[test]
    fn display_path_remove_prefixo_verbatim_do_windows() {
        assert_eq!(
            display_path(Path::new(r"\\?\C:\Users\user\Desktop\DEWRENCH")),
            r"C:\Users\user\Desktop\DEWRENCH"
        );
        assert_eq!(
            display_path(Path::new(r"\\?\UNC\servidor\share\repo")),
            r"\\servidor\share\repo"
        );
        assert_eq!(display_path(Path::new("/home/user/repo")), "/home/user/repo");
    }
}
