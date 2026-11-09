import { describe, expect, it } from "vitest";
import { firstRain } from "./weather";

describe("rain alert", () => {
  // 14:00 local time in Jakarta (UTC+7) is 07:00 UTC.
  const now = Date.parse("2026-11-09T07:00:00Z");
  const times = ["2026-11-09T14:15", "2026-11-09T14:30", "2026-11-09T14:45", "2026-11-09T15:00"];

  it("finds the first wet slot within the hour", () => {
    expect(firstRain(times, [0, 0, 1.2, 3], 7 * 3600, now)).toBe(30);
    expect(firstRain(times, [0.5, 0, 0, 0], 7 * 3600, now)).toBe(0);
  });

  it("ignores drizzle and dry hours", () => {
    expect(firstRain(times, [0, 0.1, 0, 0], 7 * 3600, now)).toBeNull();
    expect(firstRain([], [], 0, now)).toBeNull();
  });
});
