//! Red team — envenenamento de ambiente.
//!
//! Binário separado por necessidade, não por organização: `std::env::set_var`
//! altera o processo INTEIRO. Rodar estes testes junto com os outros fazia o
//! ataque atingir repositórios que não faziam parte do ataque, e o resultado
//! era falha em testes inocentes — ruído que esconderia uma falha real.
//!
//! Mesmo aqui, os dois testes são serializados por mutex: um envenena `GIT_DIR`
//! e o outro `GIT_EXTERNAL_DIFF`, e em paralelo eles se contaminariam.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use app_lib::modules::git::service as git_service;

static COUNTER: AtomicUsize = AtomicUsize::new(0);
static AMBIENTE: Mutex<()> = Mutex::new(());

struct Alvo {
    root: PathBuf,
    fora: PathBuf,
}

impl Alvo {
    fn novo(nome: &str) -> Alvo {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let seq = COUNTER.fetch_add(1, Ordering::SeqCst);
        let base = std::env::temp_dir().join(format!("dw_redenv_{nome}_{stamp}_{seq}"));

        let root = base.join("projeto");
        let fora = base.join("fora");
        fs::create_dir_all(&root).expect("criar projeto");
        fs::create_dir_all(&fora).expect("criar vizinho");

        git(&root, &["init", "-q", "-b", "main"]);
        fs::write(root.join("arquivo.txt"), "conteúdo\n").expect("escrever");
        git(&root, &["add", "."]);
        git(
            &root,
            &[
                "-c",
                "user.name=Lab",
                "-c",
                "user.email=lab@example.invalid",
                "commit",
                "-q",
                "-m",
                "inicial",
            ],
        );

        Alvo { root, fora }
    }

    fn marcador(&self, nome: &str) -> bool {
        self.root.join(nome).exists() || self.fora.join(nome).exists()
    }
}

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(dir)
        .status()
        .expect("executar git");
    assert!(status.success(), "git {args:?} falhou");
}

/// Escreve um script que cria um marcador quando executado.
fn escrever_script(dir: &Path, nome: &str, marcador: &str, fora: &Path) -> String {
    let caminho = dir.join(format!("{nome}.sh"));
    let alvo = fora.join(marcador);

    fs::write(
        &caminho,
        format!("#!/bin/sh\ntouch '{}'\nexit 0\n", alvo.display()),
    )
    .expect("escrever script");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&caminho, fs::Permissions::from_mode(0o755)).expect("chmod");
    }

    caminho.to_string_lossy().into_owned()
}


#[test]
fn git_external_diff_herdado_do_ambiente_nao_executa() {
    let _trava = AMBIENTE.lock().unwrap_or_else(|e| e.into_inner());
    let alvo = Alvo::novo("env_external_diff");
    let script = escrever_script(&alvo.root, "envdiff", "INVADIDO_ENV", &alvo.fora);

    // Simula um processo pai contaminado: o DEWRENCH herdaria isto.
    std::env::set_var("GIT_EXTERNAL_DIFF", &script);

    let aberto = git_service::open_project(&alvo.root.to_string_lossy()).expect("abrir");
    fs::write(alvo.root.join("arquivo.txt"), "modificado\n").expect("escrever");

    let _ = git_service::get_repository_details(&aberto.path);
    let _ = git_service::get_commit_diff(&aberto.path, "HEAD");

    std::env::remove_var("GIT_EXTERNAL_DIFF");

    assert!(
        !alvo.marcador("INVADIDO_ENV"),
        "GIT_EXTERNAL_DIFF herdado do ambiente foi executado"
    );
}

#[test]
fn git_dir_herdado_do_ambiente_nao_redireciona_a_leitura() {
    let _trava = AMBIENTE.lock().unwrap_or_else(|e| e.into_inner());
    let alvo_a = Alvo::novo("env_gitdir_a");
    let alvo_b = Alvo::novo("env_gitdir_b");

    // Marca B com uma branch reconhecível.
    git(&alvo_b.root, &["branch", "BRANCH_DE_B"]);

    std::env::set_var("GIT_DIR", alvo_b.root.join(".git"));

    let aberto = git_service::open_project(&alvo_a.root.to_string_lossy()).expect("abrir A");
    let detalhes = git_service::get_repository_details(&aberto.path);

    std::env::remove_var("GIT_DIR");

    if let Ok(detalhes) = detalhes {
        assert_ne!(
            detalhes.branch, "BRANCH_DE_B",
            "GIT_DIR do ambiente redirecionou a leitura para outro repositório"
        );
    }
}


/// `PAGER`/`GH_PAGER` herdados do ambiente apontam para um PROGRAMA.
///
/// O prelúdio do git já força `core.pager=cat`, mas a variável continuava sendo
/// herdada por qualquer processo iniciado pelo broker — inclusive a `gh`, que
/// não tem prelúdio. Este teste exercita o caminho real de leitura com o
/// ambiente contaminado e afirma que nada foi executado.
#[test]
fn pager_herdado_do_ambiente_nao_executa() {
    let _trava = AMBIENTE.lock().unwrap_or_else(|e| e.into_inner());
    let alvo = Alvo::novo("env_pager");
    let script = escrever_script(&alvo.root, "envpager", "INVADIDO_PAGER", &alvo.fora);

    std::env::set_var("PAGER", &script);
    std::env::set_var("GH_PAGER", &script);

    let aberto = git_service::open_project(&alvo.root.to_string_lossy()).expect("abrir");
    let _ = git_service::get_repository_details(&aberto.path);
    let _ = git_service::get_commit_diff(&aberto.path, "HEAD");

    std::env::remove_var("PAGER");
    std::env::remove_var("GH_PAGER");

    assert!(
        !alvo.marcador("INVADIDO_PAGER"),
        "PAGER herdado do ambiente foi executado"
    );
}
