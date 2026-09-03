//! Regras do provider GitHub.
//!
//! Duas fronteiras são respeitadas aqui:
//!
//! * **Detecção vem do Git**, não da rede: o repositório é reconhecido como
//!   GitHub pela URL do remote, que o módulo Git já sabe interpretar. Isso
//!   funciona offline e sem `gh`.
//! * **Dados do servidor vêm da `gh`**, que já resolve autenticação. O DEWRENCH
//!   nunca vê, guarda ou transporta token.
//!
//! Toda função degrada: sem `gh`, sem autenticação ou sem remote do GitHub, o
//! resultado descreve a limitação em vez de falhar.

use std::path::Path;

use serde_json::Value;

use crate::modules::git::errors::{codes, sanitize, GitOperationError};
use crate::modules::git::remote;

use super::models::{GithubContext, GithubPullRequest};
use super::provider;

/// Limite de PRs lidos de uma vez.
const PULL_REQUEST_LIMIT: &str = "30";

pub fn get_context(path: &Path) -> Result<GithubContext, GitOperationError> {
    let view = remote::get_view(path)?;

    // Preferir o remote que a branch atual usa; senão, o primeiro do GitHub.
    let github_remote = view
        .remotes
        .iter()
        .find(|item| item.is_upstream && item.identity.provider == "github")
        .or_else(|| {
            view.remotes
                .iter()
                .find(|item| item.is_origin && item.identity.provider == "github")
        })
        .or_else(|| {
            view.remotes
                .iter()
                .find(|item| item.identity.provider == "github")
        });

    let Some(github_remote) = github_remote else {
        return Ok(GithubContext {
            detected: false,
            cli_available: provider::is_installed(path),
            authenticated: false,
            owner: None,
            repository: None,
            remote_name: None,
            remote_url: None,
            default_branch: None,
            current_branch: view.current_branch.clone(),
            web_url: None,
            limitation: Some("Nenhum remote deste repositório aponta para o GitHub.".to_string()),
        });
    };

    let owner = github_remote.identity.owner.clone();
    let repository = github_remote.identity.repository.clone();
    let web_url = match (&owner, &repository) {
        (Some(owner), Some(repository)) => Some(format!("https://github.com/{owner}/{repository}")),
        _ => None,
    };

    let cli_available = provider::is_installed(path);
    let authenticated = cli_available && provider::is_authenticated(path);

    // A branch padrão vem do servidor e só existe com gh autenticada. Sem ela,
    // fica ausente — nunca chutamos "main".
    let default_branch = authenticated.then(|| read_default_branch(path)).flatten();

    let limitation = if !cli_available {
        Some("A CLI 'gh' não está instalada; o DEWRENCH mostra apenas o que o Git local sabe.".to_string())
    } else if !authenticated {
        Some("A CLI 'gh' não está autenticada. Rode 'gh auth login' para ver pull requests.".to_string())
    } else {
        None
    };

    Ok(GithubContext {
        detected: true,
        cli_available,
        authenticated,
        owner,
        repository,
        remote_name: Some(github_remote.name.clone()),
        remote_url: Some(sanitize(github_remote.fetch_url.clone())),
        default_branch,
        current_branch: view.current_branch,
        web_url,
        limitation,
    })
}

fn read_default_branch(path: &Path) -> Option<String> {
    let output = provider::run(
        path,
        &["repo", "view", "--json", "defaultBranchRef", "-q", ".defaultBranchRef.name"],
    )
    .ok()?;

    if !output.success {
        return None;
    }

    let value = output.stdout.trim().to_string();
    (!value.is_empty()).then_some(value)
}

/// Pull requests do repositório, opcionalmente filtrados por branch de origem.
pub fn list_pull_requests(
    path: &Path,
    head_branch: Option<&str>,
) -> Result<Vec<GithubPullRequest>, GitOperationError> {
    ensure_usable(path)?;

    let mut args: Vec<&str> = vec![
        "pr",
        "list",
        "--state",
        "all",
        "--limit",
        PULL_REQUEST_LIMIT,
        "--json",
        "number,title,state,isDraft,headRefName,baseRefName,author,url,reviewDecision",
    ];

    let branch = head_branch.map(str::trim).filter(|value| !value.is_empty());
    if let Some(branch) = branch {
        args.push("--head");
        args.push(branch);
    }

    let output = provider::run(path, &args).map_err(|error| unavailable(error.to_string()))?;

    if !output.success {
        return Err(command_failed("listar pull requests", &output.stderr));
    }

    Ok(parse_pull_requests(&output.stdout))
}

pub fn parse_pull_requests(raw: &str) -> Vec<GithubPullRequest> {
    let Ok(Value::Array(items)) = serde_json::from_str::<Value>(raw) else {
        return Vec::new();
    };

    items
        .iter()
        .filter_map(|item| {
            Some(GithubPullRequest {
                number: item.get("number")?.as_u64()?,
                title: item.get("title")?.as_str().unwrap_or_default().to_string(),
                state: item
                    .get("state")
                    .and_then(Value::as_str)
                    .unwrap_or("UNKNOWN")
                    .to_string(),
                is_draft: item.get("isDraft").and_then(Value::as_bool).unwrap_or(false),
                head_branch: item
                    .get("headRefName")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                base_branch: item
                    .get("baseRefName")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                author: item
                    .get("author")
                    .and_then(|author| author.get("login"))
                    .and_then(Value::as_str)
                    .map(str::to_string),
                url: item
                    .get("url")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                review_decision: item
                    .get("reviewDecision")
                    .and_then(Value::as_str)
                    .filter(|value| !value.is_empty())
                    .map(str::to_string),
            })
        })
        .collect()
}

