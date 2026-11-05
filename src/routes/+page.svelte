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
  import StatusBar from "$lib/StatusBar.svelte";
  import TimerPage from "$lib/TimerPage.svelte";
  import TodoPage from "$lib/TodoPage.svelte";
  import WaterPage from "$lib/WaterPage.svelte";
  import { arrange, islandWidth, type Activity, type Call, type Download } from "$lib/island";
  import { positionAt, type NowPlaying } from "$lib/media";
  import { cleanTrack, currentLine, findLyrics, type Lyrics } from "$lib/lyrics";
  import { Pomodoro, Shelf, Todo, Water, type Settings } from "$lib/productivity.svelte";
  import { fetchWeather, type Weather } from "$lib/weather";
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
    todo: { w: 420, h: 154, r: 32, e: 22 },
    shelf: { w: 420, h: 154, r: 32, e: 22 },
    clipboard: { w: 420, h: 154, r: 32, e: 22 },
    shortcuts: { w: 420, h: 154, r: 32, e: 22 },
    mirror: { w: 420, h: 140, r: 32, e: 22 },
  };

  let musicEnabled = $state(true);
  let pomodoroEnabled = $state(true);
  let waterEnabled = $state(true);
  let todoEnabled = $state(true);
  let shelfEnabled = $state(true);
  let clipboardEnabled = $state(true);
  let shortcutsEnabled = $state(true);

  const pages = $derived(
    filterPages(ALL_PAGES, {
      musicEnabled,
      pomodoroEnabled,
      waterEnabled,
      todoEnabled,
      shelfEnabled,
      clipboardEnabled,
      shortcutsEnabled,
    })
  );

  let open = $state(false);
  let tucked = $state(false);
  let blink = $state(false);
  let dozing = $state(false);
  let greeting = $state(false);
  let mascot = $state(true);
  let mascotMode = $state("idle");
  let activeEyes = $state(false);
  let minimalNotch = $state(false);
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

  const charging = $derived(power.battery && power.charging);
  const timerActive = $derived(pomo.running || pomo.left < pomo.length());
  const paged = $derived(open && !peekMgr.clickable(peekMgr.peek));

  // What runs right now, for the closed notch
  const running = $derived.by((): Activity[] => [
    ...(call ? [{ kind: "call", call } as const] : []),
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

  const islandDots = $derived((inUse.camera.length > 0 ? 10 : 0) + (inUse.mic.length > 0 && !call ? 10 : 0));
  const size = $derived(
    view === "island" && island.main
      ? { ...SIZES.island, w: islandWidth(island.main, island.side) + islandDots }
      : view === "rest" && (weather || (mascot && mascotMode === "always"))
        ? { ...SIZES.rest, w: weather && mascot && mascotMode === "always" ? 184 : 170 }
        : SIZES[view as keyof typeof SIZES] ?? SIZES.rest,
  );
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
    invoke("set_notch_size", { w: size.w, h: view === "minimal" ? 10 : size.h });
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
    minimalNotch: boolean;
    musicEnabled?: boolean;
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
    language?: string;
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
          }, 30_000);
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

    const loadWeather = async (s?: AllSettings) => {
      clearTimeout(weatherTimer);
      s ??= await invoke<AllSettings>("get_settings");
      const city = s.weatherCity;
      const at = city ? [city.latitude, city.longitude] : await invoke<[number, number] | null>("locate");
      weather = at ? await fetchWeather(at[0], at[1]) : null;
      weatherTimer = setTimeout(loadWeather, 30 * 60_000);
    };

    const applySettings = (s: AllSettings) => {
      smallLyrics = s.smallLyrics && s.musicEnabled !== false;
      mascot = s.mascot !== false;
      mascotMode = s.mascotMode ?? "idle";
      if (!mascot) {
        dozing = false;
        activeEyes = false;
      }
      wake();
      minimalNotch = s.minimalNotch === true;
      shortcuts = s.shortcuts ?? [];
      const current = pages[page];
      musicEnabled = s.musicEnabled !== false;
      pomodoroEnabled = s.pomodoroEnabled !== false;
      waterEnabled = s.waterEnabled !== false;
      todoEnabled = s.todoEnabled !== false;
      shelfEnabled = s.shelfEnabled !== false;
      clipboardEnabled = s.clipboardEnabled !== false;
      shortcutsEnabled = s.shortcutsEnabled !== false;
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
      pomo.configure(s);
      water.configure(s);
      loadWeather(s);
      loadCalendar();
    };

    invoke<string>("get_accent_color").then((c) => (accentColor = c)).catch(() => {});
    invoke<AllSettings>("get_settings").then(applySettings);

    setupNotchEvents({
      onNotchHover: (h) => {
        hovered = h;
        if (h && activeEyes) {
          activeEyes = false;
          dozing = false;
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
          if (mascotMode === "always") {
            if (dozing) {
              dozing = false;
              greeting = true;
              clearTimeout(greetTimer);
              greetTimer = setTimeout(() => (greeting = false), 1000);
            }
            clearTimeout(sleepTimer);
            sleepTimer = setTimeout(() => {
              if (mascot && mascotMode === "always") dozing = true;
            }, 30_000);
          } else if (activeEyes) {
            if (dozing) {
              dozing = false;
              greeting = true;
              clearTimeout(greetTimer);
              greetTimer = setTimeout(() => (greeting = false), 1000);
            }
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
        peekMgr.show(p, hovered);
        if (p.kind === "screenshot") shelf.add([p.path]);
      },
      onPower: (p) => (power = p),
      onInUse: (u) => (inUse = u),
      onMicMuted: (m) => (micMuted = m),
      onSettings: (s) => applySettings(s),
      onAccentColor: (a) => (accentColor = a),
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

    const schedule = () => {
      now = new Date();
      nowMs = Date.now();
      tick = setTimeout(schedule, 60_000 - (Date.now() % 60_000));
    };
    schedule();

    const scheduleBlink = () => {
      blinkTimer = setTimeout(() => {
        blink = true;
        setTimeout(() => (blink = false), 180);
        scheduleBlink();
      }, 3500 + Math.random() * 3000);
    };
    scheduleBlink();

    return () => {
      unlistenEvents?.();
      unlistenDrop.then((f) => f());
      peekMgr.clearTimer();
      clearTimeout(pressTimer);
      clearTimeout(weatherTimer);
      clearInterval(calendarPoll);
      calendarTimers.forEach(clearTimeout);
      clearTimeout(closeTimer);
      clearTimeout(pageLabelTimer);
      clearTimeout(tick);
      clearTimeout(blinkTimer);
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

<div
  class="notch"
  role="group"
  aria-label="Notchy"
  class:open={paged}
  class:tucked={view === "tucked"}
  class:minimal={view === "minimal"}
  class:greet={greeting && view === "rest"}
  class:charging
  class:tinted
  class:accent-synced={windowsAccentGlow && !!accentColor}
  class:accent-default={!windowsAccentGlow}
  class:has-peek={!!peekMgr.peek}
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
    <RestEyes {weather} {greeting} {dozing} {blink} {look} {time} {mascot} {mascotMode} {activeEyes} />
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
        camera={inUse.camera.length > 0}
        mic={inUse.mic.length > 0 && !call}
      />
    </div>
  {/if}

  <div class="layer charge" class:on={view === "charging"}>
    <svg class="ring" viewBox="0 0 20 20">
      <circle cx="10" cy="10" r="8" />
      <circle cx="10" cy="10" r="8" class="fill" pathLength="100" stroke-dasharray="{power.percent} 100" />
      <path d="M10.6 5.5 7.5 10.6h2.4l-.5 3.9 3.1-5.1h-2.4z" />
    </svg>
    <span class="charge-time">{time}</span>
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

  {#if mirror}
    <div class="layer on" in:fade={{ duration: 260, delay: 160 }} out:fade={{ duration: 100 }}>
      <Mirror title={mirror.title} link={mirror.link} onDone={closeMirror} onJoin={joinMeeting} />
    </div>
  {/if}

  {#if closedView && view !== "island" && (inUse.camera.length || inUse.mic.length || micMuted)}
    <div class="dots" class:wide={view !== "rest"}>
      {#if inUse.camera.length}<i class="cam" title="Kamera: {inUse.camera.join(', ')}"></i>{/if}
      {#if micMuted}
        <svg class="muted" viewBox="0 0 24 24"><path d="M9 9v3a3 3 0 0 0 5.1 2.1M15 9.3V6a3 3 0 0 0-5.7-1.3M19 11a7 7 0 0 1-1.2 3.9M5 11a7 7 0 0 0 10.6 6M12 18v3M3 3l18 18" /></svg>
      {:else if inUse.mic.length}<i class="mic" title="Mikrofon: {inUse.mic.join(', ')}"></i>{/if}
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
        {#if pages[page] === "pomodoro"}
          <TimerPage {pomo} now={nowMs} />
        {:else if pages[page] === "water"}
          <WaterPage {water} />
        {:else if pages[page] === "todo"}
          <TodoPage {todo} onTyping={setTyping} />
        {:else if pages[page] === "clipboard"}
          <ClipboardPage items={clipItems} />
        {:else if pages[page] === "shortcuts"}
          <ShortcutsPage {shortcuts} onCamera={() => openMirror(nextEvent?.link ? nextEvent.title : "", nextEvent?.link ?? "")} />
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
  :global(body) {
    margin: 0;
    overflow: hidden;
    user-select: none;
    -webkit-user-select: none;
    background: transparent;
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
    transform: translateX(-50%) translateY(-115%) scaleX(0.7) !important;
    opacity: 0 !important;
    pointer-events: none !important;
    transition:
      transform 0.2s var(--spring),
      opacity 0.16s ease !important;
  }

  .notch.monitor-entering {
    transform: translateX(-50%) translateY(-115%) scaleX(0.7) !important;
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
    flex-direction: row;
    gap: 16px;
  }
  .ring {
    width: 20px;
    height: 20px;
    transform: rotate(-90deg);
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
  .charge-time {
    font-size: 14px;
    font-weight: 700;
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
