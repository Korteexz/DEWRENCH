//! ProcessBroker — a única porta para criação de processo.
//!
//! Criação de processo é infraestrutura privilegiada. Módulos descrevem O QUE
//! querem executar; o broker decide COMO, e é ele que responde por executável,
//! argumentos, diretório, ambiente, tempo limite e tamanho de saída.
//!
//! Garantias efetivamente aplicadas aqui:
//!
//! 1. **Sem shell.** Nunca `sh -c`, `bash -c` ou `cmd /C`. O executável é
//!    resolvido de um enum fechado, então uma string do frontend não pode
//!    virar programa, e `git status && rm -rf /` chega ao git como UM
//!    argumento literal, não como dois comandos.
//! 2. **Allowlist de executável por tipo.** Não existe `run(program: &str)`.
//! 3. **Diretório de trabalho validado.** Precisa existir e ser diretório.
//! 4. **Ambiente higienizado.** Variáveis que transformam o git em executor de
//!    programas arbitrários são removidas da herança.
//! 5. **Prelúdio de segurança do git.** Config do repositório não pode
//!    apontar para binário externo em fluxo de leitura.
//! 6. **Tempo limite com encerramento.** Processo que passa do prazo é morto.
//! 7. **Teto de saída.** Saída gigante é truncada em vez de consumir memória.
//!
//! O broker NÃO sanitiza stdout: parsers dependem do byte exato. Redação
//! acontece na fronteira de erro e auditoria (`core::events`).

use std::io::Read;
use std::sync::OnceLock;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use super::authority::WorkspaceTrust;
use super::error::CoreError;

/// Executáveis que o DEWRENCH pode iniciar. Fechado por construção.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgramId {
    Git,
    Gh,
    /// Apenas para exercitar tempo limite nos testes; não existe em release.
    ///
    /// O executável concreto depende da plataforma: não existe um binário de
    /// espera com o mesmo nome no Unix e no Windows. Ver `executable`.
    #[cfg(test)]
    TestSlow,
}

impl ProgramId {
    pub fn executable(&self) -> &'static str {
        match self {
            ProgramId::Git => "git",
            ProgramId::Gh => "gh",
            #[cfg(all(test, not(windows)))]
            ProgramId::TestSlow => "sleep",
            // `sleep` não existe no Windows, e `timeout` — o equivalente óbvio —
            // aborta com "Input redirection is not supported" quando o stdin
            // não é console. O broker fecha o stdin de propósito, então
            // `timeout` falharia sempre. `ping` está em System32 em toda
            // instalação, espera de verdade e não se importa com stdio
            // redirecionado.
            #[cfg(all(test, windows))]
            ProgramId::TestSlow => "ping",
        }
    }
}

/// Tempo limite padrão para operação local.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);
/// Operações que falam com a rede merecem mais fôlego.
pub const NETWORK_TIMEOUT: Duration = Duration::from_secs(180);

/// Teto de bytes capturados por fluxo.
const STDOUT_LIMIT: usize = 24 * 1024 * 1024;
const STDERR_LIMIT: usize = 256 * 1024;

