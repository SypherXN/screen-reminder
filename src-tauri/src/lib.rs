mod calendar;
mod commands;
mod contrast;
mod dedupe;
mod models;
mod oauth_util;
mod overlay_window;
mod push_sync;
mod quiet_hours;
mod scheduler;
mod secrets;
mod settings_merge;
mod storage;
mod sync_engine;
mod tray;

use std::sync::Arc;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, RunEvent,
};
use tauri_plugin_autostart::MacosLauncher;

#[cfg(target_os = "windows")]
use tauri_plugin_autostart::WindowsLauncher;

use commands::AppState;
use scheduler::{spawn_push_listener, spawn_scheduler, spawn_sync_loop, SchedulerState};
use storage::Storage;
use sync_engine::SyncEngine;
use tray::{init_pause_menu_from_storage, TrayMenuState};

fn build_app() -> tauri::Builder<tauri::Wry> {
    let mut builder = tauri::Builder::default();
    #[cfg(target_os = "macos")]
    {
        builder = builder.plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]),
        ));
    }
    #[cfg(target_os = "windows")]
    {
        builder = builder.plugin(tauri_plugin_autostart::init(
            WindowsLauncher::AppName,
            Some(vec![]),
        ));
    }
    builder
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    let storage = Arc::new(Storage::new().expect("initialize storage"));
    let sync_engine = Arc::new(SyncEngine::new(storage.clone()));
    let scheduler = Arc::new(SchedulerState::new(storage.clone(), sync_engine.clone()));
    let app_state = AppState {
        storage: storage.clone(),
        scheduler: scheduler.clone(),
    };

    build_app()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings,
            commands::list_accounts,
            commands::connect_google,
            commands::connect_outlook,
            commands::connect_caldav,
            commands::connect_apple,
            commands::connect_google_tasks,
            commands::connect_microsoft_todo,
            commands::disconnect_account,
            commands::sync_now,
            commands::get_sync_status,
            commands::list_monitors,
            commands::get_autostart,
            commands::set_autostart,
            commands::get_platform_info,
            commands::dismiss_reminder,
            commands::snooze_reminder,
            commands::snooze_reminder_until_start,
            commands::open_reminder_url,
            commands::hide_reminder_overlay,
            commands::show_settings,
            commands::preview_overlay,
            commands::create_test_reminder,
            commands::set_reminders_paused,
            commands::list_composition_presets,
            commands::save_composition_preset,
            commands::load_composition_preset,
            commands::delete_composition_preset,
            commands::get_account_style,
            commands::save_account_style,
        ])
        .setup(move |app| {
            load_env_file();

            let settings_item =
                MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let pause_item = MenuItem::with_id(app, "pause", "Pause reminders", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&settings_item, &pause_item, &quit_item])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .tooltip("Screen Reminder")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "settings" => {
                        let _ = app.get_webview_window("main").map(|w| {
                            let _ = w.show();
                            let _ = w.set_focus();
                        });
                    }
                    "pause" => {
                        if let Some(state) = app.try_state::<AppState>() {
                            let paused = state
                                .storage
                                .get_settings()
                                .map(|s| !s.reminders_paused)
                                .unwrap_or(false);
                            let _ = tray::set_reminders_paused(app, &state.storage, paused);
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        let _ = app.get_webview_window("main").map(|w| {
                            let _ = w.show();
                            let _ = w.set_focus();
                        });
                    }
                })
                .build(app)?;

            app.manage(TrayMenuState {
                pause_item: pause_item.clone(),
            });
            init_pause_menu_from_storage(app.handle(), &storage);

            let app_handle = app.handle().clone();
            spawn_scheduler(app_handle.clone(), scheduler.clone());
            spawn_sync_loop(app_handle.clone(), scheduler.clone());
            spawn_push_listener(app_handle, scheduler);

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let RunEvent::Resumed = event {
                if let Some(state) = app.try_state::<AppState>() {
                    let sync = state.scheduler.clone();
                    let handle = app.handle().clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = sync.run_sync(&handle).await;
                    });
                }
            }
        });
}

fn load_env_file() {
    for path in [".env", "../.env"] {
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((key, value)) = line.split_once('=') {
                    std::env::set_var(key.trim(), value.trim().trim_matches('"'));
                }
            }
            break;
        }
    }
}
