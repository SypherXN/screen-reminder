use std::sync::Arc;

use chrono::{Duration, Utc};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_opener::OpenerExt;

use crate::calendar::{apple, caldav, google, google_tasks, microsoft_todo, outlook};
use crate::models::{
    AccountStyleOverrides, AppSettings, CalendarAccount, CaldavConnectRequest, CompositionPreset,
    MonitorInfo, OverlayPayload, PlatformInfo, ReminderEvent, SyncStatus,
};
use crate::overlay_window::list_monitors as fetch_monitors;
use crate::scheduler::{hide_all_overlays, SchedulerState};
use crate::storage::Storage;

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    state.storage.get_settings().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_settings(state: State<'_, AppState>, settings: AppSettings) -> Result<(), String> {
    state
        .storage
        .save_settings(&settings)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_accounts(state: State<'_, AppState>) -> Result<Vec<CalendarAccount>, String> {
    state.storage.list_accounts().map_err(|e| e.to_string())
}

async fn connect_and_sync(state: &AppState, app: &AppHandle, account: CalendarAccount) -> Result<CalendarAccount, String> {
    if let Some(existing) = find_duplicate_account(state, &account)? {
        cleanup_new_account_secrets(&account);
        let identity = account
            .email
            .as_deref()
            .or(account.caldav_username.as_deref())
            .unwrap_or("This account");
        return Err(format!(
            "{identity} is already connected as \"{}\".",
            existing.display_name
        ));
    }

    state
        .storage
        .upsert_account(&account)
        .map_err(|e| e.to_string())?;

    let app_for_sync = app.clone();
    let scheduler = state.scheduler.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(err) = scheduler.run_sync(&app_for_sync).await {
            log::error!("post-connect sync failed: {err}");
        }
    });

    show_settings_window(app)?;
    Ok(account)
}

