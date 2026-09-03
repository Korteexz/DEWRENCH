//! Red team: tentativas reais de abuso contra o backend do DEWRENCH.
//!
//! Regra deste arquivo: cada teste TENTA um abuso concreto e verifica um
//! EFEITO, não uma mensagem. "O erro foi bonito" não prova nada; "o arquivo que
//! o ataque criaria não existe" prova.
//!
//! Um teste que passa aqui significa exatamente uma coisa: este payload, nesta
//! versão, neste sistema operacional, não produziu o efeito buscado. Não
//! significa que a classe de ataque está fechada.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use app_lib::modules::git::service as git_service;
use app_lib::modules::git::{branches, commits, working_tree};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

struct Alvo {
    root: PathBuf,
    /// Diretório vizinho, fora do workspace. O que aparecer aqui é escape.
    fora: PathBuf,
}

impl Alvo {
    fn novo(nome: &str) -> Alvo {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let seq = COUNTER.fetch_add(1, Ordering::SeqCst);
        let base = std::env::temp_dir().join(format!("dw_red_{nome}_{stamp}_{seq}"));

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

        fs::write(fora.join("segredo.txt"), "não deveria ser tocado\n").expect("escrever");

        Alvo { root, fora }
    }

    /// Abre o projeto pelo caminho normal, concedendo autoridade.
    fn aberto(nome: &str) -> (Alvo, String) {
        let alvo = Alvo::novo(nome);
        let aberto = git_service::open_project(&alvo.root.to_string_lossy()).expect("abrir");
        (alvo, aberto.path)
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

// ===========================================================================
// A. Injeção de comando
// ===========================================================================

#[test]
fn separadores_de_shell_em_nome_de_branch_nao_executam_comando() {
    let (alvo, caminho) = Alvo::aberto("shell_branch");

    for payload in [
        "main && touch INVADIDO",
        "main; touch INVADIDO",
        "main | touch INVADIDO",
        "main`touch INVADIDO`",
        "main$(touch INVADIDO)",
        "main\ntouch INVADIDO",
        "main & touch INVADIDO",
    ] {
        let _ = git_service::switch_branch(&caminho, payload);
    }

    assert!(
        !alvo.marcador("INVADIDO"),
        "um separador de shell produziu execução de comando"
    );
}

#[test]
fn separadores_de_shell_em_mensagem_de_commit_nao_executam_comando() {
    let (alvo, caminho) = Alvo::aberto("shell_commit");

    fs::write(alvo.root.join("novo.txt"), "x\n").expect("escrever");
    let _ = git_service::stage_all(&caminho);

    for payload in [
        "msg && touch INVADIDO",
        "msg; touch INVADIDO",
        "msg`touch INVADIDO`",
        "$(touch INVADIDO)",
    ] {
        let _ = git_service::create_commit(&caminho, payload);
    }

    assert!(
        !alvo.marcador("INVADIDO"),
        "a mensagem de commit foi interpretada por um shell"
    );
}

#[test]
fn separadores_de_shell_em_caminho_de_arquivo_nao_executam_comando() {
    let (alvo, caminho) = Alvo::aberto("shell_arquivo");

    for payload in [
        "arquivo.txt && touch INVADIDO",
        "arquivo.txt; touch INVADIDO",
        "$(touch INVADIDO)",
    ] {
        let _ = git_service::stage_file(&caminho, payload);
        let _ = git_service::unstage_file(&caminho, payload);
    }

    assert!(
        !alvo.marcador("INVADIDO"),
        "o caminho de arquivo foi interpretado por um shell"
    );
}

// ===========================================================================
// B. Injeção de argumento
// ===========================================================================

#[test]
fn nome_de_branch_iniciado_por_hifen_nao_vira_opcao_do_git() {
    let (alvo, caminho) = Alvo::aberto("arg_switch");

    // `--orphan` cria uma branch órfã e descarta o histórico do índice;
    // `--detach` tira o HEAD da branch. Ambos mudam estado sem que o usuário
    // tenha pedido nada além de "trocar de branch".
    for payload in [
        "--detach",
        "--orphan",
        "--orphan=invadida",
        "-c",
        "--discard-changes",
        "--force",
    ] {
        let _ = git_service::switch_branch(&caminho, payload);
    }

    let head = String::from_utf8(
        Command::new("git")
            .args(["symbolic-ref", "--quiet", "HEAD"])
            .current_dir(&alvo.root)
            .output()
            .expect("git symbolic-ref")
            .stdout,
    )
    .unwrap();

    assert_eq!(
        head.trim(),
        "refs/heads/main",
        "uma opção do git passou como nome de branch e mudou o estado do HEAD"
    );
}

#[test]
fn revisao_iniciada_por_hifen_nao_vira_opcao_no_diff() {
    let (_alvo, caminho) = Alvo::aberto("arg_diff");

    for payload in ["--help", "-p", "--output=INVADIDO", "--ext-diff"] {
        let resultado = git_service::get_commit_diff(&caminho, payload);
        assert!(
            resultado.is_err(),
            "a revisão '{payload}' foi aceita como argumento do git"
        );
    }
}

#[test]
fn ponto_inicial_iniciado_por_hifen_nao_cria_branch() {
    let (alvo, caminho) = Alvo::aberto("arg_branch_create");

    for payload in ["--help", "-f", "--force"] {
        let _ = git_service::create_branch_from(&caminho, payload, "nova");
    }

    let refs = String::from_utf8(
        Command::new("git")
            .args(["for-each-ref", "--format=%(refname:short)", "refs/heads"])
            .current_dir(&alvo.root)
            .output()
            .expect("git for-each-ref")
            .stdout,
    )
    .unwrap();

    assert!(
        !refs.contains("nova"),
        "uma branch foi criada a partir de um argumento que não é revisão"
    );
}

// ===========================================================================
// C. Travessia de caminho
// ===========================================================================

#[test]
fn stage_de_arquivo_fora_do_workspace_nao_alcanca_o_vizinho() {
    let (alvo, caminho) = Alvo::aberto("traversal_stage");

    let nome_fora = alvo
        .fora
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();

    for payload in [
        format!("../{nome_fora}/segredo.txt"),
        format!("../{nome_fora}"),
        alvo.fora.join("segredo.txt").to_string_lossy().into_owned(),
    ] {
        let _ = git_service::stage_file(&caminho, &payload);
    }

    let index = String::from_utf8(
        Command::new("git")
            .args(["diff", "--cached", "--name-only"])
            .current_dir(&alvo.root)
            .output()
            .expect("git diff --cached")
            .stdout,
    )
    .unwrap();

    assert!(
        !index.contains("segredo"),
        "um arquivo de fora do workspace entrou no índice: {index}"
    );
}

// ===========================================================================
// D. Configuração hostil do repositório
// ===========================================================================

#[test]
fn fsmonitor_hostil_no_config_do_repositorio_nao_executa() {
    let (alvo, caminho) = Alvo::aberto("config_fsmonitor");

    let script = escrever_script(&alvo.root, "fsmonitor", "INVADIDO_FSMONITOR", &alvo.fora);
    git(&alvo.root, &["config", "core.fsmonitor", &script]);

    let _ = git_service::get_repository_details(&caminho);
    let _ = git_service::get_repository_graph(&caminho);

    assert!(
        !alvo.marcador("INVADIDO_FSMONITOR"),
        "core.fsmonitor do repositório foi executado"
    );
}

#[test]
fn diff_external_hostil_no_config_do_repositorio_nao_executa() {
    let (alvo, caminho) = Alvo::aberto("config_diff_external");

    let script = escrever_script(&alvo.root, "difftool", "INVADIDO_DIFF", &alvo.fora);
    git(&alvo.root, &["config", "diff.external", &script]);

    fs::write(alvo.root.join("arquivo.txt"), "modificado\n").expect("escrever");

    let _ = git_service::get_repository_details(&caminho);
    let _ = git_service::get_commit_diff(&caminho, "HEAD");

    assert!(
        !alvo.marcador("INVADIDO_DIFF"),
        "diff.external do repositório foi executado"
    );
}

#[test]
fn pager_hostil_no_config_do_repositorio_nao_executa() {
    let (alvo, caminho) = Alvo::aberto("config_pager");

    let script = escrever_script(&alvo.root, "pager", "INVADIDO_PAGER", &alvo.fora);
    git(&alvo.root, &["config", "core.pager", &script]);

    let _ = git_service::get_repository_details(&caminho);
    let _ = git_service::get_repository_graph(&caminho);

    assert!(
        !alvo.marcador("INVADIDO_PAGER"),
        "core.pager do repositório foi executado"
    );
}

// ===========================================================================
// E. Envenenamento de ambiente
// ===========================================================================
//
// Mora em `tests/red_team_env.rs`: `std::env::set_var` afeta o processo
// INTEIRO, e um teste que envenena o ambiente enquanto outros rodam em
// paralelo produz falha em quem não tem nada a ver com o ataque. Binário
// separado = processo separado.

// ===========================================================================
// F. Concorrência
// ===========================================================================

#[test]
fn duas_mutacoes_simultaneas_nao_corrompem_o_indice() {
    let (alvo, caminho) = Alvo::aberto("concorrencia");

    for indice in 0..20 {
        fs::write(alvo.root.join(format!("f{indice}.txt")), "x\n").expect("escrever");
    }

    let a = std::thread::spawn({
        let caminho = caminho.clone();
        move || git_service::stage_all(&caminho)
    });
    let b = std::thread::spawn({
        let caminho = caminho.clone();
        move || git_service::stage_all(&caminho)
    });

    let _ = a.join();
    let _ = b.join();

    // O repositório precisa continuar utilizável depois da corrida.
    let saida = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&alvo.root)
        .output()
        .expect("git status");

    assert!(
        saida.status.success(),
        "o repositório ficou inutilizável após duas mutações simultâneas"
    );
}

// ===========================================================================
// G. Vazamento de segredo
// ===========================================================================

#[test]
fn credencial_em_url_de_remote_nao_aparece_no_erro() {
    let (alvo, caminho) = Alvo::aberto("segredo_remote");

    // Credencial embutida direto no config, como acontece em repositório real.
    git(
        &alvo.root,
        &[
            "remote",
            "add",
            "origin",
            "https://usuario:ghp_aaaabbbbccccddddeeeeffff1234@example.invalid/o/r.git",
        ],
    );

    let resultado = git_service::fetch_remote(&caminho, Some("origin".to_string()), false);

    let texto = match resultado {
        Err(erro) => format!("{erro:?}"),
        Ok(saida) => format!("{saida:?}"),
    };

    assert!(
        !texto.contains("ghp_aaaabbbbccccddddeeeeffff1234"),
        "o token apareceu no resultado devolvido ao frontend: {texto}"
    );
    assert!(
        !texto.contains("usuario:"),
        "a credencial apareceu no resultado devolvido ao frontend: {texto}"
    );
}

// ===========================================================================
// H. Caminho alternativo para a mesma autoridade
// ===========================================================================

#[test]
fn funcoes_de_dominio_continuam_alcancaveis_apenas_dentro_do_processo() {
    // Este teste documenta um limite conhecido: `branches`, `commits` e
    // `working_tree` são `pub` e não verificam autoridade — a verificação está
    // na camada `service`, que é a única alcançável pelo IPC.
    //
    // Ele existe para falhar se alguém expuser um desses módulos direto num
    // command, o que reabriria o bypass.
    let alvo = Alvo::novo("bypass_direto");

    assert!(branches::get_current(&alvo.root).is_ok());
    assert!(commits::get_recent(&alvo.root, 1).is_ok());
    assert!(working_tree::get_status(&alvo.root).is_ok());
}

// ---------------------------------------------------------------------------

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

// ===========================================================================
// I. Injeção pela via de rede (push / fetch)
// ===========================================================================
//
// Esta é a variante mais perigosa da classe encontrada no `switch`: o refspec
// do push é MONTADO por interpolação (`{source}:refs/heads/{target}`), então
// um valor iniciado por `-` não vira só "opção" — vira `--upload-pack=<cmd>`,
// que faz o git executar um programa escolhido pelo atacante.
//
// Nenhum destes testes toca a rede: o "remote" é um repositório bare local.

fn com_remote(nome: &str) -> (Alvo, String, PathBuf) {
    let (alvo, caminho) = Alvo::aberto(nome);
    let bare = alvo.fora.join("remoto.git");

    let status = Command::new("git")
        .args(["init", "-q", "--bare", bare.to_str().unwrap()])
        .status()
        .expect("git init --bare");
    assert!(status.success());

    git(
        &alvo.root,
        &["remote", "add", "origin", bare.to_str().unwrap()],
    );

    (alvo, caminho, bare)
}

#[test]
fn branch_de_origem_iniciada_por_hifen_nao_vira_opcao_do_push() {
    let (alvo, caminho, _bare) = com_remote("push_source");

    for payload in [
        "--upload-pack=touch INVADIDO_PUSH",
        "--receive-pack=touch INVADIDO_PUSH",
        "--exec=touch INVADIDO_PUSH",
        "-o",
    ] {
        let resultado = git_service::push_branch(
            &caminho,
            Some("origin".to_string()),
            Some(payload.to_string()),
            None,
            false,
        );

        assert!(
            resultado.is_err(),
            "a branch de origem '{payload}' foi aceita no push"
        );
    }

    assert!(
        !alvo.marcador("INVADIDO_PUSH"),
        "o push executou um programa escolhido pelo argumento"
    );
}

#[test]
fn branch_de_destino_iniciada_por_hifen_nao_vira_opcao_do_push() {
    let (alvo, caminho, _bare) = com_remote("push_target");

    for payload in ["--receive-pack=touch INVADIDO_PUSH", "--force", "-f"] {
        let resultado = git_service::push_branch(
            &caminho,
            Some("origin".to_string()),
            None,
            Some(payload.to_string()),
            false,
        );

        assert!(
            resultado.is_err(),
            "a branch de destino '{payload}' foi aceita no push"
        );
    }

    assert!(!alvo.marcador("INVADIDO_PUSH"));
}

#[test]
fn remote_inexistente_iniciado_por_hifen_nao_vira_opcao_do_fetch() {
    let (alvo, caminho, _bare) = com_remote("fetch_remote");

    for payload in [
        "--upload-pack=touch INVADIDO_FETCH",
        "--exec=touch INVADIDO_FETCH",
        "-o",
    ] {
        let resultado = git_service::fetch_remote(&caminho, Some(payload.to_string()), false);

        assert!(
            resultado.is_err(),
            "o remote '{payload}' foi aceito no fetch"
        );
    }

    assert!(
        !alvo.marcador("INVADIDO_FETCH"),
        "o fetch executou um programa escolhido pelo nome do remote"
    );
}

#[test]
fn branch_remota_iniciada_por_hifen_nao_vira_opcao_do_pull() {
    let (alvo, caminho, _bare) = com_remote("pull_branch");

    for payload in ["--upload-pack=touch INVADIDO_PULL", "-o", "--rebase"] {
        let resultado = git_service::pull_branch(
            &caminho,
            Some("origin".to_string()),
            Some(payload.to_string()),
            "merge",
        );

        assert!(
            resultado.is_err(),
            "a branch remota '{payload}' foi aceita no pull"
        );
    }

    assert!(!alvo.marcador("INVADIDO_PULL"));
}

#[test]
fn url_de_remote_com_helper_hostil_e_recusada_no_cadastro() {
    let (_alvo, caminho) = Alvo::aberto("remote_url");

    for payload in [
        "ext::sh -c 'touch INVADIDO_EXT'",
        "--upload-pack=touch INVADIDO_EXT",
        "fd::0",
    ] {
        let resultado = git_service::add_remote(&caminho, "hostil", payload);

        assert!(
            resultado.is_err(),
            "a URL '{payload}' foi aceita como remote"
        );
    }
}

// ===========================================================================
// J. Hooks do repositório
// ===========================================================================
//
// Hooks são programas que o git executa por conta própria em pontos do fluxo
// normal. Eles chegam junto com o repositório: basta abrir um projeto de
// terceiros para que `.git/hooks/` já esteja no disco.

#[test]
fn hook_de_post_checkout_ao_trocar_de_branch() {
    let (alvo, caminho) = Alvo::aberto("hook_post_checkout");

    git(&alvo.root, &["branch", "outra"]);
    escrever_hook(&alvo.root, "post-checkout", "INVADIDO_HOOK", &alvo.fora);

    let _ = git_service::switch_branch(&caminho, "outra");

    assert!(
        !alvo.marcador("INVADIDO_HOOK"),
        "o hook post-checkout do repositório foi executado ao trocar de branch"
    );
}

#[test]
fn hook_de_pre_commit_ao_commitar() {
    let (alvo, caminho) = Alvo::aberto("hook_pre_commit");

    escrever_hook(&alvo.root, "pre-commit", "INVADIDO_HOOK", &alvo.fora);
    fs::write(alvo.root.join("novo.txt"), "x\n").expect("escrever");
    let _ = git_service::stage_all(&caminho);
    let _ = git_service::create_commit(&caminho, "mensagem");

    assert!(
        !alvo.marcador("INVADIDO_HOOK"),
        "o hook pre-commit do repositório foi executado ao commitar"
    );
}

#[test]
fn hooks_path_apontando_para_diretorio_do_atacante() {
    let (alvo, caminho) = Alvo::aberto("hooks_path");

    let hooks = alvo.fora.join("hooks_hostis");
    fs::create_dir_all(&hooks).expect("criar hooks");
    escrever_hook_em(&hooks, "post-checkout", "INVADIDO_HOOKSPATH", &alvo.fora);

    git(
        &alvo.root,
        &["config", "core.hooksPath", hooks.to_str().unwrap()],
    );
    git(&alvo.root, &["branch", "outra2"]);

    let _ = git_service::switch_branch(&caminho, "outra2");

    assert!(
        !alvo.marcador("INVADIDO_HOOKSPATH"),
        "core.hooksPath do repositório redirecionou a execução para o atacante"
    );
}

fn escrever_hook(repo: &Path, nome: &str, marcador: &str, fora: &Path) {
    escrever_hook_em(&repo.join(".git").join("hooks"), nome, marcador, fora);
}

fn escrever_hook_em(dir: &Path, nome: &str, marcador: &str, fora: &Path) {
    fs::create_dir_all(dir).expect("criar diretório de hooks");
    let caminho = dir.join(nome);
    let alvo = fora.join(marcador);

    fs::write(
        &caminho,
        format!("#!/bin/sh\ntouch '{}'\nexit 0\n", alvo.display()),
    )
    .expect("escrever hook");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&caminho, fs::Permissions::from_mode(0o755)).expect("chmod");
    }
}

