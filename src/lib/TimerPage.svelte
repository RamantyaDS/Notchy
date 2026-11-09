<script lang="ts">
  import { formatTime } from "./media";
  import type { Pomodoro } from "./productivity.svelte";
  import { t } from "./i18n";

  let { pomo, now }: { pomo: Pomodoro; now: number } = $props();

  const left = $derived(pomo.remaining(now));
  const done = $derived(pomo.progress(now));
  const focus = $derived(pomo.phase !== "break");
  const QUICK = [1, 3, 5, 10, 15];
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
      <button class="reset" onclick={() => pomo.reset()} aria-label={t("timer.reset")} title={t("timer.reset")}>↺</button>
    </div>
    <div class="phase">
      {pomo.phase === "quick" ? t("timer.quick") : focus ? t("timer.focus") : t("timer.break")} ·
      {pomo.phase === "quick" ? pomo.quickMin : focus ? pomo.focusMin : pomo.breakMin} {t("timer.minutesUnit")}
      <span class="quick" title={t("timer.quick")}>
        ⏲{#each QUICK as m}<button class:on={pomo.phase === "quick" && pomo.quickMin === m} onclick={() => pomo.quick(m)}>{m}</button>{/each}
      </span>
    </div>
    <!-- Quick starts on either side of the main button. -->
    <div class="buttons">
      <button onclick={() => pomo.begin("break")} title={t("timer.startBreakTitle")}>{t("timer.startBreak", { min: pomo.breakMin })}</button>
      <button class="main" onclick={() => (pomo.running ? pomo.pause() : pomo.start())}>
        {pomo.running ? t("timer.pause") : left < pomo.length() ? t("timer.resume") : t("timer.start")}
      </button>
      <button onclick={() => pomo.begin("focus")} title={t("timer.startFocusTitle")}>{t("timer.startFocus", { min: pomo.focusMin })}</button>
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
  .dial:not(.rest):has(+ .info .quick button.on) {
    color: #fbbf24;
  }
  .quick {
    margin-left: 6px;
    display: inline-flex;
    align-items: center;
    gap: 3px;
  }
  .quick button {
    border: 0;
    padding: 0 6px;
    height: 17px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.1);
    color: rgba(255, 255, 255, 0.8);
    font: inherit;
    font-size: 10.5px;
    cursor: pointer;
  }
  .quick button:hover {
    background: rgba(255, 255, 255, 0.2);
  }
  .quick button.on {
    background: #fbbf24;
    color: #000;
    font-weight: 600;
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
