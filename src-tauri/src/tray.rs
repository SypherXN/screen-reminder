use anyhow::Result;
use tauri::{AppHandle, Emitter, Manager, Wry};
use tauri::menu::MenuItem;

use crate::storage::Storage;

pub struct TrayMenuState {
    pub pause_item: MenuItem<Wry>,
}

pub fn pause_menu_label(paused: bool) -> &'static str {
    if paused {
        "Resume reminders"
    } else {
        "Pause reminders"
    }
}

pub fn sync_pause_menu(app: &AppHandle, paused: bool) {
    if let Some(state) = app.try_state::<TrayMenuState>() {
        let _ = state
            .pause_item
            .set_text(pause_menu_label(paused));
    }
}

pub fn set_reminders_paused(app: &AppHandle, storage: &Storage, paused: bool) -> Result<()> {
    let mut settings = storage.get_settings()?;
    settings.reminders_paused = paused;
    storage.save_settings(&settings)?;
    sync_pause_menu(app, paused);
    let _ = app.emit("reminders-paused-changed", paused);
    Ok(())
}

pub fn init_pause_menu_from_storage(app: &AppHandle, storage: &Storage) {
    if let Ok(settings) = storage.get_settings() {
        sync_pause_menu(app, settings.reminders_paused);
    }
}
