//! Comparação entre duas referências do MESMO repositório.
//!
//! É Git puro, e fica no módulo Git de propósito: comparar duas branches é
//! regra de domínio do Git, não do provider. A consequência prática é que o
//! Compare funciona offline, sem a `gh` e sem consumir API — branches remotas
//! entram como `refs/remotes/*`, que o `fetch` existente já mantém atualizadas.
//!
//! Somente leitura: nenhuma função daqui altera o repositório.

use std::collections::BTreeMap;
use std::path::Path;

use crate::core::process;

use super::errors::{codes, sanitize, GitOperationError};
use super::git_cli;
use super::models::{GitBranchComparison, GitComparisonFile};
use super::sync;

/// Teto de arquivos listados. Uma comparação entre histórias muito distantes
/// pode citar milhares; a lista serve para orientar, não para ser exaustiva.
const COMPARISON_FILE_LIMIT: usize = 500;

/// Compara `base` (destino) com `head` (origem).
///
/// A semântica é a do `base...head`: o que existe em `head` desde o ancestral
/// comum — a mesma que o GitHub usa na tela de compare, para que os dois
/// concordem sobre o mesmo par de branches.
pub fn compare(
    path: &Path,
    base: &str,
    head: &str,
) -> Result<GitBranchComparison, GitOperationError> {
    ensure_repository(path)?;

    let base = resolve(path, base, "a referência de destino")?;
    let head = resolve(path, head, "a referência de origem")?;

    let mut warnings: Vec<String> = Vec::new();
    let mut blocked: Option<String> = None;

    if base == head {
        blocked = Some("A origem e o destino são a mesma referência.".to_string());
    }

    let merge_base = read_merge_base(path, &base, &head);

    // Sem ancestral comum, a forma de três pontos não existe: cai para a
    // comparação direta e avisa, em vez de devolver erro cru do Git.
    let range = if merge_base.is_some() {
        format!("{base}...{head}")
    } else {
        warnings.push(
            "As duas referências não têm ancestral comum; a comparação é direta, não pelo ponto de divergência."
                .to_string(),
        );
        format!("{base}..{head}")
    };

    let (behind, ahead) = read_ahead_behind(path, &base, &head);

    let commits = if blocked.is_some() {
        Vec::new()
    } else {
        sync::range_commits(path, &format!("{base}..{head}"))
    };

    let files = if blocked.is_some() {
        Vec::new()
    } else {
        read_files(path, &range)
    };

    if files.len() >= COMPARISON_FILE_LIMIT {
        warnings.push(format!(
            "A lista de arquivos foi limitada aos primeiros {COMPARISON_FILE_LIMIT}."
        ));
    }

    Ok(GitBranchComparison {
        base,
        head,
        merge_base,
        ahead,
        behind,
        commits,
        files,
        warnings,
        blocked,
    })
}

/// Diff unificado da mesma comparação, no formato que `view/diff.ts` já lê.
///
/// Separado do resumo de propósito: o resumo é barato e a interface o recarrega
/// a cada troca de referência; o diff só é buscado quando alguém pede para ver.
pub fn diff(path: &Path, base: &str, head: &str) -> Result<String, GitOperationError> {
    ensure_repository(path)?;

    let base = resolve(path, base, "a referência de destino")?;
    let head = resolve(path, head, "a referência de origem")?;

    let range = if read_merge_base(path, &base, &head).is_some() {
        format!("{base}...{head}")
    } else {
        format!("{base}..{head}")
    };

    git_cli::run_raw(path, &["diff", &range, "--"]).map_err(|error| {
        GitOperationError::new(
            codes::GIT_COMMAND_FAILED,
            "Não foi possível calcular o diff da comparação.",
        )
        .with_details(sanitize(error))
    })
}

fn ensure_repository(path: &Path) -> Result<(), GitOperationError> {
    if path.join(".git").exists() {
        return Ok(());
    }

    Err(GitOperationError::new(
        codes::NOT_REPOSITORY,
        "Este projeto não possui repositório Git.",
    ))
}

/// Valida e confirma que a referência existe.
///
/// A ordem é a mesma que o resto do módulo Git usa e importa: `operand` **antes**
/// de `rev-parse`, porque `rev-parse` já é um processo e um valor iniciado por
/// `-` teria sido interpretado por ele como opção.
fn resolve(path: &Path, value: &str, what: &str) -> Result<String, GitOperationError> {
    let value = process::operand(value.trim()).map_err(|error| {
        GitOperationError::from(error)
            .with_action(format!("Verifique {what}: o valor não pode começar com '-'."))
    })?;

    let exists = git_cli::run(
        path,
        &["rev-parse", "--verify", "--quiet", &format!("{value}^{{commit}}")],
    )
    .map(|output| !output.trim().is_empty())
    .unwrap_or(false);

    if !exists {
        return Err(GitOperationError::new(
            codes::INVALID_COMMIT,
            format!("A referência '{value}' não existe neste repositório."),
        )
        .with_action("Use uma branch local, uma branch remota já buscada ou um commit."));
    }

    Ok(value.to_string())
}

