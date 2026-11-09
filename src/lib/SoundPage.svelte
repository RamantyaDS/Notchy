<script lang="ts">
  // Per-app volume: every app with a sound session on the default output gets a
  // slider and a mute button. Refreshed while the page is open.
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { t } from "./i18n";

  type AppVolume = { name: string; volume: number; muted: boolean };
  type Output = { name: string; headphones: boolean } | null;

  let { output, onSwitch }: { output: Output; onSwitch: () => void } = $props();
  let apps = $state<AppVolume[]>([]);
  // The app being dragged keeps its own value until the next refresh after release.
  let dragging: string | null = null;

  const COLORS = ["#1db954", "#4285f4", "#2d8cff", "#5865f2", "#f97316", "#e11d48", "#a855f7", "#14b8a6"];
  const color = (name: string) => COLORS[[...name].reduce((h, c) => h + c.charCodeAt(0), 0) % COLORS.length];

  async function refresh() {
    const list = await invoke<AppVolume[]>("mixer_list").catch(() => []);
    apps = dragging ? list.map((a) => (a.name === dragging ? (apps.find((b) => b.name === a.name) ?? a) : a)) : list;
  }

  onMount(() => {
    refresh();
    const id = setInterval(refresh, 1500);
    return () => clearInterval(id);
  });

  let last = 0;
  function set(app: AppVolume, volume: number) {
    app.volume = volume;
    // Throttled while dragging; the final value is always sent on release.
    if (Date.now() - last > 40) {
      last = Date.now();
      invoke("mixer_set", { name: app.name, volume });
    }
  }
  function mute(app: AppVolume) {
    app.muted = !app.muted;
    invoke("mixer_set", { name: app.name, muted: app.muted });
  }
</script>

<div class="page">
  <div class="header">
    <span class="title">{t("sound.title")}</span>
    <span class="hint">{t("sound.hint")}</span>
    {#if output}
      <button class="out" onclick={onSwitch} title={output.name}>
        <svg viewBox="0 0 24 24"><path d={output.headphones ? "M3 14v-2a9 9 0 0 1 18 0v2M3 14h4v6H3zM17 14h4v6h-4z" : "M11 5 6 9H3v6h3l5 4zM15.5 8.5a5 5 0 0 1 0 7"} /></svg>
        <span>{output.name}</span>
      </button>
    {/if}
  </div>

  {#if apps.length === 0}
    <div class="empty">{t("sound.empty")}</div>
  {:else}
    <div class="list scroll">
      {#each apps as app (app.name)}
        <div class="row" class:muted={app.muted}>
          <span class="icon" style:background={color(app.name)}>{app.name.slice(0, 1)}</span>
          <span class="name" title={app.name}>{app.name}</span>
          <input
            type="range"
            min="0"
            max="1"
            step="0.01"
            value={app.volume}
            style:--v="{app.volume * 100}%"
            oninput={(e) => set(app, +e.currentTarget.value)}
            onpointerdown={() => (dragging = app.name)}
            onpointerup={(e) => {
              dragging = null;
              invoke("mixer_set", { name: app.name, volume: +e.currentTarget.value });
            }}
            aria-label={app.name}
          />
          <span class="val">{app.muted ? "–" : Math.round(app.volume * 100)}</span>
          <button class="mute" onclick={() => mute(app)} aria-label={app.muted ? t("sound.unmute") : t("sound.mute")} title={app.muted ? t("sound.unmute") : t("sound.mute")}>
            <svg viewBox="0 0 24 24"><path d={app.muted ? "M11 5 6 9H3v6h3l5 4zM22 9l-6 6M16 9l6 6" : "M11 5 6 9H3v6h3l5 4zM15.5 8.5a5 5 0 0 1 0 7"} /></svg>
          </button>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .page {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    padding: 12px 40px 22px;
  }
  .header {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }
  .title {
    font-size: 14px;
    font-weight: 600;
  }
  .hint {
    flex: 1;
    font-size: 11px;
    color: rgba(255, 255, 255, 0.45);
  }
  button {
    border: 0;
    background: none;
    color: #fff;
    font: inherit;
    cursor: pointer;
  }
  .out {
    display: flex;
    align-items: center;
    gap: 5px;
    max-width: 140px;
    padding: 2px 9px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.08);
    font-size: 11px;
    color: rgba(255, 255, 255, 0.75);
  }
  .out:hover {
    background: rgba(255, 255, 255, 0.16);
  }
  .out span {
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }
  svg {
    width: 13px;
    height: 13px;
    flex: none;
    fill: none;
    stroke: currentColor;
    stroke-width: 2;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .empty {
    flex: 1;
    display: grid;
    place-items: center;
    font-size: 12px;
    color: rgba(255, 255, 255, 0.5);
  }
  .list {
    flex: 1;
    margin-top: 7px;
    display: flex;
    flex-direction: column;
    gap: 5px;
    overflow-y: auto;
    scrollbar-width: none;
  }
  .row {
    flex: none;
    display: grid;
    grid-template-columns: 20px 82px 1fr 26px 18px;
    align-items: center;
    gap: 9px;
    font-size: 12px;
  }
  .icon {
    width: 20px;
    height: 20px;
    border-radius: 6px;
    display: grid;
    place-items: center;
    font-size: 10.5px;
    font-weight: 700;
  }
  .name {
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }
  .val {
    text-align: right;
    font-variant-numeric: tabular-nums;
    color: rgba(255, 255, 255, 0.55);
  }
  .muted .name,
  .muted .icon {
    opacity: 0.45;
  }
  .muted .mute {
    color: #f87171;
  }
  input[type="range"] {
    appearance: none;
    height: 4px;
    margin: 0;
    border-radius: 2px;
    background: linear-gradient(to right, #fff var(--v), rgba(255, 255, 255, 0.15) var(--v));
    cursor: pointer;
  }
  .muted input[type="range"] {
    background: linear-gradient(to right, rgba(255, 255, 255, 0.3) var(--v), rgba(255, 255, 255, 0.12) var(--v));
  }
  input[type="range"]::-webkit-slider-thumb {
    appearance: none;
    width: 11px;
    height: 11px;
    border-radius: 50%;
    background: #fff;
  }
</style>
