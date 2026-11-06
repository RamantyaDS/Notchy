import { describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const { onDay } = await import("./productivity.svelte");

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