fn read_merge_base(path: &Path, base: &str, head: &str) -> Option<String> {
    let value = git_cli::run(path, &["merge-base", base, head]).ok()?;
    let value = value.trim().to_string();

    (!value.is_empty()).then_some(value)
}

/// `(atrás, à frente)` de `head` em relação a `base`.
fn read_ahead_behind(path: &Path, base: &str, head: &str) -> (usize, usize) {
    let raw = git_cli::run(
        path,
        &["rev-list", "--left-right", "--count", &format!("{base}...{head}")],
    )
    .unwrap_or_default();

    let mut parts = raw.split_whitespace();
    let behind = parts.next().and_then(|value| value.parse().ok()).unwrap_or(0);
    let ahead = parts.next().and_then(|value| value.parse().ok()).unwrap_or(0);

    (behind, ahead)
}

fn read_files(path: &Path, range: &str) -> Vec<GitComparisonFile> {
    let statuses = git_cli::run(path, &["diff", "--name-status", range, "--"]).unwrap_or_default();
    let numbers = git_cli::run(path, &["diff", "--numstat", range, "--"]).unwrap_or_default();

    let mut totals: BTreeMap<String, (Option<u64>, Option<u64>)> = BTreeMap::new();

    for line in numbers.lines() {
        let mut parts = line.split('\t');
        let (Some(additions), Some(deletions), Some(file)) =
            (parts.next(), parts.next(), parts.next())
        else {
            continue;
        };

        // Arquivo binário vem como `-`; vira `None` em vez de zero, que seria
        // uma afirmação falsa sobre o conteúdo.
        totals.insert(
            file.trim().to_string(),
            (additions.trim().parse().ok(), deletions.trim().parse().ok()),
        );
    }

    let mut files = Vec::new();

    for line in statuses.lines() {
        let mut parts = line.split('\t');
        let Some(status) = parts.next() else {
            continue;
        };

        // Renomeação vem como `R100\tantigo\tnovo`: o caminho relevante é o
        // último campo.
        let rest: Vec<&str> = parts.collect();
        let Some(file) = rest.last().map(|value| value.trim()) else {
            continue;
        };

        if file.is_empty() {
            continue;
        }

        let (additions, deletions) = totals.get(file).copied().unwrap_or((None, None));

        files.push(GitComparisonFile {
            path: file.to_string(),
            status: status.trim().to_string(),
            additions,
            deletions,
        });

        if files.len() >= COMPARISON_FILE_LIMIT {
            break;
        }
    }

    files
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn git(dir: &Path, args: &[&str]) {
        git_cli::run(dir, args).unwrap_or_else(|error| panic!("git {args:?}: {error}"));
    }

    fn commit(dir: &Path, file: &str, content: &str, message: &str) {
        fs::write(dir.join(file), content).expect("escrever");
        git(dir, &["add", "."]);
        git(
            dir,
            &[
                "-c",
                "user.name=Lab",
                "-c",
                "user.email=lab@example.invalid",
                "commit",
                "-q",
                "-m",
                message,
            ],
        );
    }

    fn lab(nome: &str) -> PathBuf {
        let base = std::env::temp_dir().join(format!("dw_compare_{nome}"));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).expect("criar laboratório");

        git(&base, &["init", "-q", "-b", "main"]);
        commit(&base, "a.txt", "1\n", "inicial");

        base
    }

    #[test]
    fn compara_branches_divergentes() {
        let path = lab("divergentes");

        git(&path, &["checkout", "-q", "-b", "feat"]);
        commit(&path, "b.txt", "novo\n", "feat 1");
        commit(&path, "c.txt", "outro\n", "feat 2");

        git(&path, &["checkout", "-q", "main"]);
        commit(&path, "a.txt", "1\n2\n", "main 1");

        let result = compare(&path, "main", "feat").expect("comparar");

        assert!(result.blocked.is_none());
        assert_eq!(result.ahead, 2);
        assert_eq!(result.behind, 1);
        assert!(result.merge_base.is_some());
        assert_eq!(result.commits.len(), 2);

        let files: Vec<&str> = result.files.iter().map(|file| file.path.as_str()).collect();
        assert!(files.contains(&"b.txt"));
        assert!(files.contains(&"c.txt"));
        // `a.txt` mudou só no destino: a semântica de três pontos o exclui.
        assert!(!files.contains(&"a.txt"));
    }

    #[test]
    fn mesma_referencia_e_bloqueada() {
        let path = lab("mesma");
        let result = compare(&path, "main", "main").expect("comparar");

        assert!(result.blocked.is_some());
        assert!(result.commits.is_empty());
    }

    #[test]
    fn referencia_inexistente_e_recusada() {
        let path = lab("inexistente");
        let error = compare(&path, "main", "nao-existe").unwrap_err();

        assert_eq!(error.code, codes::INVALID_COMMIT);
    }

    /// Segunda camada da defesa: o valor nunca chega ao `rev-parse`.
    #[test]
    fn referencia_iniciada_por_hifen_e_recusada_antes_do_processo() {
        let path = lab("operando");
        let error = compare(&path, "main", "--output=/tmp/invadido").unwrap_err();

        assert_eq!(error.code, "ARGUMENT_REJECTED");
        assert!(!Path::new("/tmp/invadido").exists());
    }
}