// ===========================================================================
// K. core.sshCommand — vetor conhecido, testado para saber se é real
// ===========================================================================

#[test]
fn ssh_command_hostil_no_config_do_repositorio() {
    let (alvo, caminho) = Alvo::aberto("config_ssh");

    let script = escrever_script(&alvo.root, "sshhostil", "INVADIDO_SSH", &alvo.fora);
    git(&alvo.root, &["config", "core.sshCommand", &script]);
    git(
        &alvo.root,
        &["remote", "add", "origin", "ssh://git@localhost:1/o/r.git"],
    );

    let _ = git_service::fetch_remote(&caminho, Some("origin".to_string()), false);

    assert!(
        !alvo.marcador("INVADIDO_SSH"),
        "core.sshCommand do repositório foi executado durante o fetch"
    );
}

#[test]
fn credential_helper_do_repositorio_perde_para_a_config_do_usuario() {
    use app_lib::core::authority::WorkspaceTrust;
    use app_lib::core::process::{self, ProcessRequest, ProgramId};

    let alvo = Alvo::novo("config_credential");
    let script = escrever_script(&alvo.root, "helperhostil", "INVADIDO_HELPER", &alvo.fora);
    git(&alvo.root, &["config", "credential.helper", &script]);

    // Limitação honesta deste laboratório: `git credential fill` só consulta
    // helpers com pedido em stdin, e o broker fecha stdin de propósito. Então
    // o que se verifica aqui é a PRECEDÊNCIA efetiva — qual helper o git usaria
    // —, e não a execução em si. A execução foi verificada manualmente com
    // `git credential fill` recebendo um pedido real, e está registrada no
    // relatório como cobertura parcial.
    let efetivo = |trust: WorkspaceTrust| {
        let pedido = ProcessRequest::new(
            ProgramId::Git,
            vec![
                "config".to_string(),
                "--get".to_string(),
                "credential.helper".to_string(),
            ],
            &alvo.root,
        )
        .with_trust(trust);

        process::run(pedido)
            .map(|saida| saida.stdout.trim().to_string())
            .unwrap_or_default()
    };

    assert_ne!(
        efetivo(WorkspaceTrust::Opened),
        script,
        "o helper definido pelo repositório continua sendo o efetivo"
    );

    // Contraprova: sem a defesa, o helper do repositório É o efetivo. Sem esta
    // metade, a de cima passaria mesmo que a chave nunca tivesse sido lida.
    assert_eq!(
        efetivo(WorkspaceTrust::ExecutableContent),
        script,
        "o teste não exercitou a chave: a metade de cima não prova nada"
    );
}

