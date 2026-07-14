mod commands;
mod models;
mod services;
mod state;

use std::sync::{Arc, Mutex};

use tauri::Manager;

use crate::{services::db, state::AppState};

pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            let db = db::init_db(&app_data_dir).map_err(|error| {
                eprintln!("Mnemo database initialization failed: {error}");
                error
            })?;

            let db_arc = Arc::new(Mutex::new(db));

            // Start clipboard watcher
            services::watcher::start(app.handle().clone(), Arc::clone(&db_arc));

            app.manage(AppState::new(db_arc));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::system::get_bootstrap_state,
            commands::system::healthcheck,
            commands::clips::list_clips,
            commands::clips::get_clip,
            commands::clips::delete_clip,
            commands::clips::toggle_pin,
            commands::clips::copy_clip,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Mnemo");
}
