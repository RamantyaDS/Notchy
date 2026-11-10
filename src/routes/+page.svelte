<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { onMount, untrack } from "svelte";
  import { fade, fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import ClipboardPage, { type ClipEntry } from "$lib/ClipboardPage.svelte";
  import ClockPage from "$lib/ClockPage.svelte";
  import Island from "$lib/Island.svelte";
  import Mirror from "$lib/Mirror.svelte";
  import Peek, { type PeekData } from "$lib/Peek.svelte";
  import Player from "$lib/Player.svelte";
  import RestEyes from "$lib/RestEyes.svelte";
  import ShelfPage from "$lib/ShelfPage.svelte";
  import ShortcutsPage, { type Shortcut } from "$lib/ShortcutsPage.svelte";
  import SoundPage from "$lib/SoundPage.svelte";
  import SnippetsPage from "$lib/SnippetsPage.svelte";
  import PrompterPage from "$lib/PrompterPage.svelte";
  import { upcomingCountdowns, type Countdown } from "$lib/countdown";
  import StatusBar from "$lib/StatusBar.svelte";
  import TimerPage from "$lib/TimerPage.svelte";
  import TodayPage from "$lib/TodayPage.svelte";
  import TodoPage from "$lib/TodoPage.svelte";
  import WaterPage from "$lib/WaterPage.svelte";
  import { arrange, islandWidth, type Activity, type Call, type Download } from "$lib/island";
  import { nextSource, positionAt, type NowPlaying } from "$lib/media";
  import { cleanTrack, currentLine, findLyrics, type Lyrics } from "$lib/lyrics";
  import { dayKey, Pomodoro, Shelf, Todo, Water, type Settings } from "$lib/productivity.svelte";
  import { fetchWeather, rainSoon, type Weather } from "$lib/weather";
  import { calculatePrayerTimes, getNextPrayer, type PrayerMethod } from "$lib/prayer";
  import { chime } from "$lib/chime";
  import { upcoming, type CalEvent } from "$lib/calendar";
  import { ALL_PAGES, filterPages, PAGE_LABELS, getPageLabel, resolvePageDirection, type PageName } from "$lib/pages";
  import { setLocale, t } from "$lib/i18n";
  import { isHighTraffic, type NetworkSpeed } from "$lib/network";
  import { PeekManager } from "$lib/peekManager.svelte";
  import { setupNotchEvents } from "$lib/notchEvents";
  import type { Power, InUse } from "$lib/types";

  // Black box logging & heartbeat
  const logJs = (msg: string) => invoke("log_js", { msg }).catch(() => {});
  const onError = (e: ErrorEvent) => logJs(`${e.message} (${String(e.filename).split("/").pop()}:${e.lineno})`);
  const onRejection = (e: PromiseRejectionEvent) => logJs(`unhandled: ${e.reason?.stack ?? e.reason}`);
  const onBlocked = (e: SecurityPolicyViolationEvent) => logJs(`CSP blocked ${e.blockedURI} (${e.effectiveDirective})`);

  onMount(() => {
    window.addEventListener("error", onError);
    window.addEventListener("unhandledrejection", onRejection);
    document.addEventListener("securitypolicyviolation", onBlocked);
    const beat = setInterval(() => requestAnimationFrame(() => invoke("heartbeat").catch(() => {})), 5000);
    return () => {
      window.removeEventListener("error", onError);
      window.removeEventListener("unhandledrejection", onRejection);
      document.removeEventListener("securitypolicyviolation", onBlocked);
      clearInterval(beat);
    };
  });

  const SIZES = {
    tucked: { w: 90, h: 6, r: 4, e: 4 },
    minimal: { w: 160, h: 5, r: 3, e: 3 },
    rest: { w: 150, h: 30, r: 14, e: 10 },
    island: { w: 200, h: 30, r: 14, e: 10 },
    charging: { w: 200, h: 30, r: 14, e: 10 },
    peek: { w: 310, h: 60, r: 19, e: 13 },
    shot: { w: 360, h: 96, r: 24, e: 16 },
    notify: { w: 380, h: 72, r: 22, e: 14 },
    clock: { w: 420, h: 140, r: 32, e: 22 },
    player: { w: 420, h: 154, r: 32, e: 22 },
    lyrics: { w: 520, h: 154, r: 32, e: 22 },
    pomodoro: { w: 420, h: 154, r: 32, e: 22 },
    water: { w: 420, h: 154, r: 32, e: 22 },
    today: { w: 420, h: 154, r: 32, e: 22 },
    todo: { w: 420, h: 154, r: 32, e: 22 },
    shelf: { w: 420, h: 154, r: 32, e: 22 },
    clipboard: { w: 420, h: 154, r: 32, e: 22 },
    shortcuts: { w: 420, h: 154, r: 32, e: 22 },
    mirror: { w: 420, h: 140, r: 32, e: 22 },
    sound: { w: 420, h: 154, r: 32, e: 22 },
    snippets: { w: 420, h: 154, r: 32, e: 22 },
    prompter: { w: 440, h: 154, r: 32, e: 22 },
    keylock: { w: 420, h: 92, r: 28, e: 18 },
  };

  let musicEnabled = $state(true);
  let todayEnabled = $state(true);
  let pomodoroEnabled = $state(true);
  let waterEnabled = $state(true);
  let todoEnabled = $state(true);
  let shelfEnabled = $state(true);
  let clipboardEnabled = $state(true);
  let shortcutsEnabled = $state(true);
  let soundEnabled = $state(true);
  let snippetsEnabled = $state(false);
  let prompterEnabled = $state(false);

  // Newer features (see Settings)
  let swipeTracks = $state(true);
  let rainAlert = $state(true);
  let chargeReminder = $state(0);
  let presentationSummary = $state(true);
  let eyeBreak = $state(false);
  let privacyIndicators = $state(true);
  let countdownList = $state<Countdown[]>([]);
  /** Keep-awake: 0 off, otherwise when it ends (unix ms; huge = until turned off). */
  let keepAwake = $state(0);
  /** Keyboard-cleaning lock: 0 off, otherwise when it unlocks by itself. */
  let keyLockUntil = $state(0);
  /** Claude Code, from its hooks: working, or waiting for you. */
  let agent = $state<{ waiting: boolean } | null>(null);
  let agentTimer: ReturnType<typeof setTimeout> | undefined;
  /** Hidden for a full-screen app or presentation; notifications are held meanwhile. */
  let hidden = $state(false);
  let held: string[] = [];
  /** Opened with the hotkey: arrows change page, Esc closes. */
  let keyboardMode = $state(false);
  let load = $state<[number, number]>([0, 0]);

  const pages = $derived(
    filterPages(ALL_PAGES, {
      musicEnabled,
      todayEnabled,
      pomodoroEnabled,
      waterEnabled,
      todoEnabled,
      shelfEnabled,
      clipboardEnabled,
      shortcutsEnabled,
      soundEnabled,
      snippetsEnabled,
      prompterEnabled,
    })
  );

  let open = $state(false);
  let tucked = $state(false);
  let blink = $state(false);
  let wink = $state<"left" | "right" | null>(null);
  let dizzy = $state(false);
  let dozing = $state(false);
  let greeting = $state(false);
  let mascot = $state(true);
  let mascotMode = $state("idle");
  let mascotTheme = $state("classic");
  let musicMascot = $state(false);
  let activeEyes = $state(false);
  let shocked = $state(false);
  let shockedTimer: ReturnType<typeof setTimeout> | undefined;
  let tickled = $state(false);
  let tickleTimer: ReturnType<typeof setTimeout> | undefined;
  let lastTickleX = 0;
  let lastTickleDir = 0;
  let tickleSwitches = 0;
  let lastTickleTime = 0;
  let minimalNotch = $state(false);
  let floatingNotch = $state(false);
  /** Thin ring in the closed notch: "off", "battery" or "water". */
  let statusRing = $state("off");
  let look = $state<[number, number, number]>([0, 0, 0]);
  let hovered = $state(false);
  let page = $state(0);
  let pageDirection = $state<1 | -1>(1);
  let hoveredPage = $state<PageName | null>(null);
  let showPageLabel = $state(false);
  let pageLabelTimer: ReturnType<typeof setTimeout> | undefined;
  let typing = $state(false);
  let dropping = $state(false);
  let dropTarget = $state<string | null>(null);
  let levels = $state<number[] | null>(null);
  let turn = $state(0);
  let pinned = $state(false);
  let mirror = $state<{ title: string; link: string } | null>(null);
  let scrollingVolume = $state(false);
  let scrollVolumeTimer: ReturnType<typeof setTimeout> | undefined;

  // Controllers
  const peekMgr = new PeekManager();
  const showPeek = (data: PeekData) => peekMgr.show(data, hovered);

  let power = $state<Power>({ charging: false, percent: 100, battery: false });
  let inUse = $state<InUse>({ camera: [], mic: [] });
  let micMuted = $state(false);
  let call = $state<Call | null>(null);
  let download = $state<Download | null>(null);
  let networkSpeed = $state<NetworkSpeed | null>(null);
  let netSpeedIsland = $state(false);
  let netSpeedStatus = $state(true);
  let netSpeedThresholdDownKb = $state(500);
  let netSpeedThresholdUpKb = $state(250);

  // Spring & Windows Accent state
  let monitorTransition = $state<"idle" | "leaving" | "entering">("idle");
  let springPhysics = $state("spring");
  let windowsAccentGlow = $state(false);
  let accentColor = $state("#38bdf8");

  const springMap: Record<string, string> = {
    spring: "cubic-bezier(0.34, 1.32, 0.5, 1)",
    snappy: "cubic-bezier(0.2, 0.95, 0.3, 1)",
    smooth: "ease",
    off: "none",
  };
  const activeSpring = $derived(springMap[springPhysics] ?? springMap.spring);

  let clipItems = $state<ClipEntry[]>([]);
  let shortcuts = $state<Shortcut[]>([]);
  let smallLyrics = $state(true);
  let now = $state(new Date());
  let nowMs = $state(Date.now());
  let events = $state<CalEvent[]>([]);
  let calendarTimers: ReturnType<typeof setTimeout>[] = [];

  const nextEvent = $derived(events.find((e) => !e.allDay && e.end > now.getTime()));
  const clockOf = (t: number) => new Date(t).toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit", hour12: systemHour12 }).replace(".", ":");

  // Output device for the switch button in the open notch.
  type Output = { id: string; name: string; headphones: boolean; current: boolean };
  let output = $state<Output | null>(null);
  const refreshOutput = async () => (output = (await invoke<Output[]>("list_outputs")).find((o) => o.current) ?? null);
  $effect(() => {
    if (open) refreshOutput();
  });
  async function switchOutput() {
    await invoke("next_output");
    await refreshOutput();
  }

  // Productivity stores
  const pomo = new Pomodoro(showPeek);
  const water = new Water(showPeek);
  const todo = new Todo(showPeek);
  const shelf = new Shelf();

  let weather = $state<Weather | null>(null);
  let prayerEnabled = $state(false);
  let prayerPreAlert = $state(true);
  let prayerMethod = $state("kemenag");
  let prayerFajr = $state(true);
  let prayerDhuhr = $state(true);
  let prayerAsr = $state(true);
  let prayerMaghrib = $state(true);
  let prayerIsha = $state(true);
  let prayerSound = $state(true);
  let prayerLocation = $state<[number, number] | null>(null);
  const prayerAlerted = new Set<string>();

  const charging = $derived(power.battery && power.charging);
  const timerActive = $derived(pomo.running || pomo.left < pomo.length());
  const paged = $derived(open && !peekMgr.clickable(peekMgr.peek));

  // What runs right now, for the closed notch
  const running = $derived.by((): Activity[] => [
    ...(call ? [{ kind: "call", call } as const] : []),
    ...(agent ? [{ kind: "agent", waiting: agent.waiting } as const] : []),
    ...(timerActive ? [{ kind: "timer" } as const] : []),
    ...(download ? [{ kind: "download", download } as const] : []),
    ...(musicEnabled && media?.playing ? [{ kind: "music", lyric: false } as const] : []),
    ...(netSpeedIsland &&
    isHighTraffic(networkSpeed, netSpeedThresholdDownKb * 1024, netSpeedThresholdUpKb * 1024) &&
    networkSpeed
      ? [{ kind: "network", speed: networkSpeed } as const]
      : []),
  ]);

  let media = $state<(NowPlaying & { receivedAt: number }) | null>(null);
  let lyrics = $state<Lyrics | null>(null);
  let lyricsStatus = $state<"loading" | "done" | "offline">("loading");
  let lyricsOn = $state(false);

  const track = $derived(media && cleanTrack(media.title, media.artist, media.source));
  $effect(() => {
    if (!track || !(lyricsOn || smallLyrics)) return;
    const { title, artist } = track;
    let cancelled = false;
    lyrics = null;
    lyricsStatus = "loading";
    const { album, duration } = untrack(() => media!);
    findLyrics(title, artist, album, duration).then((found) => {
      if (cancelled) return;
      lyrics = found;
      lyricsStatus = found ? "done" : "offline";
    });
    return () => (cancelled = true);
  });

  // The line being sung, for the small notch
  let lineNow = $state(Date.now());
  const smallLine = $derived.by((): string => {
    if (!smallLyrics || !media?.playing || !lyrics?.synced) return "";
    const i = currentLine(lyrics.synced, positionAt(media, lineNow));
    return i >= 0 && lyrics.synced[i].text !== "♪" ? lyrics.synced[i].text : "";
  });

  const island = $derived(
    arrange(
      running.map((a) => (a.kind === "music" ? { kind: "music", lyric: !!smallLine } : a)),
      turn,
    ),
  );

  const view = $derived(
    mirror
      ? "mirror"
      : keyLockUntil
        ? "keylock"
        : scrollingVolume && peekMgr.peek?.kind === "volume"
        ? "peek"
        : paged
          ? pages[page] === "now"
            ? media
              ? lyricsOn
                ? "lyrics"
                : "player"
              : "clock"
            : (pages[page] as Exclude<PageName, "now">)
          : peekMgr.peek
            ? peekMgr.peek.kind === "screenshot"
              ? "shot"
              : peekMgr.peek.kind === "notification" || peekMgr.peek.kind === "meeting" || peekMgr.peek.kind === "download-done"
                ? "notify"
                : "peek"
            : tucked
              ? "tucked"
              : running.length
                ? "island"
                : minimalNotch
                  ? "minimal"
                  : charging
                    ? "charging"
                    : "rest",
  );

  let tint = $state<string | null>(null);
  $effect(() => {
    const art = media?.art;
    tint = null;
    if (!art) return;
    const img = new Image();
    img.onload = () => {
      const c = document.createElement("canvas");
      c.width = c.height = 16;
      const g = c.getContext("2d")!;
      g.drawImage(img, 0, 0, 16, 16);
      let [r, gr, b, n] = [0, 0, 0, 0];
      const px = g.getImageData(0, 0, 16, 16).data;
      for (let i = 0; i < px.length; i += 4) {
        const [pr, pg, pb] = [px[i], px[i + 1], px[i + 2]];
        if (Math.max(pr, pg, pb) - Math.min(pr, pg, pb) < 40) continue;
        [r, gr, b, n] = [r + pr, gr + pg, b + pb, n + 1];
      }
      tint = n > 8 ? `${Math.round(r / n)} ${Math.round(gr / n)} ${Math.round(b / n)}` : null;
    };
    img.src = art;
  });

  const musicLeads = $derived(view === "island" && island.main?.kind === "music");
  const tinted = $derived(!!tint && (view === "player" || view === "lyrics" || musicLeads));

  $effect(() => {
    if (view !== "island" || !smallLyrics || !media?.playing || !lyrics?.synced) return;
    if (Date.now() - lineNow > 250) {
      lineNow = Date.now();
      return;
    }
    const pos = positionAt(media, lineNow);
    const next = lyrics.synced.find((l) => l.time > pos + 0.25);
    if (!next) return;
    const id = setTimeout(() => (lineNow = Date.now()), (next.time - 0.25 - pos) * 1000 + 30);
    return () => clearTimeout(id);
  });

  // Percent shown by the status ring in the resting notch, or null for none.
  const ringValue = $derived(
    statusRing === "battery" && power.battery
      ? power.percent
      : statusRing === "water" && waterEnabled
        ? Math.min(100, Math.round((water.today() / (water.settings?.goalMl || 2000)) * 100))
        : statusRing === "cpu"
          ? load[0]
          : statusRing === "ram"
            ? load[1]
            : null,
  );
  const ringed = $derived(view === "rest" && ringValue !== null);
  const floating = $derived(floatingNotch && view !== "tucked");
  const FLOAT_GAP = 8;

  const camOn = $derived(privacyIndicators && inUse.camera.length > 0);
  const micOn = $derived(privacyIndicators && inUse.mic.length > 0);
  const islandDots = $derived((camOn ? 10 : 0) + (micOn && !call ? 10 : 0));
  const countdownsAhead = $derived(upcomingCountdowns(countdownList, now));
  const baseSize = $derived(
    view === "island" && island.main
      ? { ...SIZES.island, w: islandWidth(island.main, island.side) + islandDots }
      : view === "rest" && (weather || (mascot && mascotMode === "always"))
        ? { ...SIZES.rest, w: weather && mascot && mascotMode === "always" ? 184 : 170 }
        : SIZES[view as keyof typeof SIZES] ?? SIZES.rest,
  );
  // Status dots (camera, mic, keep-awake) in the resting notch get room beside the clock.
  const restDots = $derived(view === "rest" && (camOn || micOn || micMuted || !!keepAwake));
  const size = $derived({ ...baseSize, w: baseSize.w + (ringed ? 22 : 0) + (restDots ? 18 : 0) });
  const closedView = $derived(view === "rest" || view === "island" || view === "charging");
  const withStatus = $derived(paged && !mirror && view !== "peek" && view !== "clock");

  const metering = $derived(
    view === "island" &&
      !!media?.playing &&
      island.main !== null &&
      [island.main, ...island.side].some((a) => a.kind === "music"),
  );
  $effect(() => {
    invoke("set_metering", { on: metering });
    if (!metering) levels = null;
  });

  $effect(() => {
    invoke("set_notch_size", { w: size.w, h: view === "minimal" ? 10 : size.h, top: floating ? FLOAT_GAP : 0 });
  });

  // CPU / RAM for the status ring, only while the ring is showing.
  $effect(() => {
    if (!(statusRing === "cpu" || statusRing === "ram") || view !== "rest") return;
    const read = () => invoke<[number, number]>("system_load").then((l) => (load = l)).catch(() => {});
    read();
    const id = setInterval(read, 2000);
    return () => clearInterval(id);
  });

  // Charger reminder: unplug at the chosen level, plug in at 20%. Once per charge.
  let chargeNoted = false;
  let lowNoted = false;
  $effect(() => {
    const p = power;
    if (!chargeReminder || !p.battery) return;
    untrack(() => {
      if (p.charging && p.percent >= chargeReminder && !chargeNoted) {
        chargeNoted = true;
        showPeek({ kind: "charge-full", percent: p.percent });
      }
      if (!p.charging && p.percent <= 20 && !lowNoted) {
        lowNoted = true;
        showPeek({ kind: "battery-low", percent: p.percent });
      }
      if (!p.charging) chargeNoted = false;
      if (p.charging) lowNoted = false;
    });
  });

  // 20-20-20: after 20 active minutes, a 20-second eye break. Away for 5 minutes resets it.
  $effect(() => {
    if (!eyeBreak) return;
    let active = 0;
    const id = setInterval(async () => {
      const idle = await invoke<number>("idle_seconds").catch(() => 0);
      active = idle >= 300 ? 0 : active + 1;
      if (active >= 20 && !hidden && pomo.phase !== "break" && !call) {
        active = 0;
        showPeek({ kind: "eyes" });
      }
    }, 60_000);
    return () => clearInterval(id);
  });

  // A countdown reaching its day gets one peek that day.
  $effect(() => {
    const today = countdownsAhead.filter((c) => c.days === 0);
    untrack(() => {
      for (const c of today) {
        const key = `countdown:${dayKey()}:${c.name}`;
        try {
          if (localStorage.getItem(key)) continue;
          localStorage.setItem(key, "1");
        } catch {}
        showPeek({ kind: "countdown", name: c.name });
      }
    });
  });

  // The keyboard-lock countdown ticks every second.
  $effect(() => {
    if (!keyLockUntil) return;
    nowMs = Date.now();
    const id = setInterval(() => (nowMs = Date.now()), 1000);
    return () => clearInterval(id);
  });

  $effect(() => {
    if (open && windowsAccentGlow) {
      invoke<string>("get_accent_color").then((c) => (accentColor = c)).catch(() => {});
    }
  });

  $effect(() => {
    const every =
      open && (media?.playing || pomo.running) ? 250 : view === "island" && (pomo.running || call) ? 1000 : 0;
    if (!every) return;
    nowMs = Date.now();
    const id = setInterval(() => (nowMs = Date.now()), every);
    return () => clearInterval(id);
  });

  let openTimer: ReturnType<typeof setTimeout> | undefined;
  let closeTimer: ReturnType<typeof setTimeout> | undefined;

  function setPinned(on: boolean) {
    pinned = on;
    invoke("set_pinned", { on });
    if (!on && !hovered && !typing) closeTimer = setTimeout(() => (open = false), 220);
  }
  function setTyping(on: boolean) {
    typing = on;
    invoke(on ? "begin_typing" : "end_typing");
    if (!on && !hovered) closeTimer = setTimeout(() => (open = false), 220);
  }
  function endKeyboardMode() {
    if (!keyboardMode) return;
    keyboardMode = false;
    setTyping(false);
    setPinned(false);
    open = false;
  }
  function keydown(e: KeyboardEvent) {
    if (!keyboardMode || (e.target as HTMLElement).closest("input, textarea")) return;
    if (e.key === "ArrowRight") goPage(page + 1, 1);
    else if (e.key === "ArrowLeft") goPage(page - 1, -1);
    else if (e.key === "Escape") endKeyboardMode();
  }

  function openMirror(title = "", link = "") {
    mirror = { title, link };
    invoke("set_mirror", { on: true });
    setPinned(true);
    open = true;
  }
  function closeMirror() {
    mirror = null;
    setPinned(false);
    setTimeout(() => invoke("set_mirror", { on: false }), 3000);
  }
  function joinMeeting() {
    const link = mirror?.link;
    closeMirror();
    if (link) invoke("run_link", { url: link });
  }

  let lastWheel = 0;
  let lastVolume = 0;
  function wheel(e: WheelEvent) {
    if (Math.abs(e.deltaX) > Math.abs(e.deltaY)) {
      if (Date.now() - lastWheel < 450 || Math.abs(e.deltaX) < 8) return;
      lastWheel = Date.now();
      if (swipeTracks && paged && pages[page] === "now" && media) {
        invoke(e.deltaX > 0 ? "media_next" : "media_prev");
        return;
      }
      turn += e.deltaX > 0 ? 1 : -1;
      return;
    }
    if (e.offsetY >= 0 && (e.target as HTMLElement).closest(".notch") && e.clientY < 30 && !typing) {
      if (Date.now() - lastVolume < 40 || Math.abs(e.deltaY) < 1) return;
      lastVolume = Date.now();
      invoke("volume_step", { up: e.deltaY < 0 });
      scrollingVolume = true;
      clearTimeout(scrollVolumeTimer);
      scrollVolumeTimer = setTimeout(() => (scrollingVolume = false), 1200);
      return;
    }
    if (!open || typing || (e.target as HTMLElement).closest(".scroll, .lyrics")) return;
    if (Date.now() - lastWheel < 350 || Math.abs(e.deltaY) < 4) return;
    lastWheel = Date.now();
    const step = e.deltaY > 0 ? 1 : -1;
    goPage(page + step, step);
  }

  function goPage(target: number, step?: number) {
    if (pages.length <= 1) return;
    const next = (target + pages.length) % pages.length;
    if (next === page && step === undefined) return;

    pageDirection = resolvePageDirection(page, next, step);
    page = next;
    showPageLabel = true;
    clearTimeout(pageLabelTimer);
    pageLabelTimer = setTimeout(() => (showPageLabel = false), 1200);
  }

  let pressTimer: ReturnType<typeof setTimeout> | undefined;
  let pressAt: [number, number] = [0, 0];
  function pressDown(e: PointerEvent) {
    if (e.button !== 0 || (e.target as HTMLElement).closest("button, input, textarea, .item, video")) return;
    pressAt = [e.clientX, e.clientY];
    clearTimeout(pressTimer);
    pressTimer = setTimeout(() => {
      pressTimer = undefined;
      if (mirror) return;
      setPinned(!pinned);
      open = true;
    }, 600);
  }
  function pressMove(e: PointerEvent) {
    if (pressTimer && Math.hypot(e.clientX - pressAt[0], e.clientY - pressAt[1]) > 6) {
      clearTimeout(pressTimer);
      pressTimer = undefined;
    }
    if (!open && !paged && mascot) {
      const dx = e.clientX - lastTickleX;
      const dir = Math.sign(dx);
      const nowTime = Date.now();
      if (dir !== 0 && dir !== lastTickleDir && Math.abs(dx) > 3) {
        if (nowTime - lastTickleTime < 340) {
          tickleSwitches++;
          if (tickleSwitches >= 4 && !tickled) {
            tickled = true;
            tickleSwitches = 0;
            clearTimeout(tickleTimer);
            tickleTimer = setTimeout(() => (tickled = false), 1500);
          }
        } else {
          tickleSwitches = 1;
        }
        lastTickleTime = nowTime;
        lastTickleDir = dir;
      }
      lastTickleX = e.clientX;
    }
  }
  function pressUp() {
    clearTimeout(pressTimer);
    pressTimer = undefined;
  }

  type AllSettings = Settings & {
    weatherCity: { latitude: number; longitude: number } | null;
    smallLyrics: boolean;
    shortcuts: Shortcut[];
    mascot: boolean;
    mascotMode?: string;
    mascotTheme?: string;
    musicMascot?: boolean;
    minimalNotch: boolean;
    musicEnabled?: boolean;
    todayEnabled?: boolean;
    floatingNotch?: boolean;
    statusRing?: string;
    pomodoroEnabled?: boolean;
    waterEnabled?: boolean;
    todoEnabled?: boolean;
    shelfEnabled?: boolean;
    clipboardEnabled?: boolean;
    shortcutsEnabled?: boolean;
    netSpeedIsland?: boolean;
    netSpeedStatus?: boolean;
    netSpeedThresholdDownKb?: number;
    netSpeedThresholdUpKb?: number;
    springPhysics?: string;
    windowsAccentGlow?: boolean;
    prayerEnabled?: boolean;
    prayerPreAlert?: boolean;
    prayerMethod?: string;
    prayerFajr?: boolean;
    prayerDhuhr?: boolean;
    prayerAsr?: boolean;
    prayerMaghrib?: boolean;
    prayerIsha?: boolean;
    prayerSound?: boolean;
    language?: string;
    soundEnabled?: boolean;
    snippetsEnabled?: boolean;
    prompterEnabled?: boolean;
    swipeTracks?: boolean;
    rainAlert?: boolean;
    chargeReminder?: number;
    presentationSummary?: boolean;
    eyeBreak?: boolean;
    privacyIndicators?: boolean;
    countdowns?: Countdown[];
  };

  const loadCalendar = async () => {
    const ics = await invoke<string>("fetch_calendar").catch(() => "");
    if (!ics) return;
    events = upcoming(ics);
    calendarTimers.forEach(clearTimeout);
    calendarTimers = (events ?? [])
      .flatMap((e) =>
        [10, 2].map((before) => {
          const at = e.start - before * 60_000;
          const peekData: PeekData =
            before === 2
              ? { kind: "meeting", title: e.title, start: e.start, link: e.link }
              : { kind: "calendar", title: e.title, start: e.start, before };
          return at > Date.now() && at - Date.now() < 2 * 24 * 3600_000
            ? setTimeout(() => showPeek(peekData), at - Date.now())
            : undefined;
        }),
      )
      .filter((t): t is ReturnType<typeof setTimeout> => t !== undefined);
  };

  onMount(() => {
    let unlistenEvents: (() => void) | null = null;
    let weatherTimer: ReturnType<typeof setTimeout>;
    let calendarPoll: ReturnType<typeof setInterval>;
    let tick: ReturnType<typeof setTimeout>;
    let blinkTimer: ReturnType<typeof setTimeout>;
    let sleepTimer: ReturnType<typeof setTimeout>;
    let activeEyesTimer: ReturnType<typeof setTimeout>;
    let greetTimer: ReturnType<typeof setTimeout>;

    const wake = () => {
      clearTimeout(sleepTimer);
      clearTimeout(activeEyesTimer);
      if (mascot) {
        if (mascotMode === "always") {
          sleepTimer = setTimeout(() => {
            if (mascot && mascotMode === "always") dozing = true;
          }, 60_000);
        } else {
          activeEyes = false;
          dozing = false;
          sleepTimer = setTimeout(() => {
            if (mascot && mascotMode === "idle") {
              activeEyes = true;
              dozing = true;
            }
          }, 120_000);
        }
      }
    };

    let place: number[] | null = null;
    let rainQuietUntil = 0;
    const loadWeather = async (s?: AllSettings) => {
      clearTimeout(weatherTimer);
      s ??= await invoke<AllSettings>("get_settings");
      const city = s.weatherCity;
      const at: [number, number] | null = city ? [city.latitude, city.longitude] : await invoke<[number, number] | null>("locate");
      place = at;
      prayerLocation = at;
      weather = at ? await fetchWeather(at[0], at[1]) : null;
      checkRain();
      checkPrayer(new Date());
      weatherTimer = setTimeout(loadWeather, 30 * 60_000);
    };
    // Rain alert: one peek per rainy spell (quiet for 3 hours after).
    const checkRain = async () => {
      if (!rainAlert || !place || Date.now() < rainQuietUntil) return;
      const minutes = await rainSoon(place[0], place[1]);
      if (minutes !== null && minutes <= 40) {
        rainQuietUntil = Date.now() + 3 * 3600_000;
        showPeek({ kind: "rain", minutes });
      }
    };
    const rainPoll = setInterval(checkRain, 15 * 60_000);

    const applySettings = (s: AllSettings) => {
      smallLyrics = s.smallLyrics && s.musicEnabled !== false;
      mascot = s.mascot !== false;
      mascotMode = s.mascotMode ?? "idle";
      mascotTheme = s.mascotTheme ?? "classic";
      musicMascot = s.musicMascot === true;
      if (!mascot) {
        dozing = false;
        activeEyes = false;
      }
      wake();
      minimalNotch = s.minimalNotch === true;
      floatingNotch = s.floatingNotch === true;
      statusRing = s.statusRing ?? "off";
      shortcuts = s.shortcuts ?? [];
      const current = pages[page];
      musicEnabled = s.musicEnabled !== false;
      todayEnabled = s.todayEnabled !== false;
      pomodoroEnabled = s.pomodoroEnabled !== false;
      waterEnabled = s.waterEnabled !== false;
      todoEnabled = s.todoEnabled !== false;
      shelfEnabled = s.shelfEnabled !== false;
      clipboardEnabled = s.clipboardEnabled !== false;
      shortcutsEnabled = s.shortcutsEnabled !== false;
      soundEnabled = s.soundEnabled !== false;
      snippetsEnabled = s.snippetsEnabled === true;
      prompterEnabled = s.prompterEnabled === true;
      swipeTracks = s.swipeTracks !== false;
      rainAlert = s.rainAlert !== false;
      chargeReminder = s.chargeReminder ?? 0;
      presentationSummary = s.presentationSummary !== false;
      eyeBreak = s.eyeBreak === true;
      privacyIndicators = s.privacyIndicators !== false;
      countdownList = s.countdowns ?? [];
      netSpeedIsland = s.netSpeedIsland === true;
      netSpeedStatus = s.netSpeedStatus !== false;
      netSpeedThresholdDownKb = s.netSpeedThresholdDownKb ?? 500;
      netSpeedThresholdUpKb = s.netSpeedThresholdUpKb ?? 250;
      springPhysics = s.springPhysics ?? "spring";
      windowsAccentGlow = s.windowsAccentGlow === true;
      currentLanguage = s.language ?? "en";
      setLocale(currentLanguage);
      const newIdx = pages.indexOf(current);
      page = newIdx !== -1 ? newIdx : Math.min(page, Math.max(0, pages.length - 1));
      prayerEnabled = s.prayerEnabled === true;
      prayerPreAlert = s.prayerPreAlert !== false;
      prayerMethod = s.prayerMethod ?? "kemenag";
      prayerFajr = s.prayerFajr !== false;
      prayerDhuhr = s.prayerDhuhr !== false;
      prayerAsr = s.prayerAsr !== false;
      prayerMaghrib = s.prayerMaghrib !== false;
      prayerIsha = s.prayerIsha !== false;
      prayerSound = s.prayerSound !== false;
      if (s.weatherCity) {
        prayerLocation = [s.weatherCity.latitude, s.weatherCity.longitude] as [number, number];
      }
      checkPrayer(new Date());
      pomo.configure(s);
      water.configure(s);
      loadWeather(s);
      loadCalendar();
    };

    invoke<string>("get_accent_color").then((c) => (accentColor = c)).catch(() => {});
    invoke<AllSettings>("get_settings").then(applySettings);
    // Windows only reports power changes, so read the current state once.
    invoke<Power>("power_state").then((p) => (power = p)).catch(() => {});
    invoke<number>("keep_awake_state").then((u) => (keepAwake = u)).catch(() => {});

    let lastLx = 0;
    let dirChanges = 0;
    let lastDirChangeMs = 0;
    let lastSign = 0;
    let dizzyTimeout: ReturnType<typeof setTimeout>;

    setupNotchEvents({
      onNotchHover: (h) => {
        hovered = h;
        if (h) {
          if (dozing) {
            dozing = false;
            greeting = true;
            clearTimeout(greetTimer);
            greetTimer = setTimeout(() => (greeting = false), 1000);
          }
          if (activeEyes) activeEyes = false;
        }
        clearTimeout(closeTimer);
        clearTimeout(openTimer);
        if (h) {
          if (!open) {
            openTimer = setTimeout(() => (open = true), 90);
          }
        } else {
          if (!typing && !pinned) {
            closeTimer = setTimeout(() => (open = false), 220);
          }
        }
        peekMgr.holdOrRelease(h);
      },
      onOutsideClick: () => {
        if (pinned && !mirror) setPinned(false);
      },
      onCall: (c) => (call = c),
      onDownload: (d) => (download = d),
      onNetSpeed: (s) => (networkSpeed = s),
      onAudioLevel: (l) => (levels = metering ? l : null),
      onClipboard: (clips) => (clipItems = clips),
      onEyesLook: (l) => {
        look = l;
        if (mascot) {
          if (dozing) {
            dozing = false;
            greeting = true;
            clearTimeout(greetTimer);
            greetTimer = setTimeout(() => (greeting = false), 1000);
          }
          const dx = l[0] - lastLx;
          const sign = Math.sign(dx);
          const nowMsTime = Date.now();
          if (sign !== 0 && sign !== lastSign && Math.abs(dx) > 0.22 && l[2] > 0.3) {
            if (nowMsTime - lastDirChangeMs < 480) {
              dirChanges++;
              if (dirChanges >= 5 && !dizzy) {
                dizzy = true;
                dirChanges = 0;
                clearTimeout(dizzyTimeout);
                dizzyTimeout = setTimeout(() => (dizzy = false), 2200);
              }
            } else {
              dirChanges = 1;
            }
            lastDirChangeMs = nowMsTime;
            lastSign = sign;
          }
          lastLx = l[0];
          if (mascotMode === "always") {
            clearTimeout(sleepTimer);
            sleepTimer = setTimeout(() => {
              if (mascot && mascotMode === "always") dozing = true;
            }, 60_000);
          } else if (activeEyes) {
            clearTimeout(activeEyesTimer);
            activeEyesTimer = setTimeout(() => {
              if (mascotMode === "idle") {
                activeEyes = false;
                dozing = false;
                wake();
              }
            }, 25_000);
          } else {
            clearTimeout(sleepTimer);
            sleepTimer = setTimeout(() => {
              if (mascot && mascotMode === "idle") {
                activeEyes = true;
                dozing = true;
              }
            }, 120_000);
          }
        }
      },
      onTucked: (t) => (tucked = t),
      onMedia: (m) => {
        media = m && { ...m, receivedAt: Date.now() };
        lineNow = Date.now();
        if (!m) lyricsOn = false;
      },
      onPeek: (p) => {
        if (!privacyIndicators && (p.kind === "camera" || p.kind === "mic-in-use")) return;
        if (hidden && presentationSummary && p.kind === "notification") {
          held.push(String(p.app || p.title || ""));
          return;
        }
        if (mascot && (p.kind === "notification" || p.kind === "meeting" || p.kind === "download-done" || p.kind === "screenshot")) {
          shocked = true;
          clearTimeout(shockedTimer);
          shockedTimer = setTimeout(() => (shocked = false), 600);
        }
        peekMgr.show(p, hovered);
        if (p.kind === "screenshot") shelf.add([p.path]);
      },
      onPower: (p) => (power = p),
      onInUse: (u) => (inUse = u),
      onMicMuted: (m) => (micMuted = m),
      onSettings: (s) => applySettings(s),
      onAccentColor: (a) => (accentColor = a),
      onHidden: (h) => {
        hidden = h;
        if (!h && held.length) {
          const names = [...new Set(held)].filter(Boolean);
          showPeek({ kind: "held", count: held.length, names: names.slice(0, 3).join(", ") });
          held = [];
        }
      },
      onHotkey: () => {
        if (keyboardMode) return endKeyboardMode();
        keyboardMode = true;
        open = true;
        setPinned(true);
        setTyping(true);
      },
      onKeepAwake: (until) => {
        const was = keepAwake;
        keepAwake = until;
        if (!!until !== !!was) {
          const label = until > 4e15 ? t("peek.untilOff") : `${Math.round((until - Date.now()) / 60_000)} min`;
          showPeek({ kind: "keep-awake", on: !!until, label });
        }
      },
      onKeyLock: (until) => {
        keyLockUntil = until;
        if (until) {
          open = false;
          setPinned(true);
        } else {
          setPinned(false);
        }
      },
      onAgent: (a) => {
        clearTimeout(agentTimer);
        if (a.state === "working" || a.state === "waiting") {
          agent = { waiting: a.state === "waiting" };
          if (a.state === "waiting") showPeek({ kind: "agent", state: "waiting", text: a.text });
          // A session that stops reporting (closed terminal) clears itself.
          agentTimer = setTimeout(() => (agent = null), 20 * 60_000);
        } else {
          if (agent && a.state === "done") showPeek({ kind: "agent", state: "done", text: a.text });
          agent = null;
        }
      },
      onMonitorLeave: async () => {
        if (monitorTransition !== "idle") return;
        if (springPhysics === "off") {
          await invoke("move_to_target_monitor");
          await invoke("finish_monitor_transition");
          return;
        }
        if (open) open = false;
        monitorTransition = "leaving";
        await new Promise((r) => setTimeout(r, 200));

        await invoke("move_to_target_monitor");

        monitorTransition = "entering";
        await new Promise((r) => requestAnimationFrame(() => setTimeout(r, 20)));
        monitorTransition = "idle";
        await new Promise((r) => setTimeout(r, 340));
        await invoke("finish_monitor_transition");
      },
    }).then((cleanup) => (unlistenEvents = cleanup));

    // Drag and drop setup
    const zoneAt = (p: { x: number; y: number }) => {
      const el = document.elementFromPoint(p.x / devicePixelRatio, p.y / devicePixelRatio);
      return (el?.closest("[data-drop]") as HTMLElement | null)?.dataset.drop ?? null;
    };
    const unlistenDrop = getCurrentWebviewWindow().onDragDropEvent((e) => {
      if (e.payload.type === "enter" || e.payload.type === "over") {
        dropping = true;
        page = pages.indexOf("shelf");
        dropTarget = zoneAt(e.payload.position);
      } else if (e.payload.type === "drop") {
        const target = zoneAt(e.payload.position);
        dropping = false;
        dropTarget = null;
        if (target === "share") invoke("share_files", { paths: e.payload.paths }).catch((err) => logJs(`share: ${err}`));
        else shelf.add(e.payload.paths);
      } else {
        dropping = false;
        dropTarget = null;
      }
    });

    water.load();
    todo.load();
    shelf.load();
    wake();

    calendarPoll = setInterval(loadCalendar, 15 * 60_000);

        const checkPrayer = (nowDate: Date) => {
      if (!prayerEnabled || !prayerLocation) return;
      try {
        const times = calculatePrayerTimes(
          prayerLocation[0],
          prayerLocation[1],
          nowDate,
          prayerMethod as PrayerMethod,
        );
        const next = getNextPrayer(times, nowDate, {
          fajr: prayerFajr,
          dhuhr: prayerDhuhr,
          asr: prayerAsr,
          maghrib: prayerMaghrib,
          isha: prayerIsha,
        });
        if (!next) return;

        const dayKey = `${nowDate.getFullYear()}-${nowDate.getMonth() + 1}-${nowDate.getDate()}`;
        const prayerNameKey = `s.prayer${next.name.charAt(0).toUpperCase() + next.name.slice(1)}`;

        if (next.isExact) {
          const exactKey = `${dayKey}_${next.name}_exact`;
          if (!prayerAlerted.has(exactKey)) {
            prayerAlerted.add(exactKey);
            showPeek({
              kind: "prayer",
              preAlert: false,
              title: t("peek.prayerTime", { prayer: t(prayerNameKey) }),
              detail: t("peek.prayerTimeDetail", { time: next.time }),
            });
            if (prayerSound) chime();
          }
        } else if (prayerPreAlert && next.isPreAlert) {
          const preKey = `${dayKey}_${next.name}_pre`;
          if (!prayerAlerted.has(preKey)) {
            prayerAlerted.add(preKey);
            showPeek({
              kind: "prayer",
              preAlert: true,
              title: t("peek.prayer10Min", { prayer: t(prayerNameKey) }),
              detail: t("peek.prayer10MinDetail", { time: next.time }),
            });
          }
        }

        if (prayerAlerted.size > 50) {
          for (const k of prayerAlerted) {
            if (!k.startsWith(dayKey)) prayerAlerted.delete(k);
          }
        }
      } catch (err) {
        logJs(`prayer check: ${err}`);
      }
    };

    const schedule = () => {
      now = new Date();
      nowMs = Date.now();
      checkPrayer(now);
      tick = setTimeout(schedule, 60_000 - (Date.now() % 60_000));
    };
    schedule();

    let winkTimer: ReturnType<typeof setTimeout>;
    const scheduleBlink = () => {
      blinkTimer = setTimeout(() => {
        const rand = Math.random();
        if (rand < 0.28 && !dozing && !dizzy && !greeting) {
          wink = rand < 0.14 ? "left" : "right";
          winkTimer = setTimeout(() => (wink = null), 240);
        } else {
          blink = true;
          setTimeout(() => (blink = false), 180);
        }
        scheduleBlink();
      }, 3200 + Math.random() * 3000);
    };
    scheduleBlink();

    return () => {
      unlistenEvents?.();
      unlistenDrop.then((f) => f());
      peekMgr.clearTimer();
      clearTimeout(pressTimer);
      clearTimeout(shockedTimer);
      clearTimeout(tickleTimer);
      clearTimeout(weatherTimer);
      clearInterval(calendarPoll);
      clearInterval(rainPoll);
      clearTimeout(agentTimer);
      calendarTimers.forEach(clearTimeout);
      clearTimeout(closeTimer);
      clearTimeout(pageLabelTimer);
      clearTimeout(tick);
      clearTimeout(blinkTimer);
      clearTimeout(winkTimer);
      clearTimeout(dizzyTimeout);
      clearTimeout(sleepTimer);
      clearTimeout(greetTimer);
      clearTimeout(scrollVolumeTimer);
    };
  });

  let currentLanguage = $state<string>("en");
  const dateLocale = $derived(currentLanguage === "id" ? "id-ID" : "en-US");
  const date = $derived(now.toLocaleDateString(dateLocale, { weekday: "long", day: "numeric", month: "long" }));

  // Follow system clock preference (12h AM/PM vs 24h as set in Windows OS), never forced by UI language
  const systemHour12 = typeof Intl !== "undefined"
    ? new Intl.DateTimeFormat(undefined, { hour: "numeric" }).resolvedOptions().hour12
    : false;
  const time = $derived(
    now.toLocaleTimeString(undefined, {
      hour: "2-digit",
      minute: "2-digit",
      hour12: systemHour12,
    }).replace(".", ":")
  );
</script>

<svelte:window onkeydown={keydown} />

{#snippet statusRingSvg(value: number, inline = false)}
  <svg class="status-ring" class:inline viewBox="0 0 20 20" aria-label="{value}%">
    <title>{statusRing === "battery" ? `${value}%` : statusRing === "cpu" ? `CPU ${value}%` : statusRing === "ram" ? `RAM ${value}%` : `💧 ${value}%`}</title>
    <circle cx="10" cy="10" r="7.5" />
    <circle
      cx="10"
      cy="10"
      r="7.5"
      class="fill"
      class:water={statusRing === "water"}
      class:load={statusRing === "cpu" || statusRing === "ram"}
      class:busy={(statusRing === "cpu" || statusRing === "ram") && value >= 90}
      class:low={statusRing === "battery" && value <= 20}
      pathLength="100"
      stroke-dasharray="{value} 100"
    />
  </svg>
{/snippet}

<div
  class="notch"
  role="group"
  aria-label="Notchy"
  class:open={paged}
  class:tucked={view === "tucked"}
  class:minimal={view === "minimal"}
  class:floating
  class:ringed
  class:dotted={restDots}
  class:greet={greeting && view === "rest"}
  class:charging
  class:tinted
  class:accent-synced={windowsAccentGlow && !!accentColor}
  class:accent-default={!windowsAccentGlow}
  class:has-peek={!!peekMgr.peek}
  class:shocked
  class:tickled
  class:monitor-leaving={monitorTransition === "leaving"}
  class:monitor-entering={monitorTransition === "entering"}
  style:--tint={tint}
  style:--w="{size.w}px"
  style:--h="{size.h}px"
  style:--r="{size.r}px"
  style:--e="{size.e}px"
  style:--battery="{power.percent}%"
  style:--windows-accent-glow="{accentColor}44"
  style:--windows-accent-color={accentColor}
  style:--spring={activeSpring}
  onwheel={wheel}
  onpointerdown={pressDown}
  onpointermove={pressMove}
  onpointerup={pressUp}
  onpointerleave={pressUp}
>
  {#if pinned && paged && !mirror}<i class="pin" title="Terkunci terbuka · tekan lama untuk melepas"></i>{/if}
  <div class="ambient"></div>

  {#if withStatus}
    <StatusBar {power} {networkSpeed} {netSpeedStatus} {date} {time} />
  {/if}

  <div class="layer rest" class:on={view === "rest"}>
    {#if ringValue !== null}{@render statusRingSvg(ringValue)}{/if}
    <RestEyes
      {weather}
      {greeting}
      {dozing}
      {blink}
      {wink}
      {dizzy}
      musicPlaying={!!media?.playing}
      {look}
      {time}
      {mascot}
      {mascotMode}
      {mascotTheme}
      {activeEyes}
      {shocked}
      {tickled}
    />
  </div>

  {#if island.main}
    <div class="layer" class:on={view === "island"}>
      <Island
        main={island.main}
        side={island.side}
        {media}
        line={smallLine}
        {time}
        {pomo}
        now={nowMs}
        {levels}
        {micMuted}
        active={view === "island"}
        camera={camOn}
        mic={micOn && !call}
        {mascot}
        {mascotMode}
        {mascotTheme}
        {activeEyes}
        {dozing}
        {blink}
        {wink}
        {dizzy}
        {musicMascot}
        {look}
        {greeting}
        {shocked}
        {tickled}
      />
    </div>
  {/if}

  <div class="layer charge" class:on={view === "charging"}>
    <svg class="ring" viewBox="0 0 20 20">
      <circle cx="10" cy="10" r="8" />
      <circle cx="10" cy="10" r="8" class="fill" pathLength="100" stroke-dasharray="{power.percent} 100" />
      <path d="M10.6 5.5 7.5 10.6h2.4l-.5 3.9 3.1-5.1h-2.4z" />
    </svg>

    {#if mascot}
      <div class="charge-eyes">
        <RestEyes
          weather={null}
          {greeting}
          {dozing}
          {blink}
          {wink}
          {dizzy}
          musicPlaying={!!media?.playing}
          {look}
          {time}
          {mascot}
          {mascotMode}
          {mascotTheme}
          {activeEyes}
          {shocked}
          {tickled}
          charging={true}
          batteryPercent={power.percent}
        />
      </div>
    {/if}

    <span class="charge-time">{time}</span>
    {#if statusRing === "water" && ringValue !== null}{@render statusRingSvg(ringValue, true)}{/if}
  </div>

  <div class="layer clock" class:on={view === "clock"}>
    <ClockPage {time} {date} {weather} {nextEvent} {clockOf} />
  </div>

  {#if peekMgr.peek && (view === "peek" || view === "shot" || view === "notify")}
    <div class="layer on" in:fade={{ duration: 220, delay: 120 }} out:fade={{ duration: 100 }}>
      <Peek
        peek={peekMgr.peek}
        onOpen={() => peekMgr.peek && peekMgr.openFromPeek(peekMgr.peek)}
        onAction={(a) => {
          if (a === "camera") openMirror(nextEvent?.title ?? "", nextEvent?.link ?? "");
          if (a === "join" && peekMgr.peek?.link) invoke("run_link", { url: peekMgr.peek.link });
          peekMgr.end();
        }}
      />
    </div>
  {/if}

  {#if keyLockUntil}
    {@const left = Math.max(0, Math.ceil((keyLockUntil - nowMs) / 1000))}
    <div class="layer on keylock" in:fade={{ duration: 220, delay: 120 }} out:fade={{ duration: 100 }}>
      <span class="lock-ring">
        <svg viewBox="0 0 40 40"><circle cx="20" cy="20" r="17" /><circle cx="20" cy="20" r="17" class="fill" pathLength="100" stroke-dasharray="{Math.min(100, (left / 60) * 100)} 100" /></svg>
        <svg class="lock" viewBox="0 0 24 24"><path d="M6 11h12v10H6zM8 11V7a4 4 0 0 1 8 0v4" /></svg>
      </span>
      <span class="lock-text"><b>{t("keylock.title")}</b><small>{t("keylock.detail", { s: left })}</small></span>
      <button class="unlock" onclick={() => invoke("unlock_keyboard")}>{t("keylock.unlock")}</button>
    </div>
  {/if}

  {#if mirror}
    <div class="layer on" in:fade={{ duration: 260, delay: 160 }} out:fade={{ duration: 100 }}>
      <Mirror title={mirror.title} link={mirror.link} onDone={closeMirror} onJoin={joinMeeting} />
    </div>
  {/if}

  {#if closedView && view !== "island" && (camOn || micOn || micMuted || keepAwake)}
    <div class="dots" class:wide={view !== "rest"}>
      {#if keepAwake}<svg class="awake" viewBox="0 0 24 24"><path d="M4 9h13v5a5 5 0 0 1-5 5H9a5 5 0 0 1-5-5zM17 11h1.5a2.5 2.5 0 0 1 0 5H17" /></svg>{/if}
      {#if camOn}<i class="cam" title="Kamera: {inUse.camera.join(', ')}"></i>{/if}
      {#if micMuted}
        <svg class="muted" viewBox="0 0 24 24"><path d="M9 9v3a3 3 0 0 0 5.1 2.1M15 9.3V6a3 3 0 0 0-5.7-1.3M19 11a7 7 0 0 1-1.2 3.9M5 11a7 7 0 0 0 10.6 6M12 18v3M3 3l18 18" /></svg>
      {:else if micOn}<i class="mic" title="Mikrofon: {inUse.mic.join(', ')}"></i>{/if}
    </div>
  {/if}

  {#if view === "clock"}
    <button
      class="mic-toggle"
      class:muted={micMuted}
      class:live={inUse.mic.length > 0}
      onclick={() => invoke("toggle_mic")}
      title={micMuted ? "Nyalakan mikrofon" : "Matikan mikrofon"}
      aria-label={micMuted ? "Nyalakan mikrofon" : "Matikan mikrofon"}
      in:fade={{ duration: 200, delay: 200 }}
    >
      <svg viewBox="0 0 24 24">
        {#if micMuted}
          <path d="M9 9v3a3 3 0 0 0 5.1 2.1M15 9.3V6a3 3 0 0 0-5.7-1.3M19 11a7 7 0 0 1-14 0M12 18v3" />
        {:else}
          <path d="M12 3a3 3 0 0 0-3 3v6a3 3 0 0 0 6 0V6a3 3 0 0 0-3-3zM19 11a7 7 0 0 1-14 0M12 18v3" />
        {/if}
      </svg>
      {#if inUse.camera.length}<i class="cam"></i>{/if}
    </button>
    {#if output}
      <button
        class="mic-toggle output"
        onclick={switchOutput}
        title="Audio: {output.name} (klik untuk ganti)"
        aria-label="Ganti output audio, sekarang {output.name}"
        in:fade={{ duration: 200, delay: 200 }}
      >
        <svg viewBox="0 0 24 24">
          {#if output.headphones}
            <path d="M3 14v-2a9 9 0 0 1 18 0v2M3 14h4v6H3zM17 14h4v6h-4z" />
          {:else}
            <path d="M11 5 6 9H3v6h3l5 4zM15.5 8.5a5 5 0 0 1 0 7M18.5 5.5a9 9 0 0 1 0 13" />
          {/if}
        </svg>
      </button>
    {/if}
    <button
      class="mic-toggle camera"
      onclick={() => openMirror(nextEvent?.link ? nextEvent.title : "", nextEvent?.link ?? "")}
      title="Cek kamera"
      aria-label="Cek kamera"
      in:fade={{ duration: 200, delay: 200 }}
    >
      <svg viewBox="0 0 24 24"><path d="M15 10l5-3v10l-5-3zM3 7h12v10H3z" /></svg>
    </button>
  {/if}

  {#if media && (view === "player" || view === "lyrics")}
    <div class="layer on below" in:fade={{ duration: 300, delay: 180 }} out:fade={{ duration: 120 }}>
      <Player
        {media}
        now={nowMs}
        clock=""
        withLyrics={lyricsOn}
        {lyrics}
        status={lyricsStatus}
        onLyrics={() => (lyricsOn = !lyricsOn)}
        onSource={nextSource}
      />
    </div>
  {/if}

  {#if paged && !mirror && view !== "peek" && pages[page] !== "now"}
    {#key page}
      <div
        class="layer on below"
        in:fly={{ x: pageDirection * 48, duration: 220, delay: 50, easing: cubicOut }}
        out:fly={{ x: -pageDirection * 48, duration: 120, easing: cubicOut }}
      >
        {#if pages[page] === "today"}
          <TodayPage {todo} {water} {weather} {nextEvent} {clockOf} waterOn={waterEnabled} countdowns={countdownsAhead} />
        {:else if pages[page] === "sound"}
          <SoundPage {output} onSwitch={switchOutput} />
        {:else if pages[page] === "snippets"}
          <SnippetsPage onTyping={setTyping} />
        {:else if pages[page] === "prompter"}
          <PrompterPage onTyping={setTyping} />
        {:else if pages[page] === "pomodoro"}
          <TimerPage {pomo} now={nowMs} />
        {:else if pages[page] === "water"}
          <WaterPage {water} />
        {:else if pages[page] === "todo"}
          <TodoPage {todo} onTyping={setTyping} />
        {:else if pages[page] === "clipboard"}
          <ClipboardPage items={clipItems} />
        {:else if pages[page] === "shortcuts"}
          <ShortcutsPage {shortcuts} {keepAwake} now={nowMs} onCamera={() => openMirror(nextEvent?.link ? nextEvent.title : "", nextEvent?.link ?? "")} />
        {:else}
          <ShelfPage {shelf} {dropping} {dropTarget} />
        {/if}
      </div>
    {/key}
  {/if}

  {#if paged && !mirror && view !== "lyrics" && view !== "peek"}
    {#each [-1, 1] as step}
      <button
        class="arrow"
        class:next={step > 0}
        onclick={() => goPage(page + step, step)}
        aria-label={step > 0 ? t("main.nextPage") : t("main.prevPage")}
        in:fade={{ duration: 200, delay: 200 }}
      >
        <svg viewBox="0 0 24 24"><path d={step > 0 ? "M9 6l6 6-6 6" : "M15 6l-6 6 6 6"} /></svg>
      </button>
    {/each}
    {#if hoveredPage || showPageLabel}
      <div class="page-tooltip" transition:fade={{ duration: 150 }}>
        {getPageLabel(hoveredPage ?? pages[page])}
      </div>
    {/if}
    <div class="pager" role="tablist" tabindex="-1" in:fade={{ duration: 200, delay: 200 }} onpointerleave={() => (hoveredPage = null)}>
      {#each pages as name, i}
        <button
          role="tab"
          aria-selected={i === page}
          aria-label={getPageLabel(name)}
          class:on={i === page}
          onpointerenter={() => (hoveredPage = name)}
          onclick={() => goPage(i, i >= page ? 1 : -1)}
        ></button>
      {/each}
    </div>
  {/if}
</div>

<style>
  :global(html),
  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
    user-select: none;
    -webkit-user-select: none;
    background: transparent !important;
    background-color: transparent !important;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI Variable Text", "Segoe UI", sans-serif;
    color: #fff;
  }

  .notch {
    position: absolute;
    top: 0;
    left: 50%;
    width: var(--w);
    height: var(--h);
    transform: translateX(-50%);
    transform-origin: top center;
    border-top: none !important;
    outline: none !important;
    background: #000;
    border-radius: 0 0 var(--r) var(--r);
    box-shadow:
      0 4px 14px rgba(0, 0, 0, 0.22),
      0 10px 30px rgba(0, 0, 0, 0.35);
    transition:
      width 0.62s var(--spring),
      height 0.62s var(--spring),
      transform 0.32s var(--spring),
      opacity 0.22s ease,
      border-radius 0.45s ease,
      box-shadow 0.45s ease;
  }

  /* Multi-monitor zero-flicker spring transition */
  .notch.monitor-leaving {
    transform: translateX(-50%) scaleY(0) scaleX(0.7) !important;
    transform-origin: top center !important;
    opacity: 0 !important;
    pointer-events: none !important;
    transition:
      transform 0.2s var(--spring),
      opacity 0.16s ease !important;
  }

  .notch.monitor-entering {
    transform: translateX(-50%) scaleY(0) scaleX(0.7) !important;
    transform-origin: top center !important;
    opacity: 0 !important;
    pointer-events: none !important;
    transition: none !important;
  }
  .notch.open {
    box-shadow:
      0 6px 18px rgba(0, 0, 0, 0.2),
      0 16px 42px rgba(0, 0, 0, 0.4);
  }

  /* Dynamic Accent Glow */
  .notch.accent-synced.has-peek {
    box-shadow:
      0 0 22px -2px var(--windows-accent-glow),
      0 8px 24px rgba(0, 0, 0, 0.35);
    border-bottom: 1.5px solid var(--windows-accent-color);
  }
  .notch.accent-default.has-peek {
    box-shadow:
      0 0 16px -2px rgba(255, 255, 255, 0.2),
      0 8px 24px rgba(0, 0, 0, 0.35);
    border-bottom: 1px solid rgba(255, 255, 255, 0.15);
  }

  .notch::before,
  .notch::after {
    content: "";
    position: absolute;
    top: 0;
    width: calc(var(--e) + 1px);
    height: var(--e);
    background-size: 100% 100%;
    background-repeat: no-repeat;
    pointer-events: none;
    transition:
      width 0.62s var(--spring),
      height 0.62s var(--spring);
  }
  .notch::before {
    right: calc(100% - 1px);
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 101 100' preserveAspectRatio='none'%3E%3Cpath d='M0 0 A100 100 0 0 1 100 100 L101 100 L101 0 Z' fill='%23000'/%3E%3C/svg%3E");
  }
  .notch::after {
    left: calc(100% - 1px);
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 101 100' preserveAspectRatio='none'%3E%3Cpath d='M101 0 A100 100 0 0 0 1 100 L0 100 L0 0 Z' fill='%23000'/%3E%3C/svg%3E");
  }

  /* Floating style: a pill a little below the top edge, no ears. */
  .notch.floating {
    top: 8px;
    border-radius: var(--r);
  }
  .notch.floating::before,
  .notch.floating::after {
    display: none;
  }
  .notch.floating.minimal {
    border-radius: 3px;
  }

  .status-ring {
    position: absolute;
    left: 11px;
    top: 50%;
    width: 16px;
    height: 16px;
    margin-top: -8px;
    transform: rotate(-90deg);
  }
  /* Water ring in the charging view, beside the battery ring. */
  .status-ring.inline {
    left: 40px;
  }
  .status-ring circle {
    fill: none;
    stroke: rgba(255, 255, 255, 0.16);
    stroke-width: 2.6;
  }
  .status-ring .fill {
    stroke: #34d399;
    stroke-linecap: round;
    transition: stroke-dasharray 0.6s ease;
  }
  .status-ring .fill.low {
    stroke: #f87171;
  }
  .status-ring .fill.water {
    stroke: #38bdf8;
  }
  .status-ring .fill.load {
    stroke: #a78bfa;
  }
  .status-ring .fill.load.busy {
    stroke: #f87171;
  }
  .layer.keylock {
    flex-direction: row;
    gap: 14px;
    padding: 0 22px;
  }
  .lock-ring {
    position: relative;
    flex: none;
    width: 46px;
    height: 46px;
    color: #fbbf24;
  }
  .lock-ring svg:first-child {
    width: 46px;
    height: 46px;
    transform: rotate(-90deg);
  }
  .lock-ring circle {
    fill: none;
    stroke: currentColor;
    stroke-opacity: 0.2;
    stroke-width: 3.5;
  }
  .lock-ring .fill {
    stroke-opacity: 1;
    stroke-linecap: round;
    transition: stroke-dasharray 1s linear;
  }
  .lock-ring .lock {
    position: absolute;
    inset: 14px;
    width: 18px;
    height: 18px;
    fill: none;
    stroke: currentColor;
    stroke-width: 2;
  }
  .lock-text {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  .lock-text b {
    font-size: 14px;
  }
  .lock-text small {
    font-size: 11.5px;
    color: rgba(255, 255, 255, 0.6);
  }
  .unlock {
    flex: none;
    border: 0;
    padding: 7px 14px;
    border-radius: 999px;
    background: #fff;
    color: #000;
    font: inherit;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
  }
  .dotted :global(.rest-time-side) {
    right: 30px;
  }
  .ringed :global(.weather) {
    left: 33px;
  }

  .ambient {
    position: absolute;
    inset: auto 0 0;
    height: 70%;
    border-radius: inherit;
    pointer-events: none;
    background: linear-gradient(to top, rgba(255, 255, 255, 0.09), transparent);
    transition:
      opacity 0.45s ease,
      height 0.45s ease;
  }
  .open .ambient {
    height: 40%;
  }
  .tinted .ambient {
    height: 100%;
    background: linear-gradient(to top, rgb(var(--tint) / 0.6), rgb(var(--tint) / 0.12) 55%, transparent);
  }
  .tinted:not(.open) .ambient {
    background: linear-gradient(to top, rgb(var(--tint) / 0.4), transparent);
  }

  .tucked .ambient {
    height: 100%;
    background: rgba(255, 255, 255, 0.22);
  }
  .notch.tucked.charging {
    box-shadow: 0 2px 10px -2px rgba(52, 211, 153, 0.75);
  }
  .tucked.charging .ambient {
    background: linear-gradient(90deg, #34d399 var(--battery), rgba(52, 211, 153, 0.28) var(--battery));
  }

  .notch.shocked {
    animation: shocked-hop 0.5s cubic-bezier(0.34, 1.56, 0.64, 1);
  }
  @keyframes shocked-hop {
    0% {
      transform: translateX(-50%) translateY(0);
    }
    35% {
      transform: translateX(-50%) translateY(-5px) scale(1.02);
    }
    70% {
      transform: translateX(-50%) translateY(1px);
    }
    100% {
      transform: translateX(-50%) translateY(0);
    }
  }
  .notch.tickled {
    animation: notch-tickle-wiggle 0.11s ease-in-out infinite alternate;
  }
  @keyframes notch-tickle-wiggle {
    0% {
      transform: translateX(calc(-50% - 2.5px));
    }
    100% {
      transform: translateX(calc(-50% + 2.5px));
    }
  }

  .layer {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    opacity: 0;
    pointer-events: none;
    transition: opacity 0.12s;
  }
  .layer.on {
    opacity: 1;
    pointer-events: auto;
    transition: opacity 0.3s 0.15s;
  }
  .layer.below {
    top: 16px;
  }

  .charge {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .charge .ring {
    position: absolute;
    left: 14px;
    top: 5px;
    width: 20px;
    height: 20px;
    transform: rotate(-90deg);
    flex-shrink: 0;
  }
  .ring circle {
    fill: none;
    stroke: rgba(255, 255, 255, 0.15);
    stroke-width: 2.2;
  }
  .ring .fill {
    stroke: #34d399;
  }
  .ring path {
    fill: #34d399;
    transform: rotate(90deg);
    transform-origin: 10px 10px;
  }
  .charge-eyes {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    pointer-events: none;
  }
  .charge-time {
    position: absolute;
    right: 14px;
    top: 50%;
    transform: translateY(-50%);
    font-size: 11.5px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    letter-spacing: -0.01em;
    color: #e6edf3;
    white-space: nowrap;
    display: flex;
    align-items: center;
    line-height: 1;
  }

  .dots {
    position: absolute;
    top: 50%;
    right: 12px;
    transform: translateY(-50%);
    display: flex;
    align-items: center;
    gap: 5px;
    pointer-events: none;
  }
  .dots.wide {
    right: 14px;
  }
  .dots i {
    width: 6px;
    height: 6px;
    border-radius: 50%;
  }
  .dots .cam {
    background: #34d399;
    box-shadow: 0 0 6px rgba(52, 211, 153, 0.6);
  }
  .dots .mic {
    background: #fb923c;
    box-shadow: 0 0 6px rgba(251, 146, 60, 0.6);
  }
  .dots .awake {
    width: 12px;
    height: 12px;
    fill: none;
    stroke: #34d399;
    stroke-width: 2.2;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .dots .muted {
    width: 13px;
    height: 13px;
    fill: none;
    stroke: #f87171;
    stroke-width: 2.2;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .mic-toggle {
    position: absolute;
    top: 14px;
    left: 14px;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    border: none;
    background: rgba(255, 255, 255, 0.08);
    color: #fff;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.15s;
  }
  .mic-toggle:hover {
    background: rgba(255, 255, 255, 0.16);
  }
  .mic-toggle svg {
    width: 14px;
    height: 14px;
    fill: none;
    stroke: currentColor;
    stroke-width: 2;
  }
  .mic-toggle.muted {
    color: #f87171;
  }
  .mic-toggle.live:not(.muted) {
    color: #fb923c;
  }
  .mic-toggle .cam {
    position: absolute;
    top: 0;
    right: -2px;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #34d399;
  }
  .mic-toggle.output {
    left: 48px;
  }
  .mic-toggle.camera {
    left: auto;
    right: 14px;
  }

  .arrow {
    position: absolute;
    top: 50%;
    left: 10px;
    transform: translateY(-50%);
    width: 24px;
    height: 24px;
    border-radius: 50%;
    border: none;
    background: rgba(255, 255, 255, 0.06);
    color: rgba(255, 255, 255, 0.7);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.15s;
    z-index: 10;
  }
  .arrow:hover {
    background: rgba(255, 255, 255, 0.14);
    color: #fff;
  }
  .arrow.next {
    left: auto;
    right: 10px;
  }
  .arrow svg {
    width: 14px;
    height: 14px;
    fill: none;
    stroke: currentColor;
    stroke-width: 2.2;
  }

  .pager {
    position: absolute;
    bottom: 8px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    gap: 6px;
    padding: 3px 6px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.06);
    z-index: 10;
  }
  .pager button {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    border: none;
    padding: 0;
    background: rgba(255, 255, 255, 0.25);
    cursor: pointer;
    transition:
      width 0.2s,
      background 0.2s;
  }
  .pager button.on {
    width: 14px;
    border-radius: 3px;
    background: #fff;
  }

  .page-tooltip {
    position: absolute;
    bottom: 28px;
    left: 50%;
    transform: translateX(-50%);
    background: rgba(15, 18, 26, 0.94);
    border: 1px solid rgba(255, 255, 255, 0.18);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(10px);
    -webkit-backdrop-filter: blur(10px);
    border-radius: 999px;
    padding: 2px 12px;
    font-size: 11px;
    font-weight: 600;
    color: #f0f6fc;
    letter-spacing: 0.01em;
    pointer-events: none;
    z-index: 50;
    white-space: nowrap;
  }

  .pin {
    position: absolute;
    top: 6px;
    right: 12px;
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: #38bdf8;
    box-shadow: 0 0 6px #38bdf8;
    pointer-events: none;
  }
</style>