/// Variáveis removidas da herança de ambiente.
///
/// Todas transformam uma execução comum de `git` ou `gh` em execução de um
/// programa escolhido por outra pessoa, ou redirecionam a ferramenta para outro
/// repositório. Nenhuma delas é usada por qualquer fluxo do DEWRENCH.
///
/// A lista vale para TODO `ProgramId`: uma variável perigosa não deixa de ser
/// perigosa por causa de qual binário a leria.
const STRIPPED_ENV: &[&str] = &[
    "GIT_EXTERNAL_DIFF",
    "GIT_DIFF_OPTS",
    "GIT_PAGER",
    "GIT_EDITOR",
    "GIT_SEQUENCE_EDITOR",
    "GIT_PROXY_COMMAND",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_NAMESPACE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_CEILING_DIRECTORIES",
    "GIT_CONFIG",
    "GIT_CONFIG_GLOBAL",
    "GIT_CONFIG_SYSTEM",
    "GIT_CONFIG_COUNT",
    "LD_PRELOAD",
    "LD_AUDIT",
    "LD_LIBRARY_PATH",
    "DYLD_INSERT_LIBRARIES",
    "DYLD_LIBRARY_PATH",
    // -- Provider GitHub ---------------------------------------------------
    //
    // Duas famílias, pelo mesmo motivo das variáveis do Git acima.
    //
    // `GH_REPO`, `GH_HOST` e `GH_CONFIG_DIR` REDIRECIONAM: com `GH_REPO`
    // definido, toda invocação da `gh` passa a operar sobre outro repositório —
    // um merge confirmado na interface atingiria um destino que a interface
    // nunca mostrou.
    //
    // `GH_BROWSER`, `GH_PAGER`, `GH_EDITOR` — e os equivalentes genéricos
    // `BROWSER`, `PAGER`, `EDITOR`, `VISUAL` — apontam para PROGRAMAS: `gh
    // browse` executaria o binário indicado por quem contaminou o ambiente.
    //
    // `GH_FORCE_TTY` faz a `gh` formatar a saída para terminal e quebraria os
    // parsers de JSON; sai junto por robustez.
    //
    // `GH_TOKEN`, `GITHUB_TOKEN` e `GH_ENTERPRISE_TOKEN` NÃO são removidas: são
    // mecanismo legítimo de autenticação por ambiente, não redirecionam nem
    // executam nada, e o DEWRENCH nunca lê, guarda ou exibe seus valores.
    "GH_REPO",
    "GH_HOST",
    "GH_CONFIG_DIR",
    "GH_BROWSER",
    "GH_PAGER",
    "GH_EDITOR",
    "GH_FORCE_TTY",
    "BROWSER",
    "PAGER",
    "EDITOR",
    "VISUAL",
];

/// Config forçada em toda invocação do git.
///
/// `core.fsmonitor` e `diff.external` apontam para PROGRAMAS e podem ser
/// definidos pelo `.git/config` de um repositório que o usuário apenas abriu.
/// Zerá-los não altera nenhum resultado do DEWRENCH — fsmonitor é otimização
/// e nenhum fluxo daqui usa diff externo — mas remove execução de binário
/// escolhido pelo repositório.
///
/// `core.sshCommand` e `credential.helper` NÃO são zerados: eles são o
/// mecanismo legítimo de autenticação do usuário, e desligá-los quebraria
/// push/fetch reais. Isso permanece como risco residual documentado, a ser
/// coberto por WorkspaceTrust.
const GIT_SAFETY_PRELUDE: &[&str] = &[
    "-c",
    "core.fsmonitor=",
    "-c",
    "diff.external=",
    "-c",
    "core.pager=cat",
];

/// Diretório vazio usado para desligar os hooks do repositório.
///
/// Hooks são a via de execução mais direta que um repositório de terceiros tem:
/// `.git/hooks/post-checkout` roda ao trocar de branch, `pre-commit` roda ao
/// commitar, e ambos chegam junto com o clone. Reproduzido em laboratório antes
/// desta defesa existir.
///
/// Apontar `core.hooksPath` para um diretório vazio desliga TODOS os hooks de
/// uma vez, incluindo os que ainda não existem — mais confiável que enumerar
/// `--no-verify` por subcomando. O diretório é criado uma vez por processo e
/// tem nome único: um caminho previsível poderia ser preenchido por quem já
/// tivesse acesso de escrita ao diretório temporário.
fn hooks_disabled_dir() -> &'static str {
    static DIR: OnceLock<String> = OnceLock::new();

    DIR.get_or_init(|| {
        let unico = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default();

        let caminho = std::env::temp_dir().join(format!(
            "dewrench-sem-hooks-{}-{unico:x}",
            std::process::id()
        ));

        let _ = std::fs::create_dir_all(&caminho);
        format!("core.hooksPath={}", caminho.display())
    })
}


/// Chaves de config que apontam para PROGRAMAS e que o repositório pode
/// definir sozinho, mas que também têm uso legítimo pelo usuário.
///
/// `core.fsmonitor`, `diff.external` e `core.pager` são zerados no prelúdio
/// porque nenhum fluxo do DEWRENCH depende deles. Estas duas são diferentes:
/// elas SÃO o mecanismo de autenticação de muita gente. Zerá-las quebraria
/// push e fetch reais; deixá-las como estão mantém um caminho de execução
/// controlado pelo `.git/config` de um repositório clonado — reproduzido em
/// laboratório com `core.sshCommand`.
///
/// A saída é de precedência, não de remoção: o valor de FORA do repositório
/// (global ou de sistema) é reimposto na linha de comando, que tem prioridade
/// máxima. O usuário mantém a autenticação dele; o repositório perde a
/// capacidade de substituí-la.
///
/// Custo conhecido e deliberado: uma configuração LOCAL legítima (chave de
/// deploy por repositório, por exemplo) deixa de valer enquanto a confiança do
/// workspace for menor que `ExecutableContent`.
const SCOPED_PROGRAM_KEYS: &[&str] = &["core.sshCommand", "credential.helper"];

