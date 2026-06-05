export function formatRelativeTime(iso: string | null): string {
  if (!iso) {
    return "Never";
  }

  const then = new Date(iso).getTime();
  if (Number.isNaN(then)) {
    return "Unknown";
  }

  const diffSec = Math.floor((Date.now() - then) / 1000);
  if (diffSec < 60) {
    return "Just now";
  }
  if (diffSec < 3600) {
    return `${Math.floor(diffSec / 60)}m ago`;
  }
  if (diffSec < 86400) {
    return `${Math.floor(diffSec / 3600)}h ago`;
  }
  return `${Math.floor(diffSec / 86400)}d ago`;
}
