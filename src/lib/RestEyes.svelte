<script lang="ts">
  import { fade } from "svelte/transition";
  import type { Weather } from "./weather";

  let {
    weather,
    greeting,
    dozing,
    blink,
    look,
    time,
  }: {
    weather: Weather | null;
    greeting: boolean;
    dozing: boolean;
    blink: boolean;
    look: [number, number, number];
    time: string;
  } = $props();
</script>

{#if weather}
  <span class="weather">{weather.icon}<b>{weather.temp}°</b></span>
{/if}

{#if greeting}
  <div class="eyes happy" in:fade={{ duration: 150 }}><i></i><i></i></div>
{:else if dozing}
  <div
    class="eyes"
    class:blink
    style:--lx={look[0]}
    style:--ly={look[1]}
    style:--near={look[2]}
    in:fade={{ duration: 400 }}
  >
    <i></i><i></i>
  </div>
{:else}
  <span class="rest-time" in:fade={{ duration: 400 }}>{time}</span>
{/if}

<style>
  .rest-time {
    font-size: 13.5px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    letter-spacing: -0.02em;
    white-space: nowrap;
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
  .eyes {
    display: flex;
    gap: 15px;
    transform: translate(calc(var(--lx, 0) * 9px), calc(var(--ly, 0) * 4px)) scale(calc(1 + var(--near, 0) * 0.25));
    transition: transform 0.22s var(--spring, cubic-bezier(0.34, 1.32, 0.5, 1));
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
  .eyes.happy i {
    width: 12px;
    height: 7px;
    border: 2.4px solid #fff;
    border-bottom: 0;
    border-radius: 12px 12px 0 0;
    background: none;
  }
</style>
