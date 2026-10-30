<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { onMount, untrack } from "svelte";
  import { fade, fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import ClipboardPage, { type ClipEntry } from "$lib/ClipboardPage.svelte";
  import Island from "$lib/Island.svelte";
  import Mirror from "$lib/Mirror.svelte";
  import Peek, { type PeekData } from "$lib/Peek.svelte";
  import Player from "$lib/Player.svelte";
  import ShelfPage from "$lib/ShelfPage.svelte";
  import ShortcutsPage, { type Shortcut } from "$lib/ShortcutsPage.svelte";
  import { arrange, islandWidth, type Activity, type Call, type Download } from "$lib/island";
  import TimerPage from "$lib/TimerPage.svelte";
  import TodoPage from "$lib/TodoPage.svelte";
  import WaterPage from "$lib/WaterPage.svelte";
  import { positionAt, type NowPlaying } from "$lib/media";
  import { cleanTrack, currentLine, findLyrics, type Lyrics } from "$lib/lyrics";
  import { Pomodoro, Shelf, Todo, Water, type Settings } from "$lib/productivity.svelte";
  import { fetchWeather, type Weather } from "$lib/weather";
  import { upcoming, type CalEvent } from "$lib/calendar";
  import { ALL_PAGES, filterPages, PAGE_LABELS, resolvePageDirection, type PageName } from "$lib/pages";
  import { formatSpeed, isHighTraffic, type NetworkSpeed } from "$lib/network";

  // Black box for the page: errors go to notchy.log, and a heartbeat tells Rust the
  // page still runs and draws (sent from an animation frame, which only comes while
  // the page renders). Without beats for a while, Rust reloads the page.
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

  // Sizes from notchy.md §4 (logical px).
  // Open pages are 154 tall: a 16 px status row (battery, date, time) above the page.
  const SIZES = {
    tucked: { w: 90, h: 6, r: 4, e: 4 },
    // "Notch minimal": a closed, idle notch is a thin black line.
    minimal: { w: 160, h: 5, r: 3, e: 3 },
    rest: { w: 150, h: 30, r: 14, e: 10 },
    // The island's width depends on what it shows (lib/island.ts).
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

  // Pages of the open notch, switched with the dots or the mouse wheel.
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

  type Power = { charging: boolean; percent: number; battery: boolean };
  type InUse = { camera: string[]; mic: string[] };

  let open = $state(false);
  // The active window sits under the notch (e.g. a maximized browser): shrink to a thin line.
  let tucked = $state(false);
  let blink = $state(false);
  // Nobody has moved the mouse for a while: the clock rests and the eyes come out.
  let dozing = $state(false);
  // Back after a quiet while: the eyes smile hello for a moment.
  let greeting = $state(false);
  // Settings from the tray: eyes on/off, and the thin-line "minimal" notch.
  let mascot = $state(true);
  let minimalNotch = $state(false);
  // Where the eyes look (-1..1 per axis) and how close the cursor is (0..1), from Rust.
  let look = $state<[number, number, number]>([0, 0, 0]);
  let now = $state(new Date());
  let media = $state<NowPlaying | null>(null);
  let lyricsOn = $state(false);
  let peek = $state<PeekData | null>(null);
  let power = $state<Power>({ charging: false, percent: 100, battery: false });
  let inUse = $state<InUse>({ camera: [], mic: [] });
  let micMuted = $state(false);
  let weather = $state<Weather | null>(null);
  let page = $state(0);
  // Typing a task: the notch holds keyboard focus and stays open.
  let typing = $state(false);
  let hovered = false;
  // A file is being dragged over the notch, and which drop zone it's over.
  let dropping = $state(false);
  let dropTarget = $state<string | null>(null);
  // Things running at once, shown side by side in the closed notch.
  let call = $state<Call | null>(null);
  let download = $state<Download | null>(null);
  let networkSpeed = $state<NetworkSpeed | null>(null);
  let netSpeedIsland = $state(false);
  let netSpeedStatus = $state(true);
  let netSpeedThresholdDownKb = $state(500);
  let netSpeedThresholdUpKb = $state(250);
  // Equalizer bar heights from the audio meter, while it runs.
  let levels = $state<number[] | null>(null);
  // Horizontal swipes cycle which activity leads.
  let turn = $state(0);
  let clipItems = $state<ClipEntry[]>([]);
  let shortcuts = $state<Shortcut[]>([]);
  // Long-pressed open: stays open until long-pressed again or a click elsewhere.
  let pinned = $state(false);
  // Camera check in the notch, optionally for a meeting.
  let mirror = $state<{ title: string; link: string } | null>(null);
  // The wheel is changing the volume right now.
  let scrollingVolume = $state(false);
  let scrollVolumeTimer: ReturnType<typeof setTimeout>;

  // Status peeks: a few seconds each; volume/brightness drags keep extending it.
  // Notifications queue up instead of replacing each other, and a peek you can
  // click (notification, screenshot) stays put while the cursor is on it.
  let peekTimer: ReturnType<typeof setTimeout>;
  let peekQueue: PeekData[] = [];
  const clickable = (p: PeekData | null) =>
    ["notification", "screenshot", "update", "download-done", "meeting"].includes(p?.kind ?? "");
  const peekLength = (p: PeekData) =>
    p.kind === "meeting"
      ? 30_000
      : ["screenshot", "update", "download-done"].includes(p.kind)
        ? 6000
        : p.kind === "notification"
          ? peekQueue.length
            ? 3200
            : 4500
          : ["todo", "calendar", "update-info", "earbuds-low"].includes(p.kind)
            ? 5000
            : 2200;
  function showPeek(data: PeekData) {
    if (data.kind === "notification" && peek?.kind === "notification") {
      peekQueue.push(data);
      return;
    }
    peek = data;
    clearTimeout(peekTimer);
    if (!(clickable(data) && hovered)) peekTimer = setTimeout(endPeek, peekLength(data));
  }
  function endPeek() {
    peek = null;
    const next = peekQueue.shift();
    if (next) setTimeout(() => showPeek(next), 120);
  }
  function openFromPeek(p: PeekData) {
    if (p.kind === "notification" && p.aumid) invoke("open_app", { aumid: p.aumid, id: p.id });
    if (p.kind === "update") invoke("install_update");
    if (p.kind === "download-done") invoke("open_download", { path: p.path });
    clearTimeout(peekTimer);
    endPeek();
  }
  // Buttons inside a peek (the meeting reminder's "Cek kamera" and "Join").
  function peekAction(p: PeekData, action: string) {
    if (action === "camera") openMirror(p.title, p.link ?? "");
    if (action === "join" && p.link) invoke("run_link", { url: p.link });
    clearTimeout(peekTimer);
    endPeek();
  }

  function setPinned(on: boolean) {
    pinned = on;
    invoke("set_pinned", { on });
    if (!on && !hovered && !typing) closeTimer = setTimeout(() => (open = false), 220);
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
    // The camera's in-use entry clears a moment after the stream stops.
    setTimeout(() => invoke("set_mirror", { on: false }), 3000);
  }
  function joinMeeting() {
    const link = mirror?.link;
    closeMirror();
    if (link) invoke("run_link", { url: link });
  }

  // Calendar: re-read every 15 minutes; remind 10 minutes before and at the start.
  let events = $state<CalEvent[]>([]);
  let calendarTimers: ReturnType<typeof setTimeout>[] = [];
  async function loadCalendar() {
    const ics = await invoke<string>("fetch_calendar").catch(() => "");
    events = ics ? upcoming(ics) : [];
    calendarTimers.forEach(clearTimeout);
    // Reminders 10 minutes before and at the start; for a video meeting also an
    // offer 2 minutes before to check the camera or join.
    calendarTimers = events
      .filter((e) => !e.allDay)
      .flatMap((e) =>
        (e.link ? [10, 2, 0] : [10, 0]).map((before) => {
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
      .filter((t) => t !== undefined);
  }
  const nextEvent = $derived(events.find((e) => !e.allDay && e.end > now.getTime()));
  const clockOf = (t: number) => new Date(t).toLocaleTimeString("id-ID", { hour: "2-digit", minute: "2-digit" }).replace(".", ":");

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

  const pomo = new Pomodoro(showPeek);
  const water = new Water(showPeek);
  const todo = new Todo(showPeek);
  const shelf = new Shelf();

  const charging = $derived(power.battery && power.charging);
  const timerActive = $derived(pomo.running || pomo.left < pomo.length());
  // The notch is showing its pages (not a held notification or screenshot peek).
  const paged = $derived(open && !clickable(peek));
  // What runs right now, for the closed notch (the lyric flag is filled in below).
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
  const view = $derived(
    mirror
      ? "mirror"
      : // Scrolling the volume on an open notch: show the volume, not the page.
        scrollingVolume && peek?.kind === "volume"
        ? "peek"
        : paged
        ? pages[page] === "now"
          ? media
            ? lyricsOn
              ? "lyrics"
              : "player"
            : "clock"
          : (pages[page] as Exclude<PageName, "now">)
        : peek
          ? peek.kind === "screenshot"
            ? "shot"
            : peek.kind === "notification" || peek.kind === "meeting" || peek.kind === "download-done"
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
  // Lyrics: fetched for the lyrics panel, and (if enabled) for a single line in the small notch.
  let lyrics = $state<Lyrics | null>(null);
  let lyricsStatus = $state<"loading" | "done" | "offline">("loading");
  let smallLyrics = $state(true);
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

  // The cover's main color, which tints the notch while music shows.
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
      // Average of the more colorful pixels, so a grey border doesn't wash it out.
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

  // The line being sung, for the small notch. Instead of ticking, wake up
  // exactly when the next line starts.
  let lineNow = $state(Date.now());
  const smallLine = $derived.by(() => {
    if (!smallLyrics || !media?.playing || !lyrics?.synced) return "";
    const i = currentLine(lyrics.synced, positionAt(media, lineNow));
    // Instrumental gaps ("♪") count as no line, so the clock shows instead.
    return i >= 0 && lyrics.synced[i].text !== "♪" ? lyrics.synced[i].text : "";
  });
  // The island with the sung line filled in: music with words leads (lib/island.ts).
  const island = $derived(
    arrange(
      running.map((a) => (a.kind === "music" ? { kind: "music", lyric: !!smallLine } : a)),
      turn,
    ),
  );
  const musicLeads = $derived(view === "island" && island.main?.kind === "music");
  const tinted = $derived(!!tint && (view === "player" || view === "lyrics" || musicLeads));
  // Keeps the sung line current whenever music is in the island, also while
  // another activity leads: during an instrumental gap the line is empty, the
  // timer (say) takes the lead, and the next line must still bring music back.
  $effect(() => {
    if (view !== "island" || !smallLyrics || !media?.playing || !lyrics?.synced) return;
    // A new song, a seek, a pause or coming back from another view re-runs this with
    // lineNow left behind, which would put the song near its start and leave the line
    // empty until the next wake-up. Catch up to the real time first (this re-runs it).
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

  // In the island the camera/mic dots sit at the end, so they get room of their own.
  const islandDots = $derived((inUse.camera.length > 0 ? 10 : 0) + (inUse.mic.length > 0 && !call ? 10 : 0));
  const size = $derived(
    view === "island" && island.main
      ? { ...SIZES.island, w: islandWidth(island.main, island.side) + islandDots }
      : SIZES[view],
  );
  const closedView = $derived(view === "rest" || view === "island" || view === "charging");
  // The status row sits above the open pages (not the clock, the camera or a peek).
  const withStatus = $derived(paged && !mirror && view !== "peek" && view !== "clock");

  // The audio meter runs only while the equalizer is on screen.
  const metering = $derived(view === "island" && !!media?.playing && island.main !== null && [island.main, ...island.side].some((a) => a.kind === "music"));
  $effect(() => {
    invoke("set_metering", { on: metering });
    if (!metering) levels = null;
  });

  // Tell Rust the target size right away, so the growing notch already counts as "inside".
  // Tucked, the notch is a thin line, so whatever is underneath (like browser tabs) stays clickable.
  $effect(() => {
    // The minimal line is 5 px tall; hovering it gets a 10 px target.
    invoke("set_notch_size", { w: size.w, h: view === "minimal" ? 10 : size.h });
  });

  // A clock only while something on screen moves: playback position or the
  // running timer (smooth when open, once a second in the small notch).
  let nowMs = $state(Date.now());
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
  function setTyping(on: boolean) {
    typing = on;
    invoke(on ? "begin_typing" : "end_typing");
    if (!on && !hovered) closeTimer = setTimeout(() => (open = false), 220);
  }

  let lastWheel = 0;
  let lastVolume = 0;
  function wheel(e: WheelEvent) {
    // Sideways (touchpad swipe): another activity leads the closed notch.
    if (Math.abs(e.deltaX) > Math.abs(e.deltaY)) {
      if (Date.now() - lastWheel < 450 || Math.abs(e.deltaX) < 8) return;
      lastWheel = Date.now();
      turn += e.deltaX > 0 ? 1 : -1;
      return;
    }
    // The band where the closed notch sits: volume, up = louder.
    if (e.offsetY >= 0 && (e.target as HTMLElement).closest(".notch") && e.clientY < 30 && !typing) {
      if (Date.now() - lastVolume < 40 || Math.abs(e.deltaY) < 1) return;
      lastVolume = Date.now();
      invoke("volume_step", { up: e.deltaY < 0 });
      scrollingVolume = true;
      clearTimeout(scrollVolumeTimer);
      scrollVolumeTimer = setTimeout(() => (scrollingVolume = false), 1200);
      return;
    }
    // Lists scroll themselves; elsewhere the wheel flips pages.
    if (!open || typing || (e.target as HTMLElement).closest(".scroll, .lyrics")) return;
    if (Date.now() - lastWheel < 350 || Math.abs(e.deltaY) < 4) return;
    lastWheel = Date.now();
    const step = e.deltaY > 0 ? 1 : -1;
    goPage(page + step, step);
  }

  // The page's name shows briefly after flipping, and while a dot is hovered.
  let hoveredPage = $state<PageName | null>(null);
  let showPageLabel = $state(false);
  let pageLabelTimer: ReturnType<typeof setTimeout> | undefined;
  let pageDirection = $state<1 | -1>(1);

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

  // Long press (still, on a non-button spot) pins the notch open, or lets it go.
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

  onMount(() => {
    const unlisten = listen<boolean>("notch-hover", (e) => {
      hovered = e.payload;
      clearTimeout(closeTimer);
      clearTimeout(openTimer);
      if (e.payload) {
        if (!open) {
          openTimer = setTimeout(() => (open = true), 90);
        }
      } else {
        if (!typing && !pinned) {
          closeTimer = setTimeout(() => (open = false), 220);
        }
      }
      // Hold a clickable peek while the cursor is on it; let it go shortly after.
      if (clickable(peek)) {
        clearTimeout(peekTimer);
        if (!e.payload) peekTimer = setTimeout(endPeek, 1500);
      }
    });

    // A click outside a pinned notch lets it close.
    const unlistenOutside = listen("notch-outside-click", () => {
      if (pinned && !mirror) setPinned(false);
    });

    // Files dragged onto the notch go to the shelf, or to Windows' Share window
    // when dropped on "Bagikan".
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

    // Activities and pages fed from Rust.
    const unlistenCall = listen<Call | null>("call", (e) => (call = e.payload));
    invoke<Call | null>("call_state").then((c) => (call = c));
    const unlistenDownload = listen<Download | null>("download", (e) => (download = e.payload));
    const unlistenNetSpeed = listen<NetworkSpeed>("network-speed", (e) => (networkSpeed = e.payload));
    const unlistenLevel = listen<number[]>("audio-level", (e) => (levels = metering ? e.payload : null));
    const unlistenClip = listen<ClipEntry[]>("clipboard", (e) => (clipItems = e.payload));
    invoke<ClipEntry[]>("clipboard_list").then((c) => (clipItems = c));

    water.load();
    todo.load();
    shelf.load();
    // Cursor moves keep the clock awake; after 2 quiet minutes the eyes come out.
    let sleepTimer: ReturnType<typeof setTimeout>;
    let greetTimer: ReturnType<typeof setTimeout>;
    const wake = () => {
      if (dozing) {
        greeting = true;
        clearTimeout(greetTimer);
        greetTimer = setTimeout(() => (greeting = false), 1500);
      }
      dozing = false;
      clearTimeout(sleepTimer);
      sleepTimer = setTimeout(() => (dozing = mascot), 120_000);
    };
    wake();
    const unlistenLook = listen<[number, number, number]>("eyes-look", (e) => {
      look = e.payload;
      wake();
    });
    const unlistenTuck = listen<boolean>("notch-tucked", (e) => (tucked = e.payload));
    invoke<boolean>("is_tucked").then((t) => (tucked = t));

    const setMedia = (m: Omit<NowPlaying, "receivedAt"> | null) => {
      media = m && { ...m, receivedAt: Date.now() };
      lineNow = Date.now();
      if (!m) lyricsOn = false;
    };
    const unlistenMedia = listen<Omit<NowPlaying, "receivedAt"> | null>("media", (e) => setMedia(e.payload));
    invoke<Omit<NowPlaying, "receivedAt"> | null>("media_now").then(setMedia);

    const unlistenPeek = listen<PeekData>("peek", (e) => {
      showPeek(e.payload);
      // New screenshots also land on the shelf.
      if (e.payload.kind === "screenshot") shelf.add([e.payload.path]);
    });

    const unlistenPower = listen<Power>("power", (e) => (power = e.payload));
    invoke<Power>("power_state").then((p) => (power = p));
    const unlistenInUse = listen<InUse>("in-use", (e) => (inUse = e.payload));
    invoke<InUse>("in_use_state").then((u) => (inUse = u));
    const unlistenMic = listen<boolean>("mic-muted", (e) => (micMuted = e.payload));
    invoke<boolean>("mic_state").then((m) => (micMuted = m));

    // Settings, and weather every 30 minutes for the chosen city or else the Windows location.
    type AllSettings = Settings & {
      weatherCity: { latitude: number; longitude: number } | null;
      smallLyrics: boolean;
      shortcuts: Shortcut[];
      mascot: boolean;
      minimalNotch: boolean;
      musicEnabled?: boolean;
      pomodoroEnabled?: boolean;
      todoEnabled?: boolean;
      shelfEnabled?: boolean;
      clipboardEnabled?: boolean;
      shortcutsEnabled?: boolean;
      netSpeedIsland?: boolean;
      netSpeedStatus?: boolean;
      netSpeedThresholdDownKb?: number;
      netSpeedThresholdUpKb?: number;
    };
    let weatherTimer: ReturnType<typeof setTimeout>;
    const loadWeather = async (settings?: AllSettings) => {
      clearTimeout(weatherTimer);
      settings ??= await invoke<AllSettings>("get_settings");
      const city = settings.weatherCity;
      const at = city ? [city.latitude, city.longitude] : await invoke<[number, number] | null>("locate");
      weather = at ? await fetchWeather(at[0], at[1]) : null;
      weatherTimer = setTimeout(loadWeather, 30 * 60_000);
    };
    const applySettings = (s: AllSettings) => {
      smallLyrics = s.smallLyrics && s.musicEnabled !== false;
      mascot = s.mascot !== false;
      if (!mascot) dozing = false;
      minimalNotch = s.minimalNotch === true;
      shortcuts = s.shortcuts ?? [];
      // Keep the same page in view when pages appear or go.
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
      const newIdx = pages.indexOf(current);
      page = newIdx !== -1 ? newIdx : Math.min(page, Math.max(0, pages.length - 1));
      pomo.configure(s);
      water.configure(s);
      loadWeather(s);
      loadCalendar();
    };
    const calendarPoll = setInterval(loadCalendar, 15 * 60_000);
    invoke<AllSettings>("get_settings").then(applySettings);
    const unlistenSettings = listen<AllSettings>("settings", (e) => applySettings(e.payload));

    // Tick once per minute, aligned to the minute, so idle stays idle.
    let tick: ReturnType<typeof setTimeout>;
    const schedule = () => {
      now = new Date();
      tick = setTimeout(schedule, 60_000 - (Date.now() % 60_000));
    };
    schedule();

    // Blink every few seconds. A one-shot class instead of an infinite CSS
    // animation, so nothing renders between blinks.
    let blinkTimer: ReturnType<typeof setTimeout>;
    const scheduleBlink = () => {
      blinkTimer = setTimeout(() => {
        blink = true;
        setTimeout(() => (blink = false), 180);
        scheduleBlink();
      }, 3500 + Math.random() * 3000);
    };
    scheduleBlink();

    return () => {
      unlisten.then((f) => f());
      unlistenLook.then((f) => f());
      unlistenTuck.then((f) => f());
      unlistenMedia.then((f) => f());
      for (const u of [
        unlistenPeek,
        unlistenPower,
        unlistenInUse,
        unlistenMic,
        unlistenSettings,
        unlistenDrop,
        unlistenOutside,
        unlistenCall,
        unlistenDownload,
        unlistenNetSpeed,
        unlistenLevel,
        unlistenClip,
      ])
        u.then((f) => f());
      clearTimeout(pressTimer);
      clearTimeout(peekTimer);
      clearTimeout(weatherTimer);
      clearInterval(calendarPoll);
      calendarTimers.forEach(clearTimeout);
      clearTimeout(closeTimer);
      clearTimeout(pageLabelTimer);
      clearTimeout(tick);
      clearTimeout(blinkTimer);
      clearTimeout(sleepTimer);
    };
  });

  const time = $derived(now.toLocaleTimeString("id-ID", { hour: "2-digit", minute: "2-digit" }).replace(".", ":"));
  const date = $derived(now.toLocaleDateString("id-ID", { weekday: "long", day: "numeric", month: "long" }));
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
  style:--tint={tint}
  style:--w="{size.w}px"
  style:--h="{size.h}px"
  style:--r="{size.r}px"
  style:--e="{size.e}px"
  style:--battery="{power.percent}%"
  onwheel={wheel}
  onpointerdown={pressDown}
  onpointermove={pressMove}
  onpointerup={pressUp}
  onpointerleave={pressUp}
>
  {#if pinned && paged && !mirror}<i class="pin" title="Terkunci terbuka · tekan lama untuk melepas"></i>{/if}
  <div class="ambient"></div>

  <!-- Battery, date and time above every open page (the clock page shows them big already). -->
  {#if withStatus}
    <div class="status" in:fade={{ duration: 200, delay: 200 }}>
      {#if power.battery}
        <svg class="mini-ring" class:low={!power.charging && power.percent <= 20} class:charging viewBox="0 0 20 20">
          <circle cx="10" cy="10" r="7.5" />
          <circle cx="10" cy="10" r="7.5" class="fill" pathLength="100" stroke-dasharray="{power.percent} 100" />
        </svg>
        <span>{power.percent}%{#if power.charging}&nbsp;⚡{/if}</span><span class="sep">·</span>
      {/if}
      {#if netSpeedStatus && networkSpeed}
        <span class="net-status">
          <span class="down">↓ {formatSpeed(networkSpeed.down)}</span>
          <span class="up">↑ {formatSpeed(networkSpeed.up)}</span>
        </span>
        <span class="sep">·</span>
      {/if}
      <span>{date}</span><b>{time}</b>
    </div>
  {/if}

  <div class="layer rest" class:on={view === "rest"}>
    {#if weather}<span class="weather">{weather.icon}<b>{weather.temp}°</b></span>{/if}
    {#if greeting}
      <div class="eyes happy" in:fade={{ duration: 150 }}><i></i><i></i></div>
    {:else if dozing}
      <div class="eyes" class:blink style:--lx={look[0]} style:--ly={look[1]} style:--near={look[2]} in:fade={{ duration: 400 }}>
        <i></i><i></i>
      </div>
    {:else}
      <span class="rest-time" in:fade={{ duration: 400 }}>{time}</span>
    {/if}
  </div>

  <!-- Things running (music, timer, a call, a download), side by side. -->
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

  <!-- While charging: a battery ring with a bolt, and the time. -->
  <div class="layer charge" class:on={view === "charging"}>
    <svg class="ring" viewBox="0 0 20 20">
      <circle cx="10" cy="10" r="8" />
      <circle cx="10" cy="10" r="8" class="fill" pathLength="100" stroke-dasharray="{power.percent} 100" />
      <path d="M10.6 5.5 7.5 10.6h2.4l-.5 3.9 3.1-5.1h-2.4z" />
    </svg>
    <span class="charge-time">{time}</span>
  </div>

  <div class="layer clock" class:on={view === "clock"}>
    <div class="time">{time}</div>
    <div class="date">{date}{#if weather} · {weather.icon} {weather.temp}°{/if}</div>
    {#if nextEvent}
      <div class="event" title={nextEvent.location}>
        <span class="event-time">{clockOf(nextEvent.start)}</span>{nextEvent.title}
      </div>
    {/if}
  </div>

  {#if peek && (view === "peek" || view === "shot" || view === "notify")}
    <div class="layer on" in:fade={{ duration: 220, delay: 120 }} out:fade={{ duration: 100 }}>
      <Peek {peek} onOpen={() => peek && openFromPeek(peek)} onAction={(a) => peek && peekAction(peek, a)} />
    </div>
  {/if}

  {#if mirror}
    <div class="layer on" in:fade={{ duration: 260, delay: 160 }} out:fade={{ duration: 100 }}>
      <Mirror title={mirror.title} link={mirror.link} onDone={closeMirror} onJoin={joinMeeting} />
    </div>
  {/if}

  <!-- Camera/mic in use, and a muted mic: always visible while the notch shows anything. -->
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
          <path d="M9 9v3a3 3 0 0 0 5.1 2.1M15 9.3V6a3 3 0 0 0-5.7-1.3M19 11a7 7 0 0 1-1.2 3.9M5 11a7 7 0 0 0 10.6 6M12 18v3M3 3l18 18" />
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
        aria-label={step > 0 ? "Halaman berikutnya" : "Halaman sebelumnya"}
        in:fade={{ duration: 200, delay: 200 }}
      >
        <svg viewBox="0 0 24 24"><path d={step > 0 ? "M9 6l6 6-6 6" : "M15 6l-6 6 6 6"} /></svg>
      </button>
    {/each}
    {#if hoveredPage || showPageLabel}
      <div class="page-tooltip" transition:fade={{ duration: 150 }}>
        {PAGE_LABELS[hoveredPage ?? pages[page]]}
      </div>
    {/if}
    <div class="pager" role="tablist" tabindex="-1" in:fade={{ duration: 200, delay: 200 }} onpointerleave={() => (hoveredPage = null)}>
      {#each pages as name, i}
        <button
          role="tab"
          aria-selected={i === page}
          aria-label={PAGE_LABELS[name]}
          class:on={i === page}
          onpointerenter={() => (hoveredPage = name)}
          onclick={() => goPage(i, i >= page ? 1 : -1)}
        ></button>
      {/each}
    </div>
  {/if}
</div>

<style>
  :global(html, body) {
    margin: 0;
    background: transparent;
    overflow: hidden;
    user-select: none;
    cursor: default;
    font-family: "Segoe UI Variable Text", "Segoe UI", sans-serif;
    color: #fff;
  }

  .notch {
    --spring: cubic-bezier(0.34, 1.32, 0.5, 1);
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
  }
  .notch.open {
    box-shadow:
      0 6px 18px rgba(0, 0, 0, 0.2),
      0 16px 42px rgba(0, 0, 0, 0.4);
  }

  /* "Ears": concave fillets that blend the notch seamlessly into the screen's top edge.
     1px overlap prevents subpixel rasterization seams on high-DPI (125%, 150%) screens. */
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

  /* A faint light rising from the bottom edge, for depth. */
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
  /* Music: the cover's color rises from the bottom. */
  .tinted .ambient {
    height: 100%;
    background: linear-gradient(to top, rgb(var(--tint) / 0.6), rgb(var(--tint) / 0.12) 55%, transparent);
  }
  .tinted:not(.open) .ambient {
    background: linear-gradient(to top, rgb(var(--tint) / 0.4), transparent);
  }

  .notch {
    transition:
      width 0.62s var(--spring),
      height 0.62s var(--spring),
      border-radius 0.45s ease,
      box-shadow 0.45s ease;
  }

  /* Tucked (a window under the notch): a thin line.
     While charging it turns green and fills up to the battery level. */
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

  /* One layer per view; only the current one is visible. */
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

  .rest-time {
    font-size: 14px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }
  .weather {
    position: absolute;
    left: 12px;
    display: flex;
    align-items: center;
    gap: 3px;
    font-size: 10px;
  }
  .weather b {
    font-size: 11px;
    font-weight: 600;
  }

  .charge {
    flex-direction: row;
    gap: 34px;
  }
  .ring {
    width: 20px;
    height: 20px;
    transform: rotate(-90deg);
  }
  .ring circle {
    fill: none;
    stroke: rgba(52, 211, 153, 0.25);
    stroke-width: 2.4;
  }
  .ring .fill {
    stroke: #34d399;
    stroke-linecap: round;
  }
  .ring path {
    fill: #34d399;
    transform: rotate(90deg);
    transform-origin: center;
  }
  .charge-time {
    font-size: 15px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }

  /* Previous / next page, on the notch's sides. */
  .arrow {
    position: absolute;
    top: 50%;
    left: 8px;
    width: 22px;
    height: 22px;
    margin-top: -14px;
    padding: 0;
    border: 0;
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: rgba(255, 255, 255, 0.08);
    cursor: pointer;
    opacity: 0.6;
    transition:
      opacity 0.2s,
      background 0.2s;
  }
  .arrow.next {
    left: auto;
    right: 8px;
  }
  .arrow:hover {
    opacity: 1;
    background: rgba(255, 255, 255, 0.18);
  }
  .arrow svg {
    width: 14px;
    height: 14px;
    fill: none;
    stroke: #fff;
    stroke-width: 2.4;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  /* Page dots at the bottom of the open notch, the current one stretched. */
  .pager {
    position: absolute;
    bottom: 7px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    gap: 5px;
  }
  .pager button {
    width: 5px;
    height: 5px;
    padding: 0;
    border: 0;
    border-radius: 3px;
    background: rgba(255, 255, 255, 0.3);
    cursor: pointer;
    transition:
      width 0.3s var(--spring),
      background 0.3s;
  }
  .pager button.on {
    width: 15px;
    background: #fff;
  }

  .page-tooltip {
    position: absolute;
    bottom: 22px;
    left: 50%;
    transform: translateX(-50%);
    font-size: 10.5px;
    font-weight: 500;
    color: rgba(255, 255, 255, 0.95);
    background: rgba(18, 18, 22, 0.92);
    border: 1px solid rgba(255, 255, 255, 0.14);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.6);
    padding: 2px 10px;
    border-radius: 999px;
    white-space: nowrap;
    pointer-events: none;
    z-index: 10;
  }

  /* Camera / mic indicators on the closed notch. */
  .dots {
    position: absolute;
    top: 0;
    right: 12px;
    height: 30px;
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .dots.wide {
    right: 38px;
  }
  .cam,
  .mic {
    width: 7px;
    height: 7px;
    border-radius: 50%;
  }
  .cam {
    background: #34d399;
    box-shadow: 0 0 6px #34d399;
  }
  .mic {
    background: #fb923c;
    box-shadow: 0 0 6px #fb923c;
  }
  .dots .muted {
    width: 13px;
    height: 13px;
  }
  .muted,
  .mic-toggle svg {
    fill: none;
    stroke: #f87171;
    stroke-width: 2.2;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .mic-toggle {
    position: absolute;
    top: 10px;
    left: 18px;
    width: 26px;
    height: 26px;
    padding: 0;
    border: 0;
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: rgba(255, 255, 255, 0.1);
    cursor: pointer;
  }
  .mic-toggle:hover {
    background: rgba(255, 255, 255, 0.2);
  }
  .mic-toggle svg {
    width: 14px;
    height: 14px;
    stroke: #fff;
  }
  .mic-toggle.live svg {
    stroke: #fb923c;
  }
  .mic-toggle.muted {
    background: rgba(248, 113, 113, 0.2);
  }
  .mic-toggle.muted svg {
    stroke: #f87171;
  }
  .mic-toggle.output {
    left: 50px;
  }
  .mic-toggle.camera {
    left: auto;
    right: 18px;
  }

  .event {
    max-width: 300px;
    margin-top: 6px;
    padding: 3px 10px;
    border-radius: 999px;
    background: rgba(251, 146, 60, 0.14);
    font-size: 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .event-time {
    margin-right: 6px;
    color: #fdba74;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }

  .mic-toggle .cam {
    position: absolute;
    top: 0;
    right: -2px;
    width: 6px;
    height: 6px;
  }

  /* Open pages sit under the 16 px status row. */
  .layer.below {
    top: 16px;
  }
  .open .arrow {
    margin-top: -6px;
  }
  .status {
    position: absolute;
    top: 6px;
    left: 0;
    right: 0;
    z-index: 1;
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 6px;
    font-size: 10.5px;
    color: rgba(255, 255, 255, 0.65);
    font-variant-numeric: tabular-nums;
    pointer-events: none;
  }
  .status b {
    color: #fff;
    font-weight: 600;
  }
  .status .sep {
    opacity: 0.6;
  }
  .net-status {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-weight: 500;
  }
  .net-status .down {
    color: #38bdf8;
  }
  .net-status .up {
    color: #34d399;
  }
  .mini-ring {
    width: 12px;
    height: 12px;
    transform: rotate(-90deg);
  }
  .mini-ring circle {
    fill: none;
    stroke: rgba(255, 255, 255, 0.25);
    stroke-width: 3;
  }
  .mini-ring .fill {
    stroke: #fff;
  }
  .mini-ring.charging .fill {
    stroke: #34d399;
  }
  .mini-ring.low .fill {
    stroke: #fb923c;
  }

  /* "Notch minimal": a plain black line, no glow. */
  .notch.minimal {
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.22);
  }
  .minimal .ambient {
    opacity: 0;
  }

  /* Back after a quiet while: smiling eyes and one little bounce. */
  .eyes.happy i {
    width: 12px;
    height: 7px;
    border: 2.4px solid #fff;
    border-bottom: 0;
    border-radius: 12px 12px 0 0;
    background: none;
  }
  .notch.greet {
    animation: greet 0.6s var(--spring);
  }
  @keyframes greet {
    40% {
      transform: translateX(-50%) scale(1.08);
    }
  }

  /* Pinned open (long press): a small mark in the corner. */
  .pin {
    position: absolute;
    top: 9px;
    right: 14px;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #38bdf8;
    box-shadow: 0 0 6px #38bdf8;
    z-index: 2;
  }

  .eyes {
    display: flex;
    gap: 15px;
    /* Glance toward the cursor (vertical travel is smaller because the notch is short),
       and open up a little, curious, as the cursor comes close. */
    transform: translate(calc(var(--lx, 0) * 9px), calc(var(--ly, 0) * 4px)) scale(calc(1 + var(--near, 0) * 0.25));
    transition: transform 0.22s var(--spring);
  }
  .eyes i {
    width: 8.4px;
    height: 12.4px;
    border-radius: 50%;
    background: #fff;
    box-shadow:
      0 0 6px 1px rgba(255, 255, 255, 0.45),
      0 0 14px 4px rgba(255, 255, 255, 0.12);
    transition: transform 0.09s ease;
  }
  .eyes.blink i {
    transform: scaleY(0.1);
  }

  .clock {
    transform: translateY(-6px) scale(0.96);
  }
  .clock.on {
    transform: none;
    transition:
      opacity 0.32s 0.2s,
      transform 0.5s 0.15s var(--spring);
  }

  .time {
    font-family: "Segoe UI Variable Display", "Segoe UI", sans-serif;
    font-size: 44px;
    font-weight: 700;
    letter-spacing: -0.03em;
    font-variant-numeric: tabular-nums;
  }
  .date {
    margin-top: 2px;
    font-size: 13px;
    color: rgba(255, 255, 255, 0.6);
  }
</style>
