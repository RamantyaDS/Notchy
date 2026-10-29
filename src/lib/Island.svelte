<script lang="ts">
  // The closed notch while things run (notchy.md §5F.1, style 1B): the main
  // activity, then up to two more at the right end behind a thin divider.
  import { invoke } from "@tauri-apps/api/core";
  import { untrack } from "svelte";
  import Bars from "./Bars.svelte";
  import { bytes, duration, type Activity } from "./island";
  import { formatSpeed } from "./network";
  import { formatTime } from "./media";
  import type { NowPlaying } from "./media";
  import type { Pomodoro } from "./productivity.svelte";

  let {
    main,
    side,
    media,
    line,
    time,
    pomo,
    now,
    levels,
    micMuted,
    active,
    camera = false,
    mic = false,
  }: {
    main: Activity;
    side: Activity[];
    media: NowPlaying | null;
    /** The sung line, or "" (then the clock shows). */
    line: string;
    time: string;
    pomo: Pomodoro;
    now: number;
    levels: number[] | null;
    micMuted: boolean;
    /** On screen: animations may run. */
    active: boolean;
    /** Camera / mic in use (the mic dot is left out during a call, which already says so). */
    camera?: boolean;
    mic?: boolean;
  } = $props();

  const playing = $derived(active && !!media?.playing);
  const timerLeft = $derived(formatTime(Math.ceil(pomo.remaining(now))));

  // With the meter running, the disc turns a step on each level update (5 a second)
  // instead of on its own CSS clock, so bars and disc redraw together.
  let angle = $state(0);
  $effect(() => {
    if (levels) angle = (untrack(() => angle) + 12) % 360;
  });
  const discStyle = $derived(
    (media?.art ? `background-image:url(${media.art});` : "") + (playing && levels ? `transform:rotate(${angle}deg)` : ""),
  );
</script>

