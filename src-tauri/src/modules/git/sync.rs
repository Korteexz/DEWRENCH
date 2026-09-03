//! Operações que atravessam a rede: push, fetch e pull.
//!
//! Três invariantes governam este módulo:
//!
//! 1. **Nenhuma operação de rede sem plano.** Push e pull têm um preflight
//!    read-only que devolve origem, destino, contagem e a lista real de
//!    commits. A interface mostra o plano; o usuário decide.
//! 2. **Nada de estratégia implícita.** O pull recebe a estratégia escolhida.
//!    O backend informa quais são possíveis no estado atual e recomenda uma —
//!    mas nunca escolhe sozinho.
//! 3. **Nenhum estado intermediário órfão.** Se a integração entra em
//!    conflito, o módulo desfaz a operação e reporta os arquivos conflitantes.
//!    O DEWRENCH ainda não sabe resolver conflito, e deixar o repositório
//!    parado num merge pela metade seria pior do que não ter feito nada.
//!
//! Todo texto vindo do Git passa por `sanitize` antes de cruzar o IPC: a saída
//! de rede cita URLs de remote, que podem conter credencial embutida.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use super::commits;
use super::errors::{codes, sanitize, GitOperationError};
use super::git_cli;
use super::models::{
    GitFetchOutcome, GitGraphCommit, GitPullOutcome, GitPullPlan, GitPushOutcome, GitPushPlan,
    GitRefUpdate, GitUpstream,
};
use super::remote;

/// Teto de commits listados num plano. Um push de mil commits não precisa
/// desenhar mil linhas para o usuário entender o que vai acontecer.
const PLAN_COMMIT_LIMIT: usize = 200;

// ============================================================================
// PUSH
// ============================================================================

/// Preflight de push. Não toca a rede e não altera nada.
pub fn plan_push(
    path: &Path,
    remote_name: Option<&str>,
    source_branch: Option<&str>,
    target_branch: Option<&str>,
) -> Result<GitPushPlan, GitOperationError> {
    ensure_repository(path)?;

    let source = match source_branch {
        Some(branch) => validate_branch(branch)?.to_string(),
        None => current_branch(path)?,
    };

    let upstream = remote::read_upstream(path, &source);
    let view = remote::get_view(path)?;

    let remote_name = remote_name
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| upstream.as_ref().map(|up| up.remote.clone()))
        .or(view.default_remote.clone())
        .ok_or_else(|| {
            GitOperationError::new(
                codes::REMOTE_NOT_FOUND,
                "Este repositório não tem nenhum remote configurado.",
            )
            .with_action("Adicione um remote antes de enviar commits.")
        })?;

    let remote_exists = view.remotes.iter().any(|item| item.name == remote_name);

    let target = match target_branch {
        Some(branch) => validate_branch(branch)?.to_string(),
        None => upstream
            .as_ref()
            .filter(|up| up.remote == remote_name)
            .map(|up| up.branch.clone())
            .unwrap_or_else(|| source.clone()),
    };

    let tracking_ref = format!("refs/remotes/{remote_name}/{target}");
    let remote_branch_exists = rev_parse(path, &tracking_ref).is_some();

    let (behind, ahead) = if remote_branch_exists {
        remote::read_ahead_behind(path, &tracking_ref, "HEAD").unwrap_or((0, 0))
    } else {
        // Sem ref remota conhecida, tudo que existe na branch local é novidade.
        (0, count_commits(path, "HEAD").unwrap_or(0))
    };

    let range = if remote_branch_exists {
        format!("{tracking_ref}..HEAD")
    } else {
        "HEAD".to_string()
    };

    let commits = commits::list_range(path, &range, PLAN_COMMIT_LIMIT).unwrap_or_default();

    let diverged = behind > 0 && ahead > 0;
    let will_create_upstream = upstream
        .as_ref()
        .map(|up| up.remote != remote_name || up.branch != target)
        .unwrap_or(true);

    let mut warnings = Vec::new();
    let mut blocked = None;

    if !remote_exists {
        blocked = Some(format!("O remote '{remote_name}' não existe neste repositório."));
    } else if ahead == 0 {
        blocked = Some("Não há commits para enviar.".to_string());
    } else if diverged {
        warnings.push(format!(
            "A branch remota tem {behind} commit(s) que você não possui. O push será recusado até você integrar essas mudanças."
        ));
    }

    if !remote_branch_exists && remote_exists {
        warnings.push(format!(
            "A branch '{target}' ainda não existe em '{remote_name}'; ela será criada."
        ));
    }

    if let Some(upstream) = &upstream {
        if upstream.gone {
            warnings.push(
                "O upstream configurado aponta para uma branch que não existe mais.".to_string(),
            );
        }
    }

    Ok(GitPushPlan {
        remote: remote_name,
        remote_exists,
        source_branch: source,
        target_branch: target,
        upstream,
        will_create_upstream,
        remote_branch_exists,
        ahead,
        behind,
        diverged,
        commits,
        warnings,
        blocked,
    })
}

