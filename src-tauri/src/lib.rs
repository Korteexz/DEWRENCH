//! Ponto de entrada do backend.
//!
//! `core` vem antes de `modules` de propósito, inclusive na leitura: módulos
//! descrevem intenção, o Core decide autoridade e executa as fronteiras.

// `pub` para que os testes de integração em `tests/` alcancem o domínio.
pub mod core;
pub mod modules;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            modules::git::commands::open_project,
            modules::git::commands::create_repository,
            modules::git::commands::get_repository_details,
            modules::git::commands::get_repository_graph,
            modules::git::commands::create_commit,
            modules::git::commands::get_commit_diff,
            modules::git::commands::stage_file,
            modules::git::commands::stage_all,
            modules::git::commands::unstage_file,
            modules::git::commands::create_branch_from,
            modules::git::commands::switch_branch,
            modules::git::commands::get_revert_preview,
            modules::git::commands::revert_commit,

            modules::git::commands::get_remotes,
            modules::git::commands::add_remote,
            modules::git::commands::remove_remote,
            modules::git::commands::rename_remote,
            modules::git::commands::set_remote_url,

            modules::git::commands::get_push_plan,
            modules::git::commands::push_branch,
            modules::git::commands::fetch_remote,
            modules::git::commands::get_pull_plan,
            modules::git::commands::pull_branch,

            modules::git::commands::get_branch_comparison,
            modules::git::commands::get_comparison_diff,

            modules::github::commands::get_github_context,
            modules::github::commands::list_pull_requests,
            modules::github::commands::create_pull_request,
            modules::github::commands::open_github_in_browser,
            modules::github::commands::get_pull_request,
            modules::github::commands::get_pull_request_diff,
            modules::github::commands::get_pull_request_plan,
            modules::github::commands::merge_pull_request,
            modules::github::commands::close_pull_request,

            modules::activity::commands::get_activity_stream,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
