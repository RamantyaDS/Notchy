// Countdowns on the Today page: "Payday in 16 days", repeating monthly or yearly.

export type Countdown = { name: string; date: string; repeat?: string };

const DAY = 86_400_000;
const midnight = (d: Date) => new Date(d.getFullYear(), d.getMonth(), d.getDate());

/** The next date the countdown falls on, today included; null when it's past or invalid. */
export function nextDate(c: Countdown, today: Date): Date | null {
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(c.date ?? "");
  if (!m) return null;
  const [y, mo, d] = [+m[1], +m[2] - 1, +m[3]];
  const start = midnight(today);
  // A day past the end of the month (e.g. the 31st) lands on the month's last day.
  const at = (year: number, month: number) => new Date(year, month, Math.min(d, new Date(year, month + 1, 0).getDate()));
  let date = at(y, mo);
  if (c.repeat === "monthly") {
    date = at(start.getFullYear(), start.getMonth());
    if (date < start) date = at(start.getFullYear(), start.getMonth() + 1);
  } else if (c.repeat === "yearly") {
    date = at(start.getFullYear(), mo);
    if (date < start) date = at(start.getFullYear() + 1, mo);
  }
  return date < start ? null : date;
}

/** Whole days until the countdown (0 = today), or null when it's over. */
export function daysUntil(c: Countdown, today: Date): number | null {
  const next = nextDate(c, today);
  return next ? Math.round((next.getTime() - midnight(today).getTime()) / DAY) : null;
}

/** Upcoming countdowns, nearest first. */
export function upcomingCountdowns(list: Countdown[], today: Date): { name: string; days: number }[] {
  return list
    .filter((c) => c.name?.trim())
    .map((c) => ({ name: c.name.trim(), days: daysUntil(c, today) }))
    .filter((c): c is { name: string; days: number } => c.days !== null)
    .sort((a, b) => a.days - b.days);
}
