<script lang="ts">
  import { formatTime } from "./media";
  import type { Pomodoro } from "./productivity.svelte";

  let { pomo, now }: { pomo: Pomodoro; now: number } = $props();

  const left = $derived(pomo.remaining(now));
  const done = $derived(pomo.progress(now));
  const focus = $derived(pomo.phase === "focus");
</script>

<div class="page">
  <svg class="ring" class:rest={!focus} viewBox="0 0 100 100">
    <circle cx="50" cy="50" r="44" />
    <circle cx="50" cy="50" r="44" class="fill" pathLength="100" stroke-dasharray="{done * 100} 100" />
  </svg>
  <div class="info">
    <div class="time">{formatTime(Math.ceil(left))}</div>
    <div class="phase">
      {focus ? "Fokus" : "Istirahat"} · {focus ? pomo.focusMin : pomo.breakMin} menit
    </div>
    <div class="buttons">
      <button class="main" onclick={() => (pomo.running ? pomo.pause() : pomo.start())}>
        {pomo.running ? "Jeda" : left < pomo.length() ? "Lanjut" : "Mulai"}
      </button>
      <button onclick={() => pomo.skip()}>{focus ? "Istirahat" : "Fokus"}</button>
      <button onclick={() => pomo.reset()} aria-label="Ulang">↺</button>
    </div>
  </div>
</div>

<style>
  .page {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    gap: 22px;
    padding: 0 40px 10px;
  }
  .ring {
    flex: none;
    width: 86px;
    height: 86px;
    transform: rotate(-90deg);
  }
  circle {
    fill: none;
    stroke: rgba(251, 113, 133, 0.2);
    stroke-width: 7;
  }
  .fill {
    stroke: #fb7185;
    stroke-linecap: round;
    transition: stroke-dasharray 0.3s;
  }
  .rest circle {
    stroke: rgba(52, 211, 153, 0.2);
  }
  .rest .fill {
    stroke: #34d399;
  }
  .time {
    font-family: "Segoe UI Variable Display", "Segoe UI", sans-serif;
    font-size: 38px;
    font-weight: 700;
    letter-spacing: -0.02em;
    font-variant-numeric: tabular-nums;
    line-height: 1;
  }
  .phase {
    margin-top: 4px;
    font-size: 12px;
    color: rgba(255, 255, 255, 0.55);
  }
  .buttons {
    display: flex;
    gap: 6px;
    margin-top: 10px;
  }
  button {
    padding: 5px 12px;
    border: 0;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.12);
    color: #fff;
    font: inherit;
    font-size: 12px;
    cursor: pointer;
  }
  button:hover {
    background: rgba(255, 255, 255, 0.22);
  }
  .main {
    background: #fff;
    color: #000;
    font-weight: 600;
  }
  .main:hover {
    background: #dfe3ee;
  }
</style>
