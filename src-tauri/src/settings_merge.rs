use crate::models::{AccountStyleOverrides, AppSettings, OverlayComposition};

pub fn merge_account_style(base: &AppSettings, overrides: &AccountStyleOverrides) -> AppSettings {
    if !overrides.enabled {
        return base.clone();
    }

    let mut merged = base.clone();

    if let Some(icon_id) = &overrides.icon_id {
        merged.icon_id = icon_id.clone();
    }
    if overrides.custom_icon_path.is_some() {
        merged.custom_icon_path = overrides.custom_icon_path.clone();
    }
    if let Some(icon_size) = overrides.icon_size {
        merged.icon_size = icon_size;
    }
    if let Some(animation_path) = &overrides.animation_path {
        merged.animation_path = animation_path.clone();
    }
    if let Some(animation_speed) = overrides.animation_speed {
        merged.animation_speed = animation_speed;
    }
    if let Some(font_color) = &overrides.font_color {
        merged.font_color = font_color.clone();
    }
    if let Some(composition) = &overrides.composition {
        merged.composition = composition.clone();
    } else if overrides.icon_id.is_some()
        || overrides.custom_icon_path.is_some()
        || overrides.icon_size.is_some()
    {
        merged.composition = sync_primary_icon_layer(&merged.composition, &merged);
    }

    merged
}

fn sync_primary_icon_layer(
    composition: &OverlayComposition,
    settings: &AppSettings,
) -> OverlayComposition {
    let mut layers = composition.layers.clone();
    let mut updated = false;

    for layer in &mut layers {
        if layer.layer_type == "image" {
            layer.icon_id = Some(settings.icon_id.clone());
            layer.image_path = settings.custom_icon_path.clone();
            layer.width = Some(settings.icon_size);
            updated = true;
            break;
        }
    }

    if updated {
        OverlayComposition {
            canvas_width: composition.canvas_width,
            canvas_height: composition.canvas_height,
            layers,
        }
    } else {
        composition.clone()
    }
}
