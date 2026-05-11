mod commands;

use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::detect_languages,
            commands::get_server_status,
            commands::install_servers,
            commands::install_one_server,
            commands::remove_one_server,
            commands::get_config,
            commands::save_config,
            commands::clean_cache,
            commands::check_updates,
            commands::reveal_in_explorer,
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
        .expect("error while running tauri application");
}
