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

use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::core::process;
use crate::core::state;
use crate::modules::git::errors::{codes, sanitize, GitOperationError};
use crate::modules::git::remote;

use super::models::{
    GithubContext, GithubMergeOutcome, GithubPullRequest, GithubPullRequestDetail,
    GithubPullRequestPlan,
};
use super::provider;

/// Limite de PRs lidos de uma vez.
const PULL_REQUEST_LIMIT: &str = "30";

/// Campos pedidos ao `gh pr view`.
///
/// Um único lugar de propósito: o detalhe exibido e o preflight de merge leem
/// exatamente o MESMO objeto, então a decisão nunca é tomada sobre uma leitura
/// diferente da que a interface mostrou.
const PULL_REQUEST_DETAIL_FIELDS: &str = "number,title,body,state,isDraft,headRefName,baseRefName,headRefOid,author,url,reviewDecision,mergeable,mergeStateStatus,changedFiles,additions,deletions,commits";

/// Métodos de merge que o DEWRENCH sabe pedir.
///
/// Lista fechada: o valor que vem do IPC é COMPARADO com estas chaves e o que
/// chega à `gh` é a flag constante ao lado — a string do frontend nunca vira
/// argumento. Flags administrativas (`--admin`, `--auto`) não estão aqui e não
/// devem entrar: elas existem justamente para contornar as proteções do
/// repositório.
const MERGE_METHODS: &[(&str, &str)] = &[
    ("merge", "--merge"),
    ("squash", "--squash"),
    ("rebase", "--rebase"),
];


/// Fronteira de autoridade do provider.
///
/// O caminho chega do IPC exatamente como no módulo Git, e vale a mesma regra:
/// ele não é credencial. Sem workspace registrado, nem a detecção de contexto
/// acontece — o que também evita que `gh` seja iniciada apontando para um
/// diretório arbitrário da máquina.
fn authority(path: &Path) -> Result<PathBuf, GitOperationError> {
    state::authorize_workspace(&path.to_string_lossy())
        .map(|record| record.scope.root().to_path_buf())
        .map_err(GitOperationError::from)
}

/// Valor externo que vai virar REFERÊNCIA (branch, ref) na linha da `gh`.
///
/// Segunda camada da mesma disciplina do módulo Git: o broker recusa byte nulo,
/// e aqui se recusa o valor que a `gh` leria como OPÇÃO. Sem isto, um
/// `--head` com valor iniciado por `-` transformaria "listar PRs desta branch"
/// em outra invocação inteiramente.
///
/// Não se aplica a texto livre (título, corpo): lá a ambiguidade é resolvida
/// pela forma `--flag=valor`, que não deixa o valor ser lido como opção nem
/// proíbe um título legítimo começando com `-`.
fn reference(value: &str, what: &str) -> Result<String, GitOperationError> {
    process::operand(value.trim())
        .map(str::to_string)
        .map_err(|error| {
            GitOperationError::from(error)
                .with_action(format!("Verifique {what}: o valor não pode começar com '-'."))
        })
}

pub fn get_context(path: &Path) -> Result<GithubContext, GitOperationError> {
    let path = &authority(path)?;
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
    let path = &authority(path)?;
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

    // Validado ANTES de virar argumento; o `String` precisa viver até a
    // execução, por isso é declarado fora do `if`.
    let branch = match head_branch.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) => Some(reference(value, "a branch de origem")?),
        None => None,
    };

    if let Some(branch) = branch.as_deref() {
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
    let path = &authority(path)?;
    ensure_usable(path)?;

    let title = title.trim();

    if title.is_empty() {
        return Err(GitOperationError::new(
            codes::PROVIDER_COMMAND_FAILED,
            "O título do pull request não pode estar vazio.",
        ));
    }

    let head = reference(head, "a branch de origem")?;
    let base = match base.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) => Some(reference(value, "a branch de destino")?),
        None => None,
    };

    // Texto livre vai na forma `--flag=valor`: não pode ser lido como opção, e
    // um título que legitimamente comece com `-` continua permitido.
    let title_arg = format!("--title={title}");
    let body_arg = format!("--body={body}");
    let base_arg = base.as_deref().map(|value| format!("--base={value}"));

    let mut args: Vec<&str> = vec![
        "pr",
        "create",
        title_arg.as_str(),
        body_arg.as_str(),
        "--head",
        head.as_str(),
    ];

    if let Some(base_arg) = base_arg.as_deref() {
        args.push(base_arg);
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

    let branch = match branch.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) => Some(reference(value, "a branch")?),
        None => None,
    };

    let target = match branch.as_deref() {
        Some(branch) => format!("{web_url}/tree/{branch}"),
        None => web_url,
    };

    if context.cli_available {
        let mut args: Vec<&str> = vec!["browse"];
        if let Some(branch) = branch.as_deref() {
            args.push("--branch");
            args.push(branch);
        }

        // Falha ao abrir o navegador não é falha da operação: a URL continua
        // sendo o resultado útil, e a interface a mostra.
        let _ = provider::run(path, &args);
    }

    Ok(target)
}

