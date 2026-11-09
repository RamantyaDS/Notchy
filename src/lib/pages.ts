export const ALL_PAGES = ["now", "sound", "today", "pomodoro", "water", "todo", "shelf", "clipboard", "snippets", "shortcuts", "prompter"] as const;
export type PageName = (typeof ALL_PAGES)[number];

export type PageToggles = {
  musicEnabled?: boolean;
  todayEnabled?: boolean;
  pomodoroEnabled?: boolean;
  waterEnabled?: boolean;
  todoEnabled?: boolean;
  shelfEnabled?: boolean;
  clipboardEnabled?: boolean;
  shortcutsEnabled?: boolean;
  soundEnabled?: boolean;
  /** Opt-in pages: shown only when true. */
  snippetsEnabled?: boolean;
  prompterEnabled?: boolean;
};

export const PAGE_LABELS: Record<PageName, string> = {
  now: "Pemutar Musik",
  today: "Hari Ini",
  pomodoro: "Fokus Pomodoro",
  water: "Minum Air",
  todo: "Catatan To-Do",
  shelf: "Laci File",
  clipboard: "Papan Klip",
  shortcuts: "Pintasan",
  sound: "Suara",
  snippets: "Snippet",
  prompter: "Teleprompter",
};

import { t } from "./i18n";

export function getPageLabel(page: PageName): string {
  const map: Record<PageName, string> = {
    now: "pages.music",
    today: "pages.today",
    pomodoro: "pages.pomodoro",
    water: "pages.water",
    todo: "pages.todo",
    shelf: "pages.shelf",
    clipboard: "pages.clipboard",
    shortcuts: "pages.shortcuts",
    sound: "pages.sound",
    snippets: "pages.snippets",
    prompter: "pages.prompter",
  };
  return t(map[page] || page);
}

/**
 * Filters the active pages based on user settings.
 * Ensures at least one page is returned (fallback to "now" if all are disabled).
 */
export function filterPages(allPages: readonly PageName[], toggles: PageToggles): PageName[] {
  const active = allPages.filter((p) => {
    switch (p) {
      case "now":
        return toggles.musicEnabled !== false;
      case "today":
        return toggles.todayEnabled !== false;
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
      case "sound":
        return toggles.soundEnabled !== false;
      case "snippets":
        return toggles.snippetsEnabled === true;
      case "prompter":
        return toggles.prompterEnabled === true;
      default:
        return true;
    }
  });

  return active.length > 0 ? active : ["now"];
}

/**
 * Resolves the slide direction (+1 for forward/right-to-left, -1 for backward/left-to-right).
 */
export function resolvePageDirection(
  from: number,
  to: number,
  explicitStep?: number
): 1 | -1 {
  if (explicitStep !== undefined) {
    return explicitStep > 0 ? 1 : -1;
  }
  return to >= from ? 1 : -1;
}