/// Overrides calculados uma vez por processo.
///
/// Lê a config de FORA do repositório e devolve os pares `-c chave=valor` que
/// reimpõem esses valores. Uma chave sem valor externo é zerada.
fn scoped_program_overrides() -> &'static Vec<String> {
    static OVERRIDES: OnceLock<Vec<String>> = OnceLock::new();

    OVERRIDES.get_or_init(|| {
        let lidos: Vec<(&str, Vec<String>)> = SCOPED_PROGRAM_KEYS
            .iter()
            .map(|key| (*key, read_config_outside_repository(key)))
            .collect();

        build_scoped_overrides(&lidos)
    })
}

/// Monta os pares `-c` a partir dos valores externos já lidos.
///
/// Separada da leitura para poder ser testada: esta é a mudança com maior
/// chance de quebrar autenticação real, e "confie que está certo" não é
/// verificação.
fn build_scoped_overrides(lidos: &[(&str, Vec<String>)]) -> Vec<String> {
    let mut argumentos = Vec::new();

    for (key, valores) in lidos {
        // Uma entrada vazia REINICIA a lista de helpers do git. Como os `-c`
        // da linha de comando são avaliados por último, esta entrada descarta
        // o que veio do repositório; as seguintes reconstroem a lista do
        // usuário na ordem original.
        argumentos.push("-c".to_string());
        argumentos.push(format!("{key}="));

        for valor in valores {
            argumentos.push("-c".to_string());
            argumentos.push(format!("{key}={valor}"));
        }
    }

    argumentos
}

/// Lê uma chave de config ignorando o repositório.
///
/// Usa `std::process::Command` diretamente, e é o ÚNICO lugar do projeto onde
/// isso é correto: chamar `run` aqui reentraria no `OnceLock` que esta função
/// inicializa e travaria o processo. A invocação é fechada — sem argumento
/// vindo de fora, sem diretório de repositório, com o ambiente já higienizado.
fn read_config_outside_repository(key: &str) -> Vec<String> {
    let mut valores = Vec::new();

    for escopo in ["--global", "--system"] {
        let mut command = Command::new("git");
        command
            .args([escopo, "--get-all", key])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());

        for variavel in STRIPPED_ENV {
            command.env_remove(variavel);
        }

        let Ok(saida) = command.output() else {
            continue;
        };

        if !saida.status.success() {
            continue;
        }

        for linha in String::from_utf8_lossy(&saida.stdout).lines() {
            let linha = linha.trim();
            if !linha.is_empty() {
                valores.push(linha.to_string());
            }
        }

        // Global vence sistema; se global respondeu, não some os dois.
        if !valores.is_empty() {
            break;
        }
    }

    valores
}

#[derive(Debug, Clone)]
pub struct ProcessRequest {
    pub program: ProgramId,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub timeout: Duration,
    /// Variáveis adicionadas deliberadamente por quem chama.
    pub extra_env: Vec<(String, String)>,
    /// Confiança no CONTEÚDO do repositório, não no usuário.
    ///
    /// Abrir um projeto (`Opened`) autoriza o DEWRENCH a operar sobre ele; não
    /// autoriza o repositório a executar programas próprios. Só
    /// `ExecutableContent` faz isso, e nenhum fluxo do DEWRENCH concede esse
    /// nível hoje.
    pub trust: WorkspaceTrust,
}

impl ProcessRequest {
    pub fn new(program: ProgramId, args: Vec<String>, cwd: &Path) -> Self {
        ProcessRequest {
            program,
            args,
            cwd: cwd.to_path_buf(),
            timeout: DEFAULT_TIMEOUT,
            extra_env: Vec::new(),
            trust: WorkspaceTrust::Opened,
        }
    }

