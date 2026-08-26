/** Relative timestamp for history rows. */
export function relativeTime(createdAtMs: number, nowMs: number): string {
  const delta = Math.max(0, nowMs - createdAtMs);
  const seconds = Math.floor(delta / 1000);
  if (seconds < 10) {
    return "just now";
  }
  if (seconds < 60) {
    return `${seconds}s ago`;
  }
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) {
    return `${minutes} min ago`;
  }
  const hours = Math.floor(minutes / 60);
  if (hours < 24) {
    return `${hours}h ago`;
  }
  const days = Math.floor(hours / 24);
  return `${days}d ago`;
}
