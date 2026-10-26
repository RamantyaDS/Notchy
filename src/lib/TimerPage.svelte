<script lang="ts">
  import { formatTime } from "./media";
  import type { Pomodoro } from "./productivity.svelte";

  let { pomo, now }: { pomo: Pomodoro; now: number } = $props();

  const left = $derived(pomo.remaining(now));
  const done = $derived(pomo.progress(now));
  const focus = $derived(pomo.phase === "focus");
</script>

<div class="page">
  <!-- A big ring around a stopwatch: red while focusing, green on a break. -->
  <div class="dial" class:rest={!focus}>
    <svg class="ring" viewBox="0 0 100 100">
      <circle cx="50" cy="50" r="44" />
      <circle cx="50" cy="50" r="44" class="fill" pathLength="100" stroke-dasharray="{done * 100} 100" />
    </svg>
    <svg class="watch" viewBox="0 0 24 24"><circle cx="12" cy="13" r="8" /><path d="M12 9v4l2 2M10 2h4" /></svg>
  </div>
  <div class="info">
    <div class="line">
      <span class="time">{formatTime(Math.ceil(left))}</span>
      <button class="reset" onclick={() => pomo.reset()} aria-label="Ulang" title="Ulang">↺</button>
    </div>
    <div class="phase">
      {focus ? "Fokus" : "Istirahat"} · {focus ? pomo.focusMin : pomo.breakMin} menit
    </div>
    <!-- Quick starts on either side of the main button. -->
    <div class="buttons">
      <button onclick={() => pomo.begin("break")} title="Mulai istirahat sekarang">☕ Istirahat {pomo.breakMin}</button>
      <button class="main" onclick={() => (pomo.running ? pomo.pause() : pomo.start())}>
        {pomo.running ? "❚❚ Jeda" : left < pomo.length() ? "▶ Lanjut" : "▶ Mulai"}
      </button>
      <button onclick={() => pomo.begin("focus")} title="Mulai fokus sekarang">◎ Fokus {pomo.focusMin}</button>
    </div>
  </div>
</div>

<style>
  .page {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    gap: 20px;
    padding: 0 40px 12px;
  }
  .dial {
    position: relative;
    flex: none;
    width: 80px;
    height: 80px;
    color: #fb7185;
  }
  .dial.rest {
    color: #34d399;
  }
  .ring {
    width: 100%;
    height: 100%;
    transform: rotate(-90deg);
  }
  circle {
    fill: none;
    stroke: currentColor;
    stroke-opacity: 0.22;
    stroke-width: 7;
  }
  .fill {
    stroke-opacity: 1;
    stroke-linecap: round;
    transition: stroke-dasharray 0.3s;
  }
  .watch {
    position: absolute;
    inset: 0;
    margin: auto;
    width: 26px;
    height: 26px;
    fill: none;
    stroke: currentColor;
    stroke-width: 2;
    stroke-linecap: round;
  }
  .watch circle {
    stroke-opacity: 1;
    stroke-width: 2;
  }
  .info {
    flex: 1;
    min-width: 0;
  }
  .line {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .time {
    font-family: "Segoe UI Variable Display", "Segoe UI", sans-serif;
    font-size: 34px;
    font-weight: 700;
    letter-spacing: -0.02em;
    font-variant-numeric: tabular-nums;
    line-height: 1;
  }
  .phase {
    margin-top: 3px;
    font-size: 11.5px;
    color: rgba(255, 255, 255, 0.55);
  }
  .buttons {
    display: flex;
    gap: 6px;
    margin-top: 9px;
  }
  button {
    padding: 5px 10px;
    border: 0;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.12);
    color: #fff;
    font: inherit;
    font-size: 11.5px;
    white-space: nowrap;
    cursor: pointer;
  }
  button:hover {
    background: rgba(255, 255, 255, 0.22);
  }
  .main {
    background: #fb7185;
    font-weight: 600;
  }
  .main:hover {
    background: #fda4af;
  }
  .rest ~ .info .main {
    background: #34d399;
    color: #000;
  }
  .reset {
    padding: 3px 7px;
    font-size: 12px;
  }
</style>
