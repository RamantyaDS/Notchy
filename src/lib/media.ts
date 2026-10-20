import { invoke } from "@tauri-apps/api/core";

/** Mirrors `NowPlaying` in src-tauri/src/media.rs. */
export type NowPlaying = {
  title: string;
  artist: string;
  album: string;
  art: string | null;
  source: string;
  playing: boolean;
  position: number;
  duration: number;
  /** Unix ms at which `position` was true (as reported by the player). */
  updatedAt: number;
  /** Unix ms at which Notchy received this state. */
  receivedAt: number;
};

/** Where playback is at time `now`, extrapolating while playing. */
export function positionAt(m: NowPlaying, now: number): number {
  if (!m.playing) return m.position;
  // Some players report a stale or missing timestamp; then count from when we heard about it.
  const since = m.updatedAt > 0 && now - m.updatedAt < 6 * 3600_000 ? m.updatedAt : m.receivedAt;
  const pos = m.position + (now - since) / 1000;
  return Math.max(0, m.duration > 0 ? Math.min(pos, m.duration) : pos);
}

export function formatTime(seconds: number): string {
  const s = Math.max(0, Math.floor(seconds));
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;
}

export const toggle = () => invoke("media_toggle");
export const next = () => invoke("media_next");
export const prev = () => invoke("media_prev");
export const seek = (seconds: number) => invoke("media_seek", { seconds });
