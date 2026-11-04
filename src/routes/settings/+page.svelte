<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { fade, slide } from "svelte/transition";
  import { searchCity, type Place } from "$lib/weather";
  import { t, setLocale, getLocale } from "$lib/i18n";

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
    minimalNotch: boolean;
    mirrorNotifications: boolean;
    calendarUrl: string;
    smallLyrics: boolean;
    shortcuts: Shortcut[];
    capsLockPeek: boolean;
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

  const ACTIONS: [string, string][] = [
    ["lock", "Kunci layar"],
    ["sleep", "Sleep"],
    ["screenshot", "Screenshot"],
    ["downloads", "Buka Downloads"],
    ["camera", "Cek kamera"],
    ["open", "Buka aplikasi / file / folder / link / skrip"],
  ];

  const DEFAULT_SHORTCUTS: Shortcut[] = [
    { name: "Kunci", action: "lock", target: "" },
    { name: "Sleep", action: "sleep", target: "" },
    { name: "Screenshot", action: "screenshot", target: "" },
    { name: "Downloads", action: "downloads", target: "" },
    { name: "Kamera", action: "camera", target: "" },
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
    ["musicEnabled", "Pemutar Musik", "Jam, pemutar media, lirik lagu berjalan"],
    ["pomodoroEnabled", "Fokus Pomodoro", "Timer sesi fokus dan istirahat"],
    ["waterEnabled", "Minum Air", "Pencatatan hidrasi harian & pengingat berkala"],
    ["todoEnabled", "Catatan To-Do", "Daftar to-do harian sederhana di notch"],
    ["shelfEnabled", "Laci File (Shelf)", "Tempat menaruh file/folder sementara"],
    ["clipboardEnabled", "Papan Klip", "Riwayat salinan teks dan tangkapan layar"],
    ["shortcutsEnabled", "Pintasan Cepat", "Tombol aksi cepat dan peluncur aplikasi"],
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
        <h1 class="app-title">Notchy Settings</h1>
        <span class="version-tag">v0.2.5 · Fluent Experience</span>
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
        placeholder={t("settings.searchPlaceholder")}
      />
      {#if searchQuery}
        <button class="clear-search" onclick={() => (searchQuery = "")}>✕</button>
      {/if}
    </div>
  </header>

  <!-- Tab Bar (Minimalist 4-Column Grid, No Overflow) -->
  <div class="tabs-nav" role="tablist" aria-label={t("settings.title")}>
    <button
      class="tab-btn"
      class:active={activeTab === "display"}
      onclick={() => (activeTab = "display")}
      role="tab"
      aria-selected={activeTab === "display"}
    >
      <span class="tab-icon">🖥️</span>
      <span class="tab-text">{t("tabs.display")}</span>
    </button>
    <button
      class="tab-btn"
      class:active={activeTab === "system"}
      onclick={() => (activeTab = "system")}
      role="tab"
      aria-selected={activeTab === "system"}
    >
      <span class="tab-icon">⚙️</span>
      <span class="tab-text">{t("tabs.system")}</span>
    </button>
    <button
      class="tab-btn"
      class:active={activeTab === "productivity"}
      onclick={() => (activeTab = "productivity")}
      role="tab"
      aria-selected={activeTab === "productivity"}
    >
      <span class="tab-icon">⏱️</span>
      <span class="tab-text">{t("tabs.focus")}</span>
    </button>
    <button
      class="tab-btn"
      class:active={activeTab === "media"}
      onclick={() => (activeTab = "media")}
      role="tab"
      aria-selected={activeTab === "media"}
    >
      <span class="tab-icon">🎵</span>
      <span class="tab-text">{t("tabs.media")}</span>
    </button>
  </div>

  <!-- Main Content Area -->
  <main class="settings-content">
    {#if !settings}
      <div class="loading-state">
        <div class="spinner"></div>
        <p>Memuat konfigurasi Notchy...</p>
      </div>
    {:else}
      <!-- TAB 1: TAMPILAN & LAYAR -->
      {#if activeTab === "display" || searchQuery}
        <div class="tab-panel" in:fade={{ duration: 180 }}>
          {#if matches("Language Bahasa Inggris Indonesia English")}
            <section class="setting-card">
              <div class="card-header">
                <div class="card-icon">🌐</div>
                <div class="card-info">
                  <h3>{t("settings.language")}</h3>
                  <p>{t("settings.languageDesc")}</p>
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
                  <option value="en">{t("settings.langEn")}</option>
                  <option value="id">{t("settings.langId")}</option>
                </select>
              </div>
            </section>
          {/if}

          {#if matches("Layar Multi-Monitor Penempatan Posisi Display Primary Secondary Kursor")}
            <section class="setting-card">
              <div class="card-header">
                <div class="card-icon">🖥️</div>
                <div class="card-info">
                  <h3>Penempatan Multi-Monitor</h3>
                  <p>Tentukan di layar mana Notchy ditempatkan secara otomatis.</p>
                </div>
              </div>
              <div class="control-row">
                <select
                  class="select-input"
                  value={settings.monitorPlacement ?? "primary"}
                  onchange={(e) => save({ monitorPlacement: e.currentTarget.value })}
                >
                  <option value="primary">Layar Utama (Primary Monitor)</option>
                  <option value="secondary">Layar Sekunder (Secondary Monitor)</option>
                  <option value="follow-cursor">Ikuti Kursor / Mouse Aktif</option>
                </select>
              </div>
            </section>
          {/if}

          {#if matches("Animasi Spring Physics Pegas Kecepatan Transisi Organik Bouncy")}
            <section class="setting-card">
              <div class="card-header">
                <div class="card-icon">🌀</div>
                <div class="card-info">
                  <h3>Animasi Dynamic Island (Spring Physics)</h3>
                  <p>Kurva fisika pegas adaptif saat Dynamic Island melebar dan merespons interaksi.</p>
                </div>
              </div>
              <div class="control-row">
                <select
                  class="select-input"
                  value={settings.springPhysics ?? "spring"}
                  onchange={(e) => save({ springPhysics: e.currentTarget.value })}
                >
                  <option value="spring">Spring Organik (Bouncy iOS / macOS Style)</option>
                  <option value="snappy">Snappy (Cepat & Responsif)</option>
                  <option value="smooth">Smooth (Halus & Elegan)</option>
                  <option value="off">Mati (Tanpa Efek Pegas)</option>
                </select>
              </div>
            </section>
          {/if}

          {#if matches("Sinkronisasi Warna Aksen Windows Dynamic Accent DWM Glow Border")}
            <section class="setting-card">
              <div class="card-header">
                <div class="card-icon">✨</div>
                <div class="card-info">
                  <h3>Sinkronisasi Warna Aksen Windows (Dynamic Accent)</h3>
                  <p>Efek highlight border glow elegan saat ada notifikasi aktif atau peek informasi.</p>
                </div>
                <label class="switch" title="Toggle On/Off Windows Accent Sync">
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
                    Mode saat ini:
                    <strong>{settings.windowsAccentGlow ? "Sinkronisasi Windows Accent DWM" : "Default Notchy (Netral)"}</strong>
                    {#if settings.windowsAccentGlow}
                      <code class="color-hex">{accentColor}</code>
                    {/if}
                  </span>
                </div>
                <p class="accent-hint">
                  {settings.windowsAccentGlow
                    ? "Warna glow border otomatis mengikuti tema aksen yang dipilih di Personalization Windows."
                    : "Warna glow border menggunakan tema monokrom ambient default Notchy."}
                </p>
              </div>
            </section>
          {/if}

          {#if matches("Mascot Mata Ekspresif Minimal Notch Bezel Tipis")}
            <section class="setting-card">
              <div class="card-header">
                <div class="card-icon">👀</div>
                <div class="card-info">
                  <h3>Ekspresi Maskot & Gaya Notch</h3>
                  <p>Karakter mata interaktif saat idle dan opsi tampilan tipis.</p>
                </div>
              </div>

              <div class="toggle-list">
                <div class="toggle-item">
                  <div class="toggle-text">
                    <strong>Mata Maskot Aktif</strong>
                    <small>Mata ekspresif keluar dan melihat ke arah kursor saat laptop idle.</small>
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

                <div class="toggle-item">
                  <div class="toggle-text">
                    <strong>Notch Minimal</strong>
                    <small>Bentuk garis tipis 5px saat idle (sangat cocok untuk laptop ber-bezel tebal).</small>
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

      <!-- TAB 2: SISTEM & INDIKATOR -->
      {#if activeTab === "system" || searchQuery}
        <div class="tab-panel" in:fade={{ duration: 180 }}>
          {#if matches("Bluetooth Baterai Headset TWS Mouse Keyboard Perangkat Device")}
            <section class="setting-card">
              <div class="card-header">
                <div class="card-icon">🎧</div>
                <div class="card-info">
                  <h3>Status Baterai Bluetooth (TWS / Mouse / Keyboard)</h3>
                  <p>Dynamic Island otomatis melebar saat perangkat Bluetooth tersambung atau baterai menipis.</p>
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
                    <span>Ambang batas baterai rendah (Low Battery Alert):</span>
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

          {#if matches("Kecepatan Jaringan Network Speed Download Upload Internet")}
            <section class="setting-card">
              <div class="card-header">
                <div class="card-icon">🚀</div>
                <div class="card-info">
                  <h3>Pemantau Kecepatan Jaringan</h3>
                  <p>Pantau traffic unduh (download) dan unggah (upload) internet secara real-time.</p>
                </div>
              </div>

              <div class="toggle-list">
                <div class="toggle-item">
                  <div class="toggle-text">
                    <strong>Dynamic Island (Notch Tertutup)</strong>
                    <small>Otomatis melebar saat ada unduhan/unggahan berkecepatan tinggi.</small>
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
                      <span>Batas unduh (download) minimum:</span>
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
                      <span>Batas unggah (upload) minimum:</span>
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
                    <strong>Baris Status (Notch Terbuka)</strong>
                    <small>Tampil di baris atas notch terbuka bersanding dengan baterai dan jam.</small>
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

          {#if matches("Notifikasi Windows Do Not Disturb DND Teams WhatsApp Email")}
            <section class="setting-card">
              <div class="card-header">
                <div class="card-icon">🔔</div>
                <div class="card-info">
                  <h3>Notifikasi Windows</h3>
                  <p>Tampilkan pop-up notifikasi Windows (WhatsApp, Slack, Teams, email) di notch.</p>
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
                  Tips: Nyalakan <strong>Do not disturb</strong> di Windows agar notifikasi tidak muncul ganda.
                </span>
                <button class="btn-secondary" onclick={() => invoke("open_dnd_settings")}>
                  Buka Pengaturan Windows DND
                </button>
              </div>
            </section>
          {/if}

          {#if matches("Keyboard Caps Lock Peek Bahasa Input")}
            <section class="setting-card">
              <div class="card-header">
                <div class="card-icon">⌨️</div>
                <div class="card-info">
                  <h3>Keyboard & Caps Lock Peek</h3>
                  <p>Tampilkan indikator Caps Lock dan pergantian layout bahasa keyboard di notch.</p>
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
        </div>
      {/if}

      <!-- TAB 3: PRODUKTIVITAS -->
      {#if activeTab === "productivity" || searchQuery}
        <div class="tab-panel" in:fade={{ duration: 180 }}>
          {#if matches("Halaman Menu Notch Aktif Pomodoro Air Todo Shelf Clipboard Shortcuts")}
            <section class="setting-card">
              <div class="card-header">
                <div class="card-icon">📑</div>
                <div class="card-info">
                  <h3>Daftar Halaman di Notch Terbuka</h3>
                  <p>
                    Pilih menu yang dapat diakses saat notch dibuka.
                    {#if enabledPagesCount <= 1}
                      <strong class="warn-text"> (Minimal 1 halaman harus aktif)</strong>
                    {/if}
                  </p>
                </div>
              </div>

              <div class="pages-grid">
                {#each PAGE_ITEMS as [key, label, desc]}
                  <label class="page-checkbox-card" class:disabled={settings[key] !== false && enabledPagesCount <= 1}>
                    <input
                      type="checkbox"
                      checked={settings[key] !== false}
                      disabled={settings[key] !== false && enabledPagesCount <= 1}
                      onchange={(e) => save({ [key]: e.currentTarget.checked })}
                    />
                    <div class="page-checkbox-content">
                      <strong>{label}</strong>
                      <small>{desc}</small>
                    </div>
                  </label>
                {/each}
              </div>
            </section>
          {/if}

          {#if matches("Pomodoro Fokus Istirahat Timer Menit")}
            <section class="setting-card">
              <div class="card-header">
                <div class="card-icon">⏳</div>
                <div class="card-info">
                  <h3>Timer Fokus Pomodoro</h3>
                  <p>Sesuaikan durasi siklus kerja produktif dan istirahat.</p>
                </div>
              </div>
              <div class="inline-fields-row">
                <label class="input-inline">
                  <span>Sesi Fokus:</span>
                  <div class="number-unit">
                    <input
                      type="number"
                      min="1"
                      value={settings.focusMinutes}
                      onchange={(e) => save({ focusMinutes: Math.max(1, Number(e.currentTarget.value) || 25) })}
                    />
                    <em>menit</em>
                  </div>
                </label>
                <label class="input-inline">
                  <span>Sesi Istirahat:</span>
                  <div class="number-unit">
                    <input
                      type="number"
                      min="1"
                      value={settings.breakMinutes}
                      onchange={(e) => save({ breakMinutes: Math.max(1, Number(e.currentTarget.value) || 5) })}
                    />
                    <em>menit</em>
                  </div>
                </label>
              </div>
            </section>
          {/if}

          {#if matches("Minum Air Hidrasi Glass Target Pengingat ml")}
            <section class="setting-card">
              <div class="card-header">
                <div class="card-icon">💧</div>
                <div class="card-info">
                  <h3>Pengingat Minum Air</h3>
                  <p>Pencatatan hidrasi harian dengan pengingat ramah saat fokus bekerja.</p>
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
                    <span>Sekali minum:</span>
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
                    <span>Target harian:</span>
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
                    <span>Mulai jam:</span>
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
                    <span>Sampai jam:</span>
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
                    <span>Ingatkan tiap:</span>
                    <div class="number-unit">
                      <input
                        type="number"
                        min="0"
                        value={settings.remindEvery}
                        onchange={(e) => save({ remindEvery: Math.max(0, Number(e.currentTarget.value)) })}
                      />
                      <em>menit (0 = mati)</em>
                    </div>
                  </label>
                </div>
              {/if}
            </section>
          {/if}

          {#if matches("Kalender Calendar Google Outlook ICS Rapat Meeting")}
            <section class="setting-card">
              <div class="card-header">
                <div class="card-icon">📅</div>
                <div class="card-info">
                  <h3>Sinkronisasi Kalender (ICS)</h3>
                  <p>Tempel URL rahasia format iCal (.ics) dari Google Calendar atau Outlook.</p>
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
                <small class="hint-muted">Alamat ICS hanya disimpan lokal di komputer ini.</small>
              </div>
            </section>
          {/if}

          {#if matches("Pintasan Shortcuts Cepat Launcher Tombol App File")}
            <section class="setting-card">
              <div class="card-header">
                <div class="card-icon">⚡</div>
                <div class="card-info">
                  <h3>Pintasan Cepat (Maksimal 8)</h3>
                  <p>Peluncur cepat aplikasi (.exe), folder, link web, atau file skrip otomatisasi.</p>
                </div>
              </div>

              <div class="shortcuts-list">
                {#each settings.shortcuts as s, i}
                  <div class="shortcut-item">
                    <input
                      class="shortcut-name"
                      value={s.name}
                      placeholder="Nama tombol"
                      onchange={(e) => editShortcut(i, { name: e.currentTarget.value.trim() })}
                    />
                    <select
                      class="select-input select-action"
                      value={s.action}
                      onchange={(e) => editShortcut(i, { action: e.currentTarget.value })}
                    >
                      {#each ACTIONS as [val, lbl]}
                        <option value={val}>{lbl}</option>
                      {/each}
                    </select>
                    <button
                      class="btn-delete"
                      onclick={() => save({ shortcuts: settings!.shortcuts.filter((_, j) => j !== i) })}
                      title="Hapus pintasan"
                    >
                      ✕
                    </button>
                    {#if s.action === "open"}
                      <div class="shortcut-target-row">
                        <input
                          class="text-input target-input"
                          value={s.target}
                          placeholder="C:\Tools\script.bat atau https://github.com..."
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
                    onclick={() => save({ shortcuts: [...settings!.shortcuts, { name: "Baru", action: "open", target: "" }] })}
                  >
                    + Tambah Pintasan
                  </button>
                {/if}
                <button class="btn-link" onclick={() => save({ shortcuts: DEFAULT_SHORTCUTS })}>
                  Kembalikan ke Default
                </button>
              </div>
            </section>
          {/if}
        </div>
      {/if}

      <!-- TAB 4: CUACA & MEDIA -->
      {#if activeTab === "media" || searchQuery}
        <div class="tab-panel" in:fade={{ duration: 180 }}>
          {#if matches("Musik Music Lirik Lyrics LRCLIB Lagu Player")}
            <section class="setting-card">
              <div class="card-header">
                <div class="card-icon">🎵</div>
                <div class="card-info">
                  <h3>Pemutar Musik & Lirik Berjalan</h3>
                  <p>Tampilkan baris lirik real-time dari LRCLIB pada Dynamic Island saat lagu berputar.</p>
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

          {#if matches("Cuaca Weather Lokasi City Suhu Derajat")}
            <section class="setting-card">
              <div class="card-header">
                <div class="card-icon">⛅</div>
                <div class="card-info">
                  <h3>Lokasi Cuaca</h3>
                  <p>Sumber lokasi untuk prakiraan cuaca pada notch dan layar jam.</p>
                </div>
              </div>

              <div class="weather-current-box">
                <span class="weather-label">Lokasi Saat Ini:</span>
                {#if settings.weatherCity}
                  <strong class="weather-city-name">{settings.weatherCity.name}</strong>
                  <button class="btn-link" onclick={() => save({ weatherCity: null })}>
                    (Ganti ke Lokasi Windows Otomatis)
                  </button>
                {:else}
                  <strong class="weather-city-name">Lokasi Windows (Otomatis)</strong>
                {/if}
              </div>

              <form class="city-search-form" onsubmit={searchCityHandler}>
                <input
                  type="text"
                  class="text-input"
                  bind:value={cityQuery}
                  placeholder="Ketik nama kota, contoh: Jakarta, Surabaya, Tokyo..."
                />
                <button type="submit" class="btn-primary" disabled={searchingCity}>
                  {searchingCity ? "Mencari..." : "Cari Kota"}
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
      <span>Pengaturan tersimpan otomatis</span>
    </div>
  {/if}
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    background: #0d1117;
    color: #e6edf3;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI Variable Text", "Segoe UI", sans-serif;
    user-select: none;
    -webkit-user-select: none;
    overflow-x: hidden;
  }

  .settings-shell {
    max-width: 760px;
    margin: 0 auto;
    padding: 24px 20px 48px;
    min-height: 100vh;
    box-sizing: border-box;
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
