import { invoke, isTauri } from "@tauri-apps/api/core";
import { createDefaultComposition, normalizeSettings } from "./composition";
import type {
  AccountStyleOverrides,
  AppSettings,
  CalendarAccount,
  CaldavConnectRequest,
  CompositionPreset,
  MonitorInfo,
  PlatformInfo,
  ReminderEvent,
  SyncStatus,
} from "./types";

let devPresets: CompositionPreset[] = [];

const mockSettings: AppSettings = normalizeSettings({
  icon_id: "bell",
  custom_icon_path: null,
  icon_size: 48,
  animation_path: "left_to_right",
  animation_speed: 1,
  show_title: true,
  show_countdown: true,
  font_family: "Inter, system-ui, sans-serif",
  font_size: 16,
  font_color: "#ffffff",
  composition: createDefaultComposition(),
  dedupe_reminders: true,
  quiet_hours_enabled: false,
  quiet_hours_start: "22:00",
  quiet_hours_end: "07:00",
  monitor_target: "primary",
  launch_at_login: false,
  sound_enabled: true,
  auto_contrast_text: false,
  push_sync_enabled: true,
  snooze_durations: [5, 10, 15],
  auto_dismiss_seconds: null,
  reminders_paused: false,
});

let devSettings: AppSettings = mockSettings;
let devAccounts: CalendarAccount[] = [];
let devReminders: ReminderEvent[] = [];
let devLastSync: string | null = null;
let devAutostart = false;

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (isTauri()) {
    return invoke<T>(command, args);
  }

  switch (command) {
    case "get_settings":
      return devSettings as T;
    case "save_settings":
      devSettings = normalizeSettings((args?.settings as AppSettings) ?? devSettings);
      return undefined as T;
    case "list_accounts":
      return devAccounts as T;
    case "get_platform_info":
      return {
        os: "browser",
        apple_calendar_available: false,
      } satisfies PlatformInfo as T;
    case "get_sync_status":
      return {
        last_sync: devLastSync,
        reminder_count: devReminders.length,
        account_count: devAccounts.length,
        accounts: devAccounts.map((account) => ({
          account_id: account.id,
          display_name: account.display_name,
          source: account.source,
          last_sync: devLastSync,
          last_error: null,
          reminders_synced: devReminders.filter((r) => r.account_id === account.id).length,
        })),
      } satisfies SyncStatus as T;
    case "list_upcoming_reminders":
      return devReminders as T;
    case "list_monitors":
      return [
        {
          index: 0,
          name: "Display 1",
          width: 1920,
          height: 1080,
          is_primary: true,
        },
      ] satisfies MonitorInfo[] as T;
    case "get_autostart":
      return devAutostart as T;
    case "set_autostart":
      devAutostart = Boolean(args?.enabled);
      devSettings = { ...devSettings, launch_at_login: devAutostart };
      return undefined as T;
    case "connect_google":
    case "connect_outlook":
    case "connect_apple":
    case "connect_google_tasks":
    case "connect_microsoft_todo":
    case "connect_caldav": {
      const account: CalendarAccount = {
        id: crypto.randomUUID(),
        source: command.replace("connect_", ""),
        display_name: "Demo account",
        email: "demo@example.com",
        sync_token: null,
        caldav_url: null,
        caldav_username: null,
        connected_at: new Date().toISOString(),
        style_overrides: null,
      };
      devAccounts = [...devAccounts, account];
      devLastSync = new Date().toISOString();
      const start = new Date(Date.now() + 60 * 60 * 1000).toISOString();
      const reminder = new Date(Date.now() + 50 * 60 * 1000).toISOString();
      devReminders = [
        ...devReminders,
        {
          id: crypto.randomUUID(),
          account_id: account.id,
          source: account.source,
          external_id: "demo-1",
          title: account.source.includes("task") ? "Finish quarterly report" : "Team standup",
          start_time: start,
          reminder_time: reminder,
          location: account.source.includes("task") ? null : "Zoom",
          url: null,
          fired_at: null,
          snoozed_until: null,
          dismissed: false,
        },
      ];
      return account as T;
    }
    case "disconnect_account":
      devAccounts = devAccounts.filter((a) => a.id !== args?.accountId);
      devReminders = devReminders.filter((r) => r.account_id !== args?.accountId);
      return undefined as T;
    case "sync_now":
      devLastSync = new Date().toISOString();
      return 0 as T;
    case "set_reminders_paused":
      devSettings = { ...devSettings, reminders_paused: Boolean(args?.paused) };
      return undefined as T;
    case "list_composition_presets":
      return devPresets as T;
    case "save_composition_preset": {
      const preset: CompositionPreset = {
        id: crypto.randomUUID(),
        name: String(args?.name ?? "Preset"),
        composition: devSettings.composition,
        created_at: new Date().toISOString(),
      };
      devPresets = [preset, ...devPresets];
      return preset as T;
    }
    case "load_composition_preset": {
      const preset = devPresets.find((p) => p.id === args?.presetId);
      if (preset) {
        devSettings = { ...devSettings, composition: preset.composition };
      }
      return devSettings as T;
    }
    case "delete_composition_preset":
      devPresets = devPresets.filter((p) => p.id !== args?.presetId);
      return undefined as T;
    case "get_account_style":
      return null as T;
    case "save_account_style":
      return undefined as T;
    default:
      console.info(`[browser dev] ${command}`, args);
      return undefined as T;
  }
}

