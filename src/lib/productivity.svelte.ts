// Pomodoro, water log and to-do: state, persistence and reminders.
// Data lives in %APPDATA%\Notchy\<name>.json via load_data/save_data.

import { invoke } from "@tauri-apps/api/core";
import { chime } from "./chime";
import type { PeekData } from "./Peek.svelte";

export type Settings = {
  focusMinutes: number;
  breakMinutes: number;
  glassMl: number;
  goalMl: number;
  waterFrom: number;
  waterTo: number;
  remindEvery: number;
  /** Off: no water reminders and no water page. */
  waterEnabled: boolean;
  /** The reminder switch on the water page. */
  waterReminders: boolean;
};

type Notify = (peek: PeekData) => void;

const load = <T>(name: string) => invoke<T | null>("load_data", { name });
const save = (name: string, value: unknown) => invoke("save_data", { name, value });

const MINUTE = 60_000;
const startOfDay = (d = new Date()) => new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime();

// ---------------------------------------------------------------- pomodoro

export class Pomodoro {
  phase = $state<"focus" | "break">("focus");
  running = $state(false);
  /** Seconds left while paused; while running, derived from `endsAt`. */
  left = $state(25 * 60);
  endsAt = $state(0);
  focusMin = 25;
  breakMin = 5;
  #timer: ReturnType<typeof setTimeout> | undefined;

  constructor(private notify: Notify) {}

  configure(s: Settings) {
    const idle = !this.running && this.left === this.length();
    this.focusMin = s.focusMinutes;
    this.breakMin = s.breakMinutes;
    if (idle) this.left = this.length();
  }

  length() {
    return (this.phase === "focus" ? this.focusMin : this.breakMin) * 60;
  }

  remaining(now = Date.now()) {
    return this.running ? Math.max(0, (this.endsAt - now) / 1000) : this.left;
  }

  /** 0..1 of the current phase done. */
  progress(now = Date.now()) {
    return 1 - this.remaining(now) / this.length();
  }

  start() {
    if (this.running) return;
    this.endsAt = Date.now() + this.left * 1000;
    this.running = true;
    this.#timer = setTimeout(() => this.#finish(), this.left * 1000);
  }

  pause() {
    if (!this.running) return;
    this.left = Math.ceil(this.remaining());
    this.running = false;
    clearTimeout(this.#timer);
  }

  reset() {
    clearTimeout(this.#timer);
    this.running = false;
    this.phase = "focus";
    this.left = this.length();
  }

  skip() {
    clearTimeout(this.#timer);
    this.running = false;
    this.#switch();
  }

  /** Starts a fresh focus or break right away (the quick buttons). */
  begin(phase: "focus" | "break") {
    clearTimeout(this.#timer);
    this.running = false;
    this.phase = phase;
    this.left = this.length();
    this.start();
  }

  #switch() {
    this.phase = this.phase === "focus" ? "break" : "focus";
    this.left = this.length();
  }

  #finish() {
    this.running = false;
    const ended = this.phase;
    this.#switch();
    chime();
    this.notify({ kind: "timer", phase: this.phase });
    // A break starts by itself; the next focus waits for you.
    if (ended === "focus") this.start();
  }
}

// ---------------------------------------------------------------- water

type Drink = { t: number; ml: number };

export class Water {
  log = $state<Drink[]>([]);
  settings = $state<Settings | null>(null);
  #timer: ReturnType<typeof setTimeout> | undefined;

  constructor(private notify: Notify) {}

  async load() {
    this.log = (await load<Drink[]>("water")) ?? [];
    this.#schedule();
  }

  configure(s: Settings) {
    this.settings = s;
    this.#schedule();
  }

  today() {
    const from = startOfDay();
    return this.log.filter((d) => d.t >= from).reduce((sum, d) => sum + d.ml, 0);
  }

  /** Millilitres per hour today, for the active hours. */
  byHour() {
    const s = this.settings;
    if (!s) return [];
    const from = startOfDay();
    const hours = Array.from({ length: Math.max(1, s.waterTo - s.waterFrom) }, (_, i) => ({ hour: s.waterFrom + i, ml: 0 }));
    for (const d of this.log) {
      if (d.t < from) continue;
      const slot = hours.find((h) => h.hour === new Date(d.t).getHours());
      if (slot) slot.ml += d.ml;
    }
    return hours;
  }

  drink() {
    if (!this.settings) return;
    const week = Date.now() - 7 * 24 * 60 * MINUTE;
    this.log = [...this.log.filter((d) => d.t >= week), { t: Date.now(), ml: this.settings.glassMl }];
    save("water", this.log);
    this.#schedule();
  }

  /** Next reminder: `remindEvery` minutes after the last drink, within the active hours. */
  #schedule() {
    clearTimeout(this.#timer);
    const s = this.settings;
    if (!s || s.waterEnabled === false || s.waterReminders === false || s.remindEvery <= 0) return;
    const now = new Date();
    const open = startOfDay(now) + s.waterFrom * 60 * MINUTE;
    const close = startOfDay(now) + s.waterTo * 60 * MINUTE;
    const last = Math.max(open, ...this.log.map((d) => d.t));
    let at = last + s.remindEvery * MINUTE;
    if (at >= close) at = open + 24 * 60 * MINUTE + s.remindEvery * MINUTE; // tomorrow
    if (this.today() >= s.goalMl && at < close) at = close; // goal met: rest of today is quiet
    this.#timer = setTimeout(() => {
      const t = Date.now();
      if (t >= open && t < close && this.today() < s.goalMl) {
        chime();
        this.notify({ kind: "water", today: this.today(), goal: s.goalMl });
      }
      this.#schedule();
    }, Math.max(MINUTE, at - now.getTime()));
  }
}

