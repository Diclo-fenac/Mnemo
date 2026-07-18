mod commands;
mod models;
mod services;
mod state;

use std::sync::{Arc, Mutex};

use tauri::{Emitter, Manager, PhysicalPosition};

use crate::{services::db, state::AppState};

use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        let popup_shortcut =
                            "CommandOrControl+Shift+V".parse::<Shortcut>().unwrap();
                        let capture_shortcut =
                            "CommandOrControl+Shift+M".parse::<Shortcut>().unwrap();
                        if shortcut == &popup_shortcut {
                            if let Some(popup) = app.get_webview_window("popup") {
                                if popup.is_visible().unwrap_or(false) {
                                    let _ = popup.hide();
                                } else {
                                    position_popup_at_cursor(app, &popup);
                                    let _ = popup.show();
                                    let _ = popup.set_focus();
                                }
                            }
                        } else if shortcut == &capture_shortcut {
                            if let Some(state) = app.try_state::<AppState>() {
                                let enabled =
                                    !services::capture_state::is_enabled(&state.capture_enabled);
                                let persisted = state
                                    .db
                                    .lock()
                                    .ok()
                                    .and_then(|conn| {
                                        services::capture_state::set_capture_enabled(&conn, enabled)
                                            .ok()
                                    })
                                    .is_some();
                                if persisted {
                                    services::capture_state::set_enabled(
                                        &state.capture_enabled,
                                        enabled,
                                    );
                                    let _ = app.emit("capture-state-changed", enabled);
                                }
                            }
                        }
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            let db = db::init_db(&app_data_dir).map_err(|error| {
                eprintln!("Mnemo database initialization failed: {error}");
                error
            })?;

            let model_cache_dir = app_data_dir.join("mnemo").join("models");
            std::fs::create_dir_all(&model_cache_dir)?;
            let db_arc = Arc::new(Mutex::new(db));

            let preferences = db_arc
                .lock()
                .ok()
                .and_then(|conn| services::capture_state::load(&conn).ok())
                .unwrap_or_default();

            let app_state = AppState::new(
                db_arc.clone(),
                model_cache_dir,
                preferences.capture_enabled,
                preferences.browser_context_enabled,
            );

            // Start HTTP context server
            services::http_server::start_server(
                app_state.browser_context.clone(),
                app_state.browser_context_enabled.clone(),
            );

            // Start clipboard watcher
            services::watcher::start(
                app.handle().clone(),
                Arc::clone(&db_arc),
                app_state.browser_context.clone(),
                app_state.capture_enabled.clone(),
                app_state.browser_context_enabled.clone(),
            );

            // Model download waits until onboarding has completed.
            if preferences.onboarding_completed {
                services::embedder::start_embedder(
                    Arc::clone(&db_arc),
                    app_state.embedder.clone(),
                    app_state.embedding_status.clone(),
                    app_state.model_cache_dir.clone(),
                    app_state.model_start_requested.clone(),
                );
            }
            services::reranker::start();

            let popup_shortcut = "CommandOrControl+Shift+V".parse::<Shortcut>().unwrap();
            let _ = app.global_shortcut().register(popup_shortcut);
            let capture_shortcut = "CommandOrControl+Shift+M".parse::<Shortcut>().unwrap();
            let _ = app.global_shortcut().register(capture_shortcut);

            let _tray = tauri::tray::TrayIconBuilder::new()
                .tooltip("Mnemo")
                .icon(tauri::include_image!("icons/tray-icon.png"))
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        button_state: tauri::tray::MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(main_win) = app.get_webview_window("main") {
                            let _ = main_win.show();
                            let _ = main_win.set_focus();
                        }
                    }
                })
                .build(app)?;

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
            commands::memory::list_sessions,
            commands::memory::get_session_reconstruction,
            commands::memory::get_clip_context,
            commands::memory::get_related_clips,
            commands::search::hybrid_search,
            commands::graph::get_graph_data,
            commands::models::get_supported_embedding_models,
            commands::models::get_active_embedding_model,
            commands::models::switch_embedding_model,
            commands::quality::get_quality_metrics,
            commands::quality::log_search_feedback,
            commands::quality::log_copy_again,
            commands::settings::clear_database,
            commands::settings::get_capture_preferences,
            commands::settings::update_capture_preferences,
            commands::settings::complete_onboarding,
            commands::settings::retry_embedding_model,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Mnemo");
}

fn position_popup_at_cursor<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    popup: &tauri::WebviewWindow<R>,
) {
    let Ok(cursor) = app.cursor_position() else {
        return;
    };
    let Ok(Some(monitor)) = app.monitor_from_point(cursor.x, cursor.y) else {
        return;
    };
    let size = monitor.size();
    let origin = monitor.position();
    let popup_size = popup
        .outer_size()
        .unwrap_or(tauri::PhysicalSize::new(600, 400));
    let margin = 16_i32;
    let max_x = origin.x + size.width as i32 - popup_size.width as i32 - margin;
    let max_y = origin.y + size.height as i32 - popup_size.height as i32 - margin;
    let x = ((cursor.x as i32) + margin).clamp(origin.x + margin, max_x.max(origin.x + margin));
    let y = ((cursor.y as i32) + margin).clamp(origin.y + margin, max_y.max(origin.y + margin));
    let _ = popup.set_position(PhysicalPosition::new(x, y));
}