{#snippet ring(progress: number, rest: boolean)}
  <svg class="ring" class:rest viewBox="0 0 20 20">
    <circle cx="10" cy="10" r="8" />
    <circle cx="10" cy="10" r="8" class="fill" pathLength="100" stroke-dasharray="{progress * 100} 100" />
  </svg>
{/snippet}

{#snippet phone()}
  <svg class="icon" viewBox="0 0 24 24"><path d="M22 16.9v3a2 2 0 0 1-2.2 2 19.8 19.8 0 0 1-8.6-3.1 19.5 19.5 0 0 1-6-6A19.8 19.8 0 0 1 2.1 4.2 2 2 0 0 1 4.1 2h3a2 2 0 0 1 2 1.7c.1.9.4 1.8.7 2.7a2 2 0 0 1-.5 2.1L8 9.8a16 16 0 0 0 6 6l1.3-1.3a2 2 0 0 1 2.1-.4c.9.3 1.8.6 2.7.7a2 2 0 0 1 1.7 2z" /></svg>
{/snippet}

{#snippet arrow()}
  <svg class="icon" viewBox="0 0 24 24"><path d="M12 3v12M7 10l5 5 5-5M4 20h16" /></svg>
{/snippet}

<div class="island">
  <div class="main {main.kind}">
    {#if main.kind === "music"}
      <div class="disc" class:spin={playing && !levels} style={discStyle}></div>
      {#key line}<span class="line" class:clock-line={!line}>{line || time}</span>{/key}
      <Bars live={playing} {levels} />
    {:else if main.kind === "timer"}
      {@render ring(pomo.progress(now), pomo.phase === "break")}
      <span class="big" class:rest={pomo.phase === "break"}>{timerLeft}</span>
    {:else if main.kind === "call"}
      <span class="badge">{@render phone()}</span>
      <span class="label">{main.call.app}</span>
      <span class="green num">{duration(now - main.call.start)}</span>
      <button
        class="mute"
        class:off={micMuted}
        onclick={() => invoke("toggle_mic")}
        aria-label={micMuted ? "Nyalakan mikrofon" : "Matikan mikrofon"}
        title={micMuted ? "Nyalakan mikrofon" : "Matikan mikrofon"}
      >
        <svg class="icon" viewBox="0 0 24 24">
          {#if micMuted}
            <path d="M9 9v3a3 3 0 0 0 5.1 2.1M15 9.3V6a3 3 0 0 0-5.7-1.3M19 11a7 7 0 0 1-1.2 3.9M5 11a7 7 0 0 0 10.6 6M12 18v3M3 3l18 18" />
          {:else}
            <path d="M12 3a3 3 0 0 0-3 3v6a3 3 0 0 0 6 0V6a3 3 0 0 0-3-3zM19 11a7 7 0 0 1-14 0M12 18v3" />
          {/if}
        </svg>
      </button>
    {:else if main.kind === "download"}
      <span class="blue">{@render arrow()}</span>
      <span class="label name">{main.download.name}</span>
      <span class="dim num">{bytes(main.download.bytes)}</span>
      <span class="run" class:on={active}></span>
    {:else if main.kind === "network"}
      <span class="net-pill down">
        <svg class="mini-arrow" viewBox="0 0 24 24"><path d="M12 4v13M7 12l5 5 5-5" /></svg>
        <span class="num">{formatSpeed(main.speed.down)}</span>
      </span>
      <span class="net-pill up">
        <svg class="mini-arrow" viewBox="0 0 24 24"><path d="M12 20V7M7 12l5-5 5 5" /></svg>
        <span class="num">{formatSpeed(main.speed.up)}</span>
      </span>
    {/if}
  </div>

  {#each side as a (a.kind)}
    <i class="sep"></i>
    <div class="side {a.kind}">
      {#if a.kind === "music"}
        <div class="disc small" class:spin={playing && !levels} style={discStyle}></div>
        <Bars live={playing} {levels} />
      {:else if a.kind === "timer"}
        {@render ring(pomo.progress(now), pomo.phase === "break")}
        <span class="num" class:red={pomo.phase !== "break"} class:green={pomo.phase === "break"}>{timerLeft}</span>
      {:else if a.kind === "call"}
        <span class="green">{@render phone()}</span>
        <span class="green num">{duration(now - a.call.start)}</span>
      {:else if a.kind === "download"}
        <span class="blue">{@render arrow()}</span>
        <span class="blue num">{bytes(a.download.speed)}/s</span>
      {:else if a.kind === "network"}
        <span class="blue num">↓{formatSpeed(a.speed.down)}</span>
      {/if}
    </div>
  {/each}

  {#if camera || mic}
    <span class="dots">
      {#if camera}<i class="cam"></i>{/if}
      {#if mic}<i class="mic"></i>{/if}
    </span>
  {/if}
</div>

<style>
  .island {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    padding: 0 12px 0 8px;
    gap: 8px;
  }
  .main {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 8px;
    height: 100%;
  }
  .main.timer {
    justify-content: center;
    gap: 30px;
  }
  .side {
    flex: none;
    display: flex;
    align-items: center;
    gap: 5px;
  }
  .dots {
    flex: none;
    display: flex;
    gap: 4px;
  }
  .dots i {
    width: 6px;
    height: 6px;
    border-radius: 50%;
  }
  .cam {
    background: #34d399;
    box-shadow: 0 0 6px #34d399;
  }
  .mic {
    background: #fb923c;
    box-shadow: 0 0 6px #fb923c;
  }
  .sep {
    flex: none;
    width: 1px;
    height: 16px;
    background: rgba(255, 255, 255, 0.18);
  }
  .line {
    flex: 1;
    min-width: 0;
    padding: 0 6px;
    text-align: center;
    font-size: 12px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .clock-line,
  .big {
    font-size: 14px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }
  .label {
    flex: 1;
    min-width: 0;
    font-size: 12.5px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .num {
    font-size: 12px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }
  .side .num {
    font-size: 11.5px;
  }
  .dim {
    color: rgba(255, 255, 255, 0.55);
    font-weight: 600;
  }
  .green {
    color: #34d399;
  }
  .red {
    color: #fb7185;
  }
  .blue {
    color: #38bdf8;
  }
  .main.network {
    gap: 10px;
    justify-content: center;
  }
  .net-pill {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 11.5px;
    font-weight: 600;
  }
  .net-pill.down {
    color: #38bdf8;
  }
  .net-pill.up {
    color: #34d399;
  }
  .mini-arrow {
    width: 12px;
    height: 12px;
    fill: none;
    stroke: currentColor;
    stroke-width: 2.4;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .icon {
    display: block;
    width: 13px;
    height: 13px;
    fill: none;
    stroke: currentColor;
    stroke-width: 2.2;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .badge {
    flex: none;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: #34d399;
    color: #000;
  }
  .badge .icon {
    width: 11px;
    height: 11px;
  }
  .mute {
    flex: none;
    width: 20px;
    height: 20px;
    padding: 0;
    border: 0;
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: rgba(255, 255, 255, 0.14);
    color: #fff;
    cursor: pointer;
  }
  .mute.off {
    background: #ef4444;
  }
  .mute .icon {
    width: 11px;
    height: 11px;
  }

  .ring {
    flex: none;
    width: 18px;
    height: 18px;
    transform: rotate(-90deg);
  }
  .main .ring {
    width: 20px;
    height: 20px;
  }
  .ring circle {
    fill: none;
    stroke: rgba(251, 113, 133, 0.25);
    stroke-width: 2.6;
  }
  .ring .fill {
    stroke: #fb7185;
    stroke-linecap: round;
  }
  .ring.rest circle {
    stroke: rgba(52, 211, 153, 0.25);
  }
  .ring.rest .fill {
    stroke: #34d399;
  }
  .big.rest {
    color: #fff;
  }

  /* A download has no known end, so a light runs along the bottom instead of a bar. */
  .run {
    position: absolute;
    left: 14px;
    right: 14px;
    bottom: 3px;
    height: 2px;
    border-radius: 1px;
    overflow: hidden;
    background: rgba(56, 189, 248, 0.15);
  }
  .run::after {
    content: "";
    position: absolute;
    inset: 0 auto 0 0;
    width: 35%;
    background: linear-gradient(90deg, transparent, #38bdf8, transparent);
  }
  /* Stepped like the other animations: few repaints. */
  .run.on::after {
    animation: run 1.6s steps(8) infinite;
  }
  @keyframes run {
    from {
      transform: translateX(-100%);
    }
    to {
      transform: translateX(290%);
    }
  }

  /* Cover as a little record: round, with a spindle hole, turning while playing. */
  .disc {
    position: relative;
    flex: none;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: #1b1f2e center / cover;
    box-shadow: 0 0 0 0.5px rgba(255, 255, 255, 0.25);
  }
  .disc.small {
    width: 16px;
    height: 16px;
  }
  .disc::after {
    content: "";
    position: absolute;
    inset: 37.5%;
    border-radius: 50%;
    background: #000;
    box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.35);
  }
  /* Stepped in time with the bars (one change every 0.2 s), see Bars.svelte. */
  .disc.spin {
    animation: spin 6s steps(30) infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
