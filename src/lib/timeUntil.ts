import { useEffect, useState } from "react";

export function formatTimeUntil(startTime: string, now = new Date()): string {
  const start = new Date(startTime);
  const diffMs = start.getTime() - now.getTime();

  if (Number.isNaN(start.getTime())) {
    return "—";
  }
  if (diffMs <= 0) {
    return "Starting now";
  }

  const totalSeconds = Math.floor(diffMs / 1000);
  const days = Math.floor(totalSeconds / 86400);
  const hours = Math.floor((totalSeconds % 86400) / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  if (days > 0) {
    return `in ${days}d ${hours}h`;
  }
  if (hours > 0) {
    return `in ${hours}h ${minutes}m`;
  }
  if (minutes > 0) {
    return `in ${minutes}m ${seconds}s`;
  }
  return `in ${seconds}s`;
}

export function useTimeUntil(startTime: string | null): string {
  const [label, setLabel] = useState(() =>
    startTime ? formatTimeUntil(startTime) : "—",
  );

  useEffect(() => {
    if (!startTime) {
      setLabel("—");
      return;
    }

    const tick = () => setLabel(formatTimeUntil(startTime));
    tick();
    const id = window.setInterval(tick, 1000);
    return () => window.clearInterval(id);
  }, [startTime]);

  return label;
}

export function previewStartTime(minutesFromNow = 15): string {
  return new Date(Date.now() + minutesFromNow * 60_000).toISOString();
}
