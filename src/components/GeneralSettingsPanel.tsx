import { useEffect, useState } from "react";
import type { AppSettings, MonitorInfo } from "../lib/types";
import { api } from "../lib/api";

interface Props {
  settings: AppSettings;
  onChange: (settings: AppSettings) => void;
}

export function GeneralSettingsPanel({ settings, onChange }: Props) {
  const [monitors, setMonitors] = useState<MonitorInfo[]>([]);
  const [autostart, setAutostart] = useState(settings.launch_at_login);

  useEffect(() => {
    api
      .listMonitors()
      .then(setMonitors)
      .catch(() => setMonitors([]));
    api
      .getAutostart()
      .then(setAutostart)
      .catch(() => setAutostart(settings.launch_at_login));
  }, [settings.launch_at_login]);

  const update = <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => {
    onChange({ ...settings, [key]: value });
  };

  const toggleAutostart = async (enabled: boolean) => {
    await api.setAutostart(enabled);
    setAutostart(enabled);
    update("launch_at_login", enabled);
  };

  return (
    <section className="space-y-6">
      <div className="rounded-xl border border-slate-800 bg-slate-900/60 p-5">
        <h2 className="mb-4 text-lg font-medium">Sync & duplicates</h2>
        <label className="flex items-center gap-2 text-sm">
          <input
            type="checkbox"
            checked={settings.dedupe_reminders}
            onChange={(e) => update("dedupe_reminders", e.target.checked)}
          />
          Hide duplicate reminders across calendars (same title and start time)
        </label>
      </div>

      <div className="rounded-xl border border-slate-800 bg-slate-900/60 p-5">
        <h2 className="mb-2 text-lg font-medium">Quiet hours</h2>
        <p className="mb-4 text-sm text-slate-400">
          Suppress overlay reminders during these hours (local time).
        </p>
        <div className="space-y-4">
          <label className="flex items-center gap-2 text-sm">
            <input
              type="checkbox"
              checked={settings.quiet_hours_enabled}
              onChange={(e) => update("quiet_hours_enabled", e.target.checked)}
            />
            Enable quiet hours
          </label>
          <div className="grid gap-4 md:grid-cols-2">
            <label className="block text-sm">
              <span className="mb-2 block text-slate-400">Start</span>
              <input
                type="time"
                value={settings.quiet_hours_start}
                onChange={(e) => update("quiet_hours_start", e.target.value)}
                className="w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2"
              />
            </label>
            <label className="block text-sm">
              <span className="mb-2 block text-slate-400">End</span>
              <input
                type="time"
                value={settings.quiet_hours_end}
                onChange={(e) => update("quiet_hours_end", e.target.value)}
                className="w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2"
              />
            </label>
          </div>
        </div>
      </div>

      <div className="rounded-xl border border-slate-800 bg-slate-900/60 p-5">
        <h2 className="mb-4 text-lg font-medium">Display</h2>
        <label className="block text-sm">
          <span className="mb-2 block text-slate-400">Show reminders on</span>
          <select
            value={settings.monitor_target}
            onChange={(e) => update("monitor_target", e.target.value)}
            className="w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2"
          >
            <option value="primary">Primary monitor</option>
            <option value="active">Monitor with cursor</option>
            <option value="all">All monitors</option>
            {monitors.map((monitor) => (
              <option key={monitor.index} value={String(monitor.index)}>
                {monitor.name} ({monitor.width}×{monitor.height})
                {monitor.is_primary ? " · primary" : ""}
              </option>
            ))}
          </select>
        </label>
      </div>

      <div className="rounded-xl border border-slate-800 bg-slate-900/60 p-5">
        <h2 className="mb-4 text-lg font-medium">Reminders</h2>
        <div className="space-y-3">
          <label className="flex items-center gap-2 text-sm">
            <input
              type="checkbox"
              checked={settings.sound_enabled}
              onChange={(e) => update("sound_enabled", e.target.checked)}
            />
            Play a short chime when a reminder appears
          </label>
          <label className="flex items-center gap-2 text-sm">
            <input
              type="checkbox"
              checked={settings.auto_contrast_text}
              onChange={(e) => update("auto_contrast_text", e.target.checked)}
            />
            Auto-contrast text (sample desktop and pick light/dark text)
          </label>
          <label className="flex items-center gap-2 text-sm">
            <input
              type="checkbox"
              checked={settings.push_sync_enabled}
              onChange={(e) => update("push_sync_enabled", e.target.checked)}
            />
            Enable push sync when a relay URL is configured (PUSH_RELAY_URL)
          </label>
        </div>
      </div>

      <div className="rounded-xl border border-slate-800 bg-slate-900/60 p-5">
        <h2 className="mb-4 text-lg font-medium">Startup</h2>
        <label className="flex items-center gap-2 text-sm">
          <input
            type="checkbox"
            checked={autostart}
            onChange={(e) => toggleAutostart(e.target.checked)}
          />
          Launch Screen Reminder at login (runs in system tray)
        </label>
      </div>
    </section>
  );
}