/// Cria um pull request a partir da branch informada.
///
/// Não é interativo de propósito: um prompt da `gh` dentro do app ficaria
/// invisível e travaria a operação. Título e base vêm da interface.
pub fn create_pull_request(
    path: &Path,
    title: &str,
    body: &str,
    base: Option<&str>,
    head: &str,
    draft: bool,
) -> Result<String, GitOperationError> {
    ensure_usable(path)?;

    let title = title.trim();
    let head = head.trim();

    if title.is_empty() {
        return Err(GitOperationError::new(
            codes::PROVIDER_COMMAND_FAILED,
            "O título do pull request não pode estar vazio.",
        ));
    }

    if head.is_empty() || head.starts_with('-') {
        return Err(GitOperationError::new(
            codes::PROVIDER_COMMAND_FAILED,
            "Branch de origem inválida.",
        ));
    }

    let mut args: Vec<&str> = vec!["pr", "create", "--title", title, "--head", head];

    if !body.trim().is_empty() {
        args.push("--body");
        args.push(body);
    } else {
        args.push("--body");
        args.push("");
    }

    let base = base.map(str::trim).filter(|value| !value.is_empty());
    if let Some(base) = base {
        args.push("--base");
        args.push(base);
    }

    if draft {
        args.push("--draft");
    }

    let output = provider::run(path, &args).map_err(|error| unavailable(error.to_string()))?;

    if !output.success {
        return Err(command_failed("criar o pull request", &output.stderr));
    }

    // A gh imprime a URL do PR criado na última linha útil.
    let url = output
        .stdout
        .lines()
        .map(str::trim)
        .rfind(|line| line.starts_with("https://"))
        .unwrap_or("")
        .to_string();

    Ok(url)
}

/// Abre o repositório (ou uma branch) no navegador, via `gh browse`.
///
/// Delegar para a `gh` evita adicionar ao DEWRENCH a capacidade de abrir
/// processos externos arbitrários só para isso.
pub fn open_in_browser(path: &Path, branch: Option<&str>) -> Result<String, GitOperationError> {
    let context = get_context(path)?;

    let Some(web_url) = context.web_url.clone() else {
        return Err(GitOperationError::new(
            codes::PROVIDER_UNAVAILABLE,
            "Este repositório não tem um remote do GitHub reconhecível.",
        ));
    };

    let target = match branch {
        Some(branch) if !branch.trim().is_empty() => {
            format!("{web_url}/tree/{}", branch.trim())
        }
        _ => web_url,
    };

    if context.cli_available {
        let mut args: Vec<&str> = vec!["browse"];
        let branch = branch.map(str::trim).filter(|value| !value.is_empty());
        if let Some(branch) = branch {
            args.push("--branch");
            args.push(branch);
        }

        // Falha ao abrir o navegador não é falha da operação: a URL continua
        // sendo o resultado útil, e a interface a mostra.
        let _ = provider::run(path, &args);
    }

    Ok(target)
}

fn ensure_usable(path: &Path) -> Result<(), GitOperationError> {
    if !provider::is_installed(path) {
        return Err(unavailable(
            "A CLI 'gh' não está instalada nesta máquina.".to_string(),
        ));
    }

    if !provider::is_authenticated(path) {
        return Err(GitOperationError::new(
            codes::PROVIDER_NOT_AUTHENTICATED,
            "A CLI 'gh' não está autenticada.",
        )
        .with_action("Rode 'gh auth login' no terminal e tente novamente."));
    }

    Ok(())
}

fn unavailable(details: String) -> GitOperationError {
    GitOperationError::new(
        codes::PROVIDER_UNAVAILABLE,
        "A integração com o GitHub não está disponível.",
    )
    .with_details(sanitize(details))
    .with_action("Instale a CLI 'gh' para habilitar pull requests. O Git continua funcionando sem ela.")
}

fn command_failed(action: &str, stderr: &str) -> GitOperationError {
    GitOperationError::new(
        codes::PROVIDER_COMMAND_FAILED,
        format!("Não foi possível {action}."),
    )
    .with_details(sanitize(stderr.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pull_requests_sao_lidos_do_json_da_gh() {
        let raw = r#"[
            {"number":12,"title":"Adiciona push","state":"OPEN","isDraft":false,
             "headRefName":"feat/push","baseRefName":"main",
             "author":{"login":"korteexz"},
             "url":"https://github.com/o/r/pull/12","reviewDecision":"APPROVED"},
            {"number":9,"title":"Rascunho","state":"OPEN","isDraft":true,
             "headRefName":"wip","baseRefName":"main","author":null,
             "url":"https://github.com/o/r/pull/9","reviewDecision":""}
        ]"#;

        let prs = parse_pull_requests(raw);
        assert_eq!(prs.len(), 2);
        assert_eq!(prs[0].number, 12);
        assert_eq!(prs[0].author.as_deref(), Some("korteexz"));
        assert_eq!(prs[0].review_decision.as_deref(), Some("APPROVED"));
        assert!(prs[1].is_draft);
        assert_eq!(prs[1].author, None);
        assert_eq!(prs[1].review_decision, None);
    }

    #[test]
    fn json_invalido_nao_derruba_a_leitura() {
        assert!(parse_pull_requests("isto não é json").is_empty());
        assert!(parse_pull_requests("").is_empty());
    }
}
