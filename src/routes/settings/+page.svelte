<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { searchCity, type Place } from "$lib/weather";

  type City = { name: string; latitude: number; longitude: number };
  type Settings = {
    weatherCity: City | null;
    focusMinutes: number;
    breakMinutes: number;
    glassMl: number;
    goalMl: number;
    waterFrom: number;
    waterTo: number;
    remindEvery: number;
    mirrorNotifications: boolean;
    calendarUrl: string;
    smallLyrics: boolean;
    shortcuts: Shortcut[];
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
  function editShortcut(i: number, patch: Partial<Shortcut>) {
    if (!settings) return;
    save({ shortcuts: settings.shortcuts.map((s, j) => (j === i ? { ...s, ...patch } : s)) });
  }

  let settings = $state<Settings | null>(null);
  let query = $state("");
  let results = $state<Place[]>([]);
  let searching = $state(false);
  let saved = $state(false);

  onMount(async () => {
    settings = await invoke<Settings>("get_settings");
  });

  async function search(e: SubmitEvent) {
    e.preventDefault();
    if (!query.trim()) return;
    searching = true;
    results = await searchCity(query.trim()).catch(() => []);
    searching = false;
  }

  async function save(patch: Partial<Settings> = {}) {
    if (!settings) return;
    settings = { ...settings, ...patch };
    await invoke("save_settings", { settings });
    results = [];
    query = "";
    saved = true;
    setTimeout(() => (saved = false), 1500);
  }

  // Number fields save when they lose focus.
  const numbers: [keyof Settings, string, string][] = [
    ["focusMinutes", "Fokus", "menit"],
    ["breakMinutes", "Istirahat", "menit"],
  ];
  const water: [keyof Settings, string, string][] = [
    ["glassMl", "Sekali minum", "ml"],
    ["goalMl", "Target harian", "ml"],
    ["waterFrom", "Ingatkan mulai jam", ""],
    ["waterTo", "Sampai jam", ""],
    ["remindEvery", "Ingatkan tiap", "menit (0 = mati)"],
  ];
</script>

{#snippet field(key: keyof Settings, label: string, unit: string)}
  {#if settings}
    <label class="field">
      <span>{label}</span>
      <input
        type="number"
        min="0"
        value={settings[key] as number}
        onchange={(e) => save({ [key]: Math.max(0, Math.round(Number(e.currentTarget.value))) })}
      />
      <em>{unit}</em>
    </label>
  {/if}
{/snippet}

<main>
  <h1>Notchy</h1>
  {#if saved}<p class="saved">Tersimpan</p>{/if}

  <section>
    <h2>Notifikasi</h2>
    {#if settings}
      <label class="field check">
        <input
          type="checkbox"
          checked={settings.mirrorNotifications}
          onchange={(e) => save({ mirrorNotifications: e.currentTarget.checked })}
        />
        <span>Tampilkan notifikasi Windows di notch (WhatsApp, Teams, email...)</span>
      </label>
      <p class="current">
        Supaya notifikasi hanya muncul di notch (tidak double), nyalakan <strong>Do not disturb</strong> di
        Windows. Notifikasi tetap masuk ke Notification Center, dan Notchy tetap menampilkannya.
        Klik notifikasi di notch untuk membuka aplikasinya.
      </p>
      <button class="secondary" onclick={() => invoke("open_dnd_settings")}>Buka pengaturan Do not disturb</button>
    {/if}
  </section>

  <section>
    <h2>Musik</h2>
    {#if settings}
      <label class="field check">
        <input
          type="checkbox"
          checked={settings.smallLyrics}
          onchange={(e) => save({ smallLyrics: e.currentTarget.checked })}
        />
        <span>Tampilkan baris lirik di notch kecil (judul lagu dikirim ke LRCLIB untuk tiap lagu)</span>
      </label>
    {/if}
  </section>

  <section>
    <h2>Kalender</h2>
    <p class="current">
      Tempel "secret address in iCal format" dari Google Calendar, atau link ICS dari Outlook.
      Link ini hanya disimpan di laptop ini.
    </p>
    {#if settings}
      <input
        class="wide"
        type="password"
        value={settings.calendarUrl}
        placeholder="https://calendar.google.com/calendar/ical/…/basic.ics"
        onchange={(e) => save({ calendarUrl: e.currentTarget.value.trim() })}
      />
    {/if}
  </section>

  <section>
    <h2>Pintasan</h2>
    <p class="current">
      Tombol di halaman Pintasan pada notch (maksimal 8). "Buka" menerima aplikasi (.exe), file, folder, link
      https://, atau skrip (.bat, .cmd, .ps1) milikmu sendiri.
    </p>
    {#if settings}
      {#each settings.shortcuts as s, i}
        <div class="shortcut">
          <input class="name" value={s.name} placeholder="Nama" onchange={(e) => editShortcut(i, { name: e.currentTarget.value.trim() })} />
          <select value={s.action} onchange={(e) => editShortcut(i, { action: e.currentTarget.value })}>
            {#each ACTIONS as [value, label]}<option {value}>{label}</option>{/each}
          </select>
          <button class="link" onclick={() => save({ shortcuts: settings!.shortcuts.filter((_, j) => j !== i) })} aria-label="Hapus pintasan">Hapus</button>
          {#if s.action === "open"}
            <input
              class="wide target"
              value={s.target}
              placeholder="C:\Tools\backup.bat atau https://…"
              onchange={(e) => editShortcut(i, { target: e.currentTarget.value.trim() })}
            />
          {/if}
        </div>
      {/each}
      <div class="row-buttons">
        {#if settings.shortcuts.length < 8}
          <button
            class="secondary"
            onclick={() => save({ shortcuts: [...settings!.shortcuts, { name: "Baru", action: "open", target: "" }] })}
          >
            Tambah pintasan
          </button>
        {/if}
        <button class="link" onclick={() => save({ shortcuts: DEFAULT_SHORTCUTS })}>Kembalikan default</button>
      </div>
    {/if}
  </section>

  <section>
    <h2>Pomodoro</h2>
    {#each numbers as [key, label, unit]}{@render field(key, label, unit)}{/each}
  </section>

  <section>
    <h2>Minum air</h2>
    {#each water as [key, label, unit]}{@render field(key, label, unit)}{/each}
  </section>

  <section>
    <h2>Lokasi cuaca</h2>
    <p class="current">
      {#if settings?.weatherCity}
        Sekarang: <strong>{settings.weatherCity.name}</strong>
        <button class="link" onclick={() => save({ weatherCity: null })}>Pakai lokasi Windows</button>
      {:else}
        Sekarang: <strong>lokasi Windows</strong> (kalau diizinkan)
      {/if}
    </p>

    <form onsubmit={search}>
      <input bind:value={query} placeholder="Cari kota, misalnya Bandung" />
      <button type="submit" disabled={searching}>{searching ? "Mencari…" : "Cari"}</button>
    </form>

    {#if results.length}
      <ul>
        {#each results as place}
          <li>
            <button
              onclick={() => save({ weatherCity: { name: place.name, latitude: place.latitude, longitude: place.longitude } })}
            >
              <strong>{place.name}</strong>
              <span>{place.detail}</span>
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </section>
</main>

<style>
  :global(body) {
    margin: 0;
    background: #0b0d14;
    color: #eceff7;
    font-family: "Segoe UI Variable Text", "Segoe UI", sans-serif;
  }
  main {
    padding: 22px 24px;
  }
  h1 {
    margin: 0 0 18px;
    font-size: 20px;
  }
  h2 {
    margin: 0 0 6px;
    font-size: 14px;
  }
  .current {
    margin: 0 0 12px;
    font-size: 13px;
    color: #8c94a8;
  }
  .current strong {
    color: #eceff7;
  }
  form {
    display: flex;
    gap: 8px;
  }
  input {
    flex: 1;
    padding: 8px 10px;
    border: 1px solid rgba(236, 239, 247, 0.15);
    border-radius: 8px;
    background: #141826;
    color: inherit;
    font: inherit;
    font-size: 13px;
  }
  input:focus {
    outline: 2px solid #2d5bff;
    outline-offset: 1px;
  }
  form button {
    padding: 8px 14px;
    border: 0;
    border-radius: 8px;
    background: #2d5bff;
    color: #fff;
    font: inherit;
    font-size: 13px;
    cursor: pointer;
  }
  ul {
    list-style: none;
    margin: 10px 0 0;
    padding: 0;
  }
  li button {
    width: 100%;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 2px;
    padding: 8px 10px;
    border: 0;
    border-radius: 8px;
    background: none;
    color: inherit;
    font: inherit;
    font-size: 13px;
    text-align: left;
    cursor: pointer;
  }
  li button:hover {
    background: #141826;
  }
  li span {
    font-size: 12px;
    color: #8c94a8;
  }
  .link {
    margin-left: 6px;
    padding: 0;
    border: 0;
    background: none;
    color: #19b8ff;
    font: inherit;
    cursor: pointer;
  }
  .saved {
    position: fixed;
    top: 22px;
    right: 24px;
    margin: 0;
    font-size: 12.5px;
    color: #34d399;
  }
  section {
    margin-bottom: 22px;
  }
  .field {
    display: flex;
    align-items: center;
    gap: 10px;
    margin: 6px 0;
    font-size: 13px;
  }
  .field span {
    width: 150px;
    color: #c4c9d6;
  }
  .field input {
    flex: none;
    width: 80px;
  }
  .secondary {
    padding: 7px 14px;
    border: 1px solid rgba(236, 239, 247, 0.2);
    border-radius: 8px;
    background: none;
    color: inherit;
    font: inherit;
    font-size: 13px;
    cursor: pointer;
  }
  .secondary:hover {
    background: #141826;
  }
  .field.check span {
    width: auto;
  }
  .field.check input {
    width: auto;
  }
  input.wide {
    width: 100%;
    box-sizing: border-box;
  }
  .field em {
    font-style: normal;
    font-size: 12px;
    color: #8c94a8;
  }
  .shortcut {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
    margin: 6px 0;
  }
  .shortcut .name {
    flex: none;
    width: 110px;
  }
  select {
    flex: 1;
    min-width: 0;
    padding: 8px 6px;
    border: 1px solid rgba(236, 239, 247, 0.15);
    border-radius: 8px;
    background: #141826;
    color: inherit;
    font: inherit;
    font-size: 13px;
  }
  .shortcut .target {
    flex-basis: 100%;
  }
  .row-buttons {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-top: 8px;
  }
</style>