/// Executa o push. O plano é recalculado aqui: a interface pode ter ficado
/// aberta enquanto o repositório mudou.
pub fn push(
    path: &Path,
    remote_name: Option<&str>,
    source_branch: Option<&str>,
    target_branch: Option<&str>,
    set_upstream: bool,
) -> Result<GitPushOutcome, GitOperationError> {
    let plan = plan_push(path, remote_name, source_branch, target_branch)?;

    if let Some(reason) = plan.blocked {
        return Err(GitOperationError::new(
            if plan.remote_exists {
                codes::NOTHING_TO_PUSH
            } else {
                codes::REMOTE_NOT_FOUND
            },
            reason,
        ));
    }

    let refspec = format!("{}:refs/heads/{}", plan.source_branch, plan.target_branch);
    let mut args: Vec<&str> = vec!["push"];

    if set_upstream {
        args.push("--set-upstream");
    }

    args.push(&plan.remote);
    args.push(&refspec);

    let output = run_structured(path, &args)?;

    if !output.success {
        return Err(classify_network_failure(&output.stderr, "enviar commits"));
    }

    let tracking_ref = format!("refs/remotes/{}/{}", plan.remote, plan.target_branch);
    let new_remote_hash = rev_parse(path, &tracking_ref)
        .or_else(|| rev_parse(path, "HEAD"))
        .unwrap_or_default();

    Ok(GitPushOutcome {
        remote: plan.remote,
        source_branch: plan.source_branch,
        target_branch: plan.target_branch,
        pushed_commits: plan.ahead,
        created_upstream: set_upstream && plan.will_create_upstream,
        created_remote_branch: !plan.remote_branch_exists,
        new_remote_hash,
        details: useful_lines(&output.stderr),
    })
}

// ============================================================================
// FETCH
// ============================================================================