// ============================================================================
// PULL REQUEST — LEITURA
// ============================================================================

/// Detalhe de um pull request específico.
pub fn get_pull_request(
    path: &Path,
    number: u64,
) -> Result<GithubPullRequestDetail, GitOperationError> {
    let path = &authority(path)?;
    ensure_usable(path)?;

    read_detail(path, number)
}

/// Leitura crua do PR. Usada tanto pela exibição quanto pelo preflight — de
/// propósito: decidir sobre um objeto diferente do que foi mostrado é
/// exatamente o buraco que o preflight existe para fechar.
fn read_detail(path: &Path, number: u64) -> Result<GithubPullRequestDetail, GitOperationError> {
    let number_arg = number.to_string();

    let output = provider::run(
        path,
        &["pr", "view", number_arg.as_str(), "--json", PULL_REQUEST_DETAIL_FIELDS],
    )
    .map_err(|error| unavailable(error.to_string()))?;

    if !output.success {
        return Err(command_failed(
            &format!("ler o pull request #{number}"),
            &output.stderr,
        ));
    }

    parse_pull_request_detail(&output.stdout).ok_or_else(|| {
        GitOperationError::new(
            codes::PROVIDER_COMMAND_FAILED,
            format!("A resposta da 'gh' para o pull request #{number} não pôde ser lida."),
        )
    })
}

/// Diff do pull request, no mesmo formato unificado que o resto do app já sabe
/// renderizar. O texto NÃO é reescrito: o parser do frontend depende do byte
/// exato, e é a mesma regra que vale para `get_commit_diff`.
pub fn get_pull_request_diff(path: &Path, number: u64) -> Result<String, GitOperationError> {
    let path = &authority(path)?;
    ensure_usable(path)?;

    let number_arg = number.to_string();

    let output = provider::run(path, &["pr", "diff", number_arg.as_str()])
        .map_err(|error| unavailable(error.to_string()))?;

    if !output.success {
        return Err(command_failed(
            &format!("ler o diff do pull request #{number}"),
            &output.stderr,
        ));
    }

    Ok(output.stdout)
}

