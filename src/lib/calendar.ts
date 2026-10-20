// Upcoming events from an ICS calendar feed (Google/Outlook "secret address").
// Handles time zones, all-day events, simple daily/weekly repeats, EXDATE,
// moved occurrences (RECURRENCE-ID) and cancelled events.

export type CalEvent = { title: string; start: number; end: number; allDay: boolean; location: string };

type Props = Map<string, { value: string; params: Record<string, string> }[]>;

/** Unfolds continuation lines and splits each VEVENT into its properties. */
function vevents(ics: string): Props[] {
  const lines = ics.replace(/\r?\n[ \t]/g, "").split(/\r?\n/);
  const events: Props[] = [];
  let cur: Props | null = null;
  for (const line of lines) {
    if (line === "BEGIN:VEVENT") cur = new Map();
    else if (line === "END:VEVENT") {
      if (cur) events.push(cur);
      cur = null;
    } else if (cur) {
      const colon = line.indexOf(":");
      if (colon < 0) continue;
      const [name, ...rawParams] = line.slice(0, colon).split(";");
      const params = Object.fromEntries(rawParams.map((p) => p.split("=") as [string, string]));
      const list = cur.get(name.toUpperCase()) ?? [];
      list.push({ value: line.slice(colon + 1), params });
      cur.set(name.toUpperCase(), list);
    }
  }
  return events;
}

const unescape = (s: string) => s.replace(/\\n/gi, " ").replace(/\\([,;\\])/g, "$1");

/** Offset (ms) of time zone `tz` from UTC at instant `utc`; throws for unknown zones. */
function tzOffset(utc: number, tz: string): number {
  const parts = new Intl.DateTimeFormat("en-US", {
    timeZone: tz,
    hourCycle: "h23",
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  }).formatToParts(utc);
  const get = (t: string) => Number(parts.find((p) => p.type === t)!.value);
  return Date.UTC(get("year"), get("month") - 1, get("day"), get("hour"), get("minute"), get("second")) - utc;
}

/** "20261019T090000Z", "20261019T090000" (+TZID) or "20261019" (all day) → epoch ms. */
function parseDate(value: string, tz?: string): { t: number; allDay: boolean } {
  const m = value.match(/^(\d{4})(\d{2})(\d{2})(?:T(\d{2})(\d{2})(\d{2})(Z)?)?$/);
  if (!m) return { t: NaN, allDay: false };
  const [, y, mo, d, h = "0", mi = "0", s = "0", z] = m;
  const fields = [+y, +mo - 1, +d, +h, +mi, +s] as const;
  if (m[4] === undefined) return { t: new Date(...fields).getTime(), allDay: true };
  if (z) return { t: Date.UTC(...fields), allDay: false };
  if (tz) {
    try {
      const guess = Date.UTC(...fields);
      return { t: guess - tzOffset(guess, tz), allDay: false };
    } catch {
      // Windows zone names (e.g. "SE Asia Standard Time"): fall back to local time.
    }
  }
  return { t: new Date(...fields).getTime(), allDay: false };
}

const DAYS = ["SU", "MO", "TU", "WE", "TH", "FR", "SA"];
const DAY = 24 * 3600_000;

/** Start times of a (possibly repeating) event that fall before `until`. */
function occurrences(start: number, rrule: string | undefined, until: number): number[] {
  if (!rrule) return [start];
  const r = Object.fromEntries(rrule.split(";").map((p) => p.split("=") as [string, string]));
  const interval = Number(r.INTERVAL ?? 1);
  const last = r.UNTIL ? Math.min(until, parseDate(r.UNTIL).t) : until;
  const count = r.COUNT ? Number(r.COUNT) : Infinity;
  const out: number[] = [];
  if (r.FREQ === "DAILY") {
    for (let t = start, n = 0; t <= last && n < count && out.length < 2000; t += interval * DAY, n++) out.push(t);
  } else if (r.FREQ === "WEEKLY") {
    const s = new Date(start);
    const days = r.BYDAY ? r.BYDAY.split(",").map((d: string) => DAYS.indexOf(d.slice(-2))) : [s.getDay()];
    const weekStart = new Date(s.getFullYear(), s.getMonth(), s.getDate() - s.getDay(), s.getHours(), s.getMinutes());
    let n = 0;
    for (let w = 0; out.length < 2000; w += interval) {
      for (const d of [...days].sort()) {
        const t = new Date(weekStart.getFullYear(), weekStart.getMonth(), weekStart.getDate() + w * 7 + d, s.getHours(), s.getMinutes()).getTime();
        if (t < start) continue;
        if (t > last || n >= count) return out;
        out.push(t);
        n++;
      }
    }
  } else {
    // Monthly/yearly repeats: only the first occurrence is known here.
    out.push(start);
  }
  return out;
}

/** Events that haven't ended yet, starting before tomorrow ends, soonest first. */
export function upcoming(ics: string, now = Date.now()): CalEvent[] {
  const today = new Date(now);
  const until = new Date(today.getFullYear(), today.getMonth(), today.getDate() + 2).getTime();
  const all = vevents(ics);

  // Moved or cancelled single occurrences of repeating events, by UID.
  const moved = new Map<string, Set<number>>();
  for (const e of all) {
    const rid = e.get("RECURRENCE-ID")?.[0];
    const uid = e.get("UID")?.[0]?.value;
    if (rid && uid) moved.set(uid, (moved.get(uid) ?? new Set()).add(parseDate(rid.value, rid.params.TZID).t));
  }

  const events: CalEvent[] = [];
  for (const e of all) {
    if (e.get("STATUS")?.[0]?.value === "CANCELLED") continue;
    const dtstart = e.get("DTSTART")?.[0];
    if (!dtstart) continue;
    const start = parseDate(dtstart.value, dtstart.params.TZID);
    if (isNaN(start.t)) continue;
    const dtend = e.get("DTEND")?.[0];
    const length = dtend ? parseDate(dtend.value, dtend.params.TZID).t - start.t : start.allDay ? DAY : 0;
    const uid = e.get("UID")?.[0]?.value ?? "";
    const rrule = e.has("RECURRENCE-ID") ? undefined : e.get("RRULE")?.[0]?.value;
    const skip = new Set([
      ...(e.get("EXDATE") ?? []).flatMap((x) => x.value.split(",").map((v) => parseDate(v, x.params.TZID).t)),
      ...(rrule ? (moved.get(uid) ?? []) : []),
    ]);
    for (const t of occurrences(start.t, rrule, until)) {
      if (skip.has(t) || t + length <= now || t >= until) continue;
      events.push({
        title: unescape(e.get("SUMMARY")?.[0]?.value ?? "(tanpa judul)"),
        start: t,
        end: t + length,
        allDay: start.allDay,
        location: unescape(e.get("LOCATION")?.[0]?.value ?? ""),
      });
    }
  }
  return events.sort((a, b) => a.start - b.start);
}
