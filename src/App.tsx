import { useCallback, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { api } from "./lib/api";
import { normalizeSettings } from "./lib/composition";
import type { AppSettings, CalendarAccount, PlatformInfo, ReminderEvent, SyncStatus } from "./lib/types";
import { AccountsPanel } from "./components/AccountsPanel";
import { AdvancedOptionsPanel } from "./components/AdvancedOptionsPanel";
import { AppearancePanel } from "./components/AppearancePanel";
import { AnimationPreview } from "./components/AnimationPreview";
import { GeneralSettingsPanel } from "./components/GeneralSettingsPanel";
import { StatusBar } from "./components/StatusBar";
import { UpcomingPanel } from "./components/UpcomingPanel";

type Tab = "upcoming" | "accounts" | "appearance" | "advanced" | "general";

export default function App() {
  const [tab, setTab] = useState<Tab>("upcoming");
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [accounts, setAccounts] = useState<CalendarAccount[]>([]);
  const [upcoming, setUpcoming] = useState<ReminderEvent[]>([]);
  const [platform, setPlatform] = useState<PlatformInfo | null>(null);
  const [syncStatus, setSyncStatus] = useState<SyncStatus | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const refresh = useCallback(async () => {
    const [nextSettings, nextAccounts, nextPlatform, nextSync, nextUpcoming] = await Promise.all([
      api.getSettings(),
      api.listAccounts(),
      api.getPlatformInfo(),
      api.getSyncStatus(),
      api.listUpcomingReminders(),
    ]);
    setSettings(normalizeSettings(nextSettings));
    setAccounts(nextAccounts);
    setPlatform(nextPlatform);
    setSyncStatus(nextSync);
    setUpcoming(nextUpcoming);
  }, []);

  useEffect(() => {
    refresh().catch((err) => setMessage(String(err)));
    const unlistenSync = listen("sync-complete", () => {
      refresh().catch(console.error);
    });
    const unlistenPause = listen<boolean>("reminders-paused-changed", (event) => {
      setSettings((current) =>
        current ? { ...current, reminders_paused: event.payload } : current,
      );
    });
    return () => {
      unlistenSync.then((fn) => fn());
      unlistenPause.then((fn) => fn());
    };
  }, [refresh]);

  const saveSettings = async (next: AppSettings) => {
    const normalized = normalizeSettings(next);
    setSettings(normalized);
    await api.saveSettings(normalized);
    setMessage("Settings saved");
  };

  const runAction = async (label: string, action: () => Promise<void>) => {
    setBusy(true);
    setMessage(null);
    try {
      await action();
      await refresh();
      if (label) {
        setMessage(label);
      }
    } catch (err) {
      setMessage(String(err));
    } finally {
      setBusy(false);
    }
  };

  if (!settings || !platform) {
    return (
      <div className="flex h-screen items-center justify-center overflow-hidden bg-slate-950 text-slate-300">
        Loading…
      </div>
    );
  }

  return (
    <div className="flex h-screen flex-col overflow-hidden bg-slate-950 text-slate-100">
      <header className="shrink-0 border-b border-slate-800 px-6 py-4">
        <div className="flex items-center justify-between gap-4">
          <div>
            <h1 className="text-xl font-semibold">Screen Reminder</h1>
            <p className="text-sm text-slate-400">
              Animated calendar reminders across your desktop
            </p>
          </div>
          <StatusBar
            syncStatus={syncStatus}
            paused={settings.reminders_paused}
            onSync={() => runAction("Sync complete", () => api.syncNow().then(() => {}))}
            onTogglePause={() =>
              runAction(
                settings.reminders_paused ? "Reminders resumed" : "Reminders paused",
                () => api.setRemindersPaused(!settings.reminders_paused),
              )
            }
            onTestReminder={() =>
              runAction("Test reminder scheduled", () => api.createTestReminder())
            }
            busy={busy}
          />
        </div>
      </header>

      <div className="mx-auto flex min-h-0 flex-1 max-w-6xl gap-6 overflow-hidden px-6 py-6">
        <nav className="flex w-48 shrink-0 flex-col gap-2 overflow-y-auto">
          <button
            className={`rounded-lg px-4 py-2 text-left text-sm ${
              tab === "upcoming" ? "bg-indigo-600" : "bg-slate-900 hover:bg-slate-800"
            }`}
            onClick={() => setTab("upcoming")}
          >
            Upcoming
          </button>
          <button
            className={`rounded-lg px-4 py-2 text-left text-sm ${
              tab === "accounts" ? "bg-indigo-600" : "bg-slate-900 hover:bg-slate-800"
            }`}
            onClick={() => setTab("accounts")}
          >
            Calendar accounts
          </button>
          <button
            className={`rounded-lg px-4 py-2 text-left text-sm ${
              tab === "appearance" ? "bg-indigo-600" : "bg-slate-900 hover:bg-slate-800"
            }`}
            onClick={() => setTab("appearance")}
          >
            Appearance
          </button>
          <button
            className={`rounded-lg px-4 py-2 text-left text-sm ${
              tab === "general" ? "bg-indigo-600" : "bg-slate-900 hover:bg-slate-800"
            }`}
            onClick={() => setTab("general")}
          >
            General
          </button>
          <button
            className={`rounded-lg px-4 py-2 text-left text-sm ${
              tab === "advanced" ? "bg-indigo-600" : "bg-slate-900 hover:bg-slate-800"
            }`}
            onClick={() => setTab("advanced")}
          >
            Advanced options
          </button>
        </nav>

        <main className="min-h-0 flex-1 space-y-4 overflow-y-auto">
          {message && (
            <div className="rounded-lg border border-slate-700 bg-slate-900 px-4 py-3 text-sm text-slate-200">
              {message}
            </div>
          )}

          {tab === "upcoming" ? (
            <UpcomingPanel
              reminders={upcoming}
              accounts={accounts}
              busy={busy}
              onSync={() => runAction("Sync complete", () => api.syncNow().then(() => {}))}
              onOpenUrl={(url) => api.openReminderUrl(url).catch((err) => setMessage(String(err)))}
            />
          ) : tab === "accounts" ? (
            <AccountsPanel
              accounts={accounts}
              accountSync={syncStatus?.accounts ?? []}
              platform={platform}
              globalSettings={settings}
              busy={busy}
              onConnectGoogle={() =>
                runAction("", async () => {
                  const account = await api.connectGoogle();
                  setMessage(`Connected ${account.display_name}`);
                })
              }
              onConnectOutlook={() =>
                runAction("", async () => {
                  const account = await api.connectOutlook();
                  setMessage(`Connected ${account.display_name}`);
                })
              }
              onConnectGoogleTasks={() =>
                runAction("", async () => {
                  const account = await api.connectGoogleTasks();
                  setMessage(`Connected ${account.display_name}`);
                })
              }
              onConnectMicrosoftTodo={() =>
                runAction("", async () => {
                  const account = await api.connectMicrosoftTodo();
                  setMessage(`Connected ${account.display_name}`);
                })
              }
              onConnectCaldav={async (request) =>
                runAction("", async () => {
                  const account = await api.connectCaldav(request);
                  setMessage(`Connected ${account.display_name}`);
                })
              }
              onConnectApple={() =>
                runAction("Apple Calendar connected", async () => {
                  await api.connectApple();
                })
              }
              onDisconnect={(id) =>
                runAction("Account disconnected", async () => {
                  await api.disconnectAccount(id);
                })
              }
              onStyleSaved={() => refresh().catch(console.error)}
            />
          ) : tab === "appearance" ? (
            <div className="grid gap-6 lg:grid-cols-2">
              <AppearancePanel settings={settings} onChange={saveSettings} />
              <AnimationPreview
                settings={settings}
                onPreview={() => runAction("Preview shown", () => api.previewOverlay())}
              />
            </div>
          ) : tab === "general" ? (
            <GeneralSettingsPanel settings={settings} onChange={saveSettings} />
          ) : (
            <AdvancedOptionsPanel settings={settings} onChange={saveSettings} />
          )}
        </main>
      </div>
    </div>
  );
}
