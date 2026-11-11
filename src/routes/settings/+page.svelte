<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import { fade, slide } from "svelte/transition";
  import { searchCity, type Place } from "$lib/weather";
  import { calculatePrayerTimes, type PrayerMethod } from "$lib/prayer";
  import { t, setLocale, getLocale, subscribeLocale } from "$lib/i18n";

  type City = { name: string; latitude: number; longitude: number };
  type Shortcut = { name: string; action: string; target: string };
  type Settings = {
    language?: string;
    weatherCity: City | null;
    focusMinutes: number;
    breakMinutes: number;
    glassMl: number;
    goalMl: number;
    waterFrom: number;
    waterTo: number;
    remindEvery: number;
    waterEnabled: boolean;
    waterReminders: boolean;
    mascot: boolean;
    mascotMode?: string;
    mascotTheme?: string;
    musicMascot?: boolean;
    minimalNotch: boolean;
    floatingNotch?: boolean;
    statusRing?: string;
    mirrorNotifications: boolean;
    calendarUrl: string;
    smallLyrics: boolean;
    shortcuts: Shortcut[];
    capsLockPeek: boolean;
    hideVolumeOsd?: boolean;
    musicEnabled: boolean;
    todayEnabled?: boolean;
    pomodoroEnabled: boolean;
    todoEnabled: boolean;
    shelfEnabled: boolean;
    clipboardEnabled: boolean;
    shortcutsEnabled: boolean;
    netSpeedIsland: boolean;
    netSpeedStatus: boolean;
    netSpeedThresholdDownKb: number;
    netSpeedThresholdUpKb: number;
    bluetoothPeek: boolean;
    bluetoothLowThreshold: number;
    monitorPlacement: string;
    springPhysics: string;
    windowsAccentGlow: boolean;
    prayerEnabled?: boolean;
    prayerPreAlert?: boolean;
    prayerMethod?: string;
    prayerFajr?: boolean;
    prayerDhuhr?: boolean;
    prayerAsr?: boolean;
    prayerMaghrib?: boolean;
    prayerIsha?: boolean;
    prayerSound?: boolean;
    soundEnabled?: boolean;
    snippetsEnabled?: boolean;
    prompterEnabled?: boolean;
    externalPeeks?: boolean;
    agentStatus?: boolean;
    openHotkey?: string;
    ocrHotkey?: string;
    swipeTracks?: boolean;
    rainAlert?: boolean;
    chargeReminder?: number;
    presentationSummary?: boolean;
    eyeBreak?: boolean;
    connectionPeek?: boolean;
    privacyIndicators?: boolean;
    countdowns?: Countdown[];

  };
  type Countdown = { name: string; date: string; repeat: string };
  type Key = keyof Settings;

  let currentLang = $state(getLocale());
  $effect(() => subscribeLocale((l) => (currentLang = l)));
  const tr = (key: string, params?: Record<string, string | number>) => {
    void currentLang;
    return t(key, params);
  };

  // Sidebar sections. `words` feed the search box in both languages.
  type SectionId =
    | "general"
    | "appearance"
    | "eyes"
    | "pages"
    | "music"
    | "focus"
    | "water"
    | "calendar"
    | "shortcuts"
    | "notifications"
    | "system"
    | "weather"
    | "prayer";
  const SECTIONS: { id: SectionId; color: string; icon: string; words: string }[] = [
    { id: "general", color: "#8e8e93", icon: "M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6zM19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1.1 1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z", words: "language bahasa monitor layar screen cursor kursor animation animasi spring" },
    { id: "appearance", color: "#5e5ce6", icon: "M3 7a4 4 0 0 1 4-4h10a4 4 0 0 1 4 4v10a4 4 0 0 1-4 4H7a4 4 0 0 1-4-4zM8 3v4a2 2 0 0 0 2 2h4a2 2 0 0 0 2-2V3", words: "floating melayang minimal ring cincin battery baterai accent aksen glow warna color cpu ram memory memori" },
    { id: "eyes", color: "#1c1c1e", icon: "M2 12s3.6-7 10-7 10 7 10 7-3.6 7-10 7S2 12 2 12zM12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z", words: "mascot maskot eyes mata cursor kursor idle style gaya theme tema anime pixel cat kucing cartoon kartun" },
    { id: "pages", color: "#0a84ff", icon: "M4 5h16M4 12h16M4 19h10", words: "pages halaman today hari ini pomodoro water air todo shelf clipboard shortcuts sound suara mixer volume snippet teleprompter" },
    { id: "music", color: "#ff375f", icon: "M9 18V5l12-2v13M9 18a3 3 0 1 1-6 0 3 3 0 0 1 6 0zM21 16a3 3 0 1 1-6 0 3 3 0 0 1 6 0z", words: "music musik lyrics lirik lrclib song lagu swipe geser gesture gestur" },
    { id: "focus", color: "#ff9f0a", icon: "M12 22a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM12 9v4l2.5 2.5M9 2h6", words: "pomodoro focus fokus break istirahat timer minutes menit eye mata 20-20-20 health kesehatan" },
    { id: "water", color: "#64d2ff", icon: "M12 2.7S5 10 5 14.5a7 7 0 0 0 14 0C19 10 12 2.7 12 2.7z", words: "water air minum glass gelas target reminder pengingat ml" },
    { id: "calendar", color: "#ff453a", icon: "M4 6a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v13a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2zM4 10h16M8 2v4M16 2v4", words: "calendar kalender ics google outlook meeting rapat countdown hitung mundur gajian payday" },
    { id: "shortcuts", color: "#ffd60a", icon: "M13 2 4 14h7l-1 8 9-12h-7z", words: "shortcuts pintasan launcher app link keep awake jaga menyala keyboard clean bersihkan ocr text teks dark mode gelap" },
    { id: "notifications", color: "#ff453a", icon: "M18 8a6 6 0 0 0-12 0c0 7-3 9-3 9h18s-3-2-3-9M13.7 21a2 2 0 0 1-3.4 0", words: "notifications notifikasi do not disturb dnd whatsapp teams presentation presentasi camera kamera mic mikrofon privacy privasi connection koneksi wifi charger baterai battery" },
    { id: "system", color: "#636366", icon: "M3 6a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2zM7 9h.01M11 9h.01M15 9h.01M7 13h10", words: "keyboard caps lock volume overlay osd bluetooth network jaringan speed kecepatan download upload hotkey tombol pintas script peek claude code agent ai codex aider" },
    { id: "weather", color: "#30b0c7", icon: "M17.5 19a4.5 4.5 0 1 0-1.4-8.8A6 6 0 1 0 6 16.4M6 19h11.5", words: "weather cuaca city kota location lokasi rain hujan" },
    { id: "prayer", color: "#34d399", icon: "M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z", words: "azan adhan sholat prayer subuh dzuhur ashar maghrib isya muslim islam jadwal prealert" },

  ];

  let section = $state<SectionId>("general");
  let searchQuery = $state("");
  let settings = $state<Settings | null>(null);
  let saved = $state(false);
  let savedTimer: ReturnType<typeof setTimeout> | undefined;

  let cityQuery = $state("");
  let cityResults = $state<Place[]>([]);
  let searchingCity = $state(false);
  let cityNotFound = $state(false);

    let userCoords = $state<[number, number] | null>(null);
  $effect(() => {
    if (settings?.weatherCity) {
      userCoords = [settings.weatherCity.latitude, settings.weatherCity.longitude];
    } else {
      invoke<[number, number] | null>("locate").then((loc) => {
        if (loc) userCoords = loc;
      });
    }
  });

  const previewPrayerTimes = $derived.by(() => {
    if (!userCoords || !settings?.prayerEnabled) return null;
    try {
      return calculatePrayerTimes(
        userCoords[0],
        userCoords[1],
        new Date(),
        (settings.prayerMethod as PrayerMethod) || "kemenag",
      );
    } catch {
      return null;
    }
  });

  const query = $derived(searchQuery.trim().toLowerCase());
  const visible = $derived(
    query ? SECTIONS.filter((s) => `${s.words} ${tr(`nav.${s.id}`)}`.toLowerCase().includes(query)).map((s) => s.id) : [section],
  );
  const shows = (id: SectionId) => visible.includes(id);

  const ACTION_ITEMS: [string, string][] = [
    ["lock", "actions.lock"],
    ["sleep", "actions.sleep"],
    ["screenshot", "actions.screenshot"],
    ["downloads", "actions.downloads"],
    ["camera", "actions.camera"],
    ["keepawake", "actions.keepawake"],
    ["keyboardclean", "actions.keyboardclean"],
    ["ocr", "actions.ocr"],
    ["darkmode", "actions.darkmode"],
    ["nightlight", "actions.nightlight"],
    ["batterysaver", "actions.batterysaver"],
    ["focus", "actions.focus"],
    ["open", "actions.open"],
  ];

  type PageKey =
    | "musicEnabled"
    | "todayEnabled"
    | "pomodoroEnabled"
    | "waterEnabled"
    | "todoEnabled"
    | "shelfEnabled"
    | "clipboardEnabled"
    | "shortcutsEnabled"
    | "soundEnabled"
    | "snippetsEnabled"
    | "prompterEnabled";
  const PAGE_ITEMS: [PageKey, string][] = [
    ["musicEnabled", "pageItems.music"],
    ["soundEnabled", "pageItems.sound"],
    ["todayEnabled", "pageItems.today"],
    ["pomodoroEnabled", "pageItems.pomodoro"],
    ["waterEnabled", "pageItems.water"],
    ["todoEnabled", "pageItems.todo"],
    ["shelfEnabled", "pageItems.shelf"],
    ["clipboardEnabled", "pageItems.clipboard"],
    ["shortcutsEnabled", "pageItems.shortcuts"],
    ["snippetsEnabled", "pageItems.snippets"],
    ["prompterEnabled", "pageItems.prompter"],
  ];
  const enabledPages = $derived(settings ? PAGE_ITEMS.filter(([k]) => settings![k] !== false).length : PAGE_ITEMS.length);

  let exe = $state("");
  // Opened from a peek straight to one page, e.g. ?section=prayer.
  const pick = (id: string | null) => {
    if (SECTIONS.some((s) => s.id === id)) section = id as SectionId;
  };
  onMount(() => {
    pick(new URLSearchParams(location.search).get("section"));
    const off = listen<string>("settings-section", (e) => pick(e.payload));
    return () => off.then((f) => f());
  });

  onMount(async () => {
    settings = await invoke<Settings>("get_settings");
    if (settings?.language) setLocale(settings.language);
    exe = (await invoke<string>("exe_path").catch(() => "")).replaceAll("\\", "/");
  });

  type AiTool = "claude" | "codex" | "aider" | "script";
  let selectedAi = $state<AiTool>("claude");
  const AI_TOOLS: { id: AiTool; label: string }[] = [
    { id: "claude", label: "Claude Code" },
    { id: "codex", label: "OpenAI Codex" },
    { id: "aider", label: "Aider" },
    { id: "script", label: "Script & URL" },
  ];

  const aiSnippet = $derived.by(() => {
    const exePath = exe || "notchy.exe";
    switch (selectedAi) {
      case "claude": {
        const cmd = (state: string) => [{ type: "command", command: `"${exePath}" --agent ${state} --app "Claude Code"` }];
        return JSON.stringify(
          {
            hooks: {
              UserPromptSubmit: [{ hooks: cmd("working") }],
              PostToolUse: [{ matcher: "*", hooks: cmd("working") }],
              Notification: [{ hooks: cmd("waiting") }],
              Stop: [{ hooks: cmd("done") }],
            },
          },
          null,
          2,
        );
      }
      case "codex":
        return `# ~/.codex/config.toml (berjalan saat giliran selesai)
notify = ["${exePath}", "--agent", "done", "--app", "Codex"]`;
      case "aider":
        return `# .aider.conf.yml (berjalan saat Aider menunggu input)
notifications-command: "${exePath} --agent waiting --app Aider --text \"Aider butuh respon\""`;
      case "script":
        return `# CLI: jalankan dari skrip, alias, atau terminal (PowerShell / Bash / CMD)
"${exePath}" --agent working --app "MyAgent"
"${exePath}" --agent waiting --app "MyAgent" --text "Menunggu persetujuan"
"${exePath}" --agent done --app "MyAgent"

# Skema URL notchy://agent (Bebas lokasi exe! Argumen pertama "" wajib untuk CMD):
start "" "notchy://agent?app=MyAgent&state=working"
start "" "notchy://agent?app=MyAgent&state=waiting&text=Menunggu%20persetujuan"
start "" "notchy://agent?app=MyAgent&state=done"`;
    }
  });

  let hookCopied = $state(false);
  async function copyHooks() {
    await navigator.clipboard.writeText(aiSnippet).catch(() => {});
    hookCopied = true;
    setTimeout(() => (hookCopied = false), 1500);
  }

  function editCountdown(i: number, patch: Partial<Countdown>) {
    if (!settings) return;
    save({ countdowns: (settings.countdowns ?? []).map((c, j) => (j === i ? { ...c, ...patch } : c)) });
  }
  const todayIso = () => {
    const d = new Date();
    return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
  };

  async function save(patch: Partial<Settings> = {}) {
    if (!settings) return;
    if (patch.language) setLocale(patch.language);
    settings = { ...settings, ...patch };
    await invoke("save_settings", { settings });
    saved = true;
    clearTimeout(savedTimer);
    savedTimer = setTimeout(() => (saved = false), 1600);
  }

  const clamp = (v: number, min: number, max: number) => Math.max(min, Math.min(max, v));

  function editShortcut(i: number, patch: Partial<Shortcut>) {
    if (!settings) return;
    save({ shortcuts: settings.shortcuts.map((s, j) => (j === i ? { ...s, ...patch } : s)) });
  }

  function defaultShortcuts(): Shortcut[] {
    return [
      { name: tr("shortcuts.lock"), action: "lock", target: "" },
      { name: tr("shortcuts.sleep"), action: "sleep", target: "" },
      { name: tr("shortcuts.screenshot"), action: "screenshot", target: "" },
      { name: tr("shortcuts.downloads"), action: "downloads", target: "" },
      { name: tr("shortcuts.camera"), action: "camera", target: "" },
    ];
  }

  async function searchCityHandler(e: SubmitEvent) {
    e.preventDefault();
    if (!cityQuery.trim()) return;
    searchingCity = true;
    cityResults = await searchCity(cityQuery.trim()).catch(() => []);
    cityNotFound = cityResults.length === 0;
    searchingCity = false;
  }