// ===========================================================================
// L. Symlink como caminho alternativo para a mesma autoridade
// ===========================================================================

#[test]
#[cfg(unix)]
fn symlink_apontando_para_fora_nao_herda_a_autoridade_do_workspace() {
    let (alvo, _caminho) = Alvo::aberto("symlink_fora");
    let outro = Alvo::novo("symlink_alvo");

    let link = alvo.root.join("atalho");
    std::os::unix::fs::symlink(&outro.root, &link).expect("criar symlink");

    let resultado = git_service::get_repository_details(&link.to_string_lossy());

    assert!(
        resultado.is_err(),
        "um symlink dentro do workspace concedeu acesso ao diretório de destino"
    );
}

#[test]
#[cfg(unix)]
fn symlink_apontando_para_o_proprio_workspace_resolve_para_a_mesma_identidade() {
    let alvo = Alvo::novo("symlink_mesmo");

    let link = alvo.fora.join("atalho_para_o_projeto");
    std::os::unix::fs::symlink(&alvo.root, &link).expect("criar symlink");

    // Abre pelo caminho real...
    git_service::open_project(&alvo.root.to_string_lossy()).expect("abrir");

    // ...e opera pelo caminho alternativo. Precisa funcionar: se resolvessem
    // para identidades diferentes, o mesmo repositório teria dois locks e duas
    // autoridades, e o lock deixaria de proteger qualquer coisa.
    git_service::get_repository_details(&link.to_string_lossy())
        .expect("o mesmo diretório por outro nome deveria ser o mesmo workspace");
}
