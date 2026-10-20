<script lang="ts">
  import type { Water } from "./productivity.svelte";

  let { water }: { water: Water } = $props();

  const s = $derived(water.settings);
  const today = $derived(water.today());
  const pct = $derived(s ? Math.min(100, Math.round((today / s.goalMl) * 100)) : 0);
  const hours = $derived(water.byHour());
  const top = $derived(Math.max(s ? s.glassMl * 2 : 500, ...hours.map((h) => h.ml)));
  const liters = (ml: number) => (ml / 1000).toLocaleString("id-ID", { maximumFractionDigits: 2 });
</script>

{#if s}
  <div class="page">
    <div class="summary">
      <div class="amount">{liters(today)} <span>/ {liters(s.goalMl)} L</span></div>
      <div class="status">{pct >= 100 ? "Target hari ini tercapai" : `Hari ini · ${pct}%`}</div>
      <div class="bar"><span style:width="{pct}%"></span></div>
      <button onclick={() => water.drink()}>💧 Minum +{s.glassMl} ml</button>
    </div>

    <div class="chart" aria-label="Minum per jam hari ini">
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
{/if}

<style>
  .page {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    gap: 24px;
    padding: 0 40px 10px;
  }
  .summary {
    flex: none;
    width: 160px;
  }
  .amount {
    font-family: "Segoe UI Variable Display", "Segoe UI", sans-serif;
    font-size: 26px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }
  .amount span {
    font-size: 14px;
    font-weight: 500;
    color: rgba(255, 255, 255, 0.55);
  }
  .status {
    margin-top: 2px;
    font-size: 12px;
    color: rgba(255, 255, 255, 0.55);
  }
  .bar {
    height: 5px;
    margin: 8px 0 10px;
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
    padding: 5px 12px;
    border: 0;
    border-radius: 999px;
    background: rgba(56, 189, 248, 0.2);
    color: #fff;
    font: inherit;
    font-size: 12px;
    cursor: pointer;
  }
  button:hover {
    background: rgba(56, 189, 248, 0.32);
  }
  .chart {
    flex: 1;
    height: 84px;
    display: flex;
    flex-direction: column;
  }
  .bars {
    flex: 1;
    display: flex;
    align-items: flex-end;
    gap: 3px;
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
    font-size: 10px;
    color: rgba(255, 255, 255, 0.45);
    font-variant-numeric: tabular-nums;
  }
</style>
