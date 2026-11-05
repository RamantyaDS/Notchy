import { invoke } from "@tauri-apps/api/core";
import type { PeekData } from "./Peek.svelte";

export class PeekManager {
  peek = $state<PeekData | null>(null);
  peekQueue = $state<PeekData[]>([]);
  private timer: ReturnType<typeof setTimeout> | null = null;

  clickable(p: PeekData | null): boolean {
    return ["notification", "screenshot", "update", "download-done", "meeting", "bluetooth"].includes(p?.kind ?? "");
  }

  peekLength(p: PeekData): number {
    if (p.kind === "meeting") return 30_000;
    if (["screenshot", "update", "download-done"].includes(p.kind)) return 6000;
    if (["bluetooth"].includes(p.kind)) return 4500;
    if (p.kind === "notification") return this.peekQueue.length ? 3200 : 4500;
    if (["todo", "calendar", "update-info", "earbuds-low"].includes(p.kind)) return 5000;
    return 2200;
  }

  show(data: PeekData, hovered: boolean) {
    if (data.kind === "notification" && this.peek?.kind === "notification") {
      this.peekQueue.push(data);
      return;
    }
    this.peek = data;
    this.clearTimer();
    if (!(this.clickable(data) && hovered)) {
      this.timer = setTimeout(() => this.end(), this.peekLength(data));
    }
  }

  holdOrRelease(hovered: boolean) {
    if (this.clickable(this.peek)) {
      this.clearTimer();
      if (!hovered) {
        this.timer = setTimeout(() => this.end(), 1500);
      }
    }
  }

  end() {
    this.clearTimer();
    this.peek = null;
    const next = this.peekQueue.shift();
    if (next) {
      setTimeout(() => this.show(next, false), 120);
    }
  }

  openFromPeek(p: PeekData) {
    if (p.kind === "notification" && p.aumid) invoke("open_app", { aumid: p.aumid, id: p.id });
    if (p.kind === "update") invoke("install_update");
    if (p.kind === "download-done") invoke("open_download", { path: p.path });
    if (p.kind === "bluetooth") invoke("run_link", { url: "ms-settings:bluetooth" }).catch(() => {});
    this.end();
  }

  clearTimer() {
    if (this.timer) {
      clearTimeout(this.timer);
      this.timer = null;
    }
  }
}
