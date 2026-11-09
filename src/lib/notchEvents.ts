import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Call, Download, ClipEntry, Power, InUse } from "./types";
import type { NetworkSpeed } from "./network";
import type { NowPlaying } from "./media";
import type { PeekData } from "./Peek.svelte";

export interface EventHandlers {
  onNotchHover: (hovered: boolean) => void;
  onOutsideClick: () => void;
  onCall: (call: Call | null) => void;
  onDownload: (download: Download | null) => void;
  onNetSpeed: (speed: NetworkSpeed) => void;
  onAudioLevel: (levels: number[]) => void;
  onClipboard: (clips: ClipEntry[]) => void;
  onEyesLook: (look: [number, number, number]) => void;
  onTucked: (tucked: boolean) => void;
  onMedia: (media: Omit<NowPlaying, "receivedAt"> | null) => void;
  onPeek: (peek: PeekData) => void;
  onPower: (power: Power) => void;
  onInUse: (inUse: InUse) => void;
  onMicMuted: (muted: boolean) => void;
  onSettings: (settings: any) => void;
  onAccentColor: (accent: string) => void;
  onMonitorLeave?: () => void;
  /** The window was hidden for a full-screen app or presentation (true) or shown again. */
  onHidden?: (hidden: boolean) => void;
  /** The "open the notch" hotkey. */
  onHotkey?: () => void;
  /** Keep-awake: 0 off, otherwise when it ends (unix ms; huge = until turned off). */
  onKeepAwake?: (until: number) => void;
  /** Keyboard lock: 0 unlocked, otherwise when it unlocks by itself (unix ms). */
  onKeyLock?: (until: number) => void;
  /** Claude Code hook: working, waiting, done or idle. */
  onAgent?: (agent: { state: string; text: string }) => void;
}

export async function setupNotchEvents(handlers: EventHandlers): Promise<() => void> {
  const unlistens: UnlistenFn[] = [];

  const add = async (promise: Promise<UnlistenFn>) => {
    try {
      const u = await promise;
      unlistens.push(u);
    } catch {
      // ignore in tests / non-tauri context
    }
  };

  await Promise.all([
    add(listen<boolean>("notch-hover", (e) => handlers.onNotchHover(e.payload))),
    add(listen("notch-outside-click", handlers.onOutsideClick)),
    add(listen<Call | null>("call", (e) => handlers.onCall(e.payload))),
    add(listen<Download | null>("download", (e) => handlers.onDownload(e.payload))),
    add(listen<NetworkSpeed>("network-speed", (e) => handlers.onNetSpeed(e.payload))),
    add(listen<number[]>("audio-level", (e) => handlers.onAudioLevel(e.payload))),
    add(listen<ClipEntry[]>("clipboard", (e) => handlers.onClipboard(e.payload))),
    add(listen<[number, number, number]>("eyes-look", (e) => handlers.onEyesLook(e.payload))),
    add(listen<boolean>("notch-tucked", (e) => handlers.onTucked(e.payload))),
    add(listen<Omit<NowPlaying, "receivedAt"> | null>("media", (e) => handlers.onMedia(e.payload))),
    add(listen<PeekData>("peek", (e) => handlers.onPeek(e.payload))),
    add(listen<Power>("power", (e) => handlers.onPower(e.payload))),
    add(listen<InUse>("in-use", (e) => handlers.onInUse(e.payload))),
    add(listen<boolean>("mic-muted", (e) => handlers.onMicMuted(e.payload))),
    add(listen("settings", (e) => handlers.onSettings(e.payload))),
    add(listen<string>("accent-color", (e) => handlers.onAccentColor(e.payload))),
    add(listen("monitor-transition-leave", () => handlers.onMonitorLeave?.())),
    add(listen<boolean>("notch-hidden", (e) => handlers.onHidden?.(e.payload))),
    add(listen("notch-hotkey", () => handlers.onHotkey?.())),
    add(listen<number>("keep-awake", (e) => handlers.onKeepAwake?.(e.payload))),
    add(listen<number>("keyboard-lock", (e) => handlers.onKeyLock?.(e.payload))),
    add(listen<{ state: string; text: string }>("agent", (e) => handlers.onAgent?.(e.payload))),
  ]);

  return () => {
    for (const u of unlistens) {
      try {
        u();
      } catch {}
    }
  };
}