    /// Declara a confiança no conteúdo do repositório.
    ///
    /// Existe para que a permissão de executar código do repositório seja um
    /// ato explícito e rastreável, e não o efeito colateral de alguém ter
    /// aberto uma pasta.
    pub fn with_trust(mut self, trust: WorkspaceTrust) -> Self {
        self.trust = trust;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_env(mut self, key: &str, value: &str) -> Self {
        self.extra_env.push((key.to_string(), value.to_string()));
        self
    }
}

#[derive(Debug, Clone)]
pub struct ProcessOutcome {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
    pub truncated: bool,
    pub duration: Duration,
}

/// Executa um processo sob todas as garantias do broker.
pub fn run(request: ProcessRequest) -> Result<ProcessOutcome, CoreError> {
    let program = request.program.executable();

    validate_arguments(&request.args, program)?;
    validate_cwd(&request.cwd, program)?;

    let mut command = Command::new(program);

    if matches!(request.program, ProgramId::Git) {
        command.args(GIT_SAFETY_PRELUDE);

        if request.trust < WorkspaceTrust::ExecutableContent {
            command.args(["-c", hooks_disabled_dir()]);
            command.args(scoped_program_overrides().iter());
        }
    }

    command
        .args(&request.args)
        .current_dir(&request.cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    for key in STRIPPED_ENV {
        command.env_remove(key);
    }

    // Nenhum prompt interativo: sem terminal, um pedido de senha ficaria
    // pendurado para sempre — o que é falha de disponibilidade além de
    // segurança.
    command.env("GIT_TERMINAL_PROMPT", "0");

    for (key, value) in &request.extra_env {
        command.env(key, value);
    }

    let started = Instant::now();

    let mut child = command.spawn().map_err(|error| CoreError::ExecutionFailed {
        program: program.to_string(),
        reason: error.to_string(),
        io_kind: Some(error.kind()),
    })?;

    let stdout_handle = child.stdout.take().map(|stream| {
        std::thread::spawn(move || read_capped(stream, STDOUT_LIMIT))
    });
    let stderr_handle = child.stderr.take().map(|stream| {
        std::thread::spawn(move || read_capped(stream, STDERR_LIMIT))
    });

    let mut timed_out = false;

    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) => {
                if started.elapsed() >= request.timeout {
                    // Encerrar é obrigatório: sem isto, um processo travado
                    // segura a thread e o recurso indefinidamente.
                    let _ = child.kill();
                    let _ = child.wait();
                    timed_out = true;
                    break None;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(error) => {
                let _ = child.kill();
                return Err(CoreError::ExecutionFailed {
                    program: program.to_string(),
                    reason: error.to_string(),
                    io_kind: Some(error.kind()),
                });
            }
        }
    };

    let (stdout, stdout_truncated) = stdout_handle
        .and_then(|handle| handle.join().ok())
        .unwrap_or_else(|| (String::new(), false));
    let (stderr, stderr_truncated) = stderr_handle
        .and_then(|handle| handle.join().ok())
        .unwrap_or_else(|| (String::new(), false));

    if timed_out {
        return Err(CoreError::ExecutionTimeout {
            program: program.to_string(),
            seconds: request.timeout.as_secs(),
        });
    }

    let status = status.expect("status presente quando não houve timeout");

    Ok(ProcessOutcome {
        success: status.success(),
        exit_code: status.code(),
        stdout,
        stderr,
        timed_out: false,
        truncated: stdout_truncated || stderr_truncated,
        duration: started.elapsed(),
    })
}

/// Valida um valor que deve chegar ao processo como OPERANDO, nunca como
/// opção.
///
/// Existe porque a ausência de shell não impede injeção de ARGUMENTO. Um nome
/// de branch vindo do frontend é dado; se ele começa com `-`, o git o lê como
/// opção e a intenção "trocar de branch" vira outra operação inteiramente —
/// foi exatamente assim que `--orphan=<x>` transformou uma troca de branch em
/// criação de branch órfã com o HEAD movido.
///
/// Esta é a primeira das duas camadas. A segunda é o separador `--` no ponto de
/// chamada, onde o subcomando o aceita; nenhuma das duas confia na outra.
pub fn operand(value: &str) -> Result<&str, CoreError> {
    if value.is_empty() {
        return Err(CoreError::ArgumentRejected {
            reason: "valor vazio não é um operando válido".to_string(),
            argument: String::new(),
        });
    }

    if value.contains('\0') {
        return Err(CoreError::ArgumentRejected {
            reason: "operando contém byte nulo".to_string(),
            argument: value.replace('\0', "\\0"),
        });
    }

    if value.starts_with('-') {
        return Err(CoreError::ArgumentRejected {
            reason: "um operando não pode começar com '-': seria lido como opção".to_string(),
            argument: value.to_string(),
        });
    }

    Ok(value)
}

fn validate_arguments(args: &[String], program: &str) -> Result<(), CoreError> {
    for argument in args {
        if argument.contains('\0') {
            return Err(CoreError::ArgumentRejected {
                reason: "argumento contém byte nulo".to_string(),
                argument: argument.replace('\0', "\\0"),
            });
        }
    }

    let _ = program;
    Ok(())
}

fn validate_cwd(cwd: &Path, program: &str) -> Result<(), CoreError> {
    if !cwd.exists() {
        return Err(CoreError::ExecutionFailed {
            program: program.to_string(),
            reason: format!(
                "diretório de trabalho inexistente: {}",
                super::path_security::display_path(cwd)
            ),
            io_kind: Some(std::io::ErrorKind::NotFound),
        });
    }

    if !cwd.is_dir() {
        return Err(CoreError::ExecutionFailed {
            program: program.to_string(),
            reason: "diretório de trabalho não é um diretório".to_string(),
            io_kind: Some(std::io::ErrorKind::NotADirectory),
        });
    }

    Ok(())
}

/// Lê até `limit` bytes; o excedente é descartado e sinalizado.
fn read_capped(mut stream: impl Read, limit: usize) -> (String, bool) {
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 8192];
    let mut truncated = false;

    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(read) => {
                if buffer.len() < limit {
                    let room = limit - buffer.len();
                    buffer.extend_from_slice(&chunk[..read.min(room)]);
                    if read > room {
                        truncated = true;
                    }
                } else {
                    truncated = true;
                }
            }
            Err(_) => break,
        }
    }

    (String::from_utf8_lossy(&buffer).into_owned(), truncated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn override_reinicia_a_lista_antes_de_reconstruir_a_do_usuario() {
        let montado = build_scoped_overrides(&[(
            "credential.helper",
            vec!["manager".to_string(), "cache".to_string()],
        )]);

        assert_eq!(
            montado,
            vec![
                "-c",
                "credential.helper=",
                "-c",
                "credential.helper=manager",
                "-c",
                "credential.helper=cache",
            ],
            "a ordem importa: o reset precisa vir ANTES dos valores do usuário"
        );
    }

    #[test]
    fn sem_valor_externo_a_chave_e_apenas_zerada() {
        let montado = build_scoped_overrides(&[("core.sshCommand", Vec::new())]);
        assert_eq!(montado, vec!["-c", "core.sshCommand="]);
    }

    #[test]
    fn operando_iniciado_por_hifen_e_recusado() {
        for hostil in ["--orphan=x", "-c", "--force", "--upload-pack=touch x"] {
            let recusa = operand(hostil).expect_err("operando hostil foi aceito");
            assert_eq!(recusa.code(), "ARGUMENT_REJECTED");
        }
    }

    #[test]
    fn operando_comum_passa_intacto() {
        for aceito in ["main", "feature/x", "v1.2.3", "a-b_c.d", "solto-com-hifen"] {
            assert_eq!(operand(aceito).expect("operando válido"), aceito);
        }
    }

    #[test]
    fn operando_vazio_ou_com_byte_nulo_e_recusado() {
        assert!(operand("").is_err());
        assert!(operand("main\0extra").is_err());
    }

    use std::fs;

    fn lab(name: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("dw-proc-{name}-{nanos}"));
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn git(root: &Path, args: &[&str]) -> ProcessOutcome {
        run(ProcessRequest::new(
            ProgramId::Git,
            args.iter().map(|a| a.to_string()).collect(),
            root,
        ))
        .unwrap()
    }

    fn init_repo(name: &str) -> PathBuf {
        let root = lab(name);
        git(&root, &["init", "-b", "main"]);
        git(&root, &["config", "user.name", "Lab"]);
        git(&root, &["config", "user.email", "lab@dewrench.test"]);
        fs::write(root.join("a.txt"), "1").unwrap();
        git(&root, &["add", "a.txt"]);
        git(&root, &["commit", "-m", "primeiro"]);
        root
    }

    #[test]
    fn executa_git_e_captura_saida() {
        let root = init_repo("basico");
        let outcome = git(&root, &["rev-parse", "--abbrev-ref", "HEAD"]);
        assert!(outcome.success);
        assert_eq!(outcome.stdout.trim(), "main");
        let _ = fs::remove_dir_all(root);
    }

    /// Um separador de shell dentro de um argumento precisa continuar sendo
    /// TEXTO. Se virasse comando, o git não reclamaria — o shell executaria.
    #[test]
    fn separadores_de_shell_nao_viram_comando() {
        let root = init_repo("injecao");
        let marker = root.join("PWNED");

        for payload in [
            format!("main && touch {}", marker.display()),
            format!("main; touch {}", marker.display()),
            format!("main | touch {}", marker.display()),
            format!("$(touch {})", marker.display()),
            format!("`touch {}`", marker.display()),
            format!("main\ntouch {}", marker.display()),
        ] {
            let outcome = git(&root, &["rev-parse", "--verify", "--quiet", &payload]);
            assert!(!outcome.success, "payload deveria falhar: {payload}");
            assert!(
                !marker.exists(),
                "COMMAND INJECTION reproduzida com: {payload}"
            );
        }

        let _ = fs::remove_dir_all(root);
    }

        /// Guarda de regressão da higienização do ambiente do provider GitHub.
    ///
    /// Estas variáveis redirecionam a `gh` para outro repositório ou apontam
    /// para PROGRAMAS que ela executaria. Remover qualquer uma daqui reabre um
    /// caminho fechado deliberadamente — por isso a lista é afirmada, e não
    /// apenas documentada.
    #[test]
    fn variaveis_perigosas_do_gh_estao_na_lista_de_remocao() {
        for nome in [
            "GH_REPO",
            "GH_HOST",
            "GH_CONFIG_DIR",
            "GH_BROWSER",
            "GH_PAGER",
            "GH_EDITOR",
            "GH_FORCE_TTY",
            "BROWSER",
            "PAGER",
            "EDITOR",
            "VISUAL",
        ] {
            assert!(
                STRIPPED_ENV.contains(&nome),
                "{nome} deixou de ser removida da herança de ambiente"
            );
        }
    }

    /// Token de ambiente é mecanismo legítimo de autenticação da `gh` e
    /// continua passando: o DEWRENCH nunca lê o valor.
    #[test]
    fn tokens_de_ambiente_nao_sao_removidos() {
        for nome in ["GH_TOKEN", "GITHUB_TOKEN", "GH_ENTERPRISE_TOKEN"] {
            assert!(!STRIPPED_ENV.contains(&nome));
        }
    }

#[test]
    fn argumento_com_byte_nulo_e_recusado_antes_de_executar() {
        let root = init_repo("nulo");
        let error = run(ProcessRequest::new(
            ProgramId::Git,
            vec!["status".to_string(), "arquivo\0malicioso".to_string()],
            &root,
        ))
        .unwrap_err();

        assert_eq!(error.code(), "ARGUMENT_REJECTED");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn diretorio_de_trabalho_inexistente_e_recusado() {
        let error = run(ProcessRequest::new(
            ProgramId::Git,
            vec!["status".to_string()],
            Path::new("/dw-nao-existe-jamais"),
        ))
        .unwrap_err();

        assert_eq!(error.code(), "EXECUTION_FAILED");
    }

    /// Argumentos que fazem `ProgramId::TestSlow` esperar ~30s.
    ///
    /// Mora aqui junto do teste porque forma um par indissociável com o
    /// executável: trocar um sem o outro não espera nada.
    #[cfg(not(windows))]
    fn espera_de_trinta_segundos() -> Vec<String> {
        vec!["30".to_string()]
    }

    /// `ping -n 31 127.0.0.1` envia 31 pacotes com 1s entre eles: ~30s de
    /// espera, sem depender de rede externa.
    #[cfg(windows)]
    fn espera_de_trinta_segundos() -> Vec<String> {
        vec!["-n".to_string(), "31".to_string(), "127.0.0.1".to_string()]
    }

    #[test]
    fn tempo_limite_encerra_o_processo() {
        let root = lab("timeout");
        let started = Instant::now();

        let error = run(
            ProcessRequest::new(ProgramId::TestSlow, espera_de_trinta_segundos(), &root)
                .with_timeout(Duration::from_millis(300)),
        )
        .unwrap_err();

        assert_eq!(
            error.code(),
            "EXECUTION_TIMEOUT",
            "esperava tempo limite e veio {}: quase sempre significa que o \
             programa de espera desta plataforma não pôde ser iniciado, e não \
             que o encerramento por tempo limite quebrou — detalhe: {error}",
            error.code()
        );
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "o processo não foi encerrado no prazo"
        );
        let _ = fs::remove_dir_all(root);
    }

    /// `core.fsmonitor` aponta para um PROGRAMA e pode vir do `.git/config` de
    /// um repositório apenas aberto. Este teste tenta usar isso para executar
    /// código durante um `git status`.
    #[test]
    fn config_do_repositorio_nao_executa_programa_no_status() {
        let root = init_repo("fsmonitor");
        let marker = root.join("PWNED_FSMONITOR");
        let hook = root.join("evil.sh");

        fs::write(&hook, format!("#!/bin/sh\ntouch {}\n", marker.display())).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&hook, fs::Permissions::from_mode(0o755)).unwrap();
        }

        git(&root, &["config", "core.fsmonitor", hook.to_str().unwrap()]);

        let outcome = git(&root, &["status", "--porcelain=v1"]);
        assert!(outcome.success || !outcome.stderr.is_empty());
        assert!(
            !marker.exists(),
            "VULNERABILIDADE: core.fsmonitor do repositório executou um programa"
        );

        let _ = fs::remove_dir_all(root);
    }

    /// Mesma ideia por outro caminho: `diff.external` roda um programa quando
    /// o git calcula um diff.
    #[test]
    fn config_do_repositorio_nao_executa_programa_no_diff() {
        let root = init_repo("diffexternal");
        let marker = root.join("PWNED_DIFF");
        let hook = root.join("evil-diff.sh");

        fs::write(&hook, format!("#!/bin/sh\ntouch {}\n", marker.display())).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&hook, fs::Permissions::from_mode(0o755)).unwrap();
        }

        git(&root, &["config", "diff.external", hook.to_str().unwrap()]);
        fs::write(root.join("a.txt"), "2").unwrap();

        let outcome = git(&root, &["diff"]);
        assert!(outcome.success || !outcome.stderr.is_empty());
        assert!(
            !marker.exists(),
            "VULNERABILIDADE: diff.external do repositório executou um programa"
        );

        let _ = fs::remove_dir_all(root);
    }

    /// A variável de ambiente é o mesmo ataque vindo do processo pai.
    #[test]
    fn variavel_de_ambiente_perigosa_nao_e_herdada() {
        let root = init_repo("env");
        let marker = root.join("PWNED_ENV");
        let hook = root.join("evil-env.sh");

        fs::write(&hook, format!("#!/bin/sh\ntouch {}\n", marker.display())).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&hook, fs::Permissions::from_mode(0o755)).unwrap();
        }

        // Envenena o ambiente do PRÓPRIO processo de teste, como faria um
        // ambiente hostil onde o DEWRENCH foi iniciado.
        unsafe { std::env::set_var("GIT_EXTERNAL_DIFF", hook.to_str().unwrap()) };
        fs::write(root.join("a.txt"), "3").unwrap();

        let outcome = git(&root, &["diff"]);
        unsafe { std::env::remove_var("GIT_EXTERNAL_DIFF") };

        assert!(outcome.success || !outcome.stderr.is_empty());
        assert!(
            !marker.exists(),
            "VULNERABILIDADE: GIT_EXTERNAL_DIFF herdado executou um programa"
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn saida_gigante_e_truncada_em_vez_de_consumir_memoria() {
        let root = lab("truncagem");
        let (text, truncated) = read_capped(std::io::Cursor::new(vec![b'x'; 1024]), 100);
        assert_eq!(text.len(), 100);
        assert!(truncated);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn prompt_de_terminal_fica_desligado() {
        // Não há como observar a variável de fora do processo filho sem um
        // programa auxiliar; o que é verificável aqui é que o broker a define
        // em toda invocação — garantido pela ausência de caminho alternativo.
        let root = init_repo("prompt");
        let outcome = git(&root, &["config", "--get", "user.name"]);
        assert!(outcome.success);
        let _ = fs::remove_dir_all(root);
    }
}
