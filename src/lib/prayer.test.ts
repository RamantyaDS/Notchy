import { describe, it, expect } from "vitest";
import { calculatePrayerTimes, getNextPrayer } from "./prayer";

describe("prayer times calculation", () => {
  it("calculates valid formatted prayer times for Jakarta (Kemenag standard)", () => {
    // Jakarta: lat -6.2088, lon 106.8456, UTC+7
    const date = new Date("2026-11-06T12:00:00+07:00");
    const times = calculatePrayerTimes(-6.2088, 106.8456, date, "kemenag", 7);

    // Validate format HH:mm
    const timeRegex = /^\d{2}:\d{2}$/;
    expect(times.fajr).toMatch(timeRegex);
    expect(times.sunrise).toMatch(timeRegex);
    expect(times.dhuhr).toMatch(timeRegex);
    expect(times.asr).toMatch(timeRegex);
    expect(times.maghrib).toMatch(timeRegex);
    expect(times.isha).toMatch(timeRegex);

    // Astronomical order check: fajr < sunrise < dhuhr < asr < maghrib < isha
    expect(times.fajr < times.sunrise).toBe(true);
    expect(times.sunrise < times.dhuhr).toBe(true);
    expect(times.dhuhr < times.asr).toBe(true);
    expect(times.asr < times.maghrib).toBe(true);
    expect(times.maghrib < times.isha).toBe(true);

    // Approximate Jakarta times in November:
    // Subuh ~04:00-04:25, Dzuhur ~11:35-11:55, Ashar ~14:50-15:20, Maghrib ~17:45-18:10, Isya ~18:55-19:25
    const [fajrH] = times.fajr.split(":").map(Number);
    const [dhuhrH] = times.dhuhr.split(":").map(Number);
    const [asrH] = times.asr.split(":").map(Number);
    const [maghribH] = times.maghrib.split(":").map(Number);
    const [ishaH] = times.isha.split(":").map(Number);

    expect(fajrH).toBe(4);
    expect(dhuhrH).toBe(11);
    expect(asrH === 14 || asrH === 15).toBe(true);
    expect(maghribH).toBe(17);
    expect(ishaH).toBe(19);
  });

  it("calculates prayer times for Makkah (MWL standard)", () => {
    // Makkah: lat 21.4225, lon 39.8262, UTC+3
    const date = new Date("2026-11-06T12:00:00+03:00");
    const times = calculatePrayerTimes(21.4225, 39.8262, date, "mwl", 3);

    expect(times.fajr).toMatch(/^\d{2}:\d{2}$/);
    expect(times.dhuhr).toMatch(/^\d{2}:\d{2}$/);
    expect(times.asr).toMatch(/^\d{2}:\d{2}$/);
    expect(times.maghrib).toMatch(/^\d{2}:\d{2}$/);
    expect(times.isha).toMatch(/^\d{2}:\d{2}$/);
  });

  it("detects exact and 10-minute pre-alert prayer triggers", () => {
    const date = new Date("2026-11-06T12:00:00+07:00");
    const times = calculatePrayerTimes(-6.2088, 106.8456, date, "kemenag", 7);

    // 1. Simulate 8 minutes before Dzuhur (T-10 window)
    const dhuhrDate = new Date(times.dates.dhuhr);
    const eightMinBefore = new Date(dhuhrDate.getTime() - 8 * 60_000);

    const preAlert = getNextPrayer(times, eightMinBefore);
    expect(preAlert).not.toBeNull();
    expect(preAlert?.name).toBe("dhuhr");
    expect(preAlert?.isPreAlert).toBe(true);
    expect(preAlert?.isExact).toBe(false);
    expect(preAlert?.remainingMinutes).toBe(8);

    // 2. Simulate exact Dzuhur time (T-0)
    const exactDhuhr = new Date(dhuhrDate.getTime() + 10_000); // 10 seconds into Dzuhur
    const exactAlert = getNextPrayer(times, exactDhuhr);
    expect(exactAlert).not.toBeNull();
    expect(exactAlert?.name).toBe("dhuhr");
    expect(exactAlert?.isPreAlert).toBe(false);
    expect(exactAlert?.isExact).toBe(true);
  });
});