fn show_settings_window(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn find_duplicate_account(
    state: &AppState,
    account: &CalendarAccount,
) -> Result<Option<CalendarAccount>, String> {
    if account.source == "caldav" {
        let url = account.caldav_url.as_deref().unwrap_or("");
        let username = account.caldav_username.as_deref().unwrap_or("");
        if url.is_empty() || username.is_empty() {
            return Ok(None);
        }
        return state
            .storage
            .find_caldav_account(url, username)
            .map_err(|e| e.to_string());
    }

    if let Some(email) = account.email.as_deref().filter(|value| !value.is_empty()) {
        return state
            .storage
            .find_account_by_source_and_email(&account.source, email)
            .map_err(|e| e.to_string());
    }

    Ok(None)
}

fn cleanup_new_account_secrets(account: &CalendarAccount) {
    match account.source.as_str() {
        "google" | "google_tasks" | "outlook" | "microsoft_todo" => {
            let _ = crate::secrets::delete_tokens(&account.id);
        }
        "caldav" => {
            let _ = crate::secrets::delete_password(&account.id);
        }
        _ => {}
    }
}

fn uses_oauth_tokens(source: &str) -> bool {
    matches!(
        source,
        "google" | "google_tasks" | "outlook" | "microsoft_todo"
    )
}

#[tauri::command]
pub async fn connect_google(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<CalendarAccount, String> {
    let account = google::GoogleProvider::connect()
        .await
        .map_err(|e| e.to_string())?;
    connect_and_sync(&state, &app, account).await
}

#[tauri::command]
pub async fn connect_outlook(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<CalendarAccount, String> {
    let account = outlook::OutlookProvider::connect()
        .await
        .map_err(|e| e.to_string())?;
    connect_and_sync(&state, &app, account).await
}

#[tauri::command]
pub async fn connect_caldav(
    app: AppHandle,
    state: State<'_, AppState>,
    request: CaldavConnectRequest,
) -> Result<CalendarAccount, String> {
    let account = caldav::CaldavProvider::connect(request).map_err(|e| e.to_string())?;
    connect_and_sync(&state, &app, account).await
}

#[tauri::command]
pub async fn connect_apple(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<CalendarAccount, String> {
    let account = apple::AppleProvider::connect().map_err(|e| e.to_string())?;
    connect_and_sync(&state, &app, account).await
}

#[tauri::command]
pub async fn connect_google_tasks(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<CalendarAccount, String> {
    let account = google_tasks::GoogleTasksProvider::connect()
        .await
        .map_err(|e| e.to_string())?;
    connect_and_sync(&state, &app, account).await
}

#[tauri::command]
pub async fn connect_microsoft_todo(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<CalendarAccount, String> {
    let account = microsoft_todo::MicrosoftTodoProvider::connect()
        .await
        .map_err(|e| e.to_string())?;
    connect_and_sync(&state, &app, account).await
}

#[tauri::command]
pub async fn disconnect_account(state: State<'_, AppState>, account_id: String) -> Result<(), String> {
    if let Some(account) = state
        .storage
        .get_account(&account_id)
        .map_err(|e| e.to_string())?
    {
        match account.source.as_str() {
            source if uses_oauth_tokens(source) => {
                let _ = crate::secrets::delete_tokens(&account_id);
            }
            "caldav" => {
                let _ = crate::secrets::delete_password(&account_id);
            }
            _ => {}
        }
    }
    state
        .storage
        .delete_account_sync_status(&account_id)
        .map_err(|e| e.to_string())?;
    state
        .storage
        .delete_account(&account_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sync_now(app: AppHandle, state: State<'_, AppState>) -> Result<usize, String> {
    state
        .scheduler
        .run_sync(&app)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_upcoming_reminders(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<ReminderEvent>, String> {
    state
        .storage
        .list_upcoming_reminders(Utc::now(), limit.unwrap_or(200))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_sync_status(state: State<'_, AppState>) -> Result<SyncStatus, String> {
    let last_sync = state
        .storage
        .get_sync_meta("last_sync")
        .map_err(|e| e.to_string())?
        .and_then(|s| s.parse().ok());
    Ok(SyncStatus {
        last_sync,
        reminder_count: state.storage.reminder_count().map_err(|e| e.to_string())?,
        account_count: state.storage.list_accounts().map_err(|e| e.to_string())?.len(),
        accounts: state
            .storage
            .list_account_sync_statuses()
            .map_err(|e| e.to_string())?,
    })
}

#[tauri::command]
pub fn list_monitors(app: AppHandle) -> Result<Vec<MonitorInfo>, String> {
    fetch_monitors(&app).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_autostart(app: AppHandle) -> Result<bool, String> {
    app.autolaunch()
        .is_enabled()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_autostart(app: AppHandle, state: State<'_, AppState>, enabled: bool) -> Result<(), String> {
    if enabled {
        app.autolaunch().enable().map_err(|e| e.to_string())?;
    } else {
        app.autolaunch().disable().map_err(|e| e.to_string())?;
    }

    let mut settings = state.storage.get_settings().map_err(|e| e.to_string())?;
    settings.launch_at_login = enabled;
    state
        .storage
        .save_settings(&settings)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_platform_info() -> PlatformInfo {
    PlatformInfo {
        os: std::env::consts::OS.to_string(),
        apple_calendar_available: cfg!(target_os = "macos"),
    }
}

#[tauri::command]
pub async fn dismiss_reminder(
    app: AppHandle,
    state: State<'_, AppState>,
    reminder_id: String,
) -> Result<(), String> {
    state
        .storage
        .dismiss_reminder(&reminder_id)
        .map_err(|e| e.to_string())?;
    state.scheduler.clear_active(&reminder_id);
    hide_all_overlays(&app).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn snooze_reminder(
    app: AppHandle,
    state: State<'_, AppState>,
    reminder_id: String,
    minutes: u32,
) -> Result<(), String> {
    let until = Utc::now() + Duration::minutes(minutes as i64);
    state
        .storage
        .snooze_reminder(&reminder_id, until)
        .map_err(|e| e.to_string())?;
    state.scheduler.clear_active(&reminder_id);
    hide_all_overlays(&app).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn snooze_reminder_until_start(
    app: AppHandle,
    state: State<'_, AppState>,
    reminder_id: String,
) -> Result<(), String> {
    let reminder = state
        .storage
        .get_reminder(&reminder_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "reminder not found".to_string())?;
    state
        .storage
        .snooze_reminder(&reminder_id, reminder.start_time)
        .map_err(|e| e.to_string())?;
    state.scheduler.clear_active(&reminder_id);
    hide_all_overlays(&app).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_reminder_url(app: AppHandle, url: String) -> Result<(), String> {
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn hide_reminder_overlay(app: AppHandle) -> Result<(), String> {
    hide_all_overlays(&app).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn show_settings(app: AppHandle) -> Result<(), String> {
    show_settings_window(&app)
}

#[tauri::command]
pub async fn preview_overlay(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let settings = state.storage.get_settings().map_err(|e| e.to_string())?;
    let payload = OverlayPayload {
        reminder_id: "preview".to_string(),
        account_id: "preview".to_string(),
        source: "preview".to_string(),
        title: "Preview reminder".to_string(),
        location: Some("Sample location".to_string()),
        url: None,
        start_time: Utc::now() + Duration::minutes(15),
        settings: settings.clone(),
        effective_font_color: None,
        play_sound: false,
        monitor_x: 0,
        monitor_y: 0,
        monitor_width: 0,
        monitor_height: 0,
        monitor_scale_factor: 1.0,
    };

    crate::scheduler::show_overlay_for_preview(&app, &settings, payload)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_test_reminder(state: State<'_, AppState>) -> Result<(), String> {
    use crate::models::ReminderEvent;
    use crate::storage::new_id;

    let settings = state.storage.get_settings().map_err(|e| e.to_string())?;
    if settings.reminders_paused {
        return Err("Reminders are paused".to_string());
    }

    let reminder = ReminderEvent {
        id: new_id(),
        account_id: "test".to_string(),
        source: "test".to_string(),
        external_id: new_id(),
        title: "Test reminder — team standup".to_string(),
        start_time: Utc::now() + Duration::minutes(5),
        reminder_time: Utc::now(),
        location: Some("Zoom".to_string()),
        url: Some("https://calendar.google.com".to_string()),
        fired_at: None,
        snoozed_until: None,
        dismissed: false,
    };
    state
        .storage
        .upsert_reminders(&[reminder])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn set_reminders_paused(
    app: AppHandle,
    state: State<'_, AppState>,
    paused: bool,
) -> Result<(), String> {
    crate::tray::set_reminders_paused(&app, &state.storage, paused).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_composition_presets(state: State<'_, AppState>) -> Result<Vec<CompositionPreset>, String> {
    state
        .storage
        .list_composition_presets()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_composition_preset(
    state: State<'_, AppState>,
    name: String,
) -> Result<CompositionPreset, String> {
    let settings = state.storage.get_settings().map_err(|e| e.to_string())?;
    state
        .storage
        .save_composition_preset(&name, &settings.composition)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn load_composition_preset(
    state: State<'_, AppState>,
    preset_id: String,
) -> Result<AppSettings, String> {
    let preset = state
        .storage
        .get_composition_preset(&preset_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "preset not found".to_string())?;
    let mut settings = state.storage.get_settings().map_err(|e| e.to_string())?;
    settings.composition = preset.composition;
    settings.ensure_composition();
    state
        .storage
        .save_settings(&settings)
        .map_err(|e| e.to_string())?;
    Ok(settings)
}

#[tauri::command]
pub fn delete_composition_preset(state: State<'_, AppState>, preset_id: String) -> Result<(), String> {
    state
        .storage
        .delete_composition_preset(&preset_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_account_style(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<Option<AccountStyleOverrides>, String> {
    state
        .storage
        .get_account_style(&account_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_account_style(
    state: State<'_, AppState>,
    account_id: String,
    style: AccountStyleOverrides,
) -> Result<(), String> {
    state
        .storage
        .save_account_style(&account_id, &style)
        .map_err(|e| e.to_string())
}

pub struct AppState {
    pub storage: Arc<Storage>,
    pub scheduler: Arc<SchedulerState>,
}
