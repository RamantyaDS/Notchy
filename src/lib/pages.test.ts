import { describe, expect, it } from "vitest";
import { ALL_PAGES, filterPages, PAGE_LABELS, resolvePageDirection, type PageToggles } from "./pages";

describe("pages configuration and labels", () => {
  it("defines all 7 pages with appropriate Indonesian labels", () => {
    expect(ALL_PAGES).toEqual(["now", "pomodoro", "water", "todo", "shelf", "clipboard", "shortcuts"]);
    expect(PAGE_LABELS.now).toBe("Pemutar Musik");
    expect(PAGE_LABELS.pomodoro).toBe("Fokus Pomodoro");
    expect(PAGE_LABELS.water).toBe("Minum Air");
    expect(PAGE_LABELS.todo).toBe("Catatan To-Do");
    expect(PAGE_LABELS.shelf).toBe("Laci File");
    expect(PAGE_LABELS.clipboard).toBe("Papan Klip");
    expect(PAGE_LABELS.shortcuts).toBe("Pintasan");
  });

  it("returns all pages when all toggles are true or undefined", () => {
    expect(filterPages(ALL_PAGES, {})).toEqual(ALL_PAGES);
    const allTrue: PageToggles = {
      musicEnabled: true,
      pomodoroEnabled: true,
      waterEnabled: true,
      todoEnabled: true,
      shelfEnabled: true,
      clipboardEnabled: true,
      shortcutsEnabled: true,
    };
    expect(filterPages(ALL_PAGES, allTrue)).toEqual(ALL_PAGES);
  });

  it("filters out disabled pages accurately", () => {
    const withoutMusic = filterPages(ALL_PAGES, { musicEnabled: false });
    expect(withoutMusic).not.toContain("now");
    expect(withoutMusic).toHaveLength(6);

    const onlyTodoAndPomodoro = filterPages(ALL_PAGES, {
      musicEnabled: false,
      waterEnabled: false,
      shelfEnabled: false,
      clipboardEnabled: false,
      shortcutsEnabled: false,
      pomodoroEnabled: true,
      todoEnabled: true,
    });
    expect(onlyTodoAndPomodoro).toEqual(["pomodoro", "todo"]);
  });

  it("preserves original page ordering when some pages are disabled", () => {
    const res = filterPages(ALL_PAGES, {
      pomodoroEnabled: false,
      todoEnabled: false,
    });
    expect(res).toEqual(["now", "water", "shelf", "clipboard", "shortcuts"]);
  });

  it("safety fallback: returns at least one page ('now') if all toggles are set to false", () => {
    const allFalse: PageToggles = {
      musicEnabled: false,
      pomodoroEnabled: false,
      waterEnabled: false,
      todoEnabled: false,
      shelfEnabled: false,
      clipboardEnabled: false,
      shortcutsEnabled: false,
    };
    const fallback = filterPages(ALL_PAGES, allFalse);
    expect(fallback).toEqual(["now"]);
    expect(fallback.length).toBeGreaterThan(0);
  });

  describe("resolvePageDirection", () => {
    it("returns 1 for forward navigation without explicit step", () => {
      expect(resolvePageDirection(0, 1)).toBe(1);
      expect(resolvePageDirection(2, 5)).toBe(1);
    });

    it("returns -1 for backward navigation without explicit step", () => {
      expect(resolvePageDirection(3, 1)).toBe(-1);
      expect(resolvePageDirection(5, 2)).toBe(-1);
    });

    it("respects explicit step > 0 even when wrapping backwards (e.g. 6 to 0)", () => {
      expect(resolvePageDirection(6, 0, 1)).toBe(1);
    });

    it("respects explicit step < 0 even when wrapping forwards (e.g. 0 to 6)", () => {
      expect(resolvePageDirection(0, 6, -1)).toBe(-1);
    });
  });
});