</script>

{#snippet toggle(key: Key, on: boolean, disabled = false)}
  <label class="switch">
    <input type="checkbox" checked={on} {disabled} onchange={(e) => save({ [key]: e.currentTarget.checked })} />
    <span></span>
  </label>
{/snippet}

{#snippet stepper(key: Key, value: number, min: number, max: number, step: number, unit: string)}
  <div class="stepper">
    <button disabled={value <= min} onclick={() => save({ [key]: clamp(value - step, min, max) })} aria-label="−">−</button>
    <span>{value}{unit ? ` ${unit}` : ""}</span>
    <button disabled={value >= max} onclick={() => save({ [key]: clamp(value + step, min, max) })} aria-label="+">+</button>
  </div>
{/snippet}

{#snippet row(title: string, desc: string)}
  <div class="row-text">
    <span>{title}</span>
    {#if desc}<small>{desc}</small>{/if}
  </div>
{/snippet}

<div class="shell">
  <aside class="sidebar">
    <div class="search">
      <svg viewBox="0 0 24 24"><circle cx="11" cy="11" r="7" /><path d="m20 20-3.5-3.5" /></svg>
      <input type="text" bind:value={searchQuery} placeholder={tr("s.search")} />
      {#if searchQuery}<button onclick={() => (searchQuery = "")} aria-label="✕">✕</button>{/if}
    </div>
    <nav>
      {#each SECTIONS as s (s.id)}
        <button class:on={!query && section === s.id} class:hit={query && shows(s.id)} onclick={() => ((section = s.id), (searchQuery = ""))}>
          <i style:background={s.color}><svg viewBox="0 0 24 24"><path d={s.icon} /></svg></i>
          {tr(`nav.${s.id}`)}
        </button>
      {/each}
    </nav>
  </aside>

  <main class="content">
    {#if !settings}
      <p class="muted">{tr("settings.loading")}</p>
    {:else}
      {#if !visible.length}<p class="muted">{tr("s.noResults")}</p>{/if}

      {#if shows("general")}
        <h1>{tr("nav.general")}</h1>
        <div class="group">
          <div class="row">
            {@render row(tr("s.language"), "")}
            <select value={settings.language ?? "en"} onchange={(e) => save({ language: e.currentTarget.value })}>
              <option value="en">English</option>
              <option value="id">Bahasa Indonesia</option>
            </select>
          </div>
          <div class="row">
            {@render row(tr("s.monitor"), tr("s.monitorDesc"))}
            <select value={settings.monitorPlacement ?? "primary"} onchange={(e) => save({ monitorPlacement: e.currentTarget.value })}>
              <option value="primary">{tr("s.monitorPrimary")}</option>
              <option value="secondary">{tr("s.monitorSecondary")}</option>
              <option value="follow-cursor">{tr("s.monitorFollow")}</option>
            </select>
          </div>
          <div class="row">
            {@render row(tr("s.spring"), tr("s.springDesc"))}
            <select value={settings.springPhysics ?? "spring"} onchange={(e) => save({ springPhysics: e.currentTarget.value })}>
              <option value="spring">{tr("s.springBouncy")}</option>
              <option value="snappy">{tr("s.springSnappy")}</option>
              <option value="smooth">{tr("s.springSmooth")}</option>
              <option value="off">{tr("s.off")}</option>
            </select>
          </div>
        </div>
      {/if}

      {#if shows("appearance")}
        <h1>{tr("nav.appearance")}</h1>
        <div class="group">
          <div class="row">
            {@render row(tr("s.floating"), tr("s.floatingDesc"))}
            {@render toggle("floatingNotch", settings.floatingNotch === true)}
          </div>
          <div class="row">
            {@render row(tr("s.minimal"), tr("s.minimalDesc"))}
            {@render toggle("minimalNotch", settings.minimalNotch === true)}
          </div>
          <div class="row">
            {@render row(tr("s.ring"), tr("s.ringDesc"))}
            <select value={settings.statusRing ?? "off"} onchange={(e) => save({ statusRing: e.currentTarget.value })}>
              <option value="off">{tr("s.off")}</option>
              <option value="battery">{tr("s.ringBattery")}</option>
              <option value="water">{tr("s.ringWater")}</option>
              <option value="cpu">{tr("s.ringCpu")}</option>
              <option value="ram">{tr("s.ringRam")}</option>
            </select>
          </div>
        </div>
        <div class="group">
          <div class="row">
            {@render row(tr("s.accent"), tr("s.accentDesc"))}
            {@render toggle("windowsAccentGlow", settings.windowsAccentGlow === true)}
          </div>
        </div>
      {/if}

      {#if shows("eyes")}
        <h1>{tr("nav.eyes")}</h1>
        <div class="group">
          <div class="row">
            {@render row(tr("s.mascot"), tr("s.mascotDesc"))}
            {@render toggle("mascot", settings.mascot !== false)}
          </div>
          {#if settings.mascot !== false}
            <div class="row" transition:slide={{ duration: 150 }}>
              {@render row(tr("s.mascotMode"), "")}
              <select value={settings.mascotMode ?? "idle"} onchange={(e) => save({ mascotMode: e.currentTarget.value })}>
                <option value="idle">{tr("s.mascotIdle")}</option>
                <option value="always">{tr("s.mascotAlways")}</option>
              </select>
            </div>
            <div class="row" transition:slide={{ duration: 150 }}>
              {@render row(tr("s.eyeStyle"), "")}
              <select value={settings.mascotTheme ?? "classic"} onchange={(e) => save({ mascotTheme: e.currentTarget.value })}>
                <option value="classic">{tr("s.eyeClassic")}</option>
                <option value="anime">Anime</option>
                <option value="pixel">Pixel</option>
                <option value="cat">{tr("s.eyeCat")}</option>
                <option value="cartoon">{tr("s.eyeCartoon")}</option>
              </select>
            </div>
            <div class="row" transition:slide={{ duration: 150 }}>
              {@render row(tr("s.musicMascot"), tr("s.musicMascotDesc"))}
              {@render toggle("musicMascot", settings.musicMascot === true)}
            </div>
          {/if}
        </div>
      {/if}

      {#if shows("pages")}
        <h1>{tr("nav.pages")}</h1>
        <p class="lead">{tr("s.pagesDesc")}</p>
        <div class="group">
          {#each PAGE_ITEMS as [key, label] (key)}
            {@const on = settings[key] !== false}
            <div class="row">
              {@render row(tr(label), tr(`${label}Desc`))}
              {@render toggle(key, on, on && enabledPages <= 1)}
            </div>
          {/each}
        </div>
      {/if}

      {#if shows("music")}
        <h1>{tr("nav.music")}</h1>
        <div class="group">
          <div class="row">
            {@render row(tr("s.lyrics"), tr("s.lyricsDesc"))}
            {@render toggle("smallLyrics", settings.smallLyrics)}
          </div>
          <div class="row">
            {@render row(tr("s.swipe"), tr("s.swipeDesc"))}
            {@render toggle("swipeTracks", settings.swipeTracks !== false)}
          </div>
        </div>
        <p class="hint">{tr("s.coverHint")}</p>
      {/if}

      {#if shows("focus")}
        <h1>{tr("nav.focus")}</h1>
        <div class="group">
          <div class="row">
            {@render row(tr("s.focusLen"), "")}
            {@render stepper("focusMinutes", settings.focusMinutes, 5, 180, 5, tr("s.min"))}
          </div>
          <div class="row">
            {@render row(tr("s.breakLen"), "")}
            {@render stepper("breakMinutes", settings.breakMinutes, 1, 60, 1, tr("s.min"))}
          </div>
        </div>
        <h2>{tr("s.health")}</h2>
        <div class="group">
          <div class="row">
            {@render row(tr("s.eyeBreak"), tr("s.eyeBreakDesc"))}
            {@render toggle("eyeBreak", settings.eyeBreak === true)}
          </div>
        </div>
      {/if}

      {#if shows("water")}
        <h1>{tr("nav.water")}</h1>
        <div class="group">
          <div class="row">
            {@render row(tr("s.waterOn"), tr("s.waterOnDesc"))}
            {@render toggle("waterEnabled", settings.waterEnabled)}
          </div>
        </div>
        {#if settings.waterEnabled}
          <div class="group" transition:slide={{ duration: 150 }}>
            <div class="row">
              {@render row(tr("s.glass"), "")}
              {@render stepper("glassMl", settings.glassMl, 50, 1000, 50, "ml")}
            </div>
            <div class="row">
              {@render row(tr("s.goal"), "")}
              {@render stepper("goalMl", settings.goalMl, 500, 6000, 250, "ml")}
            </div>
            <div class="row">
              {@render row(tr("s.from"), "")}
              {@render stepper("waterFrom", settings.waterFrom, 0, 23, 1, ":00")}
            </div>
            <div class="row">
              {@render row(tr("s.to"), "")}
              {@render stepper("waterTo", settings.waterTo, 0, 23, 1, ":00")}
            </div>
            <div class="row">
              {@render row(tr("s.remindEvery"), tr("s.remindEveryDesc"))}
              {@render stepper("remindEvery", settings.remindEvery, 0, 240, 15, tr("s.min"))}
            </div>
          </div>
        {/if}
      {/if}

      {#if shows("calendar")}
        <h1>{tr("nav.calendar")}</h1>
        <div class="group">
          <div class="row column">
            {@render row(tr("s.calendarUrl"), tr("s.calendarDesc"))}
            <input
              type="password"
              class="text"
              value={settings.calendarUrl}
              placeholder="https://calendar.google.com/calendar/ical/…/basic.ics"
              onchange={(e) => save({ calendarUrl: e.currentTarget.value.trim() })}
            />
          </div>
        </div>
        <p class="hint">{tr("settings.calendarHint")}</p>
        <h2>{tr("s.countdowns")}</h2>
        <p class="lead">{tr("s.countdownsDesc")}</p>
        {#if (settings.countdowns ?? []).length}
          <div class="group">
            {#each settings.countdowns ?? [] as c, i}
              <div class="row shortcut">
                <input class="text name" value={c.name} placeholder={tr("s.countdownName")} onchange={(e) => editCountdown(i, { name: e.currentTarget.value.trim() })} />
                <input class="text date" type="date" value={c.date} onchange={(e) => editCountdown(i, { date: e.currentTarget.value })} />
                <select value={c.repeat || "none"} onchange={(e) => editCountdown(i, { repeat: e.currentTarget.value })}>
                  <option value="none">{tr("s.repeatNone")}</option>
                  <option value="monthly">{tr("s.repeatMonthly")}</option>
                  <option value="yearly">{tr("s.repeatYearly")}</option>
                </select>
                <button class="icon-btn" title="✕" onclick={() => save({ countdowns: (settings!.countdowns ?? []).filter((_, j) => j !== i) })}>✕</button>
              </div>
            {/each}
          </div>
        {/if}
        <div class="actions">
          {#if (settings.countdowns ?? []).length < 10}
            <button class="btn" onclick={() => save({ countdowns: [...(settings!.countdowns ?? []), { name: "", date: todayIso(), repeat: "none" }] })}>
              {tr("s.addCountdown")}
            </button>
          {/if}
        </div>
      {/if}

      {#if shows("shortcuts")}
        <h1>{tr("nav.shortcuts")}</h1>
        <div class="group">
          {#each settings.shortcuts as s, i}
            <div class="row shortcut">
              <input class="text name" value={s.name} placeholder={tr("settings.shortcutNamePlaceholder")} onchange={(e) => editShortcut(i, { name: e.currentTarget.value.trim() })} />
              <select
                value={s.action}
                onchange={(e) => {
                  const action = e.currentTarget.value;
                  const label = tr(`actions.${action}`).replace(/ \(Settings\)$/, "");
                  editShortcut(i, action === "open" ? { action } : { action, name: label });
                }}
              >
                {#each ACTION_ITEMS as [val, label]}<option value={val}>{tr(label)}</option>{/each}
              </select>
              <button class="icon-btn" title={tr("settings.deleteShortcut")} onclick={() => save({ shortcuts: settings!.shortcuts.filter((_, j) => j !== i) })}>✕</button>
              {#if s.action === "open"}
                <input class="text target" value={s.target} placeholder={tr("settings.shortcutTargetPlaceholder")} onchange={(e) => editShortcut(i, { target: e.currentTarget.value.trim() })} />
              {/if}
            </div>
          {/each}
        </div>
        <div class="actions">
          {#if settings.shortcuts.length < 8}
            <button class="btn" onclick={() => save({ shortcuts: [...settings!.shortcuts, { name: tr("shortcuts.title"), action: "open", target: "" }] })}>
              {tr("settings.addShortcut")}
            </button>
          {/if}
          <button class="btn ghost" onclick={() => save({ shortcuts: defaultShortcuts() })}>{tr("settings.restoreDefaults")}</button>
        </div>
      {/if}

      {#if shows("notifications")}
        <h1>{tr("nav.notifications")}</h1>
        <div class="group">
          <div class="row">
            {@render row(tr("s.mirror"), tr("s.mirrorDesc"))}
            {@render toggle("mirrorNotifications", settings.mirrorNotifications)}
          </div>
          <div class="row">
            {@render row(tr("s.dnd"), tr("s.dndDesc"))}
            <button class="btn" onclick={() => invoke("open_dnd_settings")}>{tr("s.open")}</button>
          </div>
          <div class="row">
            {@render row(tr("s.presentation"), tr("s.presentationDesc"))}
            {@render toggle("presentationSummary", settings.presentationSummary !== false)}
          </div>
        </div>
        <div class="group">
          <div class="row">
            {@render row(tr("s.privacy"), tr("s.privacyDesc"))}
            {@render toggle("privacyIndicators", settings.privacyIndicators !== false)}
          </div>
          <div class="row">
            {@render row(tr("s.connection"), tr("s.connectionDesc"))}
            {@render toggle("connectionPeek", settings.connectionPeek !== false)}
          </div>
          <div class="row">
            {@render row(tr("s.charge"), tr("s.chargeDesc"))}
            <select value={String(settings.chargeReminder ?? 0)} onchange={(e) => save({ chargeReminder: +e.currentTarget.value })}>
              <option value="0">{tr("s.off")}</option>
              <option value="80">80%</option>
              <option value="85">85%</option>
              <option value="90">90%</option>
              <option value="100">100%</option>
            </select>
          </div>
        </div>
      {/if}

      {#if shows("system")}
        <h1>{tr("nav.system")}</h1>
        <h2>{tr("s.keyboard")}</h2>
        <div class="group">
          <div class="row">
            {@render row(tr("s.caps"), tr("s.capsDesc"))}
            {@render toggle("capsLockPeek", settings.capsLockPeek)}
          </div>
          <div class="row">
            {@render row(tr("s.hideOsd"), tr("s.hideOsdDesc"))}
            {@render toggle("hideVolumeOsd", settings.hideVolumeOsd === true)}
          </div>
        </div>
        <p class="hint">{tr("s.restartHint")}</p>
        <h2>{tr("s.hotkeys")}</h2>
        <div class="group">
          <div class="row">
            {@render row(tr("s.openHotkey"), tr("s.openHotkeyDesc"))}
            <input class="text hotkey" value={settings.openHotkey ?? ""} placeholder="Ctrl+Alt+N" onchange={(e) => save({ openHotkey: e.currentTarget.value.trim() })} />
          </div>
          <div class="row">
            {@render row(tr("s.ocrHotkey"), tr("s.ocrHotkeyDesc"))}
            <input class="text hotkey" value={settings.ocrHotkey ?? ""} placeholder="Ctrl+Alt+T" onchange={(e) => save({ ocrHotkey: e.currentTarget.value.trim() })} />
          </div>
        </div>
        <h2>Script & Claude Code</h2>
        <div class="group">
          <div class="row">
            {@render row(tr("s.external"), tr("s.externalDesc"))}
            {@render toggle("externalPeeks", settings.externalPeeks !== false)}
          </div>
          <div class="row">
            {@render row(tr("s.agent"), tr("s.agentDesc"))}
            {@render toggle("agentStatus", settings.agentStatus !== false)}
          </div>
          {#if settings.agentStatus !== false}
            <div class="row column" transition:slide={{ duration: 150 }}>
              {@render row(tr("s.agentHint"), "")}
              <div class="ai-chips">
                {#each AI_TOOLS as tool}
                  <button
                    class="chip"
                    class:active={selectedAi === tool.id}
                    onclick={() => (selectedAi = tool.id)}
                    type="button"
                  >
                    {tool.label}
                  </button>
                {/each}
              </div>
              <div class="snippet-box">
                <pre class="code">{aiSnippet}</pre>
                <button class="btn" onclick={copyHooks}>{hookCopied ? `✓ ${tr("s.copiedOk")}` : tr("s.copy")}</button>
              </div>
            </div>
          {/if}
        </div>
        <h2>Bluetooth</h2>
        <div class="group">
          <div class="row">
            {@render row(tr("s.bluetooth"), tr("s.bluetoothDesc"))}
            {@render toggle("bluetoothPeek", settings.bluetoothPeek === true)}
          </div>
          {#if settings.bluetoothPeek}
            <div class="row" transition:slide={{ duration: 150 }}>
              {@render row(tr("s.lowBattery"), "")}
              {@render stepper("bluetoothLowThreshold", settings.bluetoothLowThreshold ?? 20, 5, 50, 5, "%")}
            </div>
          {/if}
        </div>
        <h2>{tr("s.network")}</h2>
        <div class="group">
          <div class="row">
            {@render row(tr("s.netStatus"), tr("s.netStatusDesc"))}
            {@render toggle("netSpeedStatus", settings.netSpeedStatus !== false)}
          </div>
          <div class="row">
            {@render row(tr("s.netIsland"), tr("s.netIslandDesc"))}
            {@render toggle("netSpeedIsland", settings.netSpeedIsland === true)}
          </div>
          {#if settings.netSpeedIsland}
            <div class="row" transition:slide={{ duration: 150 }}>
              {@render row(tr("s.netDown"), "")}
              {@render stepper("netSpeedThresholdDownKb", settings.netSpeedThresholdDownKb ?? 500, 50, 100000, 250, "KB/s")}
            </div>
            <div class="row" transition:slide={{ duration: 150 }}>
              {@render row(tr("s.netUp"), "")}
              {@render stepper("netSpeedThresholdUpKb", settings.netSpeedThresholdUpKb ?? 250, 50, 100000, 250, "KB/s")}
            </div>
          {/if}
        </div>
      {/if}

      {#if shows("weather")}
        <h1>{tr("nav.weather")}</h1>
        <div class="group">
          <div class="row">
            {@render row(tr("s.location"), settings.weatherCity ? settings.weatherCity.name : tr("s.locationAuto"))}
            {#if settings.weatherCity}
              <button class="btn ghost" onclick={() => save({ weatherCity: null })}>{tr("s.useAuto")}</button>
            {/if}
          </div>
          <div class="row">
            {@render row(tr("s.rain"), tr("s.rainDesc"))}
            {@render toggle("rainAlert", settings.rainAlert !== false)}
          </div>
          <form class="row" onsubmit={searchCityHandler}>
            <input class="text grow" bind:value={cityQuery} placeholder={tr("s.cityPlaceholder")} />
            <button class="btn" type="submit" disabled={searchingCity}>{searchingCity ? "…" : tr("s.search")}</button>
          </form>
          {#if cityNotFound}
            <div class="row"><small class="muted">{tr("s.cityNotFound")}</small></div>
          {/if}
          {#each cityResults as place}
            <button
              class="row pick"
              onclick={() => {
                save({ weatherCity: { name: place.name, latitude: place.latitude, longitude: place.longitude } });
                cityResults = [];
                cityQuery = "";
              }}
            >
              {@render row(place.name, place.detail)}
            </button>
          {/each}
        </div>
      {/if}

      {#if shows("prayer")}
        <h1>{tr("nav.prayer")}</h1>
        <div class="group">
          <div class="row">
            {@render row(tr("s.prayerOn"), tr("s.prayerOnDesc"))}
            {@render toggle("prayerEnabled", settings.prayerEnabled === true)}
          </div>
          {#if settings.prayerEnabled}
            <div class="row" transition:slide={{ duration: 150 }}>
              {@render row(tr("s.prayerWeatherSync"), settings.weatherCity ? settings.weatherCity.name : tr("s.prayerAutoGps"))}
            </div>
          {/if}
        </div>

        {#if settings.prayerEnabled}
          <div transition:slide={{ duration: 150 }}>
            <h2>{tr("nav.notifications")}</h2>
            <div class="group">
              <div class="row">
                {@render row(tr("s.prayerPreAlert"), tr("s.prayerPreAlertDesc"))}
                {@render toggle("prayerPreAlert", settings.prayerPreAlert !== false)}
              </div>
              <div class="row">
                {@render row(tr("s.prayerSound"), tr("s.prayerSoundDesc"))}
                {@render toggle("prayerSound", settings.prayerSound !== false)}
              </div>
              <div class="row">
                {@render row(tr("s.prayerMethod"), tr("s.prayerMethodDesc"))}
                <select value={settings.prayerMethod ?? "kemenag"} onchange={(e) => save({ prayerMethod: e.currentTarget.value })}>
                  <option value="kemenag">{tr("s.prayerMethodKemenag")}</option>
                  <option value="mwl">{tr("s.prayerMethodMwl")}</option>
                  <option value="egyptian">{tr("s.prayerMethodEgyptian")}</option>
                  <option value="isna">{tr("s.prayerMethodIsna")}</option>
                </select>
              </div>
            </div>

            <h2>{tr("nav.prayer")}</h2>
            <div class="group">
              <div class="row">
                {@render row(tr("s.prayerFajr"), previewPrayerTimes ? previewPrayerTimes.fajr : "")}
                {@render toggle("prayerFajr", settings.prayerFajr !== false)}
              </div>
              <div class="row">
                {@render row(tr("s.prayerDhuhr"), previewPrayerTimes ? previewPrayerTimes.dhuhr : "")}
                {@render toggle("prayerDhuhr", settings.prayerDhuhr !== false)}
              </div>
              <div class="row">
                {@render row(tr("s.prayerAsr"), previewPrayerTimes ? previewPrayerTimes.asr : "")}
                {@render toggle("prayerAsr", settings.prayerAsr !== false)}
              </div>
              <div class="row">
                {@render row(tr("s.prayerMaghrib"), previewPrayerTimes ? previewPrayerTimes.maghrib : "")}
                {@render toggle("prayerMaghrib", settings.prayerMaghrib !== false)}
              </div>
              <div class="row">
                {@render row(tr("s.prayerIsha"), previewPrayerTimes ? previewPrayerTimes.isha : "")}
                {@render toggle("prayerIsha", settings.prayerIsha !== false)}
              </div>
            </div>
          </div>
        {/if}
      {/if}
    {/if}
  </main>

  {#if saved}
    <div class="toast" transition:fade={{ duration: 140 }}>✓ {tr("settings.saved")}</div>
  {/if}
</div>

<style>
  :global(html),
  :global(body) {
    margin: 0;
    height: 100%;
    background: #1e1e20;
    color: #f5f5f7;
    font-family: "Segoe UI Variable Text", "Segoe UI", system-ui, sans-serif;
    font-size: 13px;
    overflow: hidden;
    color-scheme: dark;
  }
  .shell {
    display: flex;
    height: 100vh;
  }

  /* Sidebar */
  .sidebar {
    flex: none;
    width: 212px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 14px 10px;
    background: #262628;
    border-right: 1px solid rgba(255, 255, 255, 0.06);
    box-sizing: border-box;
  }
  .search {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0 8px;
    height: 28px;
    border-radius: 7px;
    background: rgba(255, 255, 255, 0.07);
  }
  .search svg {
    flex: none;
    width: 13px;
    height: 13px;
    fill: none;
    stroke: rgba(255, 255, 255, 0.45);
    stroke-width: 2.2;
    stroke-linecap: round;
  }
  .search input {
    flex: 1;
    min-width: 0;
    border: 0;
    outline: 0;
    background: none;
    color: inherit;
    font: inherit;
  }
  .search button {
    border: 0;
    background: none;
    color: rgba(255, 255, 255, 0.45);
    cursor: pointer;
    font-size: 11px;
  }
  nav {
    display: flex;
    flex-direction: column;
    gap: 1px;
    overflow-y: auto;
    scrollbar-width: none;
  }
  nav button {
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 5px 8px;
    border: 0;
    border-radius: 7px;
    background: none;
    color: inherit;
    font: inherit;
    text-align: left;
    cursor: pointer;
  }
  nav button:hover {
    background: rgba(255, 255, 255, 0.05);
  }
  nav button.on {
    background: #0a84ff;
  }
  nav button.hit {
    background: rgba(10, 132, 255, 0.22);
  }
  nav i {
    flex: none;
    display: grid;
    place-items: center;
    width: 22px;
    height: 22px;
    border-radius: 6px;
    box-shadow: inset 0 0 0 0.5px rgba(255, 255, 255, 0.15);
  }
  nav i svg {
    width: 14px;
    height: 14px;
    fill: none;
    stroke: #fff;
    stroke-width: 2;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  /* Content */
  .content {
    flex: 1;
    min-width: 0;
    overflow-y: auto;
    padding: 18px 24px 40px;
    box-sizing: border-box;
  }
  h1 {
    margin: 0 0 14px;
    font-size: 19px;
    font-weight: 700;
  }
  h1:not(:first-child) {
    margin-top: 26px;
  }
  h2 {
    margin: 18px 0 6px 4px;
    font-size: 11.5px;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.5);
  }
  .lead,
  .hint,
  .muted {
    margin: -6px 4px 10px;
    font-size: 11.5px;
    color: rgba(255, 255, 255, 0.5);
  }
  .hint {
    margin: -4px 4px 12px;
  }
  .group {
    margin-bottom: 12px;
    border-radius: 10px;
    background: #2c2c2e;
    box-shadow: inset 0 0 0 0.5px rgba(255, 255, 255, 0.07);
    overflow: hidden;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 44px;
    padding: 7px 12px;
    box-sizing: border-box;
    margin: 0;
  }
  .row + .row {
    box-shadow: inset 0 0.5px 0 rgba(255, 255, 255, 0.09);
  }
  .row.column {
    flex-direction: column;
    align-items: stretch;
  }
  .row-text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .row-text small {
    font-size: 11.5px;
    color: rgba(255, 255, 255, 0.5);
    line-height: 1.35;
  }
  button.row.pick {
    width: 100%;
    border: 0;
    background: none;
    color: inherit;
    font: inherit;
    text-align: left;
    cursor: pointer;
  }
  button.row.pick:hover {
    background: rgba(255, 255, 255, 0.05);
  }

  /* Controls */
  select,
  .text,
  .btn,
  .stepper {
    height: 26px;
    border-radius: 6px;
    border: 0;
    background: rgba(255, 255, 255, 0.1);
    color: inherit;
    font: inherit;
    box-sizing: border-box;
  }
  select {
    flex: none;
    max-width: 210px;
    padding: 0 8px;
    cursor: pointer;
  }
  select option {
    background: #2c2c2e;
  }
  .text {
    padding: 0 9px;
    outline: 0;
  }
  .text:focus {
    box-shadow: 0 0 0 2px #0a84ff;
  }
  .grow {
    flex: 1;
    min-width: 0;
  }
  .btn {
    flex: none;
    padding: 0 12px;
    cursor: pointer;
  }
  .btn:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.16);
  }
  .btn.ghost {
    background: none;
    color: #0a84ff;
  }
  .actions {
    display: flex;
    gap: 6px;
    margin: -4px 0 12px;
  }
  .stepper {
    flex: none;
    display: flex;
    align-items: center;
    overflow: hidden;
  }
  .stepper button {
    width: 26px;
    height: 100%;
    border: 0;
    background: none;
    color: inherit;
    font-size: 15px;
    cursor: pointer;
  }
  .stepper button:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.1);
  }
  .stepper button:disabled {
    opacity: 0.3;
    cursor: default;
  }
  .stepper span {
    min-width: 64px;
    text-align: center;
    font-variant-numeric: tabular-nums;
  }
  .switch {
    position: relative;
    flex: none;
    width: 38px;
    height: 22px;
    cursor: pointer;
  }
  .switch input {
    position: absolute;
    opacity: 0;
    width: 0;
    height: 0;
  }
  .switch span {
    position: absolute;
    inset: 0;
    border-radius: 11px;
    background: rgba(255, 255, 255, 0.16);
    transition: background 0.18s;
  }
  .switch span::after {
    content: "";
    position: absolute;
    top: 2px;
    left: 2px;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: #fff;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.35);
    transition: transform 0.18s;
  }
  .switch input:checked + span {
    background: #0a84ff;
  }
  .switch input:checked + span::after {
    transform: translateX(16px);
  }
  .switch input:disabled + span {
    opacity: 0.45;
  }
  .switch input:focus-visible + span {
    box-shadow: 0 0 0 2px rgba(10, 132, 255, 0.6);
  }

  .shortcut {
    flex-wrap: wrap;
  }
  .shortcut .name {
    flex: 1;
    min-width: 0;
  }
  .shortcut .target {
    flex-basis: 100%;
  }
  .shortcut .date {
    width: 140px;
    color-scheme: dark;
  }
  .text.hotkey {
    width: 130px;
    text-align: center;
  }
  .ai-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin: 4px 0 8px;
  }
  .chip {
    padding: 4px 10px;
    border-radius: 999px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: rgba(255, 255, 255, 0.05);
    color: rgba(255, 255, 255, 0.7);
    font-size: 11px;
    cursor: pointer;
    transition: all 0.15s ease;
  }
  .chip:hover {
    background: rgba(255, 255, 255, 0.12);
    color: #fff;
  }
  .chip.active {
    background: #0a84ff;
    border-color: #0a84ff;
    color: #fff;
    font-weight: 500;
  }
  .snippet-box {
    display: flex;
    flex-direction: column;
    gap: 8px;
    width: 100%;
  }
  .code {
    margin: 0;
    max-height: 150px;
    overflow: auto;
    padding: 8px 10px;
    border-radius: 8px;
    background: rgba(0, 0, 0, 0.35);
    font: 11px/1.45 "Cascadia Mono", Consolas, monospace;
    color: rgba(255, 255, 255, 0.8);
    user-select: text;
  }
  .icon-btn {
    border: 0;
    background: none;
    color: rgba(255, 255, 255, 0.45);
    cursor: pointer;
  }
  .icon-btn:hover {
    color: #ff453a;
  }

  .toast {
    position: fixed;
    right: 16px;
    bottom: 14px;
    padding: 5px 12px;
    border-radius: 999px;
    background: rgba(48, 209, 88, 0.16);
    color: #30d158;
    font-size: 12px;
  }
</style>
