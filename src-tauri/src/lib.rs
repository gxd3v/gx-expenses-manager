mod commands;
pub mod database;
pub mod errors;
pub mod graphql;
mod managers;
mod models;
mod module;
mod repositories;
mod transformers;

use commands::Session;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Session::default())
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
            commands::status,
            commands::unlock,
            commands::lock,
            commands::graphql,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
