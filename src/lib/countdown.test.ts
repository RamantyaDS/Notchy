import { describe, expect, it } from "vitest";
import { daysUntil, upcomingCountdowns } from "./countdown";

const today = new Date(2026, 10, 9, 15, 30); // 9 Nov 2026, afternoon

describe("countdowns", () => {
  it("counts whole days to a one-off date", () => {
    expect(daysUntil({ name: "Natal", date: "2026-12-25" }, today)).toBe(46);
    expect(daysUntil({ name: "Today", date: "2026-11-09" }, today)).toBe(0);
    expect(daysUntil({ name: "Past", date: "2026-11-01" }, today)).toBeNull();
    expect(daysUntil({ name: "Bad", date: "soon" }, today)).toBeNull();
  });

  it("repeats monthly and yearly", () => {
    expect(daysUntil({ name: "Gajian", date: "2026-01-25", repeat: "monthly" }, today)).toBe(16);
    expect(daysUntil({ name: "Gajian", date: "2026-01-05", repeat: "monthly" }, today)).toBe(26);
    expect(daysUntil({ name: "Ultah", date: "1990-11-10", repeat: "yearly" }, today)).toBe(1);
    expect(daysUntil({ name: "Ultah", date: "1990-03-01", repeat: "yearly" }, today)).toBe(112);
  });

  it("clamps the 31st to short months", () => {
    // 30 Nov is the last day of November.
    expect(daysUntil({ name: "Akhir bulan", date: "2026-01-31", repeat: "monthly" }, today)).toBe(21);
  });

  it("sorts upcoming ones and drops finished or unnamed ones", () => {
    const list = upcomingCountdowns(
      [
        { name: "Natal", date: "2026-12-25" },
        { name: "", date: "2026-11-20" },
        { name: "Deadline", date: "2026-11-12" },
        { name: "Lewat", date: "2026-10-01" },
      ],
      today,
    );
    expect(list).toEqual([
      { name: "Deadline", days: 3 },
      { name: "Natal", days: 46 },
    ]);
  });
});
