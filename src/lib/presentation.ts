export function timeAgo(epoch: number): string {
  const diff = Math.max(0, Date.now() - epoch);
  if (diff < 60_000) return "just now";
  if (diff < 3_600_000) return `${Math.floor(diff / 60_000)}m ago`;
  if (diff < 86_400_000) return `${Math.floor(diff / 3_600_000)}h ago`;
  return `${Math.floor(diff / 86_400_000)}d ago`;
}

export function formatTimelineTime(epoch: number): string {
  return new Intl.DateTimeFormat(undefined, { hour: "numeric", minute: "2-digit" }).format(new Date(epoch));
}

export function formatDuration(milliseconds: number): string {
  const minutes = Math.max(0, Math.round(milliseconds / 60_000));
  if (minutes < 60) return `${minutes}m`;
  const hours = Math.floor(minutes / 60);
  const remainder = minutes % 60;
  return remainder ? `${hours}h ${remainder}m` : `${hours}h`;
}

export function formatSessionRange(startedAt: number, endedAt: number): string {
  const start = new Date(startedAt);
  const end = new Date(endedAt);
  const sameDay = start.toDateString() === end.toDateString();
  const date = new Intl.DateTimeFormat(undefined, { month: "short", day: "numeric" }).format(start);
  const time = new Intl.DateTimeFormat(undefined, { hour: "numeric", minute: "2-digit" });
  return `${date}, ${time.format(start)}${sameDay ? " - " : " - " + new Intl.DateTimeFormat(undefined, { month: "short", day: "numeric" }).format(end) + ", "}${time.format(end)}`;
}

export function topicColor(topic: string): string {
  const palette = ["#B8D8C3", "#AFCFE1", "#DAB8C5", "#D9D89D", "#C8B8DD", "#E7BE9B"];
  const hash = [...topic].reduce((value, char) => ((value * 31) + char.charCodeAt(0)) >>> 0, 7);
  return palette[hash % palette.length];
}

export function initials(value: string): string {
  return value.split(/[.\s-]+/).filter(Boolean).slice(0, 2).map((part) => part[0]?.toUpperCase()).join("") || "M";
}
