//! Testes de fronteira de segurança que só fazem sentido cruzando módulos.
//!
//! Aqui não se testa "o Core funciona" — isso é responsabilidade dos testes de
//! unidade dentro de `core::*`. O que se testa é a ligação: se um caminho que o
//! usuário nunca abriu chega ao Git assim mesmo, o Core existe mas não está
//! ENFORCED, e a diferença entre essas duas coisas é a única que importa.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use app_lib::modules::activity::service as activity_service;
use app_lib::modules::git::service as git_service;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn temp_root(nome: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let seq = COUNTER.fetch_add(1, Ordering::SeqCst);
    let root = std::env::temp_dir().join(format!("dw_sec_{nome}_{stamp}_{seq}"));
    fs::create_dir_all(&root).expect("criar diretório de teste");
    root
}

fn repositorio(nome: &str) -> PathBuf {
    let root = temp_root(nome);
    let status = Command::new("git")
        .args(["init", "-q", "-b", "main"])
        .current_dir(&root)
        .status()
        .expect("git init");
    assert!(status.success());

    fs::write(root.join("arquivo.txt"), "conteúdo\n").expect("escrever arquivo");

    let status = Command::new("git")
        .args(["add", "."])
        .current_dir(&root)
        .status()
        .expect("git add");
    assert!(status.success());

    let status = Command::new("git")
        .args([
            "-c",
            "user.name=Lab",
            "-c",
            "user.email=lab@example.invalid",
            "commit",
            "-q",
            "-m",
            "inicial",
        ])
        .current_dir(&root)
        .status()
        .expect("git commit");
    assert!(status.success());

    root
}

// ---------------------------------------------------------------------------
// Autoridade
// ---------------------------------------------------------------------------

#[test]
fn caminho_nunca_aberto_e_recusado_em_leitura() {
    let repo = repositorio("nao_aberto");

    let resultado = git_service::get_repository_details(&repo.to_string_lossy());

    let erro = match resultado {
        Err(erro) => erro,
        Ok(_) => panic!("um repositório nunca aberto não deveria ser legível"),
    };

    assert!(
        erro.contains("Nenhum projeto aberto"),
        "a recusa não veio da fronteira de autoridade: {erro}"
    );
}

#[test]
fn caminho_nunca_aberto_e_recusado_em_mutacao() {
    let repo = repositorio("nao_aberto_mutacao");

    let resultado = git_service::stage_all(&repo.to_string_lossy());

    assert!(
        resultado.is_err(),
        "mutação foi aceita sobre um diretório sem autoridade concedida"
    );
}

#[test]
fn abrir_o_projeto_concede_autoridade_e_a_leitura_passa_a_funcionar() {
    let repo = repositorio("abre_e_le");
    let caminho = repo.to_string_lossy().into_owned();

    assert!(
        git_service::get_repository_details(&caminho).is_err(),
        "leitura funcionou ANTES de abrir — a autoridade não está sendo exigida"
    );

    let aberto = git_service::open_project(&caminho).expect("abrir projeto");

    git_service::get_repository_details(&aberto.path)
        .expect("depois de aberto, a leitura deveria funcionar");
}

#[test]
fn abrir_um_projeto_nao_concede_autoridade_sobre_outro() {
    let a = repositorio("autoridade_a");
    let b = repositorio("autoridade_b");

    git_service::open_project(&a.to_string_lossy()).expect("abrir A");

    let resultado = git_service::get_repository_details(&b.to_string_lossy());

    assert!(
        resultado.is_err(),
        "abrir A concedeu acesso a B — a autoridade não é por workspace"
    );
}

#[test]
fn atravessar_para_fora_com_dot_dot_nao_alcanca_outro_repositorio() {
    let a = repositorio("traversal_a");
    let b = repositorio("traversal_b");

    git_service::open_project(&a.to_string_lossy()).expect("abrir A");

    // Caminho textualmente ancorado em A, resolvendo para B.
    let escapada = a.join("..").join(b.file_name().unwrap());

    let resultado = git_service::get_repository_details(&escapada.to_string_lossy());

    assert!(
        resultado.is_err(),
        "`..` a partir de um workspace autorizado alcançou outro diretório"
    );
}

#[test]
fn atividade_tambem_exige_workspace_registrado() {
    let repo = repositorio("atividade");

    let resultado = activity_service::collect(&repo.to_string_lossy(), Some(10));

    let erro = resultado.expect_err("a Temporal Matrix leu um repositório não autorizado");
    assert_eq!(erro.code, "WORKSPACE_NOT_REGISTERED");
}

// ---------------------------------------------------------------------------
// Guarda arquitetural
// ---------------------------------------------------------------------------

/// Nenhum módulo cria processo por conta própria.
///
/// Este teste é a única defesa contra a regressão mais provável desta
/// arquitetura: alguém (inclusive eu, numa sessão futura) acrescenta um
/// `Command::new("git")` num módulo novo, e a fronteira do broker deixa de ser
/// total sem que nenhum outro teste perceba.
#[test]
fn nenhum_modulo_cria_processo_diretamente() {
    let raiz = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join("modules");

    let mut infratores = Vec::new();
    visitar(&raiz, &mut |arquivo: &PathBuf, conteudo: &str| {
        for (numero, linha) in conteudo.lines().enumerate() {
            if linha.contains("Command::new") || linha.contains("std::process::Command") {
                infratores.push(format!("{}:{}", arquivo.display(), numero + 1));
            }
        }
    });

    assert!(
        infratores.is_empty(),
        "módulos devem descrever intenção, não criar processo. Ocorrências: {infratores:?}"
    );
}

fn visitar(dir: &PathBuf, acao: &mut impl FnMut(&PathBuf, &str)) {
    let Ok(entradas) = fs::read_dir(dir) else {
        return;
    };

    for entrada in entradas.flatten() {
        let caminho = entrada.path();

        if caminho.is_dir() {
            visitar(&caminho, acao);
            continue;
        }

        if caminho.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }

        if let Ok(conteudo) = fs::read_to_string(&caminho) {
            acao(&caminho, &conteudo);
        }
    }
}