/// Busca refs do remote sem tocar no working tree.
///
/// O relatório do que mudou é construído comparando as refs remotas ANTES e
/// DEPOIS, em vez de interpretar o texto do Git: a comparação é dado real e
/// não depende do formato de saída, que muda entre versões.
pub fn fetch(
    path: &Path,
    remote_name: Option<&str>,
    prune: bool,
) -> Result<GitFetchOutcome, GitOperationError> {
    ensure_repository(path)?;

    let view = remote::get_view(path)?;
    let remote_name = remote_name
        .map(str::to_string)
        .filter(|value| !value.trim().is_empty())
        .or(view.default_remote.clone())
        .ok_or_else(|| {
            GitOperationError::new(
                codes::REMOTE_NOT_FOUND,
                "Este repositório não tem nenhum remote configurado.",
            )
        })?;

    if !view.remotes.iter().any(|item| item.name == remote_name) {
        return Err(GitOperationError::new(
            codes::REMOTE_NOT_FOUND,
            format!("O remote '{remote_name}' não existe neste repositório."),
        ));
    }

    let before = snapshot_remote_refs(path, &remote_name);
    // Commits que já existiam em QUALQUER ref antes do fetch. É essa fronteira
    // que separa "chegou agora" de "já estava aqui por outro caminho".
    let known_before = snapshot_known_commits(path);

    let mut args: Vec<&str> = vec!["fetch"];
    if prune {
        args.push("--prune");
    }
    args.push(&remote_name);

    let output = run_structured(path, &args)?;

    if !output.success {
        return Err(classify_network_failure(&output.stderr, "buscar atualizações"));
    }

    let after = snapshot_remote_refs(path, &remote_name);
    let (updated_refs, new_branches, pruned_branches) =
        diff_snapshots(path, &before, &after, &known_before);

    // O total é a UNIÃO dos commits novos, não a soma por ref: duas branches
    // que avançaram juntas compartilham commits, e somar contaria duas vezes.
    let arriving: Vec<String> = updated_refs
        .iter()
        .filter_map(|item| item.new_hash.clone())
        .collect();
    let received_commits = count_new_commits_union(path, &arriving, &known_before);

    let current = current_branch(path).unwrap_or_default();

    Ok(GitFetchOutcome {
        remote: remote_name,
        had_changes: !updated_refs.is_empty(),
        updated_refs,
        new_branches,
        pruned_branches,
        received_commits,
        upstream: remote::read_upstream(path, &current),
    })
}

fn snapshot_remote_refs(path: &Path, remote_name: &str) -> BTreeMap<String, String> {
    let pattern = format!("refs/remotes/{remote_name}/");
    let raw = git_cli::run(
        path,
        &[
            "for-each-ref",
            "--format=%(refname)%09%(refname:short)%09%(objectname)",
            &pattern,
        ],
    )
    .unwrap_or_default();

    raw.lines()
        .filter_map(|line| {
            let mut parts = line.split('\t');
            let full = parts.next()?;
            let short = parts.next()?;
            let hash = parts.next()?;

            // `refs/remotes/origin/HEAD` é um ponteiro simbólico para a branch
            // padrão, não uma branch — e o Git o encurta para `origin`, sem o
            // sufixo, então o filtro precisa olhar o nome completo.
            if full.ends_with("/HEAD") {
                return None;
            }

            Some((short.to_string(), hash.to_string()))
        })
        .collect()
}

/// Commits alcançáveis por qualquer ref antes da operação.
///
/// Limitado porque vira argumento de linha de comando; em repositórios com
/// centenas de refs a contagem degrada para "tudo que a ref alcança", que
/// superestima em vez de mentir para menos.
fn snapshot_known_commits(path: &Path) -> Vec<String> {
    let raw = git_cli::run(path, &["for-each-ref", "--format=%(objectname)", "refs/"])
        .unwrap_or_default();

    raw.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .take(200)
        .collect()
}

/// Quantos commits de `hash` não existiam em nenhuma ref conhecida.
fn count_new_commits(path: &Path, hash: &str, known: &[String]) -> usize {
    count_new_commits_union(path, std::slice::from_ref(&hash.to_string()), known)
}

/// Commits alcançáveis por qualquer um dos `hashes` e por nenhuma ref anterior.
fn count_new_commits_union(path: &Path, hashes: &[String], known: &[String]) -> usize {
    if hashes.is_empty() {
        return 0;
    }

    let mut args: Vec<&str> = vec!["rev-list", "--count"];
    args.extend(hashes.iter().map(String::as_str));

    if !known.is_empty() {
        args.push("--not");
        args.extend(known.iter().map(String::as_str));
    }

    git_cli::run(path, &args)
        .ok()
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or(0)
}

