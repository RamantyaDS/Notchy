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
  };

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
</style>
