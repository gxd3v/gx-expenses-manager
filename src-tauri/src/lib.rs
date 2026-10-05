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
#[cfg(desktop)]
use tauri::Manager;

const MAX_LOG_BYTES: u128 = 1_000_000;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .manage(Session::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log_level())
                .max_file_size(MAX_LOG_BYTES)
                .build(),
        );

    #[cfg(desktop)]
    let builder = builder
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                bring_to_front(&window);
            }
            Ok(())
        });

    builder
        .invoke_handler(tauri::generate_handler![
            commands::status,
            commands::unlock,
            commands::lock,
            commands::graphql,
            commands::change_password,
            commands::restore_backup,
            commands::reset_data,
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

#[cfg(desktop)]
fn bring_to_front(window: &tauri::WebviewWindow) {
    let _ = window.unminimize();
    let _ = window.set_always_on_top(true);
    let _ = window.set_focus();
    let _ = window.set_always_on_top(false);
}
