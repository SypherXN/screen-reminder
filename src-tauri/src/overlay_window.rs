use anyhow::{Context, Result};
use tauri::{AppHandle, Manager, Monitor, PhysicalPosition, PhysicalSize, WebviewWindow};

use crate::models::AppSettings;

pub struct MonitorBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub scale_factor: f64,
}

pub fn monitor_bounds(monitor: &Monitor) -> MonitorBounds {
    let position = monitor.position();
    let size = monitor.size();
    MonitorBounds {
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
        scale_factor: monitor.scale_factor(),
    }
}

pub fn list_monitors(app: &AppHandle) -> Result<Vec<crate::models::MonitorInfo>> {
    let monitors = app.available_monitors().context("list monitors")?;
    let primary = app.primary_monitor().ok().flatten();

    Ok(monitors
        .into_iter()
        .enumerate()
        .map(|(index, monitor)| {
            let name = monitor.name().cloned().unwrap_or_else(|| format!("Monitor {index}"));
            let size = monitor.size();
            let is_primary = primary
                .as_ref()
                .map(|p| p.name() == monitor.name())
                .unwrap_or(index == 0);
            crate::models::MonitorInfo {
                index,
                name,
                width: size.width,
                height: size.height,
                is_primary,
            }
        })
        .collect())
}

pub fn resolve_target_monitors(app: &AppHandle, settings: &AppSettings) -> Result<Vec<Monitor>> {
    let monitors = app.available_monitors().context("list monitors")?;
    if monitors.is_empty() {
        anyhow::bail!("no monitors available");
    }

    match settings.monitor_target.as_str() {
        "all" => Ok(monitors),
        "active" => {
            if let Some(window) = app.get_webview_window("main") {
                if let Ok(Some(monitor)) = window.current_monitor() {
                    return Ok(vec![monitor]);
                }
            }
            if let Some(window) = app.get_webview_window("overlay") {
                if let Ok(Some(monitor)) = window.current_monitor() {
                    return Ok(vec![monitor]);
                }
            }
            app.primary_monitor()
                .context("primary monitor")?
                .map(|monitor| vec![monitor])
                .context("no primary monitor")
        }
        "primary" => app
            .primary_monitor()
            .context("primary monitor")?
            .map(|monitor| vec![monitor])
            .context("no primary monitor"),
        index => {
            let idx: usize = index.parse().context("invalid monitor index")?;
            monitors
                .into_iter()
                .nth(idx)
                .map(|monitor| vec![monitor])
                .context("monitor index out of range")
        }
    }
}

pub fn position_overlay_window(window: &WebviewWindow, monitor: &Monitor) -> Result<()> {
    let size = monitor.size();
    let position = monitor.position();
    window.set_size(PhysicalSize::new(size.width, size.height))?;
    window.set_position(PhysicalPosition::new(position.x, position.y))?;
    Ok(())
}