fn text(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn optional_text(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn count(value: &Value, key: &str) -> u64 {
    value.get(key).and_then(Value::as_u64).unwrap_or(0)
}

pub fn parse_pull_request_detail(raw: &str) -> Option<GithubPullRequestDetail> {
    let value = serde_json::from_str::<Value>(raw).ok()?;

    Some(GithubPullRequestDetail {
        number: value.get("number")?.as_u64()?,
        title: text(&value, "title"),
        body: text(&value, "body"),
        state: value
            .get("state")
            .and_then(Value::as_str)
            .unwrap_or("UNKNOWN")
            .to_string(),
        is_draft: value.get("isDraft").and_then(Value::as_bool).unwrap_or(false),
        head_branch: text(&value, "headRefName"),
        base_branch: text(&value, "baseRefName"),
        head_sha: optional_text(&value, "headRefOid"),
        author: value
            .get("author")
            .and_then(|author| author.get("login"))
            .and_then(Value::as_str)
            .map(str::to_string),
        url: text(&value, "url"),
        review_decision: optional_text(&value, "reviewDecision"),
        mergeable: optional_text(&value, "mergeable"),
        merge_state_status: optional_text(&value, "mergeStateStatus"),
        changed_files: count(&value, "changedFiles"),
        additions: count(&value, "additions"),
        deletions: count(&value, "deletions"),
        commit_count: value
            .get("commits")
            .and_then(Value::as_array)
            .map(|items| items.len() as u64)
            .unwrap_or(0),
    })
}

// ============================================================================
// PULL REQUEST — PREFLIGHT
// ============================================================================

/// Preflight de merge/close.
///
/// Read-only: nada é alterado aqui. O que ele produz é a base da confirmação
/// da interface — e a MESMA função é recalculada dentro da execução, o que faz
/// de `blocked` uma regra e não um aviso.
pub fn get_pull_request_plan(
    path: &Path,
    number: u64,
) -> Result<GithubPullRequestPlan, GitOperationError> {
    let path = &authority(path)?;
    ensure_usable(path)?;

    plan_for(path, number)
}

fn plan_for(path: &Path, number: u64) -> Result<GithubPullRequestPlan, GitOperationError> {
    let detail = read_detail(path, number)?;
    let methods = read_allowed_merge_methods(path);

    Ok(build_plan(detail, methods))
}

/// Métodos de merge habilitados NO REPOSITÓRIO.
///
/// Falha de leitura devolve lista vazia, e lista vazia bloqueia o merge: não
/// saber o que é permitido não pode virar permissão.
fn read_allowed_merge_methods(path: &Path) -> Vec<String> {
    let Ok(output) = provider::run(
        path,
        &[
            "repo",
            "view",
            "--json",
            "mergeCommitAllowed,squashMergeAllowed,rebaseMergeAllowed",
        ],
    ) else {
        return Vec::new();
    };

    if !output.success {
        return Vec::new();
    }

    parse_merge_methods(&output.stdout)
}

pub fn parse_merge_methods(raw: &str) -> Vec<String> {
    let Ok(value) = serde_json::from_str::<Value>(raw) else {
        return Vec::new();
    };

    let allowed = |key: &str| value.get(key).and_then(Value::as_bool).unwrap_or(false);

    let mut methods = Vec::new();
    if allowed("mergeCommitAllowed") {
        methods.push("merge".to_string());
    }
    if allowed("squashMergeAllowed") {
        methods.push("squash".to_string());
    }
    if allowed("rebaseMergeAllowed") {
        methods.push("rebase".to_string());
    }

    methods
}

/// Traduz o estado reportado pelo GitHub em permissão, avisos e bloqueio.
///
/// Função pura: é aqui que "pode mesclar?" é respondido, e por isso ela é
/// testável sem rede, sem `gh` e sem repositório. Deny-by-default — estado
/// desconhecido bloqueia.
pub fn build_plan(
    detail: GithubPullRequestDetail,
    available_methods: Vec<String>,
) -> GithubPullRequestPlan {
    let state = detail.state.to_uppercase();
    let merge_state = detail
        .merge_state_status
        .clone()
        .unwrap_or_default()
        .to_uppercase();
    let mergeable = detail.mergeable.clone().unwrap_or_default().to_uppercase();

    let mut warnings: Vec<String> = Vec::new();

    let mut blocked = if state != "OPEN" {
        Some(format!(
            "O pull request não está aberto (estado {state}); não há merge a fazer."
        ))
    } else if detail.is_draft || merge_state == "DRAFT" {
        Some(
            "O pull request é um rascunho. Marque-o como pronto para revisão antes de mesclar."
                .to_string(),
        )
    } else if mergeable == "CONFLICTING" || merge_state == "DIRTY" {
        Some(
            "Há conflitos com a branch de destino. Resolva-os antes de mesclar.".to_string(),
        )
    } else if merge_state == "BLOCKED" {
        Some(
            "O GitHub bloqueou o merge: faltam checks obrigatórios, revisões aprovadas ou permissão neste repositório."
                .to_string(),
        )
    } else if merge_state == "BEHIND" {
        Some(
            "A branch de origem está atrás da branch de destino e este repositório exige atualização antes do merge."
                .to_string(),
        )
    } else if mergeable.is_empty() || mergeable == "UNKNOWN" || merge_state.is_empty() || merge_state == "UNKNOWN" {
        Some(
            "O GitHub ainda não calculou se este pull request pode ser mesclado.".to_string(),
        )
    } else {
        None
    };

    if blocked.is_none() && available_methods.is_empty() {
        blocked = Some(
            "Nenhum método de merge está habilitado neste repositório, ou a permissão para lê-los não existe."
                .to_string(),
        );
    }

    if merge_state == "UNSTABLE" {
        warnings.push(
            "Há checks falhando ou ainda em execução; o repositório permite mesclar mesmo assim."
                .to_string(),
        );
    }

    match detail.review_decision.as_deref() {
        Some("CHANGES_REQUESTED") => {
            warnings.push("Uma revisão pediu alterações.".to_string());
        }
        Some("REVIEW_REQUIRED") => {
            warnings.push("Este pull request ainda exige revisão aprovada.".to_string());
        }
        None if state == "OPEN" => {
            warnings.push("Este pull request ainda não tem decisão de revisão.".to_string());
        }
        _ => {}
    }

    if detail.head_sha.is_none() {
        warnings.push(
            "O GitHub não informou o commit de topo da origem; o merge não poderá ser vinculado a um estado exato."
                .to_string(),
        );
    }

    let recommended_method = ["merge", "squash", "rebase"]
        .iter()
        .find(|candidate| available_methods.iter().any(|allowed| allowed.as_str() == **candidate))
        .map(|candidate| (*candidate).to_string());

    GithubPullRequestPlan {
        number: detail.number,
        title: detail.title,
        state: detail.state,
        is_draft: detail.is_draft,
        head_branch: detail.head_branch,
        base_branch: detail.base_branch,
        head_sha: detail.head_sha,
        url: detail.url,
        mergeable: detail.mergeable,
        merge_state_status: detail.merge_state_status,
        review_decision: detail.review_decision,
        available_methods,
        recommended_method,
        warnings,
        blocked,
    }
}

// ============================================================================
// PULL REQUEST — MUTAÇÃO REMOTA
// ============================================================================

/// Recusa a execução quando o estado revisado não é mais o estado atual.
fn ensure_reviewed_state(
    current_sha: Option<&str>,
    expected_head_sha: Option<&str>,
) -> Result<(), GitOperationError> {
    let Some(expected) = expected_head_sha
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(());
    };

    if current_sha == Some(expected) {
        return Ok(());
    }

    Err(GitOperationError::new(
        codes::PROVIDER_COMMAND_FAILED,
        "O pull request mudou desde a revisão.",
    )
    .with_action("Recarregue o pull request, revise o novo estado e confirme novamente."))
}

fn useful_lines(raw: &str) -> Vec<String> {
    sanitize(raw.to_string())
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

/// Executa o merge revalidando o preflight imediatamente antes de mutar.
///
/// Três barreiras, todas do backend:
///
/// 1. `method` é comparado com uma lista fechada — a string do IPC nunca vira
///    argumento — e conferida contra o que o repositório permite.
/// 2. O plano é recalculado agora; `blocked` presente aborta.
/// 3. `expected_head_sha` (o commit que o usuário revisou) precisa continuar
///    sendo o topo da origem, e a própria `gh` recebe `--match-head-commit`,
///    de modo que o GitHub recusa o merge se a branch andar entre a
///    revalidação e a execução.
pub fn merge_pull_request(
    path: &Path,
    number: u64,
    method: &str,
    delete_branch: bool,
    expected_head_sha: Option<&str>,
) -> Result<GithubMergeOutcome, GitOperationError> {
    let path = &authority(path)?;
    ensure_usable(path)?;

    let method = method.trim();
    let flag = MERGE_METHODS
        .iter()
        .find(|(name, _)| *name == method)
        .map(|(_, flag)| *flag)
        .ok_or_else(|| {
            GitOperationError::new(
                codes::PROVIDER_COMMAND_FAILED,
                "Método de merge desconhecido.",
            )
            .with_action("Use 'merge', 'squash' ou 'rebase'.")
        })?;

    let plan = plan_for(path, number)?;

    if let Some(reason) = plan.blocked.clone() {
        return Err(GitOperationError::new(
            codes::PROVIDER_COMMAND_FAILED,
            "O estado atual do pull request não permite o merge.",
        )
        .with_details(reason)
        .with_action("Atualize o painel e revise o pull request novamente."));
    }

    if !plan.available_methods.iter().any(|allowed| allowed.as_str() == method) {
        return Err(GitOperationError::new(
            codes::PROVIDER_COMMAND_FAILED,
            "Este repositório não permite esse método de merge.",
        )
        .with_details(format!(
            "Permitidos agora: {}.",
            plan.available_methods.join(", ")
        )));
    }

    ensure_reviewed_state(plan.head_sha.as_deref(), expected_head_sha)?;

    let number_arg = number.to_string();
    let match_arg = plan
        .head_sha
        .as_deref()
        .map(|sha| format!("--match-head-commit={sha}"));

    let mut args: Vec<&str> = vec!["pr", "merge", number_arg.as_str(), flag];

    if let Some(match_arg) = match_arg.as_deref() {
        args.push(match_arg);
    }

    // Apagar a branch é destrutivo e permanece opt-in: só entra na linha de
    // comando quando a interface pediu explicitamente.
    if delete_branch {
        args.push("--delete-branch");
    }

    let output = provider::run(path, &args).map_err(|error| unavailable(error.to_string()))?;

    if !output.success {
        return Err(command_failed(
            &format!("mesclar o pull request #{number}"),
            &output.stderr,
        ));
    }

    let mut notes = useful_lines(&output.stdout);
    notes.extend(useful_lines(&output.stderr));

    Ok(GithubMergeOutcome {
        number,
        method: method.to_string(),
        merged: true,
        deleted_branch: delete_branch,
        url: plan.url,
        notes,
    })
}

/// Fecha o pull request sem mesclar, pelo mesmo caminho preventivo do merge.
pub fn close_pull_request(
    path: &Path,
    number: u64,
    delete_branch: bool,
    expected_head_sha: Option<&str>,
) -> Result<GithubPullRequestDetail, GitOperationError> {
    let path = &authority(path)?;
    ensure_usable(path)?;

    let current = read_detail(path, number)?;

    if !current.state.eq_ignore_ascii_case("OPEN") {
        return Err(GitOperationError::new(
            codes::PROVIDER_COMMAND_FAILED,
            format!(
                "O pull request #{number} não está aberto (estado {}).",
                current.state
            ),
        ));
    }

    ensure_reviewed_state(current.head_sha.as_deref(), expected_head_sha)?;

    let number_arg = number.to_string();
    let mut args: Vec<&str> = vec!["pr", "close", number_arg.as_str()];

    if delete_branch {
        args.push("--delete-branch");
    }

    let output = provider::run(path, &args).map_err(|error| unavailable(error.to_string()))?;

    if !output.success {
        return Err(command_failed(
            &format!("fechar o pull request #{number}"),
            &output.stderr,
        ));
    }

    // O estado devolvido é relido do servidor, não presumido a partir do
    // sucesso do processo.
    read_detail(path, number)
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

    // -- Validação de argumento -------------------------------------------

    #[test]
    fn referencia_iniciada_por_hifen_e_recusada() {
        let error = reference("--repo=outro/repo", "a branch").unwrap_err();
        assert_eq!(error.code, "ARGUMENT_REJECTED");
    }

    #[test]
    fn referencia_vazia_e_recusada() {
        assert!(reference("   ", "a branch").is_err());
    }

    #[test]
    fn referencia_comum_passa() {
        assert_eq!(reference(" feat/x ", "a branch").unwrap(), "feat/x");
    }

    // -- Detalhe -----------------------------------------------------------

    fn detail_json(state: &str, mergeable: &str, merge_state: &str, draft: bool) -> String {
        format!(
            r#"{{"number":7,"title":"T","body":"corpo","state":"{state}","isDraft":{draft},
              "headRefName":"feat/x","baseRefName":"main","headRefOid":"abc123",
              "author":{{"login":"korteexz"}},"url":"https://github.com/o/r/pull/7",
              "reviewDecision":"APPROVED","mergeable":"{mergeable}",
              "mergeStateStatus":"{merge_state}","changedFiles":3,"additions":10,
              "deletions":2,"commits":[{{"oid":"a"}},{{"oid":"b"}}]}}"#
        )
    }

    fn detail(state: &str, mergeable: &str, merge_state: &str, draft: bool) -> GithubPullRequestDetail {
        parse_pull_request_detail(&detail_json(state, mergeable, merge_state, draft))
            .expect("detalhe legível")
    }

    #[test]
    fn detalhe_do_pr_e_lido_do_json_da_gh() {
        let parsed = detail("OPEN", "MERGEABLE", "CLEAN", false);

        assert_eq!(parsed.number, 7);
        assert_eq!(parsed.head_sha.as_deref(), Some("abc123"));
        assert_eq!(parsed.commit_count, 2);
        assert_eq!(parsed.changed_files, 3);
        assert_eq!(parsed.author.as_deref(), Some("korteexz"));
    }

    #[test]
    fn detalhe_invalido_nao_derruba_a_leitura() {
        assert!(parse_pull_request_detail("isto não é json").is_none());
        assert!(parse_pull_request_detail("{}").is_none());
    }

    // -- Métodos permitidos ------------------------------------------------

    #[test]
    fn metodos_de_merge_vem_do_repositorio() {
        let methods = parse_merge_methods(
            r#"{"mergeCommitAllowed":false,"squashMergeAllowed":true,"rebaseMergeAllowed":true}"#,
        );
        assert_eq!(methods, vec!["squash".to_string(), "rebase".to_string()]);
    }

    #[test]
    fn leitura_ilegivel_de_metodos_nao_vira_permissao() {
        assert!(parse_merge_methods("erro").is_empty());
    }

    // -- Preflight ---------------------------------------------------------

    fn plan(state: &str, mergeable: &str, merge_state: &str, draft: bool) -> GithubPullRequestPlan {
        build_plan(
            detail(state, mergeable, merge_state, draft),
            vec!["merge".to_string(), "squash".to_string()],
        )
    }

    #[test]
    fn pr_limpo_nao_e_bloqueado() {
        let plan = plan("OPEN", "MERGEABLE", "CLEAN", false);
        assert!(plan.blocked.is_none());
        assert_eq!(plan.recommended_method.as_deref(), Some("merge"));
    }

    #[test]
    fn conflito_bloqueia() {
        assert!(plan("OPEN", "CONFLICTING", "DIRTY", false).blocked.is_some());
    }

    #[test]
    fn checks_ou_revisao_pendentes_bloqueiam() {
        assert!(plan("OPEN", "MERGEABLE", "BLOCKED", false).blocked.is_some());
    }

    #[test]
    fn branch_atrasada_bloqueia() {
        assert!(plan("OPEN", "MERGEABLE", "BEHIND", false).blocked.is_some());
    }

    #[test]
    fn rascunho_bloqueia() {
        assert!(plan("OPEN", "MERGEABLE", "DRAFT", true).blocked.is_some());
    }

    #[test]
    fn pr_ja_fechado_bloqueia() {
        assert!(plan("MERGED", "MERGEABLE", "CLEAN", false).blocked.is_some());
    }

    /// Deny-by-default: não saber é bloquear.
    #[test]
    fn estado_desconhecido_bloqueia() {
        assert!(plan("OPEN", "UNKNOWN", "UNKNOWN", false).blocked.is_some());
    }

    #[test]
    fn sem_metodo_permitido_o_merge_e_bloqueado() {
        let plan = build_plan(detail("OPEN", "MERGEABLE", "CLEAN", false), Vec::new());
        assert!(plan.blocked.is_some());
        assert!(plan.recommended_method.is_none());
    }

    #[test]
    fn checks_instaveis_avisam_sem_bloquear() {
        let plan = plan("OPEN", "MERGEABLE", "UNSTABLE", false);
        assert!(plan.blocked.is_none());
        assert!(!plan.warnings.is_empty());
    }

    // -- Revalidação -------------------------------------------------------

    #[test]
    fn estado_diferente_do_revisado_aborta() {
        let error = ensure_reviewed_state(Some("novo"), Some("abc123")).unwrap_err();
        assert!(error.suggested_action.is_some());
    }

    #[test]
    fn mesmo_estado_revisado_prossegue() {
        assert!(ensure_reviewed_state(Some("abc123"), Some("abc123")).is_ok());
    }

    #[test]
    fn sem_estado_esperado_a_revalidacao_nao_bloqueia() {
        assert!(ensure_reviewed_state(Some("abc123"), None).is_ok());
    }

    #[test]
    fn origem_sem_sha_conhecido_com_estado_esperado_aborta() {
        assert!(ensure_reviewed_state(None, Some("abc123")).is_err());
    }

    // -- Método de merge ---------------------------------------------------

    #[test]
    fn nenhuma_flag_administrativa_esta_no_catalogo() {
        for (_, flag) in MERGE_METHODS {
            assert!(*flag == "--merge" || *flag == "--squash" || *flag == "--rebase");
        }
    }
}
