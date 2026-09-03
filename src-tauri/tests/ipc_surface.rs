//! Guarda da superfície IPC.
//!
//! Um `#[tauri::command]` que existe mas não está em `generate_handler!`
//! compila, passa no `cargo check` e só falha em runtime, com
//! "Command not found" — foi exatamente assim que `get_activity_stream`
//! chegou ao app quebrado. Este teste compara as duas listas.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

fn source_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Nomes de função marcados com `#[tauri::command]` em toda a árvore.
fn declared_commands(dir: &Path, found: &mut BTreeSet<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            declared_commands(&path, found);
            continue;
        }

        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }

        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };

        let mut lines = content.lines().peekable();
        while let Some(line) = lines.next() {
            if line.trim() != "#[tauri::command]" {
                continue;
            }

            // A assinatura pode estar na linha seguinte ou algumas abaixo.
            for candidate in lines.clone().take(4) {
                if let Some(rest) = candidate.trim().strip_prefix("pub fn ") {
                    if let Some(name) = rest.split('(').next() {
                        found.insert(name.trim().to_string());
                    }
                    break;
                }
            }
        }
    }
}

/// Nomes citados dentro de `generate_handler![ ... ]`.
fn registered_commands() -> BTreeSet<String> {
    let lib = fs::read_to_string(source_root().join("lib.rs")).expect("lib.rs legível");
    let start = lib.find("generate_handler![").expect("generate_handler! presente");
    let end = lib[start..].find("])").expect("fim do generate_handler!") + start;

    lib[start..end]
        .lines()
        .filter_map(|line| {
            let line = line.trim().trim_end_matches(',');
            line.rsplit("::").next().filter(|name| {
                !name.is_empty()
                    && !name.contains('!')
                    && !name.contains('[')
                    && name
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_')
            })
        })
        .map(str::to_string)
        .collect()
}

#[test]
fn todo_command_declarado_esta_registrado() {
    let mut declared = BTreeSet::new();
    declared_commands(&source_root(), &mut declared);
    let registered = registered_commands();

    assert!(!declared.is_empty(), "nenhum #[tauri::command] encontrado");

    let missing: Vec<&String> = declared.difference(&registered).collect();
    assert!(
        missing.is_empty(),
        "commands declarados e NÃO registrados em generate_handler!: {missing:?}"
    );
}

#[test]
fn todo_command_registrado_existe() {
    let mut declared = BTreeSet::new();
    declared_commands(&source_root(), &mut declared);
    let registered = registered_commands();

    let unknown: Vec<&String> = registered.difference(&declared).collect();
    assert!(
        unknown.is_empty(),
        "registrados em generate_handler! sem #[tauri::command]: {unknown:?}"
    );
}
