mod commands;
mod models;
mod services;
mod state;

use std::sync::{Arc, Mutex};

use tauri::Manager;

use crate::{services::db, state::AppState};

use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().with_handler(|app, shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                let popup_shortcut = "CommandOrControl+Shift+V".parse::<Shortcut>().unwrap();
                if shortcut == &popup_shortcut {
                    if let Some(popup) = app.get_webview_window("popup") {
                        if popup.is_visible().unwrap_or(false) {
                            let _ = popup.hide();
                        } else {
                            let _ = popup.show();
                            let _ = popup.set_focus();
                        }
                    }
                }
            }
        }).build())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            let db = db::init_db(&app_data_dir).map_err(|error| {
                eprintln!("Mnemo database initialization failed: {error}");
                error
            })?;

            let db_arc = Arc::new(Mutex::new(db));

            let app_state = AppState::new(db_arc.clone());

            // Start HTTP context server
            services::http_server::start_server(app_state.browser_context.clone());

            // Start clipboard watcher
            services::watcher::start(
                app.handle().clone(), 
                Arc::clone(&db_arc),
                app_state.browser_context.clone()
            );

            // Start background embedder
            services::embedder::start_embedder(Arc::clone(&db_arc), app_state.embedder.clone());

            let popup_shortcut = "CommandOrControl+Shift+V".parse::<Shortcut>().unwrap();
            let _ = app.global_shortcut().register(popup_shortcut);

            app.manage(app_state);
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
            commands::clips::get_session_clips,
            commands::search::hybrid_search,
            commands::graph::get_graph_data,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Mnemo");
}
