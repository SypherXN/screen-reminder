use anyhow::Result;

pub fn sample_text_color(monitor_x: i32, monitor_y: i32, width: u32, height: u32) -> Result<String> {
    sample_text_color_at(monitor_x, monitor_y, width, height, 0.5, 0.5)
}

pub fn sample_text_color_at(
    monitor_x: i32,
    monitor_y: i32,
    width: u32,
    height: u32,
    x_ratio: f32,
    y_ratio: f32,
) -> Result<String> {
    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    {
        if let Ok(color) = sample_with_xcap(monitor_x, monitor_y, width, height, x_ratio, y_ratio) {
            return Ok(color);
        }
    }

    Ok("#ffffff".to_string())
}

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
fn sample_with_xcap(
    monitor_x: i32,
    monitor_y: i32,
    width: u32,
    height: u32,
    x_ratio: f32,
    y_ratio: f32,
) -> Result<String> {
    use xcap::Monitor;

    let monitors = Monitor::all()?;
    let monitor = monitors
        .into_iter()
        .find(|m| {
            m.x() == monitor_x
                && m.y() == monitor_y
                && m.width() == width
                && m.height() == height
        })
        .or_else(|| Monitor::all().ok()?.into_iter().next())
        .ok_or_else(|| anyhow::anyhow!("no monitor found"))?;

    let image = monitor.capture_image()?;
    let img_width = image.width();
    let img_height = image.height();
    if img_width == 0 || img_height == 0 {
        anyhow::bail!("empty capture");
    }

    let sample_x = ((img_width as f32) * x_ratio.clamp(0.1, 0.9)) as u32;
    let sample_y = ((img_height as f32) * y_ratio.clamp(0.1, 0.9)) as u32;
    let pixel = image.get_pixel(sample_x.min(img_width - 1), sample_y.min(img_height - 1));
    let luminance = (0.299 * pixel[0] as f64 + 0.587 * pixel[1] as f64 + 0.114 * pixel[2] as f64)
        / 255.0;

    if luminance > 0.55 {
        Ok("#111827".to_string())
    } else {
        Ok("#f8fafc".to_string())
    }
}
