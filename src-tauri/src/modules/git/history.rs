use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::errors::{codes, GitOperationError};
use super::git_cli;
use super::models::{GitRevertFileChange, GitRevertOutcome, GitRevertPreview};

/// Limite defensivo para a revisão recebida pelo IPC.
const MAX_REVISION_LENGTH: usize = 200;

/// Marcadores de operação Git intermediária.
///
/// Os caminhos são resolvidos por `git rev-parse --git-path`, e não montados a
/// partir de `.git/`, para funcionar com worktrees e com repositórios cujo
/// `.git` é um arquivo.
const IN_PROGRESS_MARKERS: [(&str, &str); 6] = [
    ("MERGE_HEAD", "merge"),
    ("REVERT_HEAD", "revert"),
    ("CHERRY_PICK_HEAD", "cherry-pick"),
    ("BISECT_LOG", "bisect"),
    ("rebase-merge", "rebase"),
    ("rebase-apply", "rebase"),
];

/// Estado do working tree lido de `git status --porcelain=v1 -z`.
#[derive(Debug, Default, PartialEq)]
struct WorkingTreeStatus {
    staged: Vec<String>,
    unstaged: Vec<String>,
    untracked: Vec<String>,
    conflicted: Vec<String>,
}

/// Contexto validado, compartilhado entre preview e execução.
struct RevertContext {
    preview: GitRevertPreview,
}

/// Preview read-only do Revert.
///
/// Não muta o repositório em nenhuma circunstância.
pub fn get_revert_preview(
    path: &Path,
    revision: &str,
) -> Result<GitRevertPreview, GitOperationError> {
    Ok(build_context(path, revision)?.preview)
}

/// Executa `git revert --no-edit` depois de revalidar todo o preflight.
///
/// O histórico anterior é preservado: o commit original permanece e um novo
/// commit inverso é criado. Em caso de conflito, a tentativa é abortada e o
/// repositório volta ao estado anterior comprovado.
pub fn revert_commit(
    path: &Path,
    revision: &str,
) -> Result<GitRevertOutcome, GitOperationError> {
    // A revalidação imediatamente antes da mutação é obrigatória: o preview
    // pode ter sido calculado há minutos e o repositório pode ter mudado.
    let context = build_context(path, revision)?;
    let preview = context.preview;

    let status_before = read_status_raw(path)?;
    let head_before = read_head(path)?;

    let output = run(path, &["revert", "--no-edit", preview.hash.as_str()])?;

    if output.success {
        let (new_hash, new_subject) = read_head_commit(path)?;
        let new_short_hash = read_short_hash(path, &new_hash)?;

        return Ok(GitRevertOutcome {
            reverted_hash: preview.hash,
            reverted_short_hash: preview.short_hash,
            new_commit_hash: new_hash,
            new_commit_short_hash: new_short_hash,
            new_commit_subject: new_subject,
            affected_files: preview.affected_files,
            warnings: preview.warnings,
            history_preserved: true,
        });
    }

    // Falhou. Conflito é um estado intermediário, não um erro comum.
    if marker_exists(path, "REVERT_HEAD").unwrap_or(false) {
        let conflicted = read_conflicted_files(path);
        return Err(recover_from_conflict(
            path,
            conflicted,
            &status_before,
            &head_before,
            &output,
        ));
    }

    Err(
        GitOperationError::new(codes::GIT_COMMAND_FAILED, "O Git recusou o revert deste commit.")
            .with_details(describe_output(&output))
            .with_action("Consulte os detalhes técnicos e tente novamente após ajustar o repositório."),
    )
}

// ---------------------------------------------------------------------------
// Preflight
// ---------------------------------------------------------------------------

