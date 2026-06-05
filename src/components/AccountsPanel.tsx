import { useState } from "react";
import { formatRelativeTime } from "../lib/formatRelativeTime";
import { AccountStylePanel } from "./AccountStylePanel";
import type {
  AccountSyncStatus,
  AppSettings,
  CalendarAccount,
  CaldavConnectRequest,
  PlatformInfo,
} from "../lib/types";

interface Props {
  accounts: CalendarAccount[];
  accountSync: AccountSyncStatus[];
  platform: PlatformInfo;
  globalSettings: AppSettings;
  busy: boolean;
  onConnectGoogle: () => void;
  onConnectOutlook: () => void;
  onConnectGoogleTasks: () => void;
  onConnectMicrosoftTodo: () => void;
  onConnectCaldav: (request: CaldavConnectRequest) => Promise<void>;
  onConnectApple: () => void;
  onDisconnect: (accountId: string) => void;
  onStyleSaved?: () => void;
}

function syncLabel(status: AccountSyncStatus | undefined): string {
  if (!status) {
    return "Not synced yet";
  }
  if (status.last_error) {
    return `Error: ${status.last_error}`;
  }
  if (status.last_sync) {
    return `Synced ${formatRelativeTime(status.last_sync)} · ${status.reminders_synced} events`;
  }
  return "Not synced yet";
}

export function AccountsPanel({
  accounts,
  accountSync,
  platform,
  globalSettings,
  busy,
  onConnectGoogle,
  onConnectOutlook,
  onConnectGoogleTasks,
  onConnectMicrosoftTodo,
  onConnectCaldav,
  onConnectApple,
  onDisconnect,
  onStyleSaved,
}: Props) {
  const [showCaldav, setShowCaldav] = useState(false);
  const [styleAccount, setStyleAccount] = useState<CalendarAccount | null>(null);
  const [caldavForm, setCaldavForm] = useState<CaldavConnectRequest>({
    display_name: "CalDAV",
    server_url: "",
    username: "",
    password: "",
  });

  const statusByAccount = new Map(accountSync.map((s) => [s.account_id, s]));

  return (
    <section className="space-y-6">
      <div className="rounded-xl border border-slate-800 bg-slate-900/60 p-5">
        <h2 className="mb-4 text-lg font-medium">Connect a calendar</h2>
        <p className="mb-4 text-sm text-slate-400">
          Calendars sync on connect, every 5 minutes (or every 1 minute when an event is within 30
          minutes), and on wake. Push sync activates when PUSH_RELAY_URL is set.
        </p>
        <div className="flex flex-wrap gap-3">
          <button
            disabled={busy}
            onClick={onConnectGoogle}
            className="rounded-lg bg-white px-4 py-2 text-sm font-medium text-slate-900 hover:bg-slate-100 disabled:opacity-50"
          >
            Google Calendar
          </button>
          <button
            disabled={busy}
            onClick={onConnectOutlook}
            className="rounded-lg bg-blue-600 px-4 py-2 text-sm hover:bg-blue-500 disabled:opacity-50"
          >
            Outlook / Microsoft 365
          </button>
          <button
            disabled={busy}
            onClick={onConnectGoogleTasks}
            className="rounded-lg bg-slate-800 px-4 py-2 text-sm hover:bg-slate-700 disabled:opacity-50"
          >
            Google Tasks
          </button>
          <button
            disabled={busy}
            onClick={onConnectMicrosoftTodo}
            className="rounded-lg bg-slate-800 px-4 py-2 text-sm hover:bg-slate-700 disabled:opacity-50"
          >
            Microsoft To Do
          </button>
          <button
            disabled={busy}
            onClick={() => setShowCaldav((v) => !v)}
            className="rounded-lg bg-slate-800 px-4 py-2 text-sm hover:bg-slate-700 disabled:opacity-50"
          >
            CalDAV
          </button>
          {platform.apple_calendar_available && (
            <button
              disabled={busy}
              onClick={onConnectApple}
              className="rounded-lg bg-slate-800 px-4 py-2 text-sm hover:bg-slate-700 disabled:opacity-50"
            >
              Apple Calendar
            </button>
          )}
        </div>

        {showCaldav && (
          <form
            className="mt-4 grid gap-3 md:grid-cols-2"
            onSubmit={(e) => {
              e.preventDefault();
              onConnectCaldav(caldavForm);
            }}
          >
            <input
              className="rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm"
              placeholder="Display name"
              value={caldavForm.display_name}
              onChange={(e) =>
                setCaldavForm({ ...caldavForm, display_name: e.target.value })
              }
            />
            <input
              className="rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm md:col-span-2"
              placeholder="Server URL (e.g. https://caldav.example.com)"
              value={caldavForm.server_url}
              onChange={(e) =>
                setCaldavForm({ ...caldavForm, server_url: e.target.value })
              }
            />
            <input
              className="rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm"
              placeholder="Username"
              value={caldavForm.username}
              onChange={(e) =>
                setCaldavForm({ ...caldavForm, username: e.target.value })
              }
            />
            <input
              type="password"
              className="rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm"
              placeholder="Password"
              value={caldavForm.password}
              onChange={(e) =>
                setCaldavForm({ ...caldavForm, password: e.target.value })
              }
            />
            <button
              type="submit"
              disabled={busy}
              className="rounded-lg bg-indigo-600 px-4 py-2 text-sm md:col-span-2 disabled:opacity-50"
            >
              Connect CalDAV
            </button>
          </form>
        )}
      </div>

      <div className="rounded-xl border border-slate-800 bg-slate-900/60 p-5">
        <h2 className="mb-4 text-lg font-medium">Connected accounts</h2>
        {accounts.length === 0 ? (
          <p className="text-sm text-slate-400">No accounts connected yet.</p>
        ) : (
          <ul className="space-y-3">
            {accounts.map((account) => {
              const status = statusByAccount.get(account.id);
              const hasError = Boolean(status?.last_error);
              return (
                <li
                  key={account.id}
                  className="flex items-center justify-between gap-4 rounded-lg border border-slate-800 bg-slate-950 px-4 py-3"
                >
                  <div className="min-w-0">
                    <div className="font-medium">{account.display_name}</div>
                    <div className="text-xs uppercase tracking-wide text-slate-500">
                      {account.source}
                      {account.email ? ` · ${account.email}` : ""}
                      {account.style_overrides?.enabled ? " · custom style" : ""}
                    </div>
                    <div
                      className={`mt-1 truncate text-xs ${
                        hasError ? "text-red-300" : "text-slate-400"
                      }`}
                    >
                      {syncLabel(status)}
                    </div>
                  </div>
                  <div className="flex shrink-0 gap-2">
                    <button
                      disabled={busy}
                      onClick={() => setStyleAccount(account)}
                      className="rounded-lg bg-slate-800 px-3 py-1 text-sm hover:bg-slate-700 disabled:opacity-50"
                    >
                      Style
                    </button>
                    <button
                      disabled={busy}
                      onClick={() => onDisconnect(account.id)}
                      className="rounded-lg bg-red-900/40 px-3 py-1 text-sm text-red-200 hover:bg-red-900/70 disabled:opacity-50"
                    >
                      Disconnect
                    </button>
                  </div>
                </li>
              );
            })}
          </ul>
        )}
      </div>

      {styleAccount && (
        <AccountStylePanel
          accountId={styleAccount.id}
          accountName={styleAccount.display_name}
          globalSettings={globalSettings}
          onClose={() => setStyleAccount(null)}
          onSaved={() => {
            setStyleAccount(null);
            onStyleSaved?.();
          }}
        />
      )}
    </section>
  );
}
