export type LayerType = "image" | "title" | "countdown" | "text";

export interface OverlayLayer {
  id: string;
  type: LayerType;
  name: string;
  visible: boolean;
  x: number;
  y: number;
  z_index: number;
  width?: number;
  height?: number;
  icon_id?: string;
  image_path?: string | null;
  font_size?: number | null;
  text_content?: string | null;
}

export interface OverlayComposition {
  canvas_width: number;
  canvas_height: number;
  layers: OverlayLayer[];
}

/** @deprecated Legacy layout — migrated to composition on load */
export interface OverlayLayout {
  icon: { x: number; y: number };
  title: { x: number; y: number };
  countdown: { x: number; y: number };
  bubble_width: number;
  bubble_height: number;
}

export interface AppSettings {
  icon_id: string;
  custom_icon_path: string | null;
  icon_size: number;
  animation_path: string;
  animation_speed: number;
  show_title: boolean;
  show_countdown: boolean;
  font_family: string;
  font_size: number;
  font_color: string;
  composition: OverlayComposition;
  /** @deprecated */
  layout?: OverlayLayout;
  dedupe_reminders: boolean;
  quiet_hours_enabled: boolean;
  quiet_hours_start: string;
  quiet_hours_end: string;
  monitor_target: string;
  launch_at_login: boolean;
  sound_enabled: boolean;
  auto_contrast_text: boolean;
  push_sync_enabled: boolean;
  snooze_durations: number[];
  auto_dismiss_seconds: number | null;
  reminders_paused: boolean;
}

export interface CalendarAccount {
  id: string;
  source: string;
  display_name: string;
  email: string | null;
  sync_token: string | null;
  caldav_url: string | null;
  caldav_username: string | null;
  connected_at: string;
  style_overrides?: AccountStyleOverrides | null;
}

export interface AccountStyleOverrides {
  enabled: boolean;
  icon_id?: string | null;
  custom_icon_path?: string | null;
  icon_size?: number | null;
  animation_path?: string | null;
  animation_speed?: number | null;
  font_color?: string | null;
  composition?: OverlayComposition | null;
}

export interface CompositionPreset {
  id: string;
  name: string;
  composition: OverlayComposition;
  created_at: string;
}

export interface CaldavConnectRequest {
  display_name: string;
  server_url: string;
  username: string;
  password: string;
}

export interface PlatformInfo {
  os: string;
  apple_calendar_available: boolean;
}

export interface AccountSyncStatus {
  account_id: string;
  display_name: string;
  source: string;
  last_sync: string | null;
  last_error: string | null;
  reminders_synced: number;
}

export interface MonitorInfo {
  index: number;
  name: string;
  width: number;
  height: number;
  is_primary: boolean;
}

export interface SyncStatus {
  last_sync: string | null;
  reminder_count: number;
  account_count: number;
  accounts: AccountSyncStatus[];
}

export interface ReminderEvent {
  id: string;
  account_id: string;
  source: string;
  external_id: string;
  title: string;
  start_time: string;
  reminder_time: string;
  location: string | null;
  url: string | null;
  fired_at: string | null;
  snoozed_until: string | null;
  dismissed: boolean;
}

export const TASK_SOURCES = new Set(["google_tasks", "microsoft_todo"]);

export const SOURCE_LABELS: Record<string, string> = {
  google: "Google Calendar",
  outlook: "Outlook / Microsoft 365",
  google_tasks: "Google Tasks",
  microsoft_todo: "Microsoft To Do",
  caldav: "CalDAV",
  apple: "Apple Calendar",
};

export interface OverlayPayload {
  reminder_id: string;
  account_id: string;
  source: string;
  title: string;
  location: string | null;
  url: string | null;
  start_time: string;
  settings: AppSettings;
  effective_font_color?: string | null;
  play_sound: boolean;
  monitor_x: number;
  monitor_y: number;
  monitor_width: number;
  monitor_height: number;
  monitor_scale_factor: number;
}

export const BUILTIN_ICONS = [
  { id: "bell", label: "Bell", emoji: "🔔" },
  { id: "calendar", label: "Calendar", emoji: "📅" },
  { id: "star", label: "Star", emoji: "⭐" },
  { id: "rocket", label: "Rocket", emoji: "🚀" },
  { id: "coffee", label: "Coffee", emoji: "☕" },
  { id: "cat", label: "Cat", emoji: "🐱" },
] as const;

export const ANIMATION_PATHS = [
  { id: "left_to_right", label: "Left to right" },
  { id: "bounce", label: "Bounce" },
  { id: "figure_eight", label: "Figure eight" },
  { id: "random", label: "Random" },
] as const;

export const FONT_OPTIONS = [
  "Inter, system-ui, sans-serif",
  "Georgia, serif",
  "Courier New, monospace",
  "Comic Sans MS, cursive",
  "Impact, sans-serif",
];

export const MONITOR_TARGETS = [
  { id: "primary", label: "Primary monitor" },
  { id: "active", label: "Monitor with cursor" },
  { id: "all", label: "All monitors" },
] as const;

export const LAYER_TYPE_LABELS: Record<LayerType, string> = {
  image: "Image",
  title: "Event title",
  countdown: "Countdown",
  text: "Text",
};