fn build_context(path: &Path, revision: &str) -> Result<RevertContext, GitOperationError> {
    // 1. o caminho representa um repositório Git válido
    ensure_repository(path)?;

    // 2 e 3. o hash existe e o objeto é realmente um commit
    let revision = validate_revision(revision)?;
    let hash = resolve_commit(path, revision)?;

    // 7. o commit não é um merge commit
    let parent_count = read_parent_count(path, &hash)?;
    if parent_count > 1 {
        return Err(GitOperationError::new(
            codes::MERGE_COMMIT_UNSUPPORTED,
            "Este commit é um merge e exige a escolha explícita de um parent/mainline. \
             O DEWRENCH ainda não oferece essa escolha com segurança.",
        )
        .with_details(format!("O commit possui {parent_count} parents."))
        .with_action(
            "Reverta um commit comum. A escolha de mainline será tratada em uma iteração dedicada.",
        ));
    }

    // 4. não existe operação Git intermediária
    if let Some(operation) = detect_operation_in_progress(path)? {
        return Err(GitOperationError::new(
            codes::OPERATION_IN_PROGRESS,
            format!("Existe uma operação {operation} em andamento neste repositório."),
        )
        .with_action(
            "Conclua ou aborte essa operação no Git antes de reverter. O DEWRENCH não cancela \
             automaticamente uma operação que já existia.",
        ));
    }

    let status = parse_status_z(&read_status_raw(path)?);

    // 6. não existem conflitos já ativos
    if !status.conflicted.is_empty() {
        return Err(GitOperationError::new(
            codes::OPERATION_IN_PROGRESS,
            "Existem arquivos em conflito não resolvidos neste repositório.",
        )
        .with_files(status.conflicted)
        .with_action("Resolva os conflitos existentes antes de reverter."));
    }

    // 5. não existem mudanças staged
    if !status.staged.is_empty() {
        return Err(GitOperationError::new(
            codes::STAGED_CHANGES,
            "Existem alterações no staging area. O Revert cria um commit e usaria esse conteúdo.",
        )
        .with_files(status.staged)
        .with_action("Faça commit dessas alterações ou remova-as do staging antes de reverter."));
    }

    // 8. a identidade Git necessária para criar o commit está configurada
    ensure_identity(path)?;

    let affected_files = read_commit_files(path, &hash)?;

    // 9. alterações locais não colidem com arquivos afetados pelo commit
    let affected_paths = collect_affected_paths(&affected_files);
    let local_changes = collect_local_changes(&status);
    let overlapping: Vec<String> = local_changes
        .iter()
        .filter(|candidate| affected_paths.contains(*candidate))
        .cloned()
        .collect();

    if !overlapping.is_empty() {
        return Err(GitOperationError::new(
            codes::OVERLAPPING_WORKTREE_CHANGES,
            "Existem alterações locais nos mesmos arquivos afetados por este commit.",
        )
        .with_files(overlapping)
        .with_action(
            "Faça commit, mova para outro lugar ou desfaça essas alterações locais antes de reverter.",
        ));
    }

    let preserved_local_changes: Vec<String> = local_changes.into_iter().collect();
    let (author, subject) = read_commit_metadata(path, &hash)?;
    let short_hash = read_short_hash(path, &hash)?;
    let is_root_commit = parent_count == 0;

    let mut warnings = Vec::new();
    if is_root_commit {
        warnings.push(
            "Este é o commit raiz do repositório. O revert removerá os arquivos introduzidos por ele."
                .to_string(),
        );
    }
    if affected_files.is_empty() {
        warnings.push(
            "Este commit não altera arquivos. O Git pode recusar a criação de um commit vazio."
                .to_string(),
        );
    }
    if !preserved_local_changes.is_empty() {
        warnings.push(format!(
            "{} alteração(ões) local(is) não relacionada(s) permanecerá(ão) intocada(s).",
            preserved_local_changes.len()
        ));
    }

    Ok(RevertContext {
        preview: GitRevertPreview {
            hash,
            short_hash,
            subject,
            author,
            parent_count,
            is_root_commit,
            affected_files,
            preserved_local_changes,
            warnings,
            creates_new_commit: true,
            preserves_history: true,
        },
    })
}

fn ensure_repository(path: &Path) -> Result<(), GitOperationError> {
    if !path.is_dir() {
        return Err(GitOperationError::new(
            codes::NOT_REPOSITORY,
            "O caminho do projeto não é um diretório acessível.",
        ));
    }

    let output = run(path, &["rev-parse", "--is-inside-work-tree"])?;

    if !output.success || output.stdout.trim() != "true" {
        return Err(GitOperationError::new(
            codes::NOT_REPOSITORY,
            "Este projeto não é um repositório Git válido.",
        )
        .with_details(describe_output(&output))
        .with_action("Abra uma pasta que contenha um repositório Git."));
    }

    Ok(())
}

/// Rejeita entradas que poderiam virar opção do Git ou corromper o argumento.
///
/// A validação é defensiva; a segurança real vem de argumentos separados em
/// `Command` e de `--end-of-options` na resolução.
fn validate_revision(revision: &str) -> Result<&str, GitOperationError> {
    let trimmed = revision.trim();

    if trimmed.is_empty() {
        return Err(GitOperationError::new(
            codes::INVALID_COMMIT,
            "Nenhuma revisão foi informada.",
        ));
    }

    if trimmed.len() > MAX_REVISION_LENGTH {
        return Err(GitOperationError::new(
            codes::INVALID_COMMIT,
            "A revisão informada é longa demais para ser válida.",
        ));
    }

    if trimmed.starts_with('-') {
        return Err(GitOperationError::new(
            codes::INVALID_COMMIT,
            "A revisão informada não é válida.",
        ));
    }

    if trimmed.chars().any(|character| character.is_control() || character.is_whitespace()) {
        return Err(GitOperationError::new(
            codes::INVALID_COMMIT,
            "A revisão informada contém caracteres inválidos.",
        ));
    }

    Ok(trimmed)
}

