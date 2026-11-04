<script lang="ts">
  import { fade } from "svelte/transition";
  import { formatSpeed } from "./network";
  import type { NetworkSpeed } from "./network";

  type Power = { charging: boolean; percent: number; battery: boolean };

  let {
    power,
    networkSpeed,
    netSpeedStatus,
    date,
    time,
  }: {
    power: Power;
    networkSpeed: NetworkSpeed | null;
    netSpeedStatus: boolean;
    date: string;
    time: string;
  } = $props();
</script>

<div class="status" in:fade={{ duration: 200, delay: 200 }}>
  {#if power.battery}
    <svg class="mini-ring" class:low={!power.charging && power.percent <= 20} class:charging={power.charging} viewBox="0 0 20 20">
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
  <span>{date}</span><span class="sep">·</span><b>{time}</b>
</div>

<style>
  .status {
    position: absolute;
    top: 6px;
    left: 14px;
    right: 14px;
    height: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    font-size: 11px;
    color: #8c94a8;
    pointer-events: none;
    z-index: 5;
  }
  .status b {
    color: #eceff7;
    font-weight: 600;
  }
  .sep {
    opacity: 0.5;
  }
  .mini-ring {
    width: 11px;
    height: 11px;
    transform: rotate(-90deg);
  }
  .mini-ring circle {
    fill: none;
    stroke: rgba(255, 255, 255, 0.15);
    stroke-width: 2.5;
  }
  .mini-ring .fill {
    stroke: #34d399;
  }
  .mini-ring.low .fill {
    stroke: #f87171;
  }
  .mini-ring.charging .fill {
    stroke: #38bdf8;
  }
  .net-status {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-variant-numeric: tabular-nums;
  }
  .net-status .down {
    color: #38bdf8;
  }
  .net-status .up {
    color: #a78bfa;
  }
</style>
