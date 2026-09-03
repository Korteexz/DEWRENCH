//! Laboratórios Git: cenários reais de remote, push, fetch e pull.
//!
//! Cada teste monta um repositório descartável em `std::env::temp_dir()` e usa
//! um repositório bare local como "remote". Isso exercita o caminho real do
//! `git` — incluindo refs remotas, upstream e divergência — sem depender de
//! rede e sem jamais tocar no repositório do DEWRENCH.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use app_lib::modules::git::{remote, sync};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

struct Lab {
    root: PathBuf,
}

impl Lab {
    fn new(name: &str) -> Lab {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let unique = COUNTER.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!("dewrench-lab-{name}-{nanos}-{unique}"));
        std::fs::create_dir_all(&root).unwrap();
        Lab { root }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }

    fn git(&self, dir: &Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .args(args)
            .current_dir(dir)
            .output()
            .unwrap_or_else(|error| panic!("git {args:?} não executou: {error}"));

        assert!(
            output.status.success(),
            "git {args:?} falhou:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );

        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    fn init_repo(&self, name: &str) -> PathBuf {
        let path = self.path(name);
        std::fs::create_dir_all(&path).unwrap();
        self.git(&path, &["init", "-b", "main"]);
        self.git(&path, &["config", "user.name", "Lab"]);
        self.git(&path, &["config", "user.email", "lab@dewrench.test"]);
        path
    }

    fn init_bare(&self, name: &str) -> PathBuf {
        let path = self.path(name);
        std::fs::create_dir_all(&path).unwrap();
        self.git(&path, &["init", "--bare", "-b", "main"]);
        path
    }

    fn clone(&self, source: &Path, name: &str) -> PathBuf {
        let path = self.path(name);
        self.git(
            &self.root,
            &["clone", source.to_str().unwrap(), path.to_str().unwrap()],
        );
        self.git(&path, &["config", "user.name", "Lab"]);
        self.git(&path, &["config", "user.email", "lab@dewrench.test"]);
        path
    }

    fn commit(&self, repo: &Path, file: &str, content: &str, message: &str) {
        std::fs::write(repo.join(file), content).unwrap();
        self.git(repo, &["add", file]);
        self.git(repo, &["commit", "-m", message]);
    }
}

impl Drop for Lab {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

// ============================================================================
// REMOTES
// ============================================================================

#[test]
fn remotes_sao_listados_com_urls_de_fetch_e_push() {
    let lab = Lab::new("remotes");
    let repo = lab.init_repo("repo");
    let bare = lab.init_bare("origin.git");

    remote::add(&repo, "origin", bare.to_str().unwrap()).unwrap();

    let remotes = remote::list(&repo).unwrap();
    assert_eq!(remotes.len(), 1);
    assert_eq!(remotes[0].name, "origin");
    assert!(remotes[0].is_origin);
    assert_eq!(remotes[0].fetch_url, remotes[0].push_url);
}

#[test]
fn remote_pode_ser_renomeado_e_ter_url_trocada() {
    let lab = Lab::new("rename");
    let repo = lab.init_repo("repo");
    let bare = lab.init_bare("origin.git");
    let outro = lab.init_bare("fork.git");

    remote::add(&repo, "origin", bare.to_str().unwrap()).unwrap();
    remote::rename(&repo, "origin", "upstream").unwrap();

    let remotes = remote::list(&repo).unwrap();
    assert_eq!(remotes[0].name, "upstream");
    assert!(!remotes[0].is_origin);

    remote::set_url(&repo, "upstream", outro.to_str().unwrap(), false).unwrap();
    let remotes = remote::list(&repo).unwrap();
    assert!(remotes[0].fetch_url.contains("fork.git"));

    remote::remove(&repo, "upstream").unwrap();
    assert!(remote::list(&repo).unwrap().is_empty());
}

#[test]
fn remote_duplicado_e_inexistente_sao_recusados() {
    let lab = Lab::new("guardas");
    let repo = lab.init_repo("repo");
    let bare = lab.init_bare("origin.git");

    remote::add(&repo, "origin", bare.to_str().unwrap()).unwrap();

    let duplicado = remote::add(&repo, "origin", bare.to_str().unwrap()).unwrap_err();
    assert_eq!(duplicado.code, "REMOTE_ALREADY_EXISTS");

    let inexistente = remote::remove(&repo, "naoexiste").unwrap_err();
    assert_eq!(inexistente.code, "REMOTE_NOT_FOUND");
}

#[test]
fn remote_padrao_prefere_o_upstream_da_branch_atual() {
    let lab = Lab::new("padrao");
    let bare_origin = lab.init_bare("origin.git");
    let bare_fork = lab.init_bare("fork.git");
    let repo = lab.init_repo("repo");

    lab.commit(&repo, "a.txt", "1", "primeiro");
    remote::add(&repo, "origin", bare_origin.to_str().unwrap()).unwrap();
    remote::add(&repo, "fork", bare_fork.to_str().unwrap()).unwrap();

    // Sem upstream, a convenção vale: origin.
    let view = remote::get_view(&repo).unwrap();
    assert_eq!(view.default_remote.as_deref(), Some("origin"));
    assert_eq!(view.remotes.len(), 2);

    // Com upstream em 'fork', a intenção do usuário passa na frente.
    sync::push(&repo, Some("fork"), None, None, true).unwrap();
    let view = remote::get_view(&repo).unwrap();
    assert_eq!(view.default_remote.as_deref(), Some("fork"));
    assert!(view.upstream.is_some());
    assert_eq!(view.upstream.unwrap().remote, "fork");
}

// ============================================================================
// PUSH
// ============================================================================

#[test]
fn push_cria_a_branch_remota_e_o_upstream() {
    let lab = Lab::new("push-novo");
    let bare = lab.init_bare("origin.git");
    let repo = lab.init_repo("repo");

    lab.commit(&repo, "a.txt", "1", "primeiro");
    lab.commit(&repo, "b.txt", "2", "segundo");
    remote::add(&repo, "origin", bare.to_str().unwrap()).unwrap();

    let plan = sync::plan_push(&repo, None, None, None).unwrap();
    assert_eq!(plan.remote, "origin");
    assert_eq!(plan.source_branch, "main");
    assert_eq!(plan.target_branch, "main");
    assert!(!plan.remote_branch_exists);
    assert!(plan.will_create_upstream);
    assert_eq!(plan.ahead, 2);
    assert_eq!(plan.commits.len(), 2);
    assert!(plan.blocked.is_none());

    let outcome = sync::push(&repo, None, None, None, true).unwrap();
    assert!(outcome.created_remote_branch);
    assert!(outcome.created_upstream);
    assert_eq!(outcome.pushed_commits, 2);
    assert!(!outcome.new_remote_hash.is_empty());

    let upstream = remote::read_upstream(&repo, "main").unwrap();
    assert_eq!(upstream.ref_name, "origin/main");
    assert_eq!(upstream.ahead, 0);
    assert_eq!(upstream.behind, 0);
}

#[test]
fn push_sem_commits_novos_e_bloqueado() {
    let lab = Lab::new("push-vazio");
    let bare = lab.init_bare("origin.git");
    let repo = lab.init_repo("repo");

    lab.commit(&repo, "a.txt", "1", "primeiro");
    remote::add(&repo, "origin", bare.to_str().unwrap()).unwrap();
    sync::push(&repo, None, None, None, true).unwrap();

    let plan = sync::plan_push(&repo, None, None, None).unwrap();
    assert_eq!(plan.ahead, 0);
    assert!(plan.blocked.is_some());

    let error = sync::push(&repo, None, None, None, false).unwrap_err();
    assert_eq!(error.code, "NOTHING_TO_PUSH");
}

#[test]
fn push_para_branch_remota_de_outro_nome() {
    let lab = Lab::new("push-outro-nome");
    let bare = lab.init_bare("origin.git");
    let repo = lab.init_repo("repo");

    lab.commit(&repo, "a.txt", "1", "primeiro");
    remote::add(&repo, "origin", bare.to_str().unwrap()).unwrap();

    let plan = sync::plan_push(&repo, Some("origin"), None, Some("publicado")).unwrap();
    assert_eq!(plan.target_branch, "publicado");

    sync::push(&repo, Some("origin"), None, Some("publicado"), false).unwrap();

    let refs = lab.git(&repo, &["ls-remote", "--heads", "origin"]);
    assert!(refs.contains("refs/heads/publicado"));
}

#[test]
fn push_para_remote_inexistente_e_recusado() {
    let lab = Lab::new("push-sem-remote");
    let repo = lab.init_repo("repo");
    lab.commit(&repo, "a.txt", "1", "primeiro");

    let error = sync::plan_push(&repo, None, None, None).unwrap_err();
    assert_eq!(error.code, "REMOTE_NOT_FOUND");
}

#[test]
fn push_em_repositorio_sem_commits_e_recusado() {
    let lab = Lab::new("push-unborn");
    let bare = lab.init_bare("origin.git");
    let repo = lab.init_repo("repo");
    remote::add(&repo, "origin", bare.to_str().unwrap()).unwrap();

    let error = sync::plan_push(&repo, None, None, None).unwrap_err();
    assert_eq!(error.code, "UNBORN_BRANCH");
}

// ============================================================================
// FETCH
// ============================================================================

#[test]
fn fetch_relata_commits_recebidos_e_branches_novas() {
    let lab = Lab::new("fetch");
    let bare = lab.init_bare("origin.git");
    let semeador = lab.init_repo("semeador");

    lab.commit(&semeador, "a.txt", "1", "primeiro");
    remote::add(&semeador, "origin", bare.to_str().unwrap()).unwrap();
    sync::push(&semeador, None, None, None, true).unwrap();

    let clone = lab.clone(&bare, "clone");

    // O outro colaborador publica mais dois commits e uma branch nova.
    lab.commit(&semeador, "b.txt", "2", "segundo");
    lab.commit(&semeador, "c.txt", "3", "terceiro");
    sync::push(&semeador, None, None, None, false).unwrap();
    lab.git(&semeador, &["checkout", "-b", "experimento"]);
    lab.commit(&semeador, "d.txt", "4", "quarto");
    sync::push(&semeador, Some("origin"), Some("experimento"), None, false).unwrap();

    let outcome = sync::fetch(&clone, None, true).unwrap();
    assert!(outcome.had_changes);
    // União: 'segundo' e 'terceiro' chegam por main, 'quarto' por experimento.
    assert_eq!(outcome.received_commits, 3);
    assert!(outcome
        .new_branches
        .iter()
        .any(|name| name == "origin/experimento"));
    assert!(outcome
        .updated_refs
        .iter()
        .any(|item| item.ref_name == "origin/main" && item.kind == "updated"));

    // O fetch não pode ter mexido no working tree.
    let status = lab.git(&clone, &["status", "--porcelain"]);
    assert!(status.is_empty());

    let upstream = outcome.upstream.unwrap();
    assert_eq!(upstream.behind, 2);
    assert_eq!(upstream.ahead, 0);
}

#[test]
fn fetch_relata_branch_removida_no_remote() {
    let lab = Lab::new("fetch-prune");
    let bare = lab.init_bare("origin.git");
    let semeador = lab.init_repo("semeador");

    lab.commit(&semeador, "a.txt", "1", "primeiro");
    remote::add(&semeador, "origin", bare.to_str().unwrap()).unwrap();
    sync::push(&semeador, None, None, None, true).unwrap();
    lab.git(&semeador, &["checkout", "-b", "temporaria"]);
    lab.commit(&semeador, "b.txt", "2", "segundo");
    sync::push(&semeador, Some("origin"), Some("temporaria"), None, false).unwrap();

    let clone = lab.clone(&bare, "clone");
    lab.git(&semeador, &["push", "origin", "--delete", "temporaria"]);

    let outcome = sync::fetch(&clone, None, true).unwrap();
    assert!(outcome
        .pruned_branches
        .iter()
        .any(|name| name == "origin/temporaria"));
}

// ============================================================================
// PULL
// ============================================================================

#[test]
fn pull_fast_forward_aplica_os_commits_recebidos() {
    let lab = Lab::new("pull-ff");
    let bare = lab.init_bare("origin.git");
    let semeador = lab.init_repo("semeador");

    lab.commit(&semeador, "a.txt", "1", "primeiro");
    remote::add(&semeador, "origin", bare.to_str().unwrap()).unwrap();
    sync::push(&semeador, None, None, None, true).unwrap();

    let clone = lab.clone(&bare, "clone");

    lab.commit(&semeador, "b.txt", "2", "segundo");
    sync::push(&semeador, None, None, None, false).unwrap();

    sync::fetch(&clone, None, true).unwrap();
    let plan = sync::plan_pull(&clone, None, None).unwrap();
    assert!(plan.can_fast_forward);
    assert!(!plan.diverged);
    assert_eq!(plan.incoming.len(), 1);
    assert!(plan.outgoing.is_empty());
    assert_eq!(plan.recommended_strategy, "fast-forward");
    assert!(plan.blocked.is_none());

    let outcome = sync::pull(&clone, None, None, "fast-forward").unwrap();
    assert_eq!(outcome.applied_commits, 1);
    assert!(outcome.files_changed.iter().any(|file| file == "b.txt"));
    assert!(clone.join("b.txt").exists());
}

#[test]
fn historico_divergente_recusa_fast_forward_e_bloqueia_push() {
    let lab = Lab::new("divergente");
    let bare = lab.init_bare("origin.git");
    let semeador = lab.init_repo("semeador");

    lab.commit(&semeador, "a.txt", "1", "primeiro");
    remote::add(&semeador, "origin", bare.to_str().unwrap()).unwrap();
    sync::push(&semeador, None, None, None, true).unwrap();

    let clone = lab.clone(&bare, "clone");

    // Os dois lados avançam de forma independente.
    lab.commit(&semeador, "remoto.txt", "r", "commit remoto");
    sync::push(&semeador, None, None, None, false).unwrap();
    lab.commit(&clone, "local.txt", "l", "commit local");

    sync::fetch(&clone, None, true).unwrap();

    let pull_plan = sync::plan_pull(&clone, None, None).unwrap();
    assert!(pull_plan.diverged);
    assert!(!pull_plan.can_fast_forward);
    assert_eq!(pull_plan.recommended_strategy, "merge");
    assert!(!pull_plan
        .available_strategies
        .iter()
        .any(|item| item == "fast-forward"));

    let push_plan = sync::plan_push(&clone, None, None, None).unwrap();
    assert!(push_plan.diverged);
    assert_eq!(push_plan.ahead, 1);
    assert_eq!(push_plan.behind, 1);
    assert!(!push_plan.warnings.is_empty());

    // O push tem de falhar de verdade, com erro classificado.
    let error = sync::push(&clone, None, None, None, false).unwrap_err();
    assert_eq!(error.code, "NON_FAST_FORWARD");
    assert!(error.details.is_some());

    // Estratégia indisponível não pode ser executada.
    let recusa = sync::pull(&clone, None, None, "fast-forward").unwrap_err();
    assert_eq!(recusa.code, "STRATEGY_UNAVAILABLE");

    // Merge resolve e mantém os dois commits.
    let outcome = sync::pull(&clone, None, None, "merge").unwrap();
    assert_eq!(outcome.strategy, "merge");
    assert!(outcome.applied_commits >= 1);
    assert!(clone.join("remoto.txt").exists());
    assert!(clone.join("local.txt").exists());
}

#[test]
fn pull_com_alteracao_local_sobreposta_e_bloqueado() {
    let lab = Lab::new("pull-sujo");
    let bare = lab.init_bare("origin.git");
    let semeador = lab.init_repo("semeador");

    lab.commit(&semeador, "compartilhado.txt", "original", "primeiro");
    remote::add(&semeador, "origin", bare.to_str().unwrap()).unwrap();
    sync::push(&semeador, None, None, None, true).unwrap();

    let clone = lab.clone(&bare, "clone");

    lab.commit(&semeador, "compartilhado.txt", "versao remota", "muda remoto");
    sync::push(&semeador, None, None, None, false).unwrap();

    std::fs::write(clone.join("compartilhado.txt"), "versao local nao commitada").unwrap();
    sync::fetch(&clone, None, true).unwrap();

    let plan = sync::plan_pull(&clone, None, None).unwrap();
    assert!(!plan.conflict_risk.is_empty());
    assert!(plan.blocked.is_some());

    let error = sync::pull(&clone, None, None, "fast-forward").unwrap_err();
    assert_eq!(error.code, "LOCAL_CHANGES_WOULD_BE_LOST");
    assert!(error.affected_files.iter().any(|f| f == "compartilhado.txt"));

    // O arquivo local não pode ter sido tocado.
    let content = std::fs::read_to_string(clone.join("compartilhado.txt")).unwrap();
    assert_eq!(content, "versao local nao commitada");
}

#[test]
fn pull_desfaz_a_integracao_quando_ha_conflito() {
    let lab = Lab::new("pull-conflito");
    let bare = lab.init_bare("origin.git");
    let semeador = lab.init_repo("semeador");

    lab.commit(&semeador, "arquivo.txt", "base\n", "primeiro");
    remote::add(&semeador, "origin", bare.to_str().unwrap()).unwrap();
    sync::push(&semeador, None, None, None, true).unwrap();

    let clone = lab.clone(&bare, "clone");

    lab.commit(&semeador, "arquivo.txt", "linha remota\n", "muda remoto");
    sync::push(&semeador, None, None, None, false).unwrap();
    lab.commit(&clone, "arquivo.txt", "linha local\n", "muda local");

    sync::fetch(&clone, None, true).unwrap();

    let error = sync::pull(&clone, None, None, "merge").unwrap_err();
    assert_eq!(error.code, "MERGE_CONFLICT");
    assert!(error.affected_files.iter().any(|f| f == "arquivo.txt"));
    assert!(error.recoverable);

    // O repositório não pode ficar parado no meio de um merge.
    assert!(!clone.join(".git").join("MERGE_HEAD").exists());
    let content = std::fs::read_to_string(clone.join("arquivo.txt")).unwrap();
    assert_eq!(content, "linha local\n");
}
