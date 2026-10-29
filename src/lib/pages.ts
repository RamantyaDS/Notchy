export const ALL_PAGES = ["now", "pomodoro", "water", "todo", "shelf", "clipboard", "shortcuts"] as const;
export type PageName = (typeof ALL_PAGES)[number];

export type PageToggles = {
  musicEnabled?: boolean;
  pomodoroEnabled?: boolean;
  waterEnabled?: boolean;
  todoEnabled?: boolean;
  shelfEnabled?: boolean;
  clipboardEnabled?: boolean;
  shortcutsEnabled?: boolean;
};

export const PAGE_LABELS: Record<PageName, string> = {
  now: "Pemutar Musik",
  pomodoro: "Fokus Pomodoro",
  water: "Minum Air",
  todo: "Catatan To-Do",
  shelf: "Laci File",
  clipboard: "Papan Klip",
  shortcuts: "Pintasan",
};

/**
 * Filters the active pages based on user settings.
 * Ensures at least one page is returned (fallback to "now" if all are disabled).
 */
export function filterPages(allPages: readonly PageName[], toggles: PageToggles): PageName[] {
  const active = allPages.filter((p) => {
    switch (p) {
      case "now":
        return toggles.musicEnabled !== false;
      case "pomodoro":
        return toggles.pomodoroEnabled !== false;
      case "water":
        return toggles.waterEnabled !== false;
      case "todo":
        return toggles.todoEnabled !== false;
      case "shelf":
        return toggles.shelfEnabled !== false;
      case "clipboard":
        return toggles.clipboardEnabled !== false;
      case "shortcuts":
        return toggles.shortcutsEnabled !== false;
      default:
        return true;
    }
  });

  return active.length > 0 ? active : ["now"];
}