// ---------------------------------------------------------------- to-do

export type Task = {
  id: number;
  text: string;
  /** "HH:MM" for a reminder today, or "". */
  time: string;
  done: boolean;
  doneAt?: number;
  /** Day (start-of-day ms) the reminder already fired. */
  remindedOn?: number;
};

export class Todo {
  tasks = $state<Task[]>([]);
  #timers: ReturnType<typeof setTimeout>[] = [];

  constructor(private notify: Notify) {}

  async load() {
    // Finished tasks from earlier days are cleared away.
    const today = startOfDay();
    this.tasks = ((await load<Task[]>("todo")) ?? []).filter((t) => !t.done || (t.doneAt ?? 0) >= today);
    this.#schedule();
  }

  doneToday() {
    return this.tasks.filter((t) => t.done).length;
  }

  add(text: string, time: string) {
    if (!text.trim()) return;
    this.tasks = [...this.tasks, { id: Date.now(), text: text.trim(), time, done: false }];
    this.#commit();
  }

  update(id: number, text: string, time: string) {
    this.tasks = this.tasks.map((t) => (t.id === id ? { ...t, text: text.trim() || t.text, time, remindedOn: undefined } : t));
    this.#commit();
  }

  toggle(id: number) {
    this.tasks = this.tasks.map((t) => (t.id === id ? { ...t, done: !t.done, doneAt: t.done ? undefined : Date.now() } : t));
    this.#commit();
  }

  remove(id: number) {
    this.tasks = this.tasks.filter((t) => t.id !== id);
    this.#commit();
  }

  #commit() {
    save("todo", this.tasks);
    this.#schedule();
  }

  /** One timer per open task with a time later today. */
  #schedule() {
    this.#timers.forEach(clearTimeout);
    const today = startOfDay();
    this.#timers = this.tasks
      .filter((t) => !t.done && t.time && t.remindedOn !== today)
      .map((t) => {
        const [h, m] = t.time.split(":").map(Number);
        const at = today + (h * 60 + m) * MINUTE;
        return setTimeout(() => {
          chime();
          this.notify({ kind: "todo", text: t.text, time: t.time });
          this.tasks = this.tasks.map((x) => (x.id === t.id ? { ...x, remindedOn: today } : x));
          save("todo", this.tasks);
        }, Math.max(0, at - Date.now()));
      });
  }
}

// ---------------------------------------------------------------- shelf

export type ShelfItem = { path: string; name: string; size: number; dir: boolean };

export class Shelf {
  items = $state<ShelfItem[]>([]);

  async load() {
    const saved = (await load<ShelfItem[]>("shelf")) ?? [];
    // Re-allow previews and drop files that no longer exist.
    const sizes = await invoke<(number | null)[]>("shelf_add", { paths: saved.map((i) => i.path) });
    this.items = saved.filter((_, i) => sizes[i] !== null);
  }

  async add(paths: string[]) {
    const fresh = paths.filter((p) => !this.items.some((i) => i.path === p));
    if (!fresh.length) return;
    const sizes = await invoke<(number | null)[]>("shelf_add", { paths: fresh });
    const added = fresh.flatMap((path, i) => {
      const size = sizes[i];
      if (size === null) return [];
      const name = path.split(/[\\/]/).pop() ?? path;
      return [{ path, name, size, dir: size === 0 && !name.includes(".") }];
    });
    this.items = [...added, ...this.items].slice(0, 24);
    save("shelf", this.items);
  }

  remove(path: string) {
    this.items = this.items.filter((i) => i.path !== path);
    save("shelf", this.items);
  }

  clear() {
    this.items = [];
    save("shelf", this.items);
  }
}
