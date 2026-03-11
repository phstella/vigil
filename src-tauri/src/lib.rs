pub mod commands;
pub mod core;
pub mod events;
pub mod models;
pub mod state;

use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .manage(state::AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::workspace::open_workspace,
            commands::fs::list_dir,
            commands::fs::read_file,
            commands::fs::write_file,
            commands::fs::create_note,
            commands::fs::rename_file,
            commands::fs::delete_file,
            commands::git::get_git_hunks,
            commands::git::get_git_status,
            commands::search::fuzzy_find,
            commands::search::search_content,
            commands::search::get_all_tags,
            commands::search::get_files_by_tag,
            commands::links::get_backlinks,
        ])
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Vigil");
}
