<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { fade, slide } from "svelte/transition";
  import { searchCity, type Place } from "$lib/weather";
  import { t, setLocale, getLocale, subscribeLocale } from "$lib/i18n";

  type City = { name: string; latitude: number; longitude: number };
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
    minimalNotch: boolean;
    mirrorNotifications: boolean;
    calendarUrl: string;
    smallLyrics: boolean;
    shortcuts: Shortcut[];
    capsLockPeek: boolean;
    hideVolumeOsd?: boolean;
    musicEnabled: boolean;
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
  };

  type Shortcut = { name: string; action: string; target: string };

  let currentLang = $state(getLocale());
  $effect(() => subscribeLocale((l) => (currentLang = l)));

  // Localized resolver reactive to language changes
  const tr = (key: string, params?: Record<string, string | number>) => {
    void currentLang;
    return t(key, params);
  };

  const ACTION_ITEMS: [string, string][] = [
    ["lock", "actions.lock"],
    ["sleep", "actions.sleep"],
    ["screenshot", "actions.screenshot"],
    ["downloads", "actions.downloads"],
    ["camera", "actions.camera"],
    ["open", "actions.open"],
  ];

  type TabKey = "display" | "system" | "productivity" | "media";

  let activeTab = $state<TabKey>("display");
  let searchQuery = $state("");
  let settings = $state<Settings | null>(null);
  let accentColor = $state("#38bdf8");
  let saved = $state(false);

  // Weather search state
  let cityQuery = $state("");
  let cityResults = $state<Place[]>([]);
  let searchingCity = $state(false);

  type PageToggleKey =
    | "musicEnabled"
    | "pomodoroEnabled"
    | "waterEnabled"
    | "todoEnabled"
    | "shelfEnabled"
    | "clipboardEnabled"
    | "shortcutsEnabled";

  const PAGE_ITEMS: [PageToggleKey, string, string][] = [
    ["musicEnabled", "pageItems.music", "pageItems.musicDesc"],
    ["pomodoroEnabled", "pageItems.pomodoro", "pageItems.pomodoroDesc"],
    ["waterEnabled", "pageItems.water", "pageItems.waterDesc"],
    ["todoEnabled", "pageItems.todo", "pageItems.todoDesc"],
    ["shelfEnabled", "pageItems.shelf", "pageItems.shelfDesc"],
    ["clipboardEnabled", "pageItems.clipboard", "pageItems.clipboardDesc"],
    ["shortcutsEnabled", "pageItems.shortcuts", "pageItems.shortcutsDesc"],
  ];

  const enabledPagesCount = $derived.by(() => {
    const s = settings;
    return s ? PAGE_ITEMS.filter(([key]) => s[key] !== false).length : 7;
  });

  onMount(async () => {
    settings = await invoke<Settings>("get_settings");
    if (settings?.language) {
      setLocale(settings.language);
    }
    accentColor = await invoke<string>("get_accent_color").catch(() => "#38bdf8");
  });

  async function save(patch: Partial<Settings> = {}) {
    if (!settings) return;
    if (patch.language) {
      setLocale(patch.language);
    }
    settings = { ...settings, ...patch };
    await invoke("save_settings", { settings });
    saved = true;
    setTimeout(() => (saved = false), 1800);
  }

  function editShortcut(i: number, patch: Partial<Shortcut>) {
    if (!settings) return;
    save({ shortcuts: settings.shortcuts.map((s, j) => (j === i ? { ...s, ...patch } : s)) });
  }

  async function searchCityHandler(e: SubmitEvent) {
    e.preventDefault();
    if (!cityQuery.trim()) return;
    searchingCity = true;
    cityResults = await searchCity(cityQuery.trim()).catch(() => []);
    searchingCity = false;
  }

  function matches(text: string): boolean {
    if (!searchQuery.trim()) return true;
    return text.toLowerCase().includes(searchQuery.trim().toLowerCase());
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
</script>

<div class="settings-shell">
  <!-- Sidebar / Header Banner -->
  <header class="app-header">
    <div class="brand">
      <div class="logo-badge">
        <svg viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8z" />
          <path d="M7 10.5C7 9.67 7.67 9 8.5 9s1.5.67 1.5 1.5S9.33 12 8.5 12 7 11.33 7 10.5zm7 0c0-.83.67-1.5 1.5-1.5s1.5.67 1.5 1.5-.67 1.5-1.5 1.5-1.5-.67-1.5-1.5z" />
        </svg>
      </div>
      <div>
        <h1 class="app-title">{tr("settings.title")}</h1>
        <span class="version-tag">v0.2.5 · {tr("settings.subtitle")}</span>
      </div>
    </div>

    <!-- Quick Search -->
    <div class="search-box">
      <svg class="search-icon" viewBox="0 0 24 24">
        <circle cx="11" cy="11" r="8" fill="none" stroke="currentColor" stroke-width="2" />
        <path d="M21 21l-4.35-4.35" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
      </svg>
      <input
        type="text"
        bind:value={searchQuery}
        placeholder={tr("settings.searchPlaceholder")}
      />
      {#if searchQuery}
        <button class="clear-search" onclick={() => (searchQuery = "")}>✕</button>
      {/if}
    </div>
  </header>

  <!-- Tab Bar (Minimalist 4-Column Grid, No Overflow) -->
  <div class="tabs-nav" role="tablist" aria-label={tr("settings.title")}>
    <button
      class="tab-btn"
      class:active={activeTab === "display"}
      onclick={() => (activeTab = "display")}
      role="tab"
      aria-selected={activeTab === "display"}
    >
      <span class="tab-icon">🖥️</span>
      <span class="tab-text">{tr("tabs.display")}</span>
    </button>
    <button
      class="tab-btn"
      class:active={activeTab === "system"}
      onclick={() => (activeTab = "system")}
      role="tab"
      aria-selected={activeTab === "system"}
    >
      <span class="tab-icon">⚙️</span>
      <span class="tab-text">{tr("tabs.system")}</span>
    </button>
    <button
      class="tab-btn"
      class:active={activeTab === "productivity"}
      onclick={() => (activeTab = "productivity")}
      role="tab"
      aria-selected={activeTab === "productivity"}
    >
      <span class="tab-icon">⏱️</span>
      <span class="tab-text">{tr("tabs.focus")}</span>
    </button>
    <button
      class="tab-btn"
      class:active={activeTab === "media"}
      onclick={() => (activeTab = "media")}
      role="tab"
      aria-selected={activeTab === "media"}
    >
      <span class="tab-icon">🎵</span>
      <span class="tab-text">{tr("tabs.media")}</span>
    </button>
  </div>

  <!-- Main Content Area -->
  <main class="settings-content">
    {#if !settings}
      <div class="loading-state">
        <div class="spinner"></div>
        <p>{tr("settings.loading")}</p>
      </div>
    {:else}
      <!-- TAB 1: TAMPILAN & LAYAR / DISPLAY -->
      {#if activeTab === "display" || searchQuery}
        <div class="tab-panel" in:fade={{ duration: 180 }}>
          {#if matches("Language Bahasa Inggris Indonesia English")}
            <section class="setting-card">
              <div class="card-header">
                <div class="card-icon">🌐</div>
                <div class="card-info">
                  <h3>{tr("settings.language")}</h3>
                  <p>{tr("settings.languageDesc")}</p>
                </div>
              </div>
              <div class="control-row">
                <select
                  class="select-input"
                  value={settings.language ?? "en"}
                  onchange={(e) => {
                    const lang = e.currentTarget.value;
                    setLocale(lang);
                    save({ language: lang });
                  }}
                >
                  <option value="en">{tr("settings.langEn")}</option>
                  <option value="id">{tr("settings.langId")}</option>
                </select>
              </div>
            </section>
          {/if}

          {#if matches("Layar Multi-Monitor Penempatan Posisi Display Primary Secondary Kursor Screen Cursor Placement")}
            <section class="setting-card">
              <div class="card-header">
                <div class="card-icon">🖥️</div>
                <div class="card-info">
                  <h3>{tr("settings.monitorTitle")}</h3>
                  <p>{tr("settings.monitorDesc")}</p>
                </div>
              </div>
              <div class="control-row">
                <select
                  class="select-input"
                  value={settings.monitorPlacement ?? "primary"}
                  onchange={(e) => save({ monitorPlacement: e.currentTarget.value })}
                >
                  <option value="primary">{tr("settings.monitorPrimary")}</option>
                  <option value="secondary">{tr("settings.monitorSecondary")}</option>
                  <option value="follow-cursor">{tr("settings.monitorFollow")}</option>
                </select>
              </div>
            </section>
          {/if}

          {#if matches("Animasi Spring Physics Pegas Kecepatan Transisi Organik Bouncy Animation Transition")}
            <section class="setting-card">
              <div class="card-header">
                <div class="card-icon">🌀</div>
                <div class="card-info">
                  <h3>{tr("settings.springTitle")}</h3>
                  <p>{tr("settings.springDesc")}</p>
                </div>
              </div>
              <div class="control-row">
                <select
                  class="select-input"
                  value={settings.springPhysics ?? "spring"}
                  onchange={(e) => save({ springPhysics: e.currentTarget.value })}
                >
                  <option value="spring">{tr("settings.springOrganic")}</option>
                  <option value="snappy">{tr("settings.springSnappy")}</option>
                  <option value="smooth">{tr("settings.springSmooth")}</option>
                  <option value="off">{tr("settings.springOff")}</option>
                </select>
              </div>
            </section>
          {/if}

          {#if matches("Sinkronisasi Warna Aksen Windows Dynamic Accent DWM Glow Border Theme")}
            <section class="setting-card">
              <div class="card-header">
                <div class="card-icon">✨</div>
                <div class="card-info">
                  <h3>{tr("settings.accentTitle")}</h3>
                  <p>{tr("settings.accentDesc")}</p>
                </div>
                <label class="switch" title="Toggle Windows Accent Sync">
                  <input
                    type="checkbox"
                    checked={settings.windowsAccentGlow === true}
                    onchange={(e) => save({ windowsAccentGlow: e.currentTarget.checked })}
                  />
                  <span class="slider"></span>
                </label>
              </div>

              <div class="accent-status-box">
                <div class="accent-preview">
                  <span
                    class="color-dot"
                    style:background={settings.windowsAccentGlow ? accentColor : "rgba(255,255,255,0.3)"}
                  ></span>
                  <span class="accent-mode-text">
                    {tr("settings.accentCurrentMode")}
                    <strong>{settings.windowsAccentGlow ? tr("settings.accentDwmActive") : tr("settings.accentDefaultNeutral")}</strong>
                    {#if settings.windowsAccentGlow}
                      <code class="color-hex">{accentColor}</code>
                    {/if}
                  </span>
                </div>
                <p class="accent-hint">
                  {settings.windowsAccentGlow
                    ? tr("settings.accentHintActive")
                    : tr("settings.accentHintDefault")}
                </p>
              </div>
            </section>
          {/if}

          {#if matches("Mascot Mata Ekspresif Minimal Notch Bezel Tipis Eyes Thin Character")}
            <section class="setting-card">
              <div class="card-header">
                <div class="card-icon">👀</div>
                <div class="card-info">
                  <h3>{tr("settings.mascotStyleTitle")}</h3>
                  <p>{tr("settings.mascotStyleDesc")}</p>
                </div>
              </div>

              <div class="toggle-list">
                <div class="toggle-item">
                  <div class="toggle-text">
                    <strong>{tr("settings.mascotTitle")}</strong>
                    <small>{tr("settings.mascotDesc")}</small>
                  </div>
                  <label class="switch">
                    <input
                      type="checkbox"
                      checked={settings.mascot !== false}
                      onchange={(e) => save({ mascot: e.currentTarget.checked })}
                    />
                    <span class="slider"></span>
                  </label>
                </div>

                {#if settings.mascot !== false}
                  <div class="nested-control" transition:slide={{ duration: 160 }}>
                    <label class="input-inline">
                      <div class="toggle-text">
                        <strong>{tr("settings.mascotModeTitle")}</strong>
                        <small>{tr("settings.mascotModeDesc")}</small>
                      </div>
                      <select
                        class="select-input"
                        value={settings.mascotMode ?? "idle"}
                        onchange={(e) => save({ mascotMode: e.currentTarget.value })}
                      >
                        <option value="idle">{tr("settings.mascotModeIdle")}</option>
                        <option value="always">{tr("settings.mascotModeAlways")}</option>
                      </select>
                    </label>
                  </div>
                {/if}

                <div class="toggle-item">
                  <div class="toggle-text">
                    <strong>{tr("settings.minimalTitle")}</strong>
                    <small>{tr("settings.minimalDesc")}</small>
                  </div>
                  <label class="switch">
                    <input
                      type="checkbox"
                      checked={settings.minimalNotch === true}
                      onchange={(e) => save({ minimalNotch: e.currentTarget.checked })}
                    />
                    <span class="slider"></span>
                  </label>
                </div>
              </div>
            </section>
          {/if}
        </div>
      {/if}

      <!-- TAB 2: SISTEM & INDIKATOR / SYSTEM -->
      {#if activeTab === "system" || searchQuery}
        <div class="tab-panel" in:fade={{ duration: 180 }}>
          {#if matches("Bluetooth Baterai Headset TWS Mouse Keyboard Perangkat Device Battery")}
            <section class="setting-card">
              <div class="card-header">
                <div class="card-icon">🎧</div>
                <div class="card-info">
                  <h3>{tr("settings.bluetoothTitle")}</h3>
                  <p>{tr("settings.bluetoothDesc")}</p>
                </div>
                <label class="switch">
                  <input
                    type="checkbox"
                    checked={settings.bluetoothPeek !== false}
                    onchange={(e) => save({ bluetoothPeek: e.currentTarget.checked })}
                  />
                  <span class="slider"></span>
                </label>
              </div>

              {#if settings.bluetoothPeek !== false}
                <div class="nested-control" transition:slide={{ duration: 160 }}>
                  <label class="input-inline">
                    <span>{tr("settings.bluetoothLowThreshold")}</span>
                    <div class="number-unit">
                      <input
                        type="number"
                        min="5"
                        max="50"
                        value={settings.bluetoothLowThreshold ?? 20}
                        onchange={(e) => {
                          const val = Math.max(5, Math.min(50, Number(e.currentTarget.value) || 20));
                          save({ bluetoothLowThreshold: val });
                        }}
                      />
                      <em>%</em>
                    </div>
                  </label>
                </div>
              {/if}
            </section>
          {/if}

          {#if matches("Kecepatan Jaringan Network Speed Download Upload Internet Traffic")}
            <section class="setting-card">
              <div class="card-header">
                <div class="card-icon">🚀</div>
                <div class="card-info">
                  <h3>{tr("settings.netSpeedMonitorTitle")}</h3>
                  <p>{tr("settings.netSpeedMonitorDesc")}</p>
                </div>
              </div>

              <div class="toggle-list">
                <div class="toggle-item">
                  <div class="toggle-text">
                    <strong>{tr("settings.netSpeedIslandTitle")}</strong>
                    <small>{tr("settings.netSpeedIslandDesc")}</small>
                  </div>
                  <label class="switch">
                    <input
                      type="checkbox"
                      checked={settings.netSpeedIsland === true}
                      onchange={(e) => save({ netSpeedIsland: e.currentTarget.checked })}
                    />
                    <span class="slider"></span>
                  </label>
                </div>

                {#if settings.netSpeedIsland}
                  <div class="nested-grid" transition:slide={{ duration: 160 }}>
                    <label class="input-inline">
                      <span>{tr("settings.netDownThreshold")}</span>
                      <div class="number-unit">
                        <input
                          type="number"
                          min="50"
                          value={settings.netSpeedThresholdDownKb ?? 500}
                          onchange={(e) => save({ netSpeedThresholdDownKb: Math.max(0, Number(e.currentTarget.value)) })}
                        />
                        <em>KB/s</em>
                      </div>
                    </label>
                    <label class="input-inline">
                      <span>{tr("settings.netUpThreshold")}</span>
                      <div class="number-unit">
                        <input
                          type="number"
                          min="50"
                          value={settings.netSpeedThresholdUpKb ?? 250}
                          onchange={(e) => save({ netSpeedThresholdUpKb: Math.max(0, Number(e.currentTarget.value)) })}
                        />
                        <em>KB/s</em>
                      </div>
                    </label>
                  </div>
                {/if}

                <div class="toggle-item">
                  <div class="toggle-text">
                    <strong>{tr("settings.netSpeedStatusTitle")}</strong>
                    <small>{tr("settings.netSpeedStatusDesc")}</small>
                  </div>
                  <label class="switch">
                    <input
                      type="checkbox"
                      checked={settings.netSpeedStatus !== false}
                      onchange={(e) => save({ netSpeedStatus: e.currentTarget.checked })}
                    />
                    <span class="slider"></span>
                  </label>
                </div>
              </div>
            </section>
          {/if}

          {#if matches("Notifikasi Windows Do Not Disturb DND Teams WhatsApp Email Notification")}
            <section class="setting-card">
              <div class="card-header">
                <div class="card-icon">🔔</div>
                <div class="card-info">
                  <h3>{tr("settings.mirrorTitle")}</h3>
                  <p>{tr("settings.mirrorDesc")}</p>
                </div>
                <label class="switch">
                  <input
                    type="checkbox"
                    checked={settings.mirrorNotifications}
                    onchange={(e) => save({ mirrorNotifications: e.currentTarget.checked })}
                  />
                  <span class="slider"></span>
                </label>
              </div>
              <div class="action-footer">
                <span class="tip-text">
                  {tr("settings.mirrorTip")}
                </span>
                <button class="btn-secondary" onclick={() => invoke("open_dnd_settings")}>
                  {tr("settings.openDnd")}
                </button>
              </div>
            </section>
          {/if}

          {#if matches("Keyboard Caps Lock Peek Bahasa Input Language Layout")}
            <section class="setting-card">
              <div class="card-header">
                <div class="card-icon">⌨️</div>
                <div class="card-info">
                  <h3>{tr("settings.capsLockTitle")}</h3>
                  <p>{tr("settings.capsLockDesc")}</p>
                </div>
                <label class="switch">
                  <input
                    type="checkbox"
                    checked={settings.capsLockPeek}
                    onchange={(e) => save({ capsLockPeek: e.currentTarget.checked })}
                  />
                  <span class="slider"></span>
                </label>
              </div>
            </section>
          {/if}

          {#if matches("Volume Overlay OSD Suara Windows Sembunyikan Hide Volume Audio")}
            <section class="setting-card">
              <div class="card-header">
                <div class="card-icon">🔊</div>
                <div class="card-info">
                  <h3>{tr("settings.hideVolumeOsdTitle")}</h3>
                  <p>{tr("settings.hideVolumeOsdDesc")}</p>
                </div>
                <label class="switch">
                  <input
                    type="checkbox"
                    checked={settings.hideVolumeOsd}
                    onchange={(e) => save({ hideVolumeOsd: e.currentTarget.checked })}
                  />
                  <span class="slider"></span>
                </label>
              </div>
            </section>
          {/if}
        </div>
      {/if}

      <!-- TAB 3: PRODUKTIVITAS / PRODUCTIVITY -->
      {#if activeTab === "productivity" || searchQuery}
        <div class="tab-panel" in:fade={{ duration: 180 }}>
          {#if matches("Halaman Menu Notch Aktif Pomodoro Air Todo Shelf Clipboard Shortcuts Pages Focus")}
            <section class="setting-card">
              <div class="card-header">
                <div class="card-icon">📑</div>
                <div class="card-info">
                  <h3>{tr("settings.activePagesTitle")}</h3>
                  <p>
                    {tr("settings.activePagesDesc")}
                    {#if enabledPagesCount <= 1}
                      <strong class="warn-text"> {tr("settings.activePagesMinWarn")}</strong>
                    {/if}
                  </p>
                </div>
              </div>

              <div class="pages-grid">
                {#each PAGE_ITEMS as [key, titleKey, descKey]}
                  <label class="page-checkbox-card" class:disabled={settings[key] !== false && enabledPagesCount <= 1}>
                    <input
                      type="checkbox"
                      checked={settings[key] !== false}
                      disabled={settings[key] !== false && enabledPagesCount <= 1}
                      onchange={(e) => save({ [key]: e.currentTarget.checked })}
                    />
                    <div class="page-checkbox-content">
                      <strong>{tr(titleKey)}</strong>
                      <small>{tr(descKey)}</small>
                    </div>
                  </label>
                {/each}
              </div>
            </section>
          {/if}

          {#if matches("Pomodoro Fokus Istirahat Timer Menit Focus Break Minutes")}
            <section class="setting-card">
              <div class="card-header">
                <div class="card-icon">⏳</div>
                <div class="card-info">
                  <h3>{tr("settings.pomodoroTitle")}</h3>
                  <p>{tr("settings.pomodoroDesc")}</p>
                </div>
              </div>
              <div class="inline-fields-row">
                <label class="input-inline">
                  <span>{tr("settings.focusSession")}</span>
                  <div class="number-unit">
                    <input
                      type="number"
                      min="1"
                      value={settings.focusMinutes}
                      onchange={(e) => save({ focusMinutes: Math.max(1, Number(e.currentTarget.value) || 25) })}
                    />
                    <em>{tr("settings.minutes")}</em>
                  </div>
                </label>
                <label class="input-inline">
                  <span>{tr("settings.shortBreak")}</span>
                  <div class="number-unit">
                    <input
                      type="number"
                      min="1"
                      value={settings.breakMinutes}
                      onchange={(e) => save({ breakMinutes: Math.max(1, Number(e.currentTarget.value) || 5) })}
                    />
                    <em>{tr("settings.minutes")}</em>
                  </div>
                </label>
              </div>
            </section>
          {/if}

          {#if matches("Minum Air Hidrasi Glass Target Pengingat ml Water Hydration Drink Reminder")}
            <section class="setting-card">
              <div class="card-header">
                <div class="card-icon">💧</div>
                <div class="card-info">
                  <h3>{tr("settings.waterTitle")}</h3>
                  <p>{tr("settings.waterDesc")}</p>
                </div>
                <label class="switch">
                  <input
                    type="checkbox"
                    checked={settings.waterEnabled}
                    onchange={(e) => save({ waterEnabled: e.currentTarget.checked })}
                  />
                  <span class="slider"></span>
                </label>
              </div>

              {#if settings.waterEnabled}
                <div class="water-grid" transition:slide={{ duration: 160 }}>
                  <label class="input-inline">
                    <span>{tr("settings.glassSize")}</span>
                    <div class="number-unit">
                      <input
                        type="number"
                        min="50"
                        step="50"
                        value={settings.glassMl}
                        onchange={(e) => save({ glassMl: Math.max(50, Number(e.currentTarget.value)) })}
                      />
                      <em>ml</em>
                    </div>
                  </label>
                  <label class="input-inline">
                    <span>{tr("settings.waterTarget")}</span>
                    <div class="number-unit">
                      <input
                        type="number"
                        min="500"
                        step="100"
                        value={settings.goalMl}
                        onchange={(e) => save({ goalMl: Math.max(500, Number(e.currentTarget.value)) })}
                      />
                      <em>ml</em>
                    </div>
                  </label>
                  <label class="input-inline">
                    <span>{tr("settings.waterFrom")}</span>
                    <div class="number-unit">
                      <input
                        type="number"
                        min="0"
                        max="23"
                        value={settings.waterFrom}
                        onchange={(e) => save({ waterFrom: Math.max(0, Math.min(23, Number(e.currentTarget.value))) })}
                      />
                      <em>:00</em>
                    </div>
                  </label>
                  <label class="input-inline">
                    <span>{tr("settings.waterTo")}</span>
                    <div class="number-unit">
                      <input
                        type="number"
                        min="0"
                        max="23"
                        value={settings.waterTo}
                        onchange={(e) => save({ waterTo: Math.max(0, Math.min(23, Number(e.currentTarget.value))) })}
                      />
                      <em>:00</em>
                    </div>
                  </label>
                  <label class="input-inline">
                    <span>{tr("settings.remindEvery")}</span>
                    <div class="number-unit">
                      <input
                        type="number"
                        min="0"
                        value={settings.remindEvery}
                        onchange={(e) => save({ remindEvery: Math.max(0, Number(e.currentTarget.value)) })}
                      />
                      <em>{tr("settings.minutesOff")}</em>
                    </div>
                  </label>
                </div>
              {/if}
            </section>
          {/if}

          {#if matches("Kalender Calendar Google Outlook ICS Rapat Meeting Schedule")}
            <section class="setting-card">
              <div class="card-header">
                <div class="card-icon">📅</div>
                <div class="card-info">
                  <h3>{tr("settings.calendarTitle")}</h3>
                  <p>{tr("settings.calendarDesc")}</p>
                </div>
              </div>
              <div class="input-block">
                <input
                  type="password"
                  class="text-input"
                  value={settings.calendarUrl}
                  placeholder="https://calendar.google.com/calendar/ical/.../basic.ics"
                  onchange={(e) => save({ calendarUrl: e.currentTarget.value.trim() })}
                />
                <small class="hint-muted">{tr("settings.calendarHint")}</small>
              </div>
            </section>
          {/if}

          {#if matches("Pintasan Shortcuts Cepat Launcher Tombol App File Quick Action")}
            <section class="setting-card">
              <div class="card-header">
                <div class="card-icon">⚡</div>
                <div class="card-info">
                  <h3>{tr("settings.shortcutsTitle")}</h3>
                  <p>{tr("settings.shortcutsDesc")}</p>
                </div>
              </div>

              <div class="shortcuts-list">
                {#each settings.shortcuts as s, i}
                  <div class="shortcut-item">
                    <input
                      class="shortcut-name"
                      value={s.name}
                      placeholder={tr("settings.shortcutNamePlaceholder")}
                      onchange={(e) => editShortcut(i, { name: e.currentTarget.value.trim() })}
                    />
                    <select
                      class="select-input select-action"
                      value={s.action}
                      onchange={(e) => editShortcut(i, { action: e.currentTarget.value })}
                    >
                      {#each ACTION_ITEMS as [val, lblKey]}
                        <option value={val}>{tr(lblKey)}</option>
                      {/each}
                    </select>
                    <button
                      class="btn-delete"
                      onclick={() => save({ shortcuts: settings!.shortcuts.filter((_, j) => j !== i) })}
                      title={tr("settings.deleteShortcut")}
                    >
                      ✕
                    </button>
                    {#if s.action === "open"}
                      <div class="shortcut-target-row">
                        <input
                          class="text-input target-input"
                          value={s.target}
                          placeholder={tr("settings.shortcutTargetPlaceholder")}
                          onchange={(e) => editShortcut(i, { target: e.currentTarget.value.trim() })}
                        />
                      </div>
                    {/if}
                  </div>
                {/each}
              </div>

              <div class="card-actions">
                {#if settings.shortcuts.length < 8}
                  <button
                    class="btn-secondary"
                    onclick={() => save({ shortcuts: [...settings!.shortcuts, { name: tr("shortcuts.title"), action: "open", target: "" }] })}
                  >
                    {tr("settings.addShortcut")}
                  </button>
                {/if}
                <button class="btn-link" onclick={() => save({ shortcuts: defaultShortcuts() })}>
                  {tr("settings.restoreDefaults")}
                </button>
              </div>
            </section>
          {/if}
        </div>
      {/if}

      <!-- TAB 4: CUACA & MEDIA / MEDIA -->
      {#if activeTab === "media" || searchQuery}
        <div class="tab-panel" in:fade={{ duration: 180 }}>
          {#if matches("Musik Music Lirik Lyrics LRCLIB Lagu Player Song")}
            <section class="setting-card">
              <div class="card-header">
                <div class="card-icon">🎵</div>
                <div class="card-info">
                  <h3>{tr("settings.lyricsTitle")}</h3>
                  <p>{tr("settings.lyricsDesc")}</p>
                </div>
                <label class="switch">
                  <input
                    type="checkbox"
                    checked={settings.smallLyrics}
                    onchange={(e) => save({ smallLyrics: e.currentTarget.checked })}
                  />
                  <span class="slider"></span>
                </label>
              </div>
            </section>
          {/if}

          {#if matches("Cuaca Weather Lokasi City Suhu Derajat Forecast Temperature")}
            <section class="setting-card">
              <div class="card-header">
                <div class="card-icon">⛅</div>
                <div class="card-info">
                  <h3>{tr("settings.weatherTitle")}</h3>
                  <p>{tr("settings.weatherDesc")}</p>
                </div>
              </div>

              <div class="weather-current-box">
                <span class="weather-label">{tr("settings.currentLocation")}</span>
                {#if settings.weatherCity}
                  <strong class="weather-city-name">{settings.weatherCity.name}</strong>
                  <button class="btn-link" onclick={() => save({ weatherCity: null })}>
                    {tr("settings.switchToAutoLocation")}
                  </button>
                {:else}
                  <strong class="weather-city-name">{tr("settings.windowsLocationAuto")}</strong>
                {/if}
              </div>

              <form class="city-search-form" onsubmit={searchCityHandler}>
                <input
                  type="text"
                  class="text-input"
                  bind:value={cityQuery}
                  placeholder={tr("settings.searchCityPlaceholder")}
                />
                <button type="submit" class="btn-primary" disabled={searchingCity}>
                  {searchingCity ? tr("settings.searchingCity") : tr("settings.searchCityBtn")}
                </button>
              </form>

              {#if cityResults.length > 0}
                <div class="city-results-list" transition:slide={{ duration: 160 }}>
                  {#each cityResults as place}
                    <button
                      class="city-result-item"
                      onclick={() => save({ weatherCity: { name: place.name, latitude: place.latitude, longitude: place.longitude } })}
                    >
                      <strong class="place-name">{place.name}</strong>
                      <span class="place-detail">{place.detail}</span>
                    </button>
                  {/each}
                </div>
              {/if}
            </section>
          {/if}
        </div>
      {/if}
    {/if}
  </main>

  <!-- Auto-save Toast Floating Notification -->
  {#if saved}
    <div class="saved-toast" transition:fade={{ duration: 150 }}>
      <svg class="check-icon" viewBox="0 0 20 20" fill="currentColor">
        <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
      </svg>
      <span>{tr("settings.savedToast")}</span>
    </div>
  {/if}
</div>

<style>
  :global(html),
  :global(body) {
    margin: 0;
    padding: 0;
    background: #0d1117 !important;
    background-color: #0d1117 !important;
    color: #e6edf3;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI Variable Text", "Segoe UI", sans-serif;
    user-select: none;
    -webkit-user-select: none;
    overflow-x: hidden !important;
    overflow-y: auto !important;
    height: 100%;
  }

  :global(body)::-webkit-scrollbar,
  :global(html)::-webkit-scrollbar {
    width: 6px;
  }
  :global(body)::-webkit-scrollbar-track,
  :global(html)::-webkit-scrollbar-track {
    background: #0d1117;
  }
  :global(body)::-webkit-scrollbar-thumb,
  :global(html)::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.18);
    border-radius: 999px;
  }
  :global(body)::-webkit-scrollbar-thumb:hover,
  :global(html)::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.35);
  }

  .settings-shell {
    max-width: 760px;
    margin: 0 auto;
    padding: 24px 20px 48px;
    min-height: 100vh;
    box-sizing: border-box;
    background: #0d1117;
  }

  /* Header */
  .app-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 20px;
    flex-wrap: wrap;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .logo-badge {
    width: 38px;
    height: 38px;
    border-radius: 10px;
    background: linear-gradient(135deg, #1f6feb 0%, #38bdf8 100%);
    display: flex;
    align-items: center;
    justify-content: center;
    color: #fff;
    box-shadow: 0 4px 12px rgba(31, 111, 235, 0.35);
  }

  .logo-badge svg {
    width: 22px;
    height: 22px;
  }

  .app-title {
    margin: 0;
    font-size: 20px;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: #f0f6fc;
  }

  .version-tag {
    font-size: 11px;
    color: #8b949e;
  }

  /* Search */
  .search-box {
    position: relative;
    flex: 1;
    min-width: 220px;
    max-width: 320px;
  }

  .search-icon {
    position: absolute;
    left: 10px;
    top: 50%;
    transform: translateY(-50%);
    width: 15px;
    height: 15px;
    color: #8b949e;
    pointer-events: none;
  }

  .search-box input {
    width: 100%;
    box-sizing: border-box;
    padding: 8px 30px 8px 32px;
    border-radius: 8px;
    border: 1px solid rgba(240, 246, 252, 0.12);
    background: rgba(22, 27, 34, 0.8);
    color: #f0f6fc;
    font-size: 12.5px;
    transition: all 0.2s ease;
  }

  .search-box input:focus {
    outline: none;
    border-color: #38bdf8;
    background: #161b22;
    box-shadow: 0 0 0 3px rgba(56, 189, 248, 0.15);
  }

  .clear-search {
    position: absolute;
    right: 8px;
    top: 50%;
    transform: translateY(-50%);
    background: none;
    border: none;
    color: #8b949e;
    cursor: pointer;
    font-size: 12px;
  }

  /* Tabs (Minimalist 4-Column Grid, Zero Scroll) */
  .tabs-nav {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 4px;
    background: rgba(22, 27, 34, 0.6);
    padding: 4px;
    border-radius: 10px;
    border: 1px solid rgba(240, 246, 252, 0.08);
    margin-bottom: 20px;
    overflow: hidden;
  }

  .tabs-nav::-webkit-scrollbar {
    display: none;
  }

  .tab-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 3px;
    padding: 7px 4px;
    border-radius: 7px;
    border: none;
    background: transparent;
    color: #8b949e;
    font-size: 11.5px;
    font-weight: 500;
    cursor: pointer;
    white-space: nowrap;
    transition: all 0.16s ease;
  }

  .tab-btn:hover {
    color: #f0f6fc;
    background: rgba(255, 255, 255, 0.04);
  }

  .tab-btn.active {
    background: rgba(56, 189, 248, 0.14);
    color: #38bdf8;
    font-weight: 600;
  }

  /* Cards */
  .tab-panel {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .setting-card {
    background: rgba(22, 27, 34, 0.7);
    border: 1px solid rgba(240, 246, 252, 0.09);
    border-radius: 12px;
    padding: 16px 18px;
    transition: transform 0.15s ease, box-shadow 0.15s ease, border-color 0.15s ease;
    backdrop-filter: blur(12px);
  }

  .setting-card:hover {
    border-color: rgba(240, 246, 252, 0.16);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.25);
  }

  .card-header {
    display: flex;
    align-items: flex-start;
    gap: 12px;
  }

  .card-icon {
    font-size: 20px;
    line-height: 1;
    padding-top: 2px;
  }

  .card-info {
    flex: 1;
  }

  .card-info h3 {
    margin: 0 0 3px;
    font-size: 14px;
    font-weight: 600;
    color: #f0f6fc;
  }

  .card-info p {
    margin: 0;
    font-size: 12.5px;
    color: #8b949e;
    line-height: 1.4;
  }

  /* Controls */
  .control-row {
    margin-top: 14px;
    display: flex;
    justify-content: flex-end;
  }

  .select-input {
    padding: 8px 12px;
    border-radius: 8px;
    border: 1px solid rgba(240, 246, 252, 0.12);
    background: #0d1117;
    color: #f0f6fc;
    font-size: 13px;
    cursor: pointer;
    outline: none;
    transition: border-color 0.18s ease;
  }

  .select-input:focus {
    border-color: #38bdf8;
  }

  /* Toggle Switch */
  .switch {
    position: relative;
    display: inline-block;
    width: 38px;
    height: 22px;
    flex-shrink: 0;
  }

  .switch input {
    opacity: 0;
    width: 0;
    height: 0;
  }

  .slider {
    position: absolute;
    cursor: pointer;
    inset: 0;
    background-color: rgba(240, 246, 252, 0.14);
    transition: 0.22s cubic-bezier(0.4, 0, 0.2, 1);
    border-radius: 999px;
    border: 1px solid rgba(240, 246, 252, 0.1);
  }

  .slider:before {
    position: absolute;
    content: "";
    height: 16px;
    width: 16px;
    left: 2px;
    bottom: 2px;
    background-color: #f0f6fc;
    transition: 0.22s cubic-bezier(0.4, 0, 0.2, 1);
    border-radius: 50%;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.35);
  }

  input:checked + .slider {
    background-color: #38bdf8;
    border-color: #38bdf8;
  }

  input:checked + .slider:before {
    transform: translateX(16px);
    background-color: #0d1117;
  }

  /* Accent status box */
  .accent-status-box {
    margin-top: 14px;
    padding: 10px 14px;
    border-radius: 8px;
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid rgba(240, 246, 252, 0.06);
  }

  .accent-preview {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
  }

  .color-dot {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    display: inline-block;
    box-shadow: 0 0 8px currentColor;
  }

  .color-hex {
    background: rgba(240, 246, 252, 0.08);
    padding: 2px 6px;
    border-radius: 4px;
    font-family: monospace;
    font-size: 11px;
    margin-left: 6px;
    color: #38bdf8;
  }

  .accent-hint {
    margin: 6px 0 0;
    font-size: 11.5px;
    color: #8b949e;
  }

  /* Toggle list */
  .toggle-list {
    margin-top: 12px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .toggle-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding-top: 8px;
    border-top: 1px solid rgba(240, 246, 252, 0.05);
  }

  .toggle-text strong {
    display: block;
    font-size: 13px;
    color: #f0f6fc;
  }

  .toggle-text small {
    display: block;
    font-size: 11.5px;
    color: #8b949e;
    margin-top: 1px;
  }

  /* Nested grid & controls */
  .nested-control,
  .nested-grid {
    margin-top: 12px;
    padding: 12px;
    border-radius: 8px;
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid rgba(240, 246, 252, 0.05);
  }

  .nested-grid {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .input-inline {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    font-size: 12.5px;
    color: #8b949e;
  }

  .number-unit {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .number-unit input {
    width: 68px;
    padding: 6px 8px;
    border-radius: 6px;
    border: 1px solid rgba(240, 246, 252, 0.12);
    background: #0d1117;
    color: #f0f6fc;
    font-size: 13px;
    text-align: right;
  }

  .number-unit em {
    font-style: normal;
    font-size: 12px;
    color: #8b949e;
  }

  /* Pages multi-checkbox grid */
  .pages-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(210px, 1fr));
    gap: 10px;
    margin-top: 14px;
  }

  .page-checkbox-card {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 10px 12px;
    border-radius: 8px;
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid rgba(240, 246, 252, 0.06);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .page-checkbox-card:hover {
    background: rgba(0, 0, 0, 0.35);
    border-color: rgba(56, 189, 248, 0.3);
  }

  .page-checkbox-card.disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .page-checkbox-card input[type="checkbox"] {
    margin-top: 3px;
    accent-color: #38bdf8;
  }

  .page-checkbox-content strong {
    display: block;
    font-size: 13px;
    color: #f0f6fc;
  }

  .page-checkbox-content small {
    display: block;
    font-size: 11px;
    color: #8b949e;
    margin-top: 2px;
    line-height: 1.3;
  }

  .warn-text {
    color: #fb923c;
    font-size: 12px;
  }

  /* Productivity grids */
  .inline-fields-row {
    display: flex;
    gap: 20px;
    margin-top: 14px;
  }

  .water-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(190px, 1fr));
    gap: 12px;
    margin-top: 14px;
    padding: 12px;
    border-radius: 8px;
    background: rgba(0, 0, 0, 0.2);
  }

  .input-block {
    margin-top: 12px;
  }

  .text-input {
    width: 100%;
    box-sizing: border-box;
    padding: 8px 12px;
    border-radius: 8px;
    border: 1px solid rgba(240, 246, 252, 0.12);
    background: #0d1117;
    color: #f0f6fc;
    font-size: 13px;
  }

  .text-input:focus {
    outline: none;
    border-color: #38bdf8;
  }

  .hint-muted {
    display: block;
    margin-top: 5px;
    font-size: 11.5px;
    color: #8b949e;
  }

  /* Shortcuts */
  .shortcuts-list {
    margin-top: 14px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .shortcut-item {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
    background: rgba(0, 0, 0, 0.2);
    padding: 8px 10px;
    border-radius: 8px;
    border: 1px solid rgba(240, 246, 252, 0.05);
  }

  .shortcut-name {
    width: 90px;
    padding: 6px 8px;
    border-radius: 6px;
    border: 1px solid rgba(240, 246, 252, 0.12);
    background: #0d1117;
    color: #f0f6fc;
    font-size: 12.5px;
  }

  .select-action {
    flex: 1;
    min-width: 130px;
  }

  .btn-delete {
    background: none;
    border: none;
    color: #f87171;
    cursor: pointer;
    padding: 6px 8px;
    font-size: 13px;
    border-radius: 6px;
  }

  .btn-delete:hover {
    background: rgba(248, 113, 113, 0.15);
  }

  .shortcut-target-row {
    width: 100%;
    margin-top: 4px;
  }

  .target-input {
    font-size: 12px;
    padding: 6px 10px;
  }

  .card-actions {
    margin-top: 14px;
    display: flex;
    align-items: center;
    gap: 12px;
  }

  /* Weather */
  .weather-current-box {
    margin-top: 10px;
    font-size: 13px;
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .weather-label {
    color: #8b949e;
  }

  .weather-city-name {
    color: #38bdf8;
  }

  .city-search-form {
    margin-top: 12px;
    display: flex;
    gap: 8px;
  }

  .city-results-list {
    margin-top: 10px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    max-height: 180px;
    overflow-y: auto;
    border: 1px solid rgba(240, 246, 252, 0.08);
    border-radius: 8px;
    padding: 4px;
  }

  .city-result-item {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    padding: 8px 10px;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: #f0f6fc;
    cursor: pointer;
    text-align: left;
    transition: background 0.15s ease;
  }

  .city-result-item:hover {
    background: rgba(56, 189, 248, 0.12);
  }

  .place-name {
    font-size: 13px;
  }

  .place-detail {
    font-size: 11.5px;
    color: #8b949e;
  }

  /* Buttons */
  .btn-primary {
    padding: 8px 16px;
    border-radius: 8px;
    border: none;
    background: #1f6feb;
    color: #fff;
    font-weight: 500;
    font-size: 13px;
    cursor: pointer;
    transition: background 0.18s ease;
  }

  .btn-primary:hover:not(:disabled) {
    background: #388bfd;
  }

  .btn-secondary {
    padding: 8px 14px;
    border-radius: 8px;
    border: 1px solid rgba(240, 246, 252, 0.14);
    background: rgba(240, 246, 252, 0.06);
    color: #f0f6fc;
    font-size: 12.5px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.18s ease;
  }

  .btn-secondary:hover {
    background: rgba(240, 246, 252, 0.12);
    border-color: rgba(240, 246, 252, 0.25);
  }

  .btn-link {
    background: none;
    border: none;
    color: #38bdf8;
    font-size: 12px;
    cursor: pointer;
    padding: 0;
    text-decoration: underline;
    text-underline-offset: 3px;
  }

  .btn-link:hover {
    color: #7dd3fc;
  }

  .action-footer {
    margin-top: 14px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    flex-wrap: wrap;
    padding-top: 10px;
    border-top: 1px solid rgba(240, 246, 252, 0.05);
  }

  .tip-text {
    font-size: 12px;
    color: #8b949e;
  }

  /* Saved Toast */
  .saved-toast {
    position: fixed;
    bottom: 24px;
    right: 24px;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 16px;
    border-radius: 999px;
    background: rgba(35, 134, 54, 0.95);
    color: #fff;
    font-size: 12.5px;
    font-weight: 500;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
    backdrop-filter: blur(8px);
    z-index: 1000;
  }

  .check-icon {
    width: 16px;
    height: 16px;
  }

  /* Loading */
  .loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 60px 0;
    color: #8b949e;
  }

  .spinner {
    width: 28px;
    height: 28px;
    border: 3px solid rgba(240, 246, 252, 0.12);
    border-top-color: #38bdf8;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
    margin-bottom: 12px;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
