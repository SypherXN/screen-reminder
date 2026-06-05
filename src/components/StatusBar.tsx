import { formatRelativeTime } from "../lib/formatRelativeTime";
import type { SyncStatus } from "../lib/types";

interface Props {
  syncStatus: SyncStatus | null;
  paused: boolean;
  busy: boolean;
  onSync: () => void;
  onTogglePause: () => void;
  onTestReminder: () => void;
}

export function StatusBar({
  syncStatus,
  paused,
  busy,
  onSync,
  onTogglePause,
  onTestReminder,
}: Props) {
  const syncErrors = syncStatus?.accounts.filter((a) => a.last_error).length ?? 0;

  return (
    <div className="flex flex-wrap items-center gap-2">
      {syncStatus && (
        <span className="rounded-full bg-slate-900 px-3 py-1 text-xs text-slate-400">
          {syncStatus.account_count} accounts · {syncStatus.reminder_count} reminders
          {syncStatus.last_sync ? ` · synced ${formatRelativeTime(syncStatus.last_sync)}` : ""}
          {syncErrors > 0 ? ` · ${syncErrors} sync error${syncErrors > 1 ? "s" : ""}` : ""}
        </span>
      )}
      <button
        disabled={busy}
        onClick={onSync}
        className="rounded-lg bg-slate-800 px-3 py-2 text-sm hover:bg-slate-700 disabled:opacity-50"
      >
        Sync now
      </button>
      <button
        disabled={busy}
        onClick={onTogglePause}
        className="rounded-lg bg-slate-800 px-3 py-2 text-sm hover:bg-slate-700 disabled:opacity-50"
      >
        {paused ? "Resume" : "Pause"}
      </button>
      <button
        disabled={busy}
        onClick={onTestReminder}
        className="rounded-lg bg-indigo-600 px-3 py-2 text-sm hover:bg-indigo-500 disabled:opacity-50"
      >
        Test reminder
      </button>
    </div>
  );
}