fn diff_snapshots(
    path: &Path,
    before: &BTreeMap<String, String>,
    after: &BTreeMap<String, String>,
    known_before: &[String],
) -> (Vec<GitRefUpdate>, Vec<String>, Vec<String>) {
    let mut updates = Vec::new();
    let mut new_branches = Vec::new();
    let mut pruned = Vec::new();

    for (name, new_hash) in after {
        match before.get(name) {
            None => {
                new_branches.push(name.clone());
                updates.push(GitRefUpdate {
                    ref_name: name.clone(),
                    old_hash: None,
                    new_hash: Some(new_hash.clone()),
                    kind: "new".to_string(),
                    received_commits: count_new_commits(path, new_hash, known_before),
                });
            }
            Some(old_hash) if old_hash != new_hash => {
                let received = count_new_commits(path, new_hash, known_before);
                // Ref que avançou sem conter o commit anterior foi reescrita no
                // remote: isso é informação de risco, não um update comum.
                let forced = !is_ancestor(path, old_hash, new_hash);

                updates.push(GitRefUpdate {
                    ref_name: name.clone(),
                    old_hash: Some(old_hash.clone()),
                    new_hash: Some(new_hash.clone()),
                    kind: if forced { "forced" } else { "updated" }.to_string(),
                    received_commits: received,
                });
            }
            _ => {}
        }
    }

    for name in before.keys() {
        if !after.contains_key(name) {
            pruned.push(name.clone());
            updates.push(GitRefUpdate {
                ref_name: name.clone(),
                old_hash: before.get(name).cloned(),
                new_hash: None,
                kind: "pruned".to_string(),
                received_commits: 0,
            });
        }
    }

    (updates, new_branches, pruned)
}

// ============================================================================
// PULL
// ============================================================================

