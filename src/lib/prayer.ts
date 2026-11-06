/**
 * Precise, offline astronomical prayer times calculation.
 * Follows the standard astronomical algorithms used by Kemenag RI, MWL, and Egyptian Survey.
 * Zero external dependencies, 100% offline, zero network latency.
 */

export type PrayerMethod = "kemenag" | "mwl" | "egyptian" | "isna";

export interface MethodParams {
  fajrAngle: number;
  ishaAngle: number;
  dhuhrBufferMin: number;
  maghribBufferMin: number;
}

export const METHODS: Record<PrayerMethod, MethodParams> = {
  kemenag: {
    fajrAngle: 20, // Kemenag RI standard: Subuh 20°
    ishaAngle: 18, // Kemenag RI standard: Isya 18°
    dhuhrBufferMin: 2, // Ihtiyat / precaution: +2 min
    maghribBufferMin: 2,
  },
  mwl: {
    fajrAngle: 18, // Muslim World League
    ishaAngle: 17,
    dhuhrBufferMin: 1,
    maghribBufferMin: 1,
  },
  egyptian: {
    fajrAngle: 19.5, // Egyptian General Authority of Survey
    ishaAngle: 17.5,
    dhuhrBufferMin: 1,
    maghribBufferMin: 1,
  },
  isna: {
    fajrAngle: 15, // Islamic Society of North America
    ishaAngle: 15,
    dhuhrBufferMin: 1,
    maghribBufferMin: 1,
  },
};

export interface PrayerTimes {
  fajr: string;
  sunrise: string;
  dhuhr: string;
  asr: string;
  maghrib: string;
  isha: string;
  // Raw Date objects for precise timers
  dates: {
    fajr: Date;
    sunrise: Date;
    dhuhr: Date;
    asr: Date;
    maghrib: Date;
    isha: Date;
  };
}

export type PrayerName = "fajr" | "dhuhr" | "asr" | "maghrib" | "isha";

export interface NextPrayerInfo {
  name: PrayerName;
  time: string;
  date: Date;
  remainingMinutes: number;
  isPreAlert: boolean; // true when within 10 minutes before
  isExact: boolean; // true when within the 1-minute alert window
}

// Math helpers in degrees
const rad = (deg: number) => (deg * Math.PI) / 180;
const deg = (rad: number) => (rad * 180) / Math.PI;
const dsin = (d: number) => Math.sin(rad(d));
const dcos = (d: number) => Math.cos(rad(d));
const dtan = (d: number) => Math.tan(rad(d));
const darcsin = (x: number) => deg(Math.asin(Math.max(-1, Math.min(1, x))));
const darccos = (x: number) => deg(Math.acos(Math.max(-1, Math.min(1, x))));
const darctan = (x: number) => deg(Math.atan(x));
const darctan2 = (y: number, x: number) => deg(Math.atan2(y, x));
const fixAngle = (a: number) => ((a % 360) + 360) % 360;
const fixHour = (h: number) => ((h % 24) + 24) % 24;

/**
 * Calculates prayer times for a given geographic coordinate and date.
 */
