mod modules;

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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
