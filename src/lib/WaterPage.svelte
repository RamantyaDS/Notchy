<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type { Water } from "./productivity.svelte";

  let { water }: { water: Water } = $props();

  const s = $derived(water.settings);
  const today = $derived(water.today());
  const pct = $derived(s ? Math.min(100, Math.round((today / s.goalMl) * 100)) : 0);
  const hours = $derived(water.byHour());
  // A round top for the chart's scale, so its labels read nicely (250, 500, 750...).
  const top = $derived(Math.ceil(Math.max(s ? s.glassMl * 2 : 500, ...hours.map((h) => h.ml)) / 250) * 250);
  const liters = (ml: number) => (ml / 1000).toLocaleString("id-ID", { maximumFractionDigits: 2 });
  const reminders = $derived(s?.waterReminders !== false);

  // The switch saves through Settings, which sends the new settings back to the page.
  const toggle = () => invoke("patch_settings", { patch: { waterReminders: !reminders } });
</script>

{#if s}
  <div class="page">
    <div class="summary">
      <div class="top">
        <div class="amount">{liters(today)} <span>/ {liters(s.goalMl)} L</span></div>
        <button class="switch" class:on={reminders} onclick={toggle} role="switch" aria-checked={reminders} title="Pengingat minum">
          <span>Pengingat</span><i></i>
        </button>
      </div>
      <div class="status">{pct >= 100 ? "Target hari ini tercapai" : `Hari ini · ${pct}%`}</div>
      <div class="bar"><span style:width="{pct}%"></span></div>
      <div class="buttons">
        <button class="drink" onclick={() => water.drink()}>💧 Minum +{s.glassMl} ml</button>
        <button class="plain" onclick={() => invoke("open_settings")} title="Pengaturan minum air">
          <svg viewBox="0 0 24 24"><path d="M4 7h10M18 7h2M4 17h4M12 17h8" /><circle cx="16" cy="7" r="2" /><circle cx="10" cy="17" r="2" /></svg>Atur
        </button>
      </div>
    </div>

    <div class="chart" aria-label="Minum per jam hari ini">
      <div class="axis"><span>{top}</span><span>{top / 2}</span><span>0</span></div>
      <div class="plot">
        <div class="bars">
          {#each hours as h}
            <i style:height="{(h.ml / top) * 100}%" class:now={h.hour === new Date().getHours()}></i>
          {/each}
        </div>
        <div class="labels">
          <span>{String(hours[0]?.hour ?? 0).padStart(2, "0")}</span>
          <span>{String(hours[Math.floor(hours.length / 2)]?.hour ?? 0).padStart(2, "0")}</span>
          <span>{String(hours.at(-1)?.hour ?? 0).padStart(2, "0")}</span>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .page {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    gap: 20px;
    padding: 0 40px 14px;
  }
  .summary {
    flex: none;
    width: 190px;
  }
  .top {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .amount {
    font-family: "Segoe UI Variable Display", "Segoe UI", sans-serif;
    font-size: 24px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }
  .amount span {
    font-size: 13px;
    font-weight: 500;
    color: rgba(255, 255, 255, 0.55);
  }
  .status {
    margin-top: 1px;
    font-size: 11.5px;
    color: rgba(255, 255, 255, 0.55);
  }
  .bar {
    height: 5px;
    margin: 6px 0 9px;
    border-radius: 3px;
    background: rgba(56, 189, 248, 0.18);
    overflow: hidden;
  }
  .bar span {
    display: block;
    height: 100%;
    border-radius: 3px;
    background: linear-gradient(90deg, #19b8ff, #2d5bff);
  }
  button {
    border: 0;
    color: #fff;
    font: inherit;
    cursor: pointer;
  }
  .buttons {
    display: flex;
    gap: 6px;
  }
  .drink,
  .plain {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 5px 11px;
    border-radius: 999px;
    font-size: 11.5px;
    white-space: nowrap;
  }
  .drink {
    background: rgba(56, 189, 248, 0.2);
  }
  .drink:hover {
    background: rgba(56, 189, 248, 0.32);
  }
  .plain {
    background: rgba(255, 255, 255, 0.1);
  }
  .plain:hover {
    background: rgba(255, 255, 255, 0.18);
  }
  .plain svg {
    width: 12px;
    height: 12px;
    fill: none;
    stroke: currentColor;
    stroke-width: 2;
    stroke-linecap: round;
  }
  /* The reminder switch. */
  .switch {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0;
    background: none;
    font-size: 10.5px;
    color: rgba(255, 255, 255, 0.55);
  }
  .switch i {
    position: relative;
    width: 28px;
    height: 16px;
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.2);
    transition: background 0.2s;
  }
  .switch i::after {
    content: "";
    position: absolute;
    top: 2px;
    left: 2px;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: #fff;
    transition: transform 0.2s;
  }
  .switch.on i {
    background: #38bdf8;
  }
  .switch.on i::after {
    transform: translateX(12px);
  }
  .chart {
    flex: 1;
    height: 84px;
    display: flex;
    gap: 6px;
  }
  .axis {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    height: 66px;
    font-size: 9.5px;
    text-align: right;
    color: rgba(255, 255, 255, 0.4);
    font-variant-numeric: tabular-nums;
  }
  .plot {
    flex: 1;
    display: flex;
    flex-direction: column;
  }
  .bars {
    height: 66px;
    display: flex;
    align-items: flex-end;
    gap: 3px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.12);
  }
  .bars i {
    flex: 1;
    min-height: 2px;
    border-radius: 3px 3px 1px 1px;
    background: rgba(56, 189, 248, 0.55);
  }
  .bars i.now {
    background: #38bdf8;
  }
  .labels {
    display: flex;
    justify-content: space-between;
    margin-top: 4px;
    font-size: 9.5px;
    color: rgba(255, 255, 255, 0.45);
    font-variant-numeric: tabular-nums;
  }
</style>