pub fn plan_pull(
    path: &Path,
    remote_name: Option<&str>,
    remote_branch: Option<&str>,
) -> Result<GitPullPlan, GitOperationError> {
    ensure_repository(path)?;

    let branch = current_branch(path)?;
    let upstream = remote::read_upstream(path, &branch);
    let view = remote::get_view(path)?;

    let remote_name = remote_name
        .map(str::to_string)
        .filter(|value| !value.trim().is_empty())
        .or_else(|| upstream.as_ref().map(|up| up.remote.clone()))
        .or(view.default_remote.clone())
        .ok_or_else(|| {
            GitOperationError::new(
                codes::REMOTE_NOT_FOUND,
                "Este repositório não tem nenhum remote configurado.",
            )
        })?;

    let target = remote_branch
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| upstream.as_ref().map(|up| up.branch.clone()))
        .unwrap_or_else(|| branch.clone());

    let tracking_ref = format!("refs/remotes/{remote_name}/{target}");
    let tracking_exists = rev_parse(path, &tracking_ref).is_some();

    let (behind, ahead) = if tracking_exists {
        remote::read_ahead_behind(path, &tracking_ref, "HEAD").unwrap_or((0, 0))
    } else {
        (0, 0)
    };

    let incoming = if tracking_exists {
        commits::list_range(path, &format!("HEAD..{tracking_ref}"), PLAN_COMMIT_LIMIT)
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let outgoing = if tracking_exists {
        commits::list_range(path, &format!("{tracking_ref}..HEAD"), PLAN_COMMIT_LIMIT)
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let diverged = ahead > 0 && behind > 0;
    let can_fast_forward = behind > 0 && ahead == 0;

    let local_changes = dirty_files(path);
    let incoming_files = if tracking_exists && behind > 0 {
        changed_files(path, &format!("HEAD..{tracking_ref}"))
    } else {
        BTreeSet::new()
    };

    let conflict_risk: Vec<String> = local_changes
        .iter()
        .filter(|file| incoming_files.contains(*file))
        .cloned()
        .collect();

    let mut warnings = Vec::new();
    let mut blocked = None;

    if !tracking_exists {
        warnings.push(format!(
            "Ainda não há informação local sobre '{remote_name}/{target}'. Faça um fetch primeiro."
        ));
    }

    if let Some(operation) = operation_in_progress(path) {
        blocked = Some(format!(
            "Há uma operação de {operation} em andamento neste repositório."
        ));
    } else if !conflict_risk.is_empty() {
        blocked = Some(
            "Há alterações locais não commitadas nos mesmos arquivos que chegariam do remote."
                .to_string(),
        );
    } else if behind == 0 && tracking_exists {
        blocked = Some("Já está atualizado: não há commits para receber.".to_string());
    }

    if diverged {
        warnings.push(format!(
            "Histórico divergente: {ahead} commit(s) local(is) e {behind} remoto(s). Fast-forward não é possível."
        ));
    }

    let available_strategies = if can_fast_forward {
        vec![
            "fast-forward".to_string(),
            "merge".to_string(),
            "rebase".to_string(),
        ]
    } else if diverged {
        vec!["merge".to_string(), "rebase".to_string()]
    } else {
        Vec::new()
    };

    let recommended_strategy = if can_fast_forward {
        "fast-forward".to_string()
    } else if diverged {
        // Merge preserva os commits locais como estão; rebase os reescreve.
        // Recomendar o que não reescreve histórico é a escolha conservadora.
        "merge".to_string()
    } else {
        String::new()
    };

    Ok(GitPullPlan {
        remote: remote_name,
        branch,
        upstream,
        incoming,
        outgoing,
        available_strategies,
        recommended_strategy,
        can_fast_forward,
        diverged,
        local_changes: local_changes.into_iter().collect(),
        conflict_risk,
        warnings,
        blocked,
    })
}

/// Executa o pull com a estratégia ESCOLHIDA pelo usuário.
///
/// Pull aqui é fetch seguido de integração explícita — nunca o `git pull`
/// genérico, cujo comportamento depende de configuração global invisível
/// (`pull.rebase`) e mudaria de máquina para máquina.
pub fn pull(
    path: &Path,
    remote_name: Option<&str>,
    remote_branch: Option<&str>,
    strategy: &str,
) -> Result<GitPullOutcome, GitOperationError> {
    let strategy = match strategy {
        "fast-forward" | "merge" | "rebase" => strategy,
        _ => {
            return Err(GitOperationError::new(
                codes::STRATEGY_UNAVAILABLE,
                "Estratégia de integração desconhecida.",
            ))
        }
    };

    let fetch_outcome = fetch(path, remote_name, true)?;
    let plan = plan_pull(path, Some(&fetch_outcome.remote), remote_branch)?;

    if let Some(reason) = plan.blocked {
        let code = if reason.starts_with("Já está atualizado") {
            codes::NOTHING_TO_PUSH
        } else if reason.starts_with("Há alterações locais") {
            codes::LOCAL_CHANGES_WOULD_BE_LOST
        } else {
            codes::OPERATION_IN_PROGRESS
        };

        return Err(GitOperationError::new(code, reason)
            .with_files(plan.conflict_risk.clone()));
    }

    if !plan.available_strategies.iter().any(|item| item == strategy) {
        return Err(GitOperationError::new(
            codes::STRATEGY_UNAVAILABLE,
            format!("A estratégia '{strategy}' não é possível no estado atual."),
        )
        .with_details(format!(
            "Disponíveis: {}",
            plan.available_strategies.join(", ")
        )));
    }

    let target_ref = format!(
        "refs/remotes/{}/{}",
        plan.remote,
        remote_branch
            .map(str::to_string)
            .or_else(|| plan.upstream.as_ref().map(|up| up.branch.clone()))
            .unwrap_or_else(|| plan.branch.clone())
    );

    let previous_head = rev_parse(path, "HEAD").unwrap_or_default();

    let args: Vec<&str> = match strategy {
        "fast-forward" => vec!["merge", "--ff-only", &target_ref],
        "merge" => vec!["merge", "--no-edit", &target_ref],
        _ => vec!["rebase", &target_ref],
    };

    let output = run_structured(path, &args)?;

    if !output.success {
        return Err(recover_from_failed_integration(path, strategy, &output));
    }

    let new_head = rev_parse(path, "HEAD").unwrap_or_default();
    let applied = if previous_head == new_head {
        0
    } else {
        count_commits(path, &format!("{previous_head}..{new_head}")).unwrap_or(0)
    };

    let files_changed = if previous_head == new_head {
        Vec::new()
    } else {
        changed_files(path, &format!("{previous_head}..{new_head}"))
            .into_iter()
            .collect()
    };

    Ok(GitPullOutcome {
        remote: plan.remote,
        branch: plan.branch,
        strategy: strategy.to_string(),
        applied_commits: applied,
        files_changed,
        previous_head,
        new_head,
        fetch: fetch_outcome,
    })
}

/// Desfaz uma integração que falhou, para não deixar o repositório num estado
/// que a interface não sabe operar.
fn recover_from_failed_integration(
    path: &Path,
    strategy: &str,
    output: &git_cli::GitCommandOutput,
) -> GitOperationError {
    let conflicted = conflicted_files(path);
    let is_conflict = !conflicted.is_empty()
        || output.stdout.contains("CONFLICT")
        || output.stderr.contains("CONFLICT");

    if !is_conflict {
        return GitOperationError::new(codes::GIT_COMMAND_FAILED, "A integração falhou.")
            .with_details(sanitize(describe(output)));
    }

    let abort_args: Vec<&str> = if strategy == "rebase" {
        vec!["rebase", "--abort"]
    } else {
        vec!["merge", "--abort"]
    };

    match git_cli::run_structured(path, &abort_args) {
        Ok(abort) if abort.success => GitOperationError::new(
            codes::MERGE_CONFLICT,
            "A integração gerou conflito e foi desfeita.",
        )
        .with_files(conflicted)
        .with_action(
            "O DEWRENCH ainda não resolve conflitos. Resolva no terminal ou no seu editor e tente novamente.",
        )
        .with_details(sanitize(describe(output))),

        _ => GitOperationError::critical(
            codes::MERGE_CONFLICT,
            "A integração gerou conflito e não foi possível desfazê-la automaticamente.",
        )
        .with_files(conflicted)
        .with_action("O repositório está em estado de conflito; resolva no terminal.")
        .with_details(sanitize(describe(output))),
    }
}

// ============================================================================
// APOIO
// ============================================================================

fn ensure_repository(path: &Path) -> Result<(), GitOperationError> {
    if path.join(".git").exists() {
        return Ok(());
    }

    Err(GitOperationError::new(
        codes::NOT_REPOSITORY,
        "Este projeto não possui repositório Git.",
    ))
}

/// Branch atual, recusando os dois estados em que push/pull não fazem sentido.
fn current_branch(path: &Path) -> Result<String, GitOperationError> {
    if rev_parse(path, "HEAD").is_none() {
        return Err(GitOperationError::new(
            codes::UNBORN_BRANCH,
            "Este repositório ainda não tem commits.",
        ));
    }

    let branch = git_cli::run(path, &["branch", "--show-current"])
        .map_err(|error| {
            GitOperationError::new(codes::GIT_COMMAND_FAILED, "Não foi possível ler a branch atual.")
                .with_details(sanitize(error))
        })?
        .trim()
        .to_string();

    if branch.is_empty() {
        return Err(GitOperationError::new(
            codes::DETACHED_HEAD,
            "HEAD está destacado: não há branch atual para sincronizar.",
        )
        .with_action("Troque para uma branch antes de enviar ou receber commits."));
    }

    Ok(branch)
}

fn validate_branch(branch: &str) -> Result<&str, GitOperationError> {
    let branch = branch.trim();

    if branch.is_empty() || branch.starts_with('-') {
        return Err(GitOperationError::new(
            codes::INVALID_COMMIT,
            "Nome de branch inválido.",
        ));
    }

    Ok(branch)
}

fn rev_parse(path: &Path, reference: &str) -> Option<String> {
    let value = git_cli::run(path, &["rev-parse", "--verify", "--quiet", reference]).ok()?;
    let value = value.trim().to_string();
    (!value.is_empty()).then_some(value)
}

fn count_commits(path: &Path, range: &str) -> Option<usize> {
    git_cli::run(path, &["rev-list", "--count", range])
        .ok()?
        .trim()
        .parse()
        .ok()
}

fn is_ancestor(path: &Path, ancestor: &str, descendant: &str) -> bool {
    git_cli::run_structured(
        path,
        &["merge-base", "--is-ancestor", ancestor, descendant],
    )
    .map(|output| output.success)
    .unwrap_or(false)
}

/// Arquivos com alteração local (index ou working tree).
fn dirty_files(path: &Path) -> BTreeSet<String> {
    let raw = git_cli::run_raw(path, &["status", "--porcelain=v1", "-z"]).unwrap_or_default();

    raw.split('\0')
        .filter(|entry| entry.len() > 3)
        .map(|entry| entry[3..].to_string())
        .collect()
}

fn conflicted_files(path: &Path) -> Vec<String> {
    let raw = git_cli::run(path, &["diff", "--name-only", "--diff-filter=U"]).unwrap_or_default();

    raw.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

fn changed_files(path: &Path, range: &str) -> BTreeSet<String> {
    let raw = git_cli::run(path, &["diff", "--name-only", range]).unwrap_or_default();

    raw.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

/// Merge, rebase, cherry-pick ou revert pela metade.
fn operation_in_progress(path: &Path) -> Option<String> {
    let git_dir = path.join(".git");
    let markers = [
        ("MERGE_HEAD", "merge"),
        ("REBASE_HEAD", "rebase"),
        ("rebase-merge", "rebase"),
        ("rebase-apply", "rebase"),
        ("CHERRY_PICK_HEAD", "cherry-pick"),
        ("REVERT_HEAD", "revert"),
    ];

    markers
        .iter()
        .find(|(marker, _)| git_dir.join(marker).exists())
        .map(|(_, label)| label.to_string())
}

fn run_structured(
    path: &Path,
    args: &[&str],
) -> Result<git_cli::GitCommandOutput, GitOperationError> {
    git_cli::run_structured(path, args).map_err(|error| {
        GitOperationError::critical(codes::GIT_NOT_FOUND, "Não foi possível executar o Git.")
            .with_details(error.to_string())
    })
}

fn describe(output: &git_cli::GitCommandOutput) -> String {
    let mut text = String::new();

    if !output.stdout.trim().is_empty() {
        text.push_str(output.stdout.trim());
    }

    if !output.stderr.trim().is_empty() {
        if !text.is_empty() {
            text.push('\n');
        }
        text.push_str(output.stderr.trim());
    }

    text
}

/// Linhas informativas do Git, saneadas, para exibir sem virar despejo de log.
fn useful_lines(raw: &str) -> Vec<String> {
    raw.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| !line.starts_with("remote:"))
        .map(|line| sanitize(line.to_string()))
        .filter(|line| !line.is_empty())
        .take(12)
        .collect()
}

/// Traduz a saída de uma falha de rede em erro tipado.
///
/// A classificação é por conteúdo do stderr porque o Git não oferece exit codes
/// distintos para esses casos. O texto original continua acessível em `details`
/// — normalizar não pode significar esconder.
pub fn classify_network_failure(stderr: &str, action: &str) -> GitOperationError {
    let lowered = stderr.to_ascii_lowercase();
    let details = sanitize(stderr.to_string());

    let build = |code: &str, message: String, suggestion: &str| {
        GitOperationError::new(code, message)
            .with_details(details.clone())
            .with_action(suggestion)
    };

    if lowered.contains("could not read username")
        || lowered.contains("authentication failed")
        || lowered.contains("permission denied (publickey)")
        || lowered.contains("invalid username or password")
        || lowered.contains("terminal prompts disabled")
    {
        return build(
            codes::AUTHENTICATION_REQUIRED,
            format!("O remote recusou a autenticação ao {action}."),
            "Configure suas credenciais (credential helper, chave SSH ou 'gh auth login') e tente novamente.",
        );
    }

    if lowered.contains("could not resolve host")
        || lowered.contains("connection refused")
        || lowered.contains("connection timed out")
        || lowered.contains("network is unreachable")
        || lowered.contains("failed to connect")
        || lowered.contains("temporary failure in name resolution")
    {
        return build(
            codes::NETWORK_UNREACHABLE,
            format!("Não foi possível alcançar o remote ao {action}."),
            "Verifique sua conexão e o endereço do remote.",
        );
    }

    if lowered.contains("repository not found")
        || lowered.contains("does not appear to be a git repository")
        || lowered.contains("not found")
            && lowered.contains("remote")
    {
        return build(
            codes::REMOTE_REPOSITORY_NOT_FOUND,
            "O repositório remoto não foi encontrado.".to_string(),
            "Confirme a URL do remote e se você tem acesso a ele.",
        );
    }

    if lowered.contains("non-fast-forward")
        || lowered.contains("fetch first")
        || lowered.contains("behind its remote counterpart")
    {
        return build(
            codes::NON_FAST_FORWARD,
            "O remote recusou: sua branch está atrás da branch remota.".to_string(),
            "Traga as mudanças do remote (fetch e integração) antes de enviar.",
        );
    }

    if lowered.contains("rejected") || lowered.contains("protected branch") {
        return build(
            codes::PUSH_REJECTED,
            "O remote recusou o push.".to_string(),
            "Leia os detalhes técnicos: a regra de recusa vem do servidor.",
        );
    }

    build(
        codes::GIT_COMMAND_FAILED,
        format!("Não foi possível {action}."),
        "Os detalhes técnicos abaixo vêm direto do Git.",
    )
}

/// Upstream exposto para quem só precisa da relação de rastreamento.
pub fn upstream_of(path: &Path, branch: &str) -> Option<GitUpstream> {
    remote::read_upstream(path, branch)
}

/// Commits de um intervalo, com teto — reexportado para o serviço.
pub fn range_commits(path: &Path, range: &str) -> Vec<GitGraphCommit> {
    commits::list_range(path, range, PLAN_COMMIT_LIMIT).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifica_falha_de_autenticacao() {
        let error = classify_network_failure(
            "fatal: Authentication failed for 'https://github.com/owner/repo.git/'",
            "enviar commits",
        );
        assert_eq!(error.code, codes::AUTHENTICATION_REQUIRED);
        assert!(error.details.is_some());
    }

    #[test]
    fn classifica_falha_de_rede() {
        let error = classify_network_failure(
            "fatal: unable to access 'https://github.com/o/r.git/': Could not resolve host: github.com",
            "buscar atualizações",
        );
        assert_eq!(error.code, codes::NETWORK_UNREACHABLE);
    }

    #[test]
    fn classifica_non_fast_forward() {
        let error = classify_network_failure(
            "! [rejected] main -> main (non-fast-forward)\nhint: Updates were rejected",
            "enviar commits",
        );
        assert_eq!(error.code, codes::NON_FAST_FORWARD);
    }

    #[test]
    fn falha_desconhecida_preserva_texto_tecnico() {
        let error = classify_network_failure("fatal: coisa estranha do servidor", "enviar commits");
        assert_eq!(error.code, codes::GIT_COMMAND_FAILED);
        assert!(error.details.unwrap().contains("coisa estranha"));
    }

    #[test]
    fn credencial_nao_vaza_na_classificacao() {
        let error = classify_network_failure(
            "fatal: unable to access 'https://user:supersecret@github.com/o/r.git/': Could not resolve host",
            "buscar",
        );
        let details = error.details.unwrap();
        assert!(!details.contains("supersecret"));
    }

    #[test]
    fn linhas_uteis_removem_ruido_do_servidor() {
        let lines = useful_lines("remote: Enumerating objects\nTo github.com:o/r.git\n * [new branch] x -> x\n");
        assert!(lines.iter().all(|line| !line.starts_with("remote:")));
        assert_eq!(lines.len(), 2);
    }
}
