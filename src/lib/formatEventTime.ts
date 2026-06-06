function startOfDay(date: Date): Date {
  return new Date(date.getFullYear(), date.getMonth(), date.getDate());
}

export function formatEventDayLabel(iso: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) {
    return "Unknown date";
  }

  const today = startOfDay(new Date());
  const target = startOfDay(date);
  const diffDays = Math.round((target.getTime() - today.getTime()) / 86_400_000);

  if (diffDays === 0) {
    return "Today";
  }
  if (diffDays === 1) {
    return "Tomorrow";
  }
  if (diffDays > 1 && diffDays < 7) {
    return date.toLocaleDateString(undefined, { weekday: "long" });
  }

  return date.toLocaleDateString(undefined, {
    weekday: "short",
    month: "short",
    day: "numeric",
  });
}

export function formatEventTime(iso: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) {
    return "Unknown time";
  }

  return date.toLocaleTimeString(undefined, {
    hour: "numeric",
    minute: "2-digit",
  });
}

export function formatReminderLead(startIso: string, reminderIso: string): string | null {
  const start = new Date(startIso).getTime();
  const reminder = new Date(reminderIso).getTime();
  if (Number.isNaN(start) || Number.isNaN(reminder)) {
    return null;
  }

  const minutes = Math.round((start - reminder) / 60_000);
  if (minutes <= 0) {
    return "At start time";
  }
  if (minutes < 60) {
    return `${minutes}m before`;
  }
  if (minutes % 60 === 0) {
    return `${minutes / 60}h before`;
  }
  const hours = Math.floor(minutes / 60);
  const mins = minutes % 60;
  return `${hours}h ${mins}m before`;
}

export function dayGroupKey(iso: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) {
    return "unknown";
  }
  return `${date.getFullYear()}-${date.getMonth()}-${date.getDate()}`;
}
