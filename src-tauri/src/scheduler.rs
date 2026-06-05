use std::sync::Arc;

use anyhow::Result;
use chrono::Utc;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::contrast::sample_text_color_at;
use crate::models::{AppSettings, OverlayPayload};
use crate::overlay_window::{position_overlay_window, resolve_target_monitors};
use crate::quiet_hours::is_quiet_hours;
use crate::settings_merge::merge_account_style;
use crate::storage::Storage;
use crate::sync_engine::SyncEngine;

pub struct SchedulerState {
    pub storage: Arc<Storage>,
    pub sync_engine: Arc<SyncEngine>,
    active_overlays: std::sync::Mutex<std::collections::HashSet<String>>,
}

impl SchedulerState {
    pub fn new(storage: Arc<Storage>, sync_engine: Arc<SyncEngine>) -> Self {
        Self {
            storage,
            sync_engine,
            active_overlays: std::sync::Mutex::new(std::collections::HashSet::new()),
        }
    }

    pub async fn tick(&self, app: &AppHandle) -> Result<()> {
        let settings = self.storage.get_settings()?;
        if settings.reminders_paused {
            return Ok(());
        }

        if is_quiet_hours(&settings, Utc::now()) {
            return Ok(());
        }

        let now = Utc::now();
        let due = self
            .storage
            .due_reminders(now, settings.dedupe_reminders)?;

        for reminder in due {
            {
                let mut active = self.active_overlays.lock().unwrap();
                if active.contains(&reminder.id) {
                    continue;
                }
                active.insert(reminder.id.clone());
            }

            let mut effective_settings = settings.clone();
            if let Some(account) = self.storage.get_account(&reminder.account_id)? {
                if let Some(style) = account.style_overrides {
                    effective_settings = merge_account_style(&settings, &style);
                }
            }

            let mut effective_font_color = None;
            if effective_settings.auto_contrast_text {
                if let Ok(monitors) = resolve_target_monitors(app, &effective_settings) {
                    if let Some(monitor) = monitors.first() {
                        if let Ok(color) = sample_text_color_at(
                            monitor.position().x,
                            monitor.position().y,
                            monitor.size().width,
                            monitor.size().height,
                            0.25,
                            0.5,
                        ) {
                            effective_font_color = Some(color.clone());
                            effective_settings.font_color = color;
                        }
                    }
                }
            }

            let play_sound = effective_settings.sound_enabled && !settings.reminders_paused;

            let payload = OverlayPayload {
                reminder_id: reminder.id.clone(),
                account_id: reminder.account_id.clone(),
                source: reminder.source.clone(),
                title: reminder.title.clone(),
                location: reminder.location.clone(),
                url: reminder.url.clone(),
                start_time: reminder.start_time,
                settings: effective_settings,
                effective_font_color,
                play_sound,
            };

            if let Err(err) = show_overlay(app, &settings, payload).await {
                log::error!("failed to show overlay: {err}");
                self.active_overlays.lock().unwrap().remove(&reminder.id);
            } else {
                let _ = self.storage.mark_fired(&reminder.id, now);
            }
        }
        Ok(())
    }

    pub fn clear_active(&self, reminder_id: &str) {
        self.active_overlays.lock().unwrap().remove(reminder_id);
    }

    pub async fn run_sync(&self, app: &AppHandle) -> Result<usize> {
        let report = self.sync_engine.sync_all().await?;
        if let Err(err) = crate::push_sync::register_all(self.storage.clone()).await {
            log::debug!("push registration skipped: {err}");
        }
        let _ = app.emit("sync-complete", ());
        Ok(report.reminders_upserted)
    }
}

pub async fn show_overlay_for_preview(
    app: &AppHandle,
    settings: &AppSettings,
    payload: OverlayPayload,
) -> Result<()> {
    show_overlay(app, settings, payload).await
}

async fn show_overlay(
    app: &AppHandle,
    settings: &AppSettings,
    payload: OverlayPayload,
) -> Result<()> {
    let monitors = resolve_target_monitors(app, settings)?;

    for (index, monitor) in monitors.iter().enumerate() {
        let label = if monitors.len() == 1 {
            "overlay".to_string()
        } else {
            format!("overlay-{index}")
        };

        let window = if let Some(existing) = app.get_webview_window(&label) {
            existing
        } else {
            let url = WebviewUrl::App("overlay.html".into());
            WebviewWindowBuilder::new(app, &label, url)
                .title("Screen Reminder Overlay")
                .decorations(false)
                .transparent(true)
                .always_on_top(true)
                .skip_taskbar(true)
                .focused(false)
                .resizable(false)
                .build()?
        };

        position_overlay_window(&window, monitor)?;
        window.show()?;
        if monitors.len() == 1 {
            window.set_focus()?;
        }
        window.emit("show-reminder", &payload)?;
    }

    Ok(())
}

pub fn spawn_scheduler(app: AppHandle, state: Arc<SchedulerState>) {
    tauri::async_runtime::spawn(async move {
        loop {
            if let Err(err) = state.tick(&app).await {
                log::error!("scheduler tick failed: {err}");
            }
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        }
    });
}

fn sync_interval_seconds(storage: &Storage) -> u64 {
    match storage.nearest_reminder_minutes() {
        Ok(Some(minutes)) if minutes <= 30 => 60,
        _ => 300,
    }
}

pub fn spawn_sync_loop(app: AppHandle, state: Arc<SchedulerState>) {
    tauri::async_runtime::spawn(async move {
        if let Err(err) = state.run_sync(&app).await {
            log::error!("initial sync failed: {err}");
        }

        loop {
            let interval = sync_interval_seconds(&state.storage);
            tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
            if let Err(err) = state.run_sync(&app).await {
                log::error!("sync failed: {err}");
            }
        }
    });
}

pub fn spawn_push_listener(app: AppHandle, state: Arc<SchedulerState>) {
    crate::push_sync::spawn_push_poll_loop(state.storage.clone(), move || {
        let app = app.clone();
        let state = state.clone();
        tauri::async_runtime::spawn(async move {
            if let Err(err) = state.run_sync(&app).await {
                log::error!("push-triggered sync failed: {err}");
            }
        });
    });
}

pub async fn hide_all_overlays(app: &AppHandle) -> Result<()> {
    if let Some(window) = app.get_webview_window("overlay") {
        window.hide()?;
    }
    for index in 0..8 {
        if let Some(window) = app.get_webview_window(&format!("overlay-{index}")) {
            window.hide()?;
        }
    }
    Ok(())
}