export function calculatePrayerTimes(
  latitude: number,
  longitude: number,
  date: Date = new Date(),
  method: PrayerMethod = "kemenag",
  timezoneOffsetHours?: number
): PrayerTimes {
  const params = METHODS[method] ?? METHODS.kemenag;
  const tz = timezoneOffsetHours ?? -date.getTimezoneOffset() / 60;

  const year = date.getFullYear();
  const month = date.getMonth() + 1;
  const day = date.getDate();

  // Julian Day
  let Y = year;
  let M = month;
  if (M <= 2) {
    Y -= 1;
    M += 12;
  }
  const A = Math.floor(Y / 100);
  const B = 2 - A + Math.floor(A / 4);
  const JD = Math.floor(365.25 * (Y + 4716)) + Math.floor(30.6001 * (M + 1)) + day + B - 1524.5;

  // Sun position calculation
  const D = JD - 2451545.0;
  const g = fixAngle(357.529 + 0.98560028 * D);
  const q = fixAngle(280.459 + 0.98564736 * D);
  const L = fixAngle(q + 1.915 * dsin(g) + 0.02 * dsin(2 * g));
  const e = 23.439 - 0.00000036 * D;

  // Right ascension and Declination
  const RA = fixHour(darctan2(dcos(e) * dsin(L), dcos(L)) / 15);
  const declination = darcsin(dsin(e) * dsin(L));

  // Equation of Time (hours)
  const EqT = q / 15 - RA;

  // Solar noon / transit (hours)
  const transit = fixHour(12 + tz - longitude / 15 - EqT);

  // Hour angle for an altitude/depression angle
  const hourAngle = (alpha: number): number => {
    const val = (dsin(alpha) - dsin(latitude) * dsin(declination)) / (dcos(latitude) * dcos(declination));
    if (val > 1 || val < -1) return 0; // Polar day/night fallback
    return darccos(val) / 15;
  };

  // Asr altitude angle calculation (Standard Shafi'i shadow ratio = 1)
  // cot(h) = 1 + tan(|lat - dec|) => h = atan(1 / (1 + tan(|lat - dec|)))
  const asrAlt = darctan(1 / (1 + dtan(Math.abs(latitude - declination))));
  const asrHourAngle = hourAngle(asrAlt);

  // Calculations in hours
  const fajrHours = fixHour(transit - hourAngle(-params.fajrAngle));
  const sunriseHours = fixHour(transit - hourAngle(-0.8333));
  const dhuhrHours = fixHour(transit + params.dhuhrBufferMin / 60);
  const asrHours = fixHour(transit + asrHourAngle);
  const maghribHours = fixHour(transit + hourAngle(-0.8333) + params.maghribBufferMin / 60);
  const ishaHours = fixHour(transit + hourAngle(-params.ishaAngle) + params.maghribBufferMin / 60);

  // Helper to convert decimal hours to Date and string
  const toDate = (hours: number): Date => {
    const totalMinutes = Math.round(hours * 60);
    const h = Math.floor(totalMinutes / 60) % 24;
    const m = totalMinutes % 60;
    const d = new Date(date);
    d.setHours(h, m, 0, 0);
    return d;
  };

  const pad = (n: number) => String(n).padStart(2, "0");
  const toStr = (d: Date) => `${pad(d.getHours())}:${pad(d.getMinutes())}`;

  const dFajr = toDate(fajrHours);
  const dSunrise = toDate(sunriseHours);
  const dDhuhr = toDate(dhuhrHours);
  const dAsr = toDate(asrHours);
  const dMaghrib = toDate(maghribHours);
  const dIsha = toDate(ishaHours);

  return {
    fajr: toStr(dFajr),
    sunrise: toStr(dSunrise),
    dhuhr: toStr(dDhuhr),
    asr: toStr(dAsr),
    maghrib: toStr(dMaghrib),
    isha: toStr(dIsha),
    dates: {
      fajr: dFajr,
      sunrise: dSunrise,
      dhuhr: dDhuhr,
      asr: dAsr,
      maghrib: dMaghrib,
      isha: dIsha,
    },
  };
}

/**
 * Finds the next upcoming prayer or active prayer alert based on current time.
 */
export function getNextPrayer(
  times: PrayerTimes,
  now: Date = new Date(),
  enabledPrayers: Record<PrayerName, boolean> = {
    fajr: true,
    dhuhr: true,
    asr: true,
    maghrib: true,
    isha: true,
  }
): NextPrayerInfo | null {
  const prayerList: { name: PrayerName; time: string; date: Date }[] = [
    { name: "fajr" as const, time: times.fajr, date: times.dates.fajr },
    { name: "dhuhr" as const, time: times.dhuhr, date: times.dates.dhuhr },
    { name: "asr" as const, time: times.asr, date: times.dates.asr },
    { name: "maghrib" as const, time: times.maghrib, date: times.dates.maghrib },
    { name: "isha" as const, time: times.isha, date: times.dates.isha },
  ].filter((p) => enabledPrayers[p.name]);

  if (!prayerList.length) return null;

  const nowMs = now.getTime();

  for (const p of prayerList) {
    const diffMs = p.date.getTime() - nowMs;

    // If within 1 minute of prayer time (T-0 window: -59s to +59s)
    if (Math.abs(diffMs) <= 60_000) {
      return {
        name: p.name,
        time: p.time,
        date: p.date,
        remainingMinutes: 0,
        isPreAlert: false,
        isExact: true,
      };
    }

    const diffMinutes = Math.ceil(diffMs / 60_000);

    // If within 10 minutes before (T-10 pre-alert window: 1 to 10 minutes)
    if (diffMinutes > 0 && diffMinutes <= 10) {
      return {
        name: p.name,
        time: p.time,
        date: p.date,
        remainingMinutes: diffMinutes,
        isPreAlert: true,
        isExact: false,
      };
    }

    // Next future prayer
    if (diffMs > 0) {
      return {
        name: p.name,
        time: p.time,
        date: p.date,
        remainingMinutes: diffMinutes,
        isPreAlert: false,
        isExact: false,
      };
    }
  }

  // If all prayers today have passed, return tomorrow's Fajr
  const tomorrow = new Date(now);
  tomorrow.setDate(tomorrow.getDate() + 1);
  const tomorrowFajr = new Date(times.dates.fajr);
  tomorrowFajr.setDate(tomorrowFajr.getDate() + 1);
  const diffMinutes = Math.ceil((tomorrowFajr.getTime() - nowMs) / 60_000);

  return {
    name: "fajr",
    time: times.fajr,
    date: tomorrowFajr,
    remainingMinutes: diffMinutes,
    isPreAlert: false,
    isExact: false,
  };
}