fn resolve_commit(path: &Path, revision: &str) -> Result<String, GitOperationError> {
    let expression = format!("{revision}^{{commit}}");
    let output = run(
        path,
        &["rev-parse", "--verify", "--end-of-options", expression.as_str()],
    )?;

    if !output.success {
        return Err(GitOperationError::new(
            codes::INVALID_COMMIT,
            "A revisão informada não corresponde a um commit deste repositório.",
        )
        .with_details(describe_output(&output))
        .with_action("Selecione um commit existente no grafo."));
    }

    let hash = output.stdout.trim().to_string();

    if hash.len() < 40 || !hash.chars().all(|character| character.is_ascii_hexdigit()) {
        return Err(GitOperationError::new(
            codes::INVALID_COMMIT,
            "O Git não devolveu um hash de commit reconhecível.",
        )
        .with_details(describe_output(&output)));
    }

    Ok(hash)
}

fn read_parent_count(path: &Path, hash: &str) -> Result<usize, GitOperationError> {
    let output = run(path, &["rev-list", "--parents", "-n", "1", hash])?;

    if !output.success {
        return Err(command_failed(&output, "Não foi possível ler os parents do commit."));
    }

    Ok(parse_parent_count(&output.stdout))
}

fn detect_operation_in_progress(path: &Path) -> Result<Option<String>, GitOperationError> {
    for (marker, label) in IN_PROGRESS_MARKERS {
        if marker_exists(path, marker)? {
            return Ok(Some(label.to_string()));
        }
    }

    Ok(None)
}

fn marker_exists(path: &Path, marker: &str) -> Result<bool, GitOperationError> {
    Ok(git_path(path, marker)?.exists())
}

fn git_path(path: &Path, name: &str) -> Result<PathBuf, GitOperationError> {
    let output = run(path, &["rev-parse", "--git-path", name])?;

    if !output.success {
        return Err(command_failed(
            &output,
            "Não foi possível localizar os metadados do repositório.",
        ));
    }

    let candidate = PathBuf::from(output.stdout.trim());

    Ok(if candidate.is_absolute() {
        candidate
    } else {
        path.join(candidate)
    })
}

