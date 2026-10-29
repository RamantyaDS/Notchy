import type { NetworkSpeed } from "./network";

// What the closed notch shows when several things run at once (notchy.md §5F.1,
// style 1B): one pill, the main activity first, up to two more at its right end.

export type Call = { app: string; start: number };
export type Download = { name: string; bytes: number; speed: number };

export type Activity =
  | { kind: "call"; call: Call }
  | { kind: "timer" }
  | { kind: "download"; download: Download }
  | { kind: "music"; lyric: boolean }
  | { kind: "network"; speed: NetworkSpeed };

const ORDER: Activity["kind"][] = ["call", "timer", "download", "music", "network"];

/** Main activity and the ones beside it. `turn` (from horizontal swipes) cycles the main one. */
export function arrange(activities: Activity[], turn = 0): { main: Activity | null; side: Activity[] } {
  if (!activities.length) return { main: null, side: [] };
  let list = [...activities].sort((a, b) => ORDER.indexOf(a.kind) - ORDER.indexOf(b.kind));
  // A sung line needs the room, so music with lyrics stays in front.
  const sung = list.find((a) => a.kind === "music" && a.lyric);
  if (sung) list = [sung, ...list.filter((a) => a !== sung)];
  const n = list.length;
  const r = ((turn % n) + n) % n;
  list = [...list.slice(r), ...list.slice(0, r)];
  return { main: list[0], side: list.slice(1, 3) };
}

/** Width of a main activity, logical px (notchy.md §4). */
export function mainWidth(a: Activity): number {
  switch (a.kind) {
    case "music":
      return a.lyric ? 380 : 200;
    case "call":
      return 230;
    case "download":
      return 260;
    case "timer":
      return 200;
    case "network":
      return 210;
  }
}

/** Width of an activity shown beside the main one, divider included. */
export function sideWidth(a: Activity): number {
  switch (a.kind) {
    case "call":
      return 86;
    case "timer":
      return 84;
    case "download":
      return 92;
    case "music":
      return 56;
    case "network":
      return 82;
  }
}

export function islandWidth(main: Activity, side: Activity[]): number {
  return mainWidth(main) + side.reduce((w, a) => w + sideWidth(a), 0);
}

/** "12:34", or "1:02:03" past an hour. */
export function duration(ms: number): string {
  const s = Math.max(0, Math.floor(ms / 1000));
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const ss = String(s % 60).padStart(2, "0");
  return h ? `${h}:${String(m).padStart(2, "0")}:${ss}` : `${m}:${ss}`;
}

/** "12,4 MB" style sizes, Indonesian decimal comma. */
export function bytes(n: number): string {
  const fmt = (v: number, unit: string) => `${v.toLocaleString("id-ID", { maximumFractionDigits: v < 10 ? 1 : 0 })} ${unit}`;
  if (n >= 1e9) return fmt(n / 1e9, "GB");
  if (n >= 1e6) return fmt(n / 1e6, "MB");
  if (n >= 1e3) return fmt(n / 1e3, "KB");
  return `${n} B`;
}
