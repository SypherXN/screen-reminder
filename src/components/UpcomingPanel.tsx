import { useMemo, useState } from "react";
import {
  dayGroupKey,
  formatEventDayLabel,
  formatEventTime,
  formatReminderLead,
} from "../lib/formatEventTime";
import type { CalendarAccount, ReminderEvent } from "../lib/types";
import { SOURCE_LABELS, TASK_SOURCES } from "../lib/types";

type Filter = "all" | "events" | "tasks";

interface Props {
  reminders: ReminderEvent[];
  accounts: CalendarAccount[];
  busy: boolean;
  onSync: () => void;
  onOpenUrl: (url: string) => void;
}

function accountName(accounts: CalendarAccount[], accountId: string): string {
  return accounts.find((account) => account.id === accountId)?.display_name ?? "Unknown account";
}

function statusLabel(reminder: ReminderEvent): string | null {
  if (reminder.snoozed_until) {
    return `Snoozed until ${formatEventTime(reminder.snoozed_until)}`;
  }
  if (reminder.fired_at) {
    return "Reminder shown";
  }
  return null;
}

export function UpcomingPanel({
  reminders,
  accounts,
  busy,
  onSync,
  onOpenUrl,
}: Props) {
  const [filter, setFilter] = useState<Filter>("all");

  const filtered = useMemo(() => {
    return reminders.filter((reminder) => {
      const isTask = TASK_SOURCES.has(reminder.source);
      if (filter === "events") {
        return !isTask;
      }
      if (filter === "tasks") {
        return isTask;
      }
      return true;
    });
  }, [filter, reminders]);

  const grouped = useMemo(() => {
    const groups = new Map<string, ReminderEvent[]>();
    for (const reminder of filtered) {
      const key = dayGroupKey(reminder.start_time);
      const list = groups.get(key) ?? [];
      list.push(reminder);
      groups.set(key, list);
    }
    return [...groups.entries()];
  }, [filtered]);

  const eventCount = reminders.filter((r) => !TASK_SOURCES.has(r.source)).length;
  const taskCount = reminders.filter((r) => TASK_SOURCES.has(r.source)).length;

  return (
    <section className="space-y-6">
      <div className="rounded-xl border border-slate-800 bg-slate-900/60 p-5">
        <div className="mb-4 flex flex-wrap items-center justify-between gap-3">
          <div>
            <h2 className="text-lg font-medium">Upcoming events & tasks</h2>
            <p className="mt-1 text-sm text-slate-400">
              Synced from your connected accounts. Screen reminders fire at the reminder time, not
              necessarily when the event starts.
            </p>
          </div>
          <button
            type="button"
            disabled={busy}
            onClick={onSync}
            className="rounded-lg bg-indigo-600 px-3 py-2 text-sm hover:bg-indigo-500 disabled:opacity-50"
          >
            Refresh
          </button>
        </div>

        <div className="flex flex-wrap gap-2">
          {(
            [
              ["all", `All (${reminders.length})`],
              ["events", `Events (${eventCount})`],
              ["tasks", `Tasks (${taskCount})`],
            ] as const
          ).map(([value, label]) => (
            <button
              key={value}
              type="button"
              onClick={() => setFilter(value)}
              className={`rounded-lg px-3 py-1.5 text-sm ${
                filter === value
                  ? "bg-indigo-600 text-white"
                  : "bg-slate-800 text-slate-300 hover:bg-slate-700"
              }`}
            >
              {label}
            </button>
          ))}
        </div>
      </div>

      {accounts.length === 0 ? (
        <div className="rounded-xl border border-slate-800 bg-slate-900/60 p-5 text-sm text-slate-400">
          Connect a calendar or task account to see upcoming items here.
        </div>
      ) : filtered.length === 0 ? (
        <div className="rounded-xl border border-slate-800 bg-slate-900/60 p-5 text-sm text-slate-400">
          No upcoming {filter === "tasks" ? "tasks" : filter === "events" ? "events" : "items"} in
          the next sync window. Try refreshing sync from the header.
        </div>
      ) : (
        grouped.map(([key, items]) => (
          <div key={key} className="rounded-xl border border-slate-800 bg-slate-900/60 p-5">
            <h3 className="mb-3 text-sm font-medium text-indigo-300">
              {formatEventDayLabel(items[0].start_time)}
            </h3>
            <ul className="space-y-3">
              {items.map((reminder) => {
                const isTask = TASK_SOURCES.has(reminder.source);
                const lead = formatReminderLead(reminder.start_time, reminder.reminder_time);
                const status = statusLabel(reminder);
                return (
                  <li
                    key={reminder.id}
                    className="rounded-lg border border-slate-800 bg-slate-950 px-4 py-3"
                  >
                    <div className="flex items-start justify-between gap-4">
                      <div className="min-w-0 flex-1">
                        <div className="flex flex-wrap items-center gap-2">
                          <span className="font-medium">{reminder.title}</span>
                          <span
                            className={`rounded-full px-2 py-0.5 text-[10px] uppercase tracking-wide ${
                              isTask
                                ? "bg-amber-900/40 text-amber-200"
                                : "bg-blue-900/40 text-blue-200"
                            }`}
                          >
                            {isTask ? "Task" : "Event"}
                          </span>
                        </div>
                        <div className="mt-1 text-sm text-slate-300">
                          {formatEventTime(reminder.start_time)}
                          {lead ? ` · Reminder ${lead}` : ""}
                        </div>
                        <div className="mt-1 text-xs text-slate-500">
                          {accountName(accounts, reminder.account_id)}
                          {" · "}
                          {SOURCE_LABELS[reminder.source] ?? reminder.source}
                          {reminder.location ? ` · ${reminder.location}` : ""}
                        </div>
                        {status && (
                          <div className="mt-1 text-xs text-slate-400">{status}</div>
                        )}
                      </div>
                      {reminder.url && (
                        <button
                          type="button"
                          onClick={() => onOpenUrl(reminder.url!)}
                          className="shrink-0 rounded-lg bg-slate-800 px-3 py-1 text-sm hover:bg-slate-700"
                        >
                          Open
                        </button>
                      )}
                    </div>
                  </li>
                );
              })}
            </ul>
          </div>
        ))
      )}
    </section>
  );
}