fn ensure_identity(path: &Path) -> Result<(), GitOperationError> {
    for variable in ["GIT_AUTHOR_IDENT", "GIT_COMMITTER_IDENT"] {
        let output = run(path, &["var", variable])?;

        // Uma identidade resolvida mas vazia ("Nome <>") faz o commit falhar
        // depois, já com o revert aplicado; por isso é barrada aqui.
        if !output.success || output.stdout.contains("<>") {
            return Err(GitOperationError::new(
                codes::IDENTITY_NOT_CONFIGURED,
                "O Git não possui identidade configurada para criar commits.",
            )
            .with_details(describe_output(&output))
            .with_action("Configure user.name e user.email no Git antes de reverter."));
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Leitura de estado
// ---------------------------------------------------------------------------

fn read_status_raw(path: &Path) -> Result<String, GitOperationError> {
    let output = run(
        path,
        &["status", "--porcelain=v1", "-z", "--untracked-files=all"],
    )?;

    if !output.success {
        return Err(command_failed(
            &output,
            "Não foi possível ler o estado do working tree.",
        ));
    }

    Ok(output.stdout)
}

fn read_conflicted_files(path: &Path) -> Vec<String> {
    match read_status_raw(path) {
        Ok(raw) => parse_status_z(&raw).conflicted,
        Err(_) => Vec::new(),
    }
}

fn read_commit_files(
    path: &Path,
    hash: &str,
) -> Result<Vec<GitRevertFileChange>, GitOperationError> {
    // `--no-renames` mantém a saída determinística: um rename aparece como
    // remoção e adição, o que também amplia a checagem de sobreposição.
    let output = run(
        path,
        &[
            "diff-tree",
            "--no-commit-id",
            "--name-status",
            "--no-renames",
            "-r",
            "-z",
            "--root",
            hash,
        ],
    )?;

    if !output.success {
        return Err(command_failed(
            &output,
            "Não foi possível ler os arquivos alterados pelo commit.",
        ));
    }

    Ok(parse_name_status_z(&output.stdout))
}

fn read_commit_metadata(
    path: &Path,
    hash: &str,
) -> Result<(String, String), GitOperationError> {
    let output = run(path, &["log", "-1", "--format=%an%x1f%s", hash])?;

    if !output.success {
        return Err(command_failed(&output, "Não foi possível ler os dados do commit."));
    }

    let line = output.stdout.trim_end().to_string();
    let mut parts = line.splitn(2, '\u{1f}');
    let author = parts.next().unwrap_or("").to_string();
    let subject = parts.next().unwrap_or("").to_string();

    Ok((author, subject))
}

fn read_short_hash(path: &Path, hash: &str) -> Result<String, GitOperationError> {
    let output = run(path, &["rev-parse", "--short", hash])?;

    if !output.success {
        return Ok(hash.chars().take(7).collect());
    }

    Ok(output.stdout.trim().to_string())
}

fn read_head(path: &Path) -> Result<String, GitOperationError> {
    let output = run(path, &["rev-parse", "HEAD"])?;

    if !output.success {
        return Err(command_failed(&output, "Não foi possível ler o HEAD atual."));
    }

    Ok(output.stdout.trim().to_string())
}

fn read_head_commit(path: &Path) -> Result<(String, String), GitOperationError> {
    let hash = read_head(path)?;
    let output = run(path, &["log", "-1", "--format=%s", hash.as_str()])?;
    let subject = if output.success {
        output.stdout.trim_end().to_string()
    } else {
        String::new()
    };

    Ok((hash, subject))
}

// ---------------------------------------------------------------------------
// Recuperação de conflito
// ---------------------------------------------------------------------------

/// Aborta o revert conflitado e só declara restauração com comprovação.
///
/// `reset --hard` nunca é usado: ele descartaria trabalho do usuário.
fn recover_from_conflict(
    path: &Path,
    conflicted: Vec<String>,
    status_before: &str,
    head_before: &str,
    attempt: &git_cli::GitCommandOutput,
) -> GitOperationError {
    let abort_succeeded = match run(path, &["revert", "--abort"]) {
        Ok(output) => output.success,
        Err(_) => false,
    };

    let marker_cleared = matches!(marker_exists(path, "REVERT_HEAD"), Ok(false));
    let head_restored = matches!(read_head(path), Ok(head) if head == head_before);
    let status_restored = matches!(read_status_raw(path), Ok(status) if status == status_before);

    if abort_succeeded && marker_cleared && head_restored && status_restored {
        return GitOperationError::new(
            codes::REVERT_CONFLICT_ABORTED,
            "O revert gerou conflitos e foi abortado. Nenhum commit de revert foi criado e o \
             repositório voltou ao estado anterior.",
        )
        .with_files(conflicted)
        .with_details(describe_output(attempt))
        .with_action(
            "Resolva manualmente as diferenças nesses arquivos ou reverta o commit pela linha de \
             comando, onde é possível continuar o revert após resolver os conflitos.",
        );
    }

    GitOperationError::critical(
        codes::REVERT_CONFLICT_ABORT_FAILED,
        "O revert gerou conflitos e o DEWRENCH não conseguiu comprovar a restauração do estado \
         anterior. Inspecione o repositório antes de continuar.",
    )
    .with_files(conflicted)
    .with_details(format!(
        "abort executado: {abort_succeeded} | REVERT_HEAD removido: {marker_cleared} | \
         HEAD restaurado: {head_restored} | status restaurado: {status_restored} | {}",
        describe_output(attempt)
    ))
    .with_action(
        "Verifique `git status` e `git revert --abort` manualmente. Não execute operações \
         destrutivas antes de entender o estado atual.",
    )
}

// ---------------------------------------------------------------------------
// Adapter
// ---------------------------------------------------------------------------

fn run(
    path: &Path,
    args: &[&str],
) -> Result<git_cli::GitCommandOutput, GitOperationError> {
    git_cli::run_structured(path, args).map_err(|error| match error.kind() {
        std::io::ErrorKind::NotFound => GitOperationError::new(
            codes::GIT_NOT_FOUND,
            "O executável do Git não foi encontrado neste sistema.",
        )
        .with_action("Instale o Git e confirme que ele está disponível no PATH."),
        std::io::ErrorKind::PermissionDenied => GitOperationError::new(
            codes::PERMISSION_DENIED,
            "O sistema negou permissão para executar o Git neste diretório.",
        )
        .with_action("Verifique as permissões da pasta do repositório."),
        _ => GitOperationError::new(codes::GIT_COMMAND_FAILED, "Não foi possível executar o Git.")
            .with_details(error.to_string()),
    })
}

fn command_failed(output: &git_cli::GitCommandOutput, message: &str) -> GitOperationError {
    GitOperationError::new(codes::GIT_COMMAND_FAILED, message).with_details(describe_output(output))
}

fn describe_output(output: &git_cli::GitCommandOutput) -> String {
    let code = output
        .exit_code
        .map(|value| value.to_string())
        .unwrap_or_else(|| "desconhecido".to_string());

    let mut parts = vec![format!("exit code: {code}")];

    if !output.stderr.trim().is_empty() {
        parts.push(format!("stderr: {}", output.stderr.trim()));
    }

    if !output.stdout.trim().is_empty() {
        parts.push(format!("stdout: {}", output.stdout.trim()));
    }

    parts.join(" | ")
}

// ---------------------------------------------------------------------------
// Parsers puros
// ---------------------------------------------------------------------------

fn collect_affected_paths(files: &[GitRevertFileChange]) -> BTreeSet<String> {
    let mut paths = BTreeSet::new();

    for file in files {
        paths.insert(file.path.clone());

        if let Some(original) = &file.original_path {
            paths.insert(original.clone());
        }
    }

    paths
}

fn collect_local_changes(status: &WorkingTreeStatus) -> BTreeSet<String> {
    let mut changes = BTreeSet::new();

    for path in status.unstaged.iter().chain(status.untracked.iter()) {
        changes.insert(path.clone());
    }

    changes
}

fn parse_parent_count(raw: &str) -> usize {
    raw.lines()
        .next()
        .map(|line| line.split_whitespace().count().saturating_sub(1))
        .unwrap_or(0)
}

/// Interpreta `git status --porcelain=v1 -z`.
///
/// A saída delimitada por NUL preserva espaços, Unicode e paths incomuns. Em
/// entradas de rename ou cópia, o path de origem vem no token seguinte.
fn parse_status_z(raw: &str) -> WorkingTreeStatus {
    let mut status = WorkingTreeStatus::default();
    let mut tokens = raw.split('\u{0}');

    while let Some(entry) = tokens.next() {
        if entry.len() < 3 {
            continue;
        }

        let mut characters = entry.chars();
        let index = characters.next().unwrap_or(' ');
        let worktree = characters.next().unwrap_or(' ');
        // Os dois códigos e o separador são sempre ASCII, então o byte 3 é um
        // limite de caractere válido mesmo com paths Unicode.
        let path = entry.get(3..).unwrap_or("").to_string();

        if index == 'R' || index == 'C' {
            if let Some(original) = tokens.next() {
                if !original.is_empty() {
                    status.staged.push(original.to_string());
                }
            }
        }

        if index == '?' && worktree == '?' {
            status.untracked.push(path);
            continue;
        }

        if index == '!' && worktree == '!' {
            continue;
        }

        if index == 'U' || worktree == 'U' || (index == 'A' && worktree == 'A')
            || (index == 'D' && worktree == 'D')
        {
            status.conflicted.push(path);
            continue;
        }

        if index != ' ' {
            status.staged.push(path.clone());
        }

        if worktree != ' ' {
            status.unstaged.push(path);
        }
    }

    status
}

/// Interpreta `git diff-tree --name-status -z`.
fn parse_name_status_z(raw: &str) -> Vec<GitRevertFileChange> {
    let mut changes = Vec::new();
    let mut tokens = raw.split('\u{0}');

    while let Some(token) = tokens.next() {
        if token.is_empty() {
            continue;
        }

        let status = token.to_string();
        let is_rename_like = status.starts_with('R') || status.starts_with('C');

        let first = match tokens.next() {
            Some(value) if !value.is_empty() => value.to_string(),
            _ => continue,
        };

        if is_rename_like {
            let second = tokens.next().unwrap_or("").to_string();

            changes.push(GitRevertFileChange {
                status,
                path: second,
                original_path: Some(first),
            });
        } else {
            changes.push(GitRevertFileChange {
                status,
                path: first,
                original_path: None,
            });
        }
    }

    changes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_status_z_classifica_index_e_working_tree() {
        let raw = "M  staged.txt\u{0} M unstaged.txt\u{0}?? novo.txt\u{0}";
        let status = parse_status_z(raw);

        assert_eq!(status.staged, vec!["staged.txt".to_string()]);
        assert_eq!(status.unstaged, vec!["unstaged.txt".to_string()]);
        assert_eq!(status.untracked, vec!["novo.txt".to_string()]);
        assert!(status.conflicted.is_empty());
    }

    #[test]
    fn parse_status_z_preserva_espacos_e_unicode() {
        let raw = " M dir com espaco/arq ç.txt\u{0}";
        let status = parse_status_z(raw);

        assert_eq!(status.unstaged, vec!["dir com espaco/arq ç.txt".to_string()]);
    }

    #[test]
    fn parse_status_z_trata_rename_com_path_de_origem() {
        let raw = "R  novo nome.txt\u{0}nome antigo.txt\u{0} M outro.txt\u{0}";
        let status = parse_status_z(raw);

        assert!(status.staged.contains(&"nome antigo.txt".to_string()));
        assert!(status.staged.contains(&"novo nome.txt".to_string()));
        assert_eq!(status.unstaged, vec!["outro.txt".to_string()]);
    }

    #[test]
    fn parse_status_z_detecta_conflito() {
        let raw = "UU conflito.txt\u{0}";
        let status = parse_status_z(raw);

        assert_eq!(status.conflicted, vec!["conflito.txt".to_string()]);
        assert!(status.staged.is_empty());
    }

    #[test]
    fn parse_name_status_z_le_paths_incomuns() {
        let raw = "M\u{0}dir com espaco/arq ç.txt\u{0}A\u{0}novo.txt\u{0}";
        let changes = parse_name_status_z(raw);

        assert_eq!(changes.len(), 2);
        assert_eq!(changes[0].path, "dir com espaco/arq ç.txt");
        assert_eq!(changes[0].status, "M");
        assert_eq!(changes[1].original_path, None);
    }

    #[test]
    fn parse_name_status_z_trata_rename() {
        let raw = "R100\u{0}antigo.txt\u{0}novo.txt\u{0}";
        let changes = parse_name_status_z(raw);

        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].path, "novo.txt");
        assert_eq!(changes[0].original_path, Some("antigo.txt".to_string()));
    }

    #[test]
    fn parse_parent_count_conta_root_e_merge() {
        assert_eq!(parse_parent_count("aaa\n"), 0);
        assert_eq!(parse_parent_count("aaa bbb\n"), 1);
        assert_eq!(parse_parent_count("aaa bbb ccc\n"), 2);
    }

    #[test]
    fn validate_revision_rejeita_entradas_hostis() {
        assert!(validate_revision("").is_err());
        assert!(validate_revision("--upload-pack=rm").is_err());
        assert!(validate_revision("abc def").is_err());
        assert!(validate_revision(&"a".repeat(MAX_REVISION_LENGTH + 1)).is_err());
        assert_eq!(validate_revision("  abc123  ").unwrap(), "abc123");
    }

    #[test]
    fn sobreposicao_usa_path_de_origem_do_rename() {
        let files = vec![GitRevertFileChange {
            status: "R100".to_string(),
            path: "novo.txt".to_string(),
            original_path: Some("antigo.txt".to_string()),
        }];

        let paths = collect_affected_paths(&files);

        assert!(paths.contains("novo.txt"));
        assert!(paths.contains("antigo.txt"));
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    /// Repositório Git descartável, criado fora do projeto e removido no Drop.
    ///
    /// A identidade é gravada apenas no escopo local: a configuração global e a
    /// de sistema do usuário nunca são alteradas. Nenhum remote é configurado.
    struct TempRepo {
        path: PathBuf,
    }

    impl TempRepo {
        fn new(label: &str) -> TempRepo {
            let unique = format!(
                "dewrench-revert-test-{}-{}-{}",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::SeqCst),
                label
            );

            let path = std::env::temp_dir().join(unique);
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).expect("criar diretório temporário");

            let repo = TempRepo { path };
            assert!(repo.run(&["init", "-q", "-b", "main", "."]).success);

            // Neutraliza a conversão de fim de linha ANTES de qualquer arquivo
            // ser criado ou adicionado. No Windows, `core.autocrlf=true` no
            // escopo global/de sistema faria o checkout devolver CRLF e as
            // asserções byte a byte deste laboratório falhariam sem que nada
            // estivesse errado na implementação.
            //
            // A configuração é estritamente local a este repositório
            // descartável: a configuração global e a de sistema do usuário não
            // são tocadas, e a aplicação nunca impõe isso a repositórios reais.
            assert!(repo.run(&["config", "--local", "core.autocrlf", "false"]).success);
            assert!(repo.run(&["config", "--local", "core.eol", "lf"]).success);
            assert_eq!(
                repo.run(&["config", "--local", "--get", "core.autocrlf"]).stdout.trim(),
                "false",
                "o laboratório precisa desabilitar core.autocrlf localmente"
            );

            repo.run(&["config", "--local", "user.name", "DEWRENCH Test"]);
            repo.run(&["config", "--local", "user.email", "test@dewrench.local"]);
            repo.run(&["config", "--local", "commit.gpgsign", "false"]);
            repo
        }

        fn run(&self, args: &[&str]) -> git_cli::GitCommandOutput {
            git_cli::run_structured(&self.path, args).expect("executar git")
        }

        fn write(&self, name: &str, content: &str) {
            let target = self.path.join(name);

            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).expect("criar pasta do arquivo");
            }

            fs::write(target, content).expect("escrever arquivo");
        }

        fn read(&self, name: &str) -> String {
            fs::read_to_string(self.path.join(name)).expect("ler arquivo")
        }

        fn commit_all(&self, message: &str) -> String {
            assert!(self.run(&["add", "-A"]).success);
            let output = self.run(&["commit", "-m", message]);
            assert!(output.success, "commit falhou: {}", output.stderr);
            self.head()
        }

        fn head(&self) -> String {
            self.run(&["rev-parse", "HEAD"]).stdout.trim().to_string()
        }

        fn status(&self) -> String {
            self.run(&["status", "--porcelain=v1", "-z", "--untracked-files=all"])
                .stdout
        }

        fn commit_count(&self) -> String {
            self.run(&["rev-list", "--count", "HEAD"]).stdout.trim().to_string()
        }
    }

    impl Drop for TempRepo {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn preview_nao_altera_o_repositorio() {
        let repo = TempRepo::new("preview-readonly");
        repo.write("f.txt", "a\n");
        repo.commit_all("c1");
        repo.write("f.txt", "a\nb\n");
        let target = repo.commit_all("c2");

        let head_before = repo.head();
        let status_before = repo.status();

        let preview = get_revert_preview(&repo.path, &target).expect("preview deve funcionar");

        assert!(preview.creates_new_commit);
        assert!(preview.preserves_history);
        assert_eq!(repo.head(), head_before);
        assert_eq!(repo.status(), status_before);
        assert_eq!(repo.commit_count(), "2");
    }

    #[test]
    fn revert_de_commit_comum_cria_commit_inverso_e_preserva_historico() {
        let repo = TempRepo::new("comum");
        repo.write("f.txt", "a\n");
        let first = repo.commit_all("c1");
        repo.write("f.txt", "a\nb\n");
        let second = repo.commit_all("c2");

        let outcome = revert_commit(&repo.path, &second).expect("revert deve concluir");

        assert_eq!(outcome.reverted_hash, second);
        assert!(outcome.history_preserved);
        assert_ne!(outcome.new_commit_hash, second);

        // A -> B -> B': o commit original continua no histórico.
        let log = repo.run(&["log", "--format=%H"]).stdout;
        let hashes: Vec<&str> = log.lines().collect();

        assert_eq!(hashes.len(), 3);
        assert_eq!(hashes[0], outcome.new_commit_hash);
        assert_eq!(hashes[1], second);
        assert_eq!(hashes[2], first);
        assert_eq!(repo.read("f.txt"), "a\n");
    }

    #[test]
    fn revert_de_root_commit_e_suportado() {
        let repo = TempRepo::new("root");
        repo.write("f.txt", "a\n");
        let root = repo.commit_all("c1 root");

        let preview = get_revert_preview(&repo.path, &root).expect("preview do root");

        assert!(preview.is_root_commit);
        assert_eq!(preview.parent_count, 0);
        assert!(!preview.warnings.is_empty());

        let outcome = revert_commit(&repo.path, &root).expect("revert do root");

        assert!(outcome.history_preserved);
        assert!(!repo.path.join("f.txt").exists());
        assert_eq!(repo.commit_count(), "2");
    }

    #[test]
    fn merge_commit_e_bloqueado_no_preview_e_na_execucao() {
        let repo = TempRepo::new("merge");
        repo.write("base.txt", "base\n");
        repo.commit_all("base");
        assert!(repo.run(&["checkout", "-q", "-b", "feature"]).success);
        repo.write("feature.txt", "feature\n");
        repo.commit_all("feature");
        assert!(repo.run(&["checkout", "-q", "main"]).success);
        repo.write("main.txt", "main\n");
        repo.commit_all("main");
        assert!(repo
            .run(&["merge", "--no-ff", "-m", "merge feature", "feature"])
            .success);

        let merge_hash = repo.head();
        let head_before = repo.head();

        let error = get_revert_preview(&repo.path, &merge_hash).expect_err("merge deve bloquear");
        assert_eq!(error.code, codes::MERGE_COMMIT_UNSUPPORTED);

        let error = revert_commit(&repo.path, &merge_hash).expect_err("execução também bloqueia");
        assert_eq!(error.code, codes::MERGE_COMMIT_UNSUPPORTED);
        assert_eq!(repo.head(), head_before);
    }

    #[test]
    fn mudancas_staged_bloqueiam_o_revert() {
        let repo = TempRepo::new("staged");
        repo.write("f.txt", "a\n");
        repo.commit_all("c1");
        repo.write("f.txt", "a\nb\n");
        let target = repo.commit_all("c2");

        repo.write("outro.txt", "novo\n");
        assert!(repo.run(&["add", "outro.txt"]).success);

        let error = get_revert_preview(&repo.path, &target).expect_err("staged deve bloquear");

        assert_eq!(error.code, codes::STAGED_CHANGES);
        assert!(error.affected_files.contains(&"outro.txt".to_string()));
    }

    #[test]
    fn alteracoes_locais_nao_relacionadas_sao_preservadas() {
        let repo = TempRepo::new("preserva");
        repo.write("f.txt", "a\n");
        repo.write("outro.txt", "intacto\n");
        repo.commit_all("c1");
        repo.write("f.txt", "a\nb\n");
        let target = repo.commit_all("c2");

        repo.write("outro.txt", "modificado localmente\n");

        let preview = get_revert_preview(&repo.path, &target).expect("preview");
        assert_eq!(preview.preserved_local_changes, vec!["outro.txt".to_string()]);

        let outcome = revert_commit(&repo.path, &target).expect("revert deve concluir");

        assert!(outcome.history_preserved);
        assert_eq!(repo.read("outro.txt"), "modificado localmente\n");
        assert_eq!(repo.read("f.txt"), "a\n");
    }

    #[test]
    fn alteracoes_locais_sobrepostas_bloqueiam_o_revert() {
        let repo = TempRepo::new("sobreposta");
        repo.write("f.txt", "a\n");
        repo.commit_all("c1");
        repo.write("f.txt", "a\nb\n");
        let target = repo.commit_all("c2");

        repo.write("f.txt", "a\nb\nlocal\n");

        let head_before = repo.head();
        let error = get_revert_preview(&repo.path, &target).expect_err("sobreposição bloqueia");

        assert_eq!(error.code, codes::OVERLAPPING_WORKTREE_CHANGES);
        assert!(error.affected_files.contains(&"f.txt".to_string()));

        let error = revert_commit(&repo.path, &target).expect_err("execução também bloqueia");

        assert_eq!(error.code, codes::OVERLAPPING_WORKTREE_CHANGES);
        assert_eq!(repo.head(), head_before);
        assert_eq!(repo.read("f.txt"), "a\nb\nlocal\n");
    }

    #[test]
    fn arquivo_untracked_sobreposto_bloqueia_o_revert() {
        let repo = TempRepo::new("untracked");
        repo.write("f.txt", "a\n");
        repo.commit_all("c1");
        repo.write("novo.txt", "conteudo\n");
        let target = repo.commit_all("c2 adiciona novo");

        assert!(repo.run(&["rm", "--cached", "-q", "novo.txt"]).success);
        assert!(repo.run(&["commit", "-qm", "remove do index"]).success);
        // novo.txt agora é untracked e continua sendo alterado por c2.

        let error = get_revert_preview(&repo.path, &target).expect_err("untracked sobreposto bloqueia");
        assert_eq!(error.code, codes::OVERLAPPING_WORKTREE_CHANGES);
    }

    #[test]
    fn revisao_invalida_e_recusada() {
        let repo = TempRepo::new("invalida");
        repo.write("f.txt", "a\n");
        repo.commit_all("c1");

        let revisions = [
            "0000000000000000000000000000000000000000",
            "nao-existe",
            "",
            "--upload-pack=x",
            "com espaco",
        ];

        for revision in revisions {
            let error = get_revert_preview(&repo.path, revision).expect_err("deve recusar");
            assert_eq!(error.code, codes::INVALID_COMMIT, "revisão: {revision}");
        }
    }

    #[test]
    fn identidade_ausente_bloqueia_o_revert() {
        let repo = TempRepo::new("identidade");
        repo.write("f.txt", "a\n");
        repo.commit_all("c1");
        repo.write("f.txt", "a\nb\n");
        let target = repo.commit_all("c2");

        // Valores locais vazios vencem qualquer configuração global do ambiente.
        repo.run(&["config", "--local", "user.name", ""]);
        repo.run(&["config", "--local", "user.email", ""]);

        let head_before = repo.head();
        let error = get_revert_preview(&repo.path, &target).expect_err("sem identidade bloqueia");

        assert_eq!(error.code, codes::IDENTITY_NOT_CONFIGURED);
        assert_eq!(repo.head(), head_before);
    }

    #[test]
    fn operacao_em_andamento_bloqueia_o_revert() {
        let repo = TempRepo::new("emandamento");
        repo.write("f.txt", "base\n");
        repo.commit_all("base");
        assert!(repo.run(&["checkout", "-q", "-b", "feature"]).success);
        repo.write("f.txt", "feature\n");
        repo.commit_all("feature");
        assert!(repo.run(&["checkout", "-q", "main"]).success);
        repo.write("f.txt", "main\n");
        let target = repo.commit_all("main");

        assert!(!repo.run(&["merge", "feature"]).success, "o merge deveria conflitar");

        let error = get_revert_preview(&repo.path, &target).expect_err("estado intermediário bloqueia");
        assert_eq!(error.code, codes::OPERATION_IN_PROGRESS);

        repo.run(&["merge", "--abort"]);
    }

    #[test]
    fn conflito_e_abortado_e_o_estado_anterior_e_comprovadamente_restaurado() {
        let repo = TempRepo::new("conflito");
        repo.write("f.txt", "l1\nl2\nl3\n");
        repo.commit_all("c1");
        repo.write("f.txt", "l1\nALVO\nl3\n");
        let target = repo.commit_all("c2");
        repo.write("f.txt", "l1\nPOSTERIOR\nl3\n");
        repo.commit_all("c3");

        let head_before = repo.head();
        let status_before = repo.status();
        let count_before = repo.commit_count();

        let error = revert_commit(&repo.path, &target).expect_err("deve conflitar");

        assert_eq!(error.code, codes::REVERT_CONFLICT_ABORTED);
        assert!(error.recoverable);
        assert!(error.affected_files.contains(&"f.txt".to_string()));

        // Nenhum REVERT_HEAD remanescente e nenhum commit criado.
        assert!(!marker_exists(&repo.path, "REVERT_HEAD").expect("checar marcador"));
        assert_eq!(repo.head(), head_before);
        assert_eq!(repo.status(), status_before);
        assert_eq!(repo.commit_count(), count_before);
        assert_eq!(repo.read("f.txt"), "l1\nPOSTERIOR\nl3\n");
    }

    #[test]
    fn caminho_sem_repositorio_e_recusado() {
        let path = std::env::temp_dir().join(format!(
            "dewrench-revert-test-sem-repo-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(&path).expect("criar pasta");

        let error = get_revert_preview(&path, "HEAD").expect_err("pasta sem repo deve recusar");
        assert_eq!(error.code, codes::NOT_REPOSITORY);

        let _ = fs::remove_dir_all(&path);
    }
}