export const api = {
  getSettings: () => call<AppSettings>("get_settings"),
  saveSettings: (settings: AppSettings) => call<void>("save_settings", { settings }),
  listAccounts: () => call<CalendarAccount[]>("list_accounts"),
  connectGoogle: () => call<CalendarAccount>("connect_google"),
  connectOutlook: () => call<CalendarAccount>("connect_outlook"),
  connectCaldav: (request: CaldavConnectRequest) =>
    call<CalendarAccount>("connect_caldav", { request }),
  connectApple: () => call<CalendarAccount>("connect_apple"),
  connectGoogleTasks: () => call<CalendarAccount>("connect_google_tasks"),
  connectMicrosoftTodo: () => call<CalendarAccount>("connect_microsoft_todo"),
  disconnectAccount: (accountId: string) =>
    call<void>("disconnect_account", { accountId }),
  syncNow: () => call<number>("sync_now"),
  listUpcomingReminders: (limit?: number) =>
    call<ReminderEvent[]>("list_upcoming_reminders", { limit }),
  getSyncStatus: () => call<SyncStatus>("get_sync_status"),
  listMonitors: () => call<MonitorInfo[]>("list_monitors"),
  getAutostart: () => call<boolean>("get_autostart"),
  setAutostart: (enabled: boolean) => call<void>("set_autostart", { enabled }),
  getPlatformInfo: () => call<PlatformInfo>("get_platform_info"),
  dismissReminder: (reminderId: string) =>
    call<void>("dismiss_reminder", { reminderId }),
  snoozeReminder: (reminderId: string, minutes: number) =>
    call<void>("snooze_reminder", { reminderId, minutes }),
  snoozeReminderUntilStart: (reminderId: string) =>
    call<void>("snooze_reminder_until_start", { reminderId }),
  openReminderUrl: (url: string) => call<void>("open_reminder_url", { url }),
  hideReminderOverlay: () => call<void>("hide_reminder_overlay"),
  previewOverlay: () => call<void>("preview_overlay"),
  createTestReminder: () => call<void>("create_test_reminder"),
  setRemindersPaused: (paused: boolean) =>
    call<void>("set_reminders_paused", { paused }),
  listCompositionPresets: () => call<CompositionPreset[]>("list_composition_presets"),
  saveCompositionPreset: (name: string) =>
    call<CompositionPreset>("save_composition_preset", { name }),
  loadCompositionPreset: (presetId: string) =>
    call<AppSettings>("load_composition_preset", { presetId }),
  deleteCompositionPreset: (presetId: string) =>
    call<void>("delete_composition_preset", { presetId }),
  getAccountStyle: (accountId: string) =>
    call<AccountStyleOverrides | null>("get_account_style", { accountId }),
  saveAccountStyle: (accountId: string, style: AccountStyleOverrides) =>
    call<void>("save_account_style", { accountId, style }),
};
