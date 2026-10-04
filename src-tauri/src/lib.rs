#[macro_use]
mod macros;

mod commands;
pub mod database;
pub mod errors;
pub mod graphql;
mod managers;
mod models;
pub mod module;
mod repositories;
mod transformers;

#[cfg(test)]
mod tests;

use commands::Session;

const MAX_LOG_BYTES: u128 = 1_000_000;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Session::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log_level())
                .max_file_size(MAX_LOG_BYTES)
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            commands::status,
            commands::unlock,
            commands::lock,
            commands::graphql,
            commands::change_password,
            commands::restore_backup,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn log_level() -> log::LevelFilter {
    if cfg!(debug_assertions) {
        log::LevelFilter::Info
    } else {
        log::LevelFilter::Warn
    }
}
