import { describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const { onDay, doneOn } = await import("./productivity.svelte");

describe("to-do days", () => {
  const today = "2026-11-06";
  const task = (date: string | undefined, done = false) => ({ id: 1, text: "x", time: "", done, date });

  it("older tasks without a date count as today", () => {
    expect(onDay(task(undefined), today, today)).toBe(true);
    expect(onDay(task(undefined), "2026-11-07", today)).toBe(false);
  });

  it("unfinished tasks from earlier days carry over to today only", () => {
    expect(onDay(task("2026-11-04"), today, today)).toBe(true);
    expect(onDay(task("2026-11-04", true), today, today)).toBe(false);
    expect(onDay(task("2026-11-04"), "2026-11-04", today)).toBe(true);
  });

  it("future tasks show only on their day", () => {
    expect(onDay(task("2026-11-08"), today, today)).toBe(false);
    expect(onDay(task("2026-11-08"), "2026-11-08", today)).toBe(true);
  });
});

describe("repeating to-dos", () => {
  // 2026-11-06 is a Friday.
  const today = "2026-11-06";
  const task = (repeat: "daily" | "weekdays" | "weekly", doneDays?: string[]) => ({
    id: 1, text: "x", time: "", done: false, date: today, repeat, doneDays,
  });

  it("daily shows every day from its start, not before", () => {
    expect(onDay(task("daily"), today, today)).toBe(true);
    expect(onDay(task("daily"), "2026-11-09", today)).toBe(true);
    expect(onDay(task("daily"), "2026-11-05", today)).toBe(false);
  });

  it("weekdays skips the weekend", () => {
    expect(onDay(task("weekdays"), "2026-11-07", today)).toBe(false);
    expect(onDay(task("weekdays"), "2026-11-08", today)).toBe(false);
    expect(onDay(task("weekdays"), "2026-11-09", today)).toBe(true);
  });

  it("weekly shows on the same weekday", () => {
    expect(onDay(task("weekly"), "2026-11-13", today)).toBe(true);
    expect(onDay(task("weekly"), "2026-11-12", today)).toBe(false);
  });

  it("is ticked off per day", () => {
    const t = task("daily", [today]);
    expect(doneOn(t, today)).toBe(true);
    expect(doneOn(t, "2026-11-07")).toBe(false);
  });

  it("does not carry over when left undone", () => {
    const old = { ...task("weekly"), date: "2026-11-04" };
    expect(onDay(old, today, today)).toBe(false);
  });
});
