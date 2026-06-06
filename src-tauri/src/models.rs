use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CalendarSource {
    Google,
    Outlook,
    Caldav,
    Apple,
}

impl CalendarSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Google => "google",
            Self::Outlook => "outlook",
            Self::Caldav => "caldav",
            Self::Apple => "apple",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "google" => Some(Self::Google),
            "outlook" => Some(Self::Outlook),
            "caldav" => Some(Self::Caldav),
            "apple" => Some(Self::Apple),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReminderEvent {
    pub id: String,
    pub account_id: String,
    pub source: String,
    pub external_id: String,
    pub title: String,
    pub start_time: DateTime<Utc>,
    pub reminder_time: DateTime<Utc>,
    pub location: Option<String>,
    pub url: Option<String>,
    pub fired_at: Option<DateTime<Utc>>,
    pub snoozed_until: Option<DateTime<Utc>>,
    pub dismissed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountStyleOverrides {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub icon_id: Option<String>,
    #[serde(default)]
    pub custom_icon_path: Option<String>,
    #[serde(default)]
    pub icon_size: Option<u32>,
    #[serde(default)]
    pub animation_path: Option<String>,
    #[serde(default)]
    pub animation_speed: Option<f32>,
    #[serde(default)]
    pub font_color: Option<String>,
    #[serde(default)]
    pub composition: Option<OverlayComposition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionPreset {
    pub id: String,
    pub name: String,
    pub composition: OverlayComposition,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushSubscription {
    pub account_id: String,
    pub source: String,
    pub channel_id: String,
    pub resource_id: Option<String>,
    pub expiration: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarAccount {
    pub id: String,
    pub source: String,
    pub display_name: String,
    pub email: Option<String>,
    pub sync_token: Option<String>,
    pub caldav_url: Option<String>,
    pub caldav_username: Option<String>,
    pub connected_at: DateTime<Utc>,
    #[serde(default)]
    pub style_overrides: Option<AccountStyleOverrides>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimationPath {
    LeftToRight,
    Bounce,
    FigureEight,
    Random,
}

impl AnimationPath {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LeftToRight => "left_to_right",
            Self::Bounce => "bounce",
            Self::FigureEight => "figure_eight",
            Self::Random => "random",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "bounce" => Self::Bounce,
            "figure_eight" => Self::FigureEight,
            "random" => Self::Random,
            _ => Self::LeftToRight,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutPoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayLayer {
    pub id: String,
    #[serde(rename = "type")]
    pub layer_type: String,
    pub name: String,
    pub visible: bool,
    pub x: f32,
    pub y: f32,
    pub z_index: u32,
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
    #[serde(default)]
    pub icon_id: Option<String>,
    #[serde(default)]
    pub image_path: Option<String>,
    #[serde(default)]
    pub font_size: Option<u32>,
    #[serde(default)]
    pub text_content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayComposition {
    pub canvas_width: u32,
    pub canvas_height: u32,
    #[serde(default)]
    pub layers: Vec<OverlayLayer>,
}

impl Default for OverlayComposition {
    fn default() -> Self {
        Self {
            canvas_width: 320,
            canvas_height: 88,
            layers: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayLayout {
    pub icon: LayoutPoint,
    pub title: LayoutPoint,
    pub countdown: LayoutPoint,
    pub bubble_width: u32,
    pub bubble_height: u32,
}

impl Default for OverlayLayout {
    fn default() -> Self {
        Self {
            icon: LayoutPoint { x: 12.0, y: 50.0 },
            title: LayoutPoint { x: 42.0, y: 38.0 },
            countdown: LayoutPoint { x: 42.0, y: 62.0 },
            bubble_width: 320,
            bubble_height: 88,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub icon_id: String,
    pub custom_icon_path: Option<String>,
    pub icon_size: u32,
    pub animation_path: String,
    pub animation_speed: f32,
    pub show_title: bool,
    #[serde(default = "default_show_countdown")]
    pub show_countdown: bool,
    pub font_family: String,
    pub font_size: u32,
    pub font_color: String,
    #[serde(default)]
    pub composition: OverlayComposition,
    #[serde(default)]
    pub layout: Option<OverlayLayout>,
    pub snooze_durations: Vec<u32>,
    pub auto_dismiss_seconds: Option<u32>,
    pub reminders_paused: bool,
    #[serde(default = "default_dedupe_reminders")]
    pub dedupe_reminders: bool,
    #[serde(default = "default_quiet_hours_enabled")]
    pub quiet_hours_enabled: bool,
    #[serde(default = "default_quiet_hours_start")]
    pub quiet_hours_start: String,
    #[serde(default = "default_quiet_hours_end")]
    pub quiet_hours_end: String,
    #[serde(default = "default_monitor_target")]
    pub monitor_target: String,
    #[serde(default)]
    pub launch_at_login: bool,
    #[serde(default = "default_sound_enabled")]
    pub sound_enabled: bool,
    #[serde(default)]
    pub auto_contrast_text: bool,
    #[serde(default = "default_push_sync_enabled")]
    pub push_sync_enabled: bool,
}

fn default_sound_enabled() -> bool {
    true
}

fn default_push_sync_enabled() -> bool {
    true
}

fn default_show_countdown() -> bool {
    true
}

fn default_dedupe_reminders() -> bool {
    true
}

fn default_quiet_hours_enabled() -> bool {
    false
}

fn default_quiet_hours_start() -> String {
    "22:00".to_string()
}

fn default_quiet_hours_end() -> String {
    "07:00".to_string()
}

fn default_monitor_target() -> String {
    "primary".to_string()
}

impl AppSettings {
    pub fn ensure_composition(&mut self) {
        if !self.composition.layers.is_empty() {
            return;
        }

        if let Some(layout) = self.layout.clone() {
            self.composition = composition_from_layout(self, &layout);
            return;
        }

        self.composition = default_composition_from_settings(self);
    }
}

fn default_composition_from_settings(settings: &AppSettings) -> OverlayComposition {
    use uuid::Uuid;

    let mut layers = vec![OverlayLayer {
        id: Uuid::new_v4().to_string(),
        layer_type: "image".to_string(),
        name: "Main icon".to_string(),
        visible: true,
        x: 15.0,
        y: 50.0,
        z_index: 0,
        width: Some(settings.icon_size),
        icon_id: Some(settings.icon_id.clone()),
        image_path: settings.custom_icon_path.clone(),
        height: None,
        font_size: None,
        text_content: None,
    }];

    let mut z = 1u32;
    if settings.show_title {
        layers.push(OverlayLayer {
            id: Uuid::new_v4().to_string(),
            layer_type: "title".to_string(),
            name: "Event title".to_string(),
            visible: true,
            x: 48.0,
            y: 38.0,
            z_index: z,
            width: None,
            icon_id: None,
            image_path: None,
            height: None,
            font_size: None,
            text_content: None,
        });
        z += 1;
    }

    if settings.show_countdown {
        layers.push(OverlayLayer {
            id: Uuid::new_v4().to_string(),
            layer_type: "countdown".to_string(),
            name: "Countdown".to_string(),
            visible: true,
            x: 48.0,
            y: 62.0,
            z_index: z,
            width: None,
            icon_id: None,
            image_path: None,
            height: None,
            font_size: None,
            text_content: None,
        });
    }

    OverlayComposition {
        canvas_width: 320,
        canvas_height: 88,
        layers,
    }
}

fn composition_from_layout(settings: &AppSettings, layout: &OverlayLayout) -> OverlayComposition {
    use uuid::Uuid;

    let mut layers = vec![OverlayLayer {
        id: Uuid::new_v4().to_string(),
        layer_type: "image".to_string(),
        name: "Main icon".to_string(),
        visible: true,
        x: layout.icon.x,
        y: layout.icon.y,
        z_index: 0,
        width: Some(settings.icon_size),
        icon_id: Some(settings.icon_id.clone()),
        image_path: settings.custom_icon_path.clone(),
        height: None,
        font_size: None,
        text_content: None,
    }];

    let mut z = 1u32;
    if settings.show_title {
        layers.push(OverlayLayer {
            id: Uuid::new_v4().to_string(),
            layer_type: "title".to_string(),
            name: "Event title".to_string(),
            visible: true,
            x: layout.title.x,
            y: layout.title.y,
            z_index: z,
            width: None,
            icon_id: None,
            image_path: None,
            height: None,
            font_size: None,
            text_content: None,
        });
        z += 1;
    }

    if settings.show_countdown {
        layers.push(OverlayLayer {
            id: Uuid::new_v4().to_string(),
            layer_type: "countdown".to_string(),
            name: "Countdown".to_string(),
            visible: true,
            x: layout.countdown.x,
            y: layout.countdown.y,
            z_index: z,
            width: None,
            icon_id: None,
            image_path: None,
            height: None,
            font_size: None,
            text_content: None,
        });
    }

    OverlayComposition {
        canvas_width: layout.bubble_width,
        canvas_height: layout.bubble_height,
        layers,
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        let mut settings = Self {
            icon_id: "bell".to_string(),
            custom_icon_path: None,
            icon_size: 48,
            animation_path: AnimationPath::LeftToRight.as_str().to_string(),
            animation_speed: 1.0,
            show_title: true,
            show_countdown: true,
            font_family: "Inter, system-ui, sans-serif".to_string(),
            font_size: 16,
            font_color: "#ffffff".to_string(),
            composition: OverlayComposition::default(),
            layout: None,
            snooze_durations: vec![5, 10, 15],
            auto_dismiss_seconds: None,
            reminders_paused: false,
            dedupe_reminders: true,
            quiet_hours_enabled: false,
            quiet_hours_start: default_quiet_hours_start(),
            quiet_hours_end: default_quiet_hours_end(),
            monitor_target: default_monitor_target(),
            launch_at_login: false,
            sound_enabled: true,
            auto_contrast_text: false,
            push_sync_enabled: true,
        };
        settings.composition = default_composition_from_settings(&settings);
        settings
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayPayload {
    pub reminder_id: String,
    pub account_id: String,
    pub source: String,
    pub title: String,
    pub location: Option<String>,
    pub url: Option<String>,
    pub start_time: DateTime<Utc>,
    pub settings: AppSettings,
    #[serde(default)]
    pub effective_font_color: Option<String>,
    #[serde(default = "default_play_sound")]
    pub play_sound: bool,
    #[serde(default)]
    pub monitor_x: i32,
    #[serde(default)]
    pub monitor_y: i32,
    #[serde(default)]
    pub monitor_width: u32,
    #[serde(default)]
    pub monitor_height: u32,
    #[serde(default = "default_monitor_scale_factor")]
    pub monitor_scale_factor: f64,
}

fn default_monitor_scale_factor() -> f64 {
    1.0
}

fn default_play_sound() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaldavConnectRequest {
    pub display_name: String,
    pub server_url: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub os: String,
    pub apple_calendar_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSyncStatus {
    pub account_id: String,
    pub display_name: String,
    pub source: String,
    pub last_sync: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub reminders_synced: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorInfo {
    pub index: usize,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub is_primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub last_sync: Option<DateTime<Utc>>,
    pub reminder_count: usize,
    pub account_count: usize,
    pub accounts: Vec<AccountSyncStatus>,
}
