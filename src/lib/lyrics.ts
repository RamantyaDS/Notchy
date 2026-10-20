// Synced lyrics from LRCLIB (lrclib.net): free, no login.

export type Line = { time: number; text: string };
export type Lyrics = { synced: Line[] | null; plain: string | null };

const API = "https://lrclib.net/api";
const BROWSER = /chrome|msedge|edge|firefox|brave|opera|vivaldi|arc/i;
/** Bracketed video-title noise: "(Official Music Video)", "[Lyrics]", "(Lirik)", ... */
const NOISE =
  /\s*[([][^)\]]*?(official|video|audio|lyrics?|lirik|visuali[sz]er|mv|m\/v|hd|4k|remaster(ed)?|live|explicit)[^)\]]*?[)\]]/gi;

/** The same noise without brackets at the end: "... LYRIC OFFICIAL", "... Official Music Video". */
const TRAILING_NOISE =
  /(\s+(official|lyrics?|lirik|music|video|audio|visuali[sz]er|mv|hd|4k|live|explicit))+\s*$/i;

/**
 * Turns what the player reports into something LRCLIB can match. Browsers
 * report the video title and channel ("Tulus - Hati-Hati di Jalan (Official
 * Video)" by "TulusVEVO"), so there the artist is taken from the title.
 * The result is also what the player shows.
 */
export function cleanTrack(title: string, artist: string, source: string) {
  let t = title.replace(NOISE, "").replace(/\s*\|.*$/, "").replace(TRAILING_NOISE, "").trim();
  let a = artist.replace(/\s*-\s*Topic$/i, "").replace(/\s*VEVO$/i, "").replace(/\s+Official$/i, "").trim();
  const m = t.match(/^(.+?)\s+[-–—]\s+(.+)$/);
  if (m) {
    const [left, right] = [m[1].trim(), m[2].trim()];
    const same = (x: string) => x.toLowerCase() === a.toLowerCase();
    // "Title - Artist" when the right side is the reported artist; "Artist - Title"
    // when the left side is, or for any "x - y" from a browser (where the reported
    // artist is often just the channel name).
    if (same(right)) t = left;
    else if (same(left) || BROWSER.test(source)) [a, t] = [left, right];
  }
  t = t
    .replace(/\s+-\s+.*remaster.*$/i, "")
    .replace(/\s*[([]?\b(feat|ft)\.?\s.*$/i, "")
    .replace(/["“”]/g, "")
    .trim();
  return { title: t, artist: a };
}

/** Parses LRC text: "[01:23.45] line". A line may carry several timestamps. */
export function parseLrc(lrc: string): Line[] {
  const lines: Line[] = [];
  for (const raw of lrc.split(/\r?\n/)) {
    const stamps = [...raw.matchAll(/\[(\d+):(\d+(?:\.\d+)?)\]/g)];
    if (!stamps.length) continue;
    const text = raw.replace(/\[[^\]]*\]/g, "").trim() || "♪";
    for (const s of stamps) lines.push({ time: Number(s[1]) * 60 + Number(s[2]), text });
  }
  return lines.sort((a, b) => a.time - b.time);
}

/** Index of the line being sung at `position`, or -1 before the first one. */
export function currentLine(lines: Line[], position: number): number {
  let i = -1;
  for (let j = 0; j < lines.length && lines[j].time <= position + 0.25; j++) i = j;
  return i;
}

async function getJson(url: string) {
  const res = await fetch(url);
  return res.ok ? res.json() : null;
}

type Record = { duration: number; syncedLyrics: string | null; plainLyrics: string | null };

/**
 * Finds lyrics, preferring synced ones whose length matches the song.
 * Hits are cached in localStorage; misses are not, so newly added lyrics show up later.
 * Returns null when LRCLIB can't be reached.
 */
export async function findLyrics(title: string, artist: string, album: string, duration: number): Promise<Lyrics | null> {
  const key = `lyrics:v1:${artist}|${title}|${Math.round(duration)}`;
  try {
    const cached = localStorage.getItem(key);
    if (cached) return JSON.parse(cached);
  } catch {}

  try {
    let hit: Record | null = null;
    const q = new URLSearchParams({ track_name: title, artist_name: artist });
    if (duration > 0 && album) {
      const exact = new URLSearchParams(q);
      exact.set("album_name", album);
      exact.set("duration", String(Math.round(duration)));
      hit = await getJson(`${API}/get?${exact}`);
    }
    if (!hit?.syncedLyrics) {
      let pool: Record[] = (await getJson(`${API}/search?${q}`)) ?? [];
      if (!pool.length) pool = (await getJson(`${API}/search?q=${encodeURIComponent(`${artist} ${title}`)}`)) ?? [];
      const fits = (r: Record) => !duration || Math.abs(r.duration - duration) < 6;
      hit = pool.find((r) => r.syncedLyrics && fits(r)) ?? pool.find((r) => r.syncedLyrics) ?? hit ?? pool.find((r) => r.plainLyrics) ?? null;
    }

    const lyrics: Lyrics = {
      synced: hit?.syncedLyrics ? parseLrc(hit.syncedLyrics) : null,
      plain: hit?.plainLyrics ?? null,
    };
    if (lyrics.synced || lyrics.plain) {
      try {
        localStorage.setItem(key, JSON.stringify(lyrics));
      } catch {}
    }
    return lyrics;
  } catch {
    return null;
  }
}
