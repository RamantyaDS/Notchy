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
    mascot = true,
    mascotMode = "idle",
    activeEyes = false,
    wink = null,
    dizzy = false,
    musicPlaying = false,

    charging = false,
    batteryPercent = 100,
    island = false,
    mascotTheme = "classic",
    shocked = false,
    tickled = false,
  }: {
    weather: Weather | null;
    greeting: boolean;
    dozing: boolean;
    blink: boolean;
    look: [number, number, number];
    time: string;
    mascot?: boolean;
    mascotMode?: string;
    activeEyes?: boolean;
    wink?: "left" | "right" | null;
    dizzy?: boolean;
    musicPlaying?: boolean;

    charging?: boolean;
    batteryPercent?: number;
    island?: boolean;
    mascotTheme?: string;
    shocked?: boolean;
    tickled?: boolean;
  } = $props();


  const isHappy = $derived(greeting || tickled);

  const showEyes = $derived(
    mascot && (island || mascotMode === "always" || activeEyes || dozing || shocked || tickled)
  );

  const showSideClock = $derived(
    !island && !charging && mascot && mascotMode === "always" && (showEyes || greeting)
  );
</script>

{#if weather && !charging && !island}
  <span class="weather">{weather.icon}<b>{weather.temp}°</b></span>
{/if}

{#if greeting && !charging && !island}
  <div class="eyes happy" class:theme-anime={mascotTheme === "anime"} class:theme-pixel={mascotTheme === "pixel"} class:theme-cat={mascotTheme === "cat"} class:theme-cartoon={mascotTheme === "cartoon"} in:fade={{ duration: 150 }}><i></i><i></i></div>
{:else if showEyes}
  <div class="eyes-container">
    {#if dozing}
      <span class="zzz z1">z</span>
      <span class="zzz z2">Z</span>
    {/if}
    {#if tickled}
      <span class="giggle-sparkle g1">✨</span>
      <span class="giggle-sparkle g2">ha!</span>
    {/if}
    <div
      class="eyes-wrapper"
      class:music-bop={musicPlaying && !dozing && !tickled}
      class:tickle-shake={tickled}
    >
      <div
        class="eyes"
        class:blink
        class:wink-left={wink === "left"}
        class:wink-right={wink === "right"}
        class:sleepy={dozing}
        class:dizzy

        class:charging
        class:happy={isHappy}
        class:shocked
        class:theme-classic={mascotTheme === "classic" || !mascotTheme}
        class:theme-anime={mascotTheme === "anime"}
        class:theme-pixel={mascotTheme === "pixel"}
        class:theme-cat={mascotTheme === "cat"}
        class:theme-cartoon={mascotTheme === "cartoon"}
        style:--lx={look[0]}
        style:--ly={look[1]}
        style:--near={look[2]}
        in:fade={{ duration: 250 }}
      >
        <i></i><i></i>
      </div>
    </div>
  </div>
{:else if !charging && !island}
  <span class="rest-time" in:fade={{ duration: 300 }}>{time}</span>
{/if}

{#if showSideClock}
  <span class="rest-time-side" in:fade={{ duration: 300 }}>{time}</span>
{/if}

<style>
  .rest-time {
    font-size: 13.5px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    letter-spacing: -0.02em;
    white-space: nowrap;
  }
  .rest-time-side {
    position: absolute;
    right: 12px;
    top: 50%;
    transform: translateY(-50%);
    z-index: 2;
    font-size: 11.5px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    letter-spacing: -0.01em;
    color: #e6edf3;
    white-space: nowrap;
    display: flex;
    align-items: center;
    line-height: 1;
  }
  .weather {
    position: absolute;
    left: 12px;
    top: 50%;
    transform: translateY(-50%);
    z-index: 2;
    display: flex;
    align-items: center;
    gap: 3px;
    font-size: 10px;
  }
  .weather b {
    font-size: 11px;
    font-weight: 600;
  }
  .eyes-container {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .eyes-wrapper {
    display: flex;
    align-items: center;
    justify-content: center;
    transition: transform 0.2s ease;
  }
  .eyes-wrapper.music-bop {
    animation: music-bounce 0.52s ease-in-out infinite alternate;
  }
  @keyframes music-bounce {
    0% {
      transform: translateY(0);
    }
    100% {
      transform: translateY(-2.2px);
    }
  }
  .eyes-wrapper.tickle-shake {
    animation: tickle-wiggle 0.11s ease-in-out infinite alternate;
  }
  @keyframes tickle-wiggle {
    0% {
      transform: translateX(-3px) rotate(-5deg);
    }
    100% {
      transform: translateX(3px) rotate(5deg);
    }
  }

  .eyes {
    position: relative;
    display: flex;
    gap: 15px;
    transform: translate(calc(var(--lx, 0) * 9px), calc(var(--ly, 0) * 4px))
      rotate(calc(var(--lx, 0) * 8deg))
      scale(calc(1 + var(--near, 0) * 0.25));
    transition: transform 0.22s var(--spring, cubic-bezier(0.34, 1.32, 0.5, 1));
  }
  .eyes i {
    position: relative;
    width: 8.4px;
    height: 12.4px;
    border-radius: 50%;
    background: #fff;
    box-shadow:
      0 0 6px 1px rgba(255, 255, 255, 0.45),
      0 0 14px 4px rgba(255, 255, 255, 0.12);
    transition:
      transform 0.14s ease,
      border-radius 0.18s ease,
      background 0.18s ease,
      box-shadow 0.25s ease;
  }

  /* 🎨 Theme: Anime Sparkle ✨ */
  .eyes.theme-anime i::after {
    content: "";
    position: absolute;
    top: 2px;
    right: 1.5px;
    width: 3.8px;
    height: 3.8px;
    background: #ffffff;
    clip-path: polygon(50% 0%, 65% 35%, 100% 50%, 65% 65%, 50% 100%, 35% 65%, 0% 50%, 35% 35%);
    filter: drop-shadow(0 0 2px rgba(255, 255, 255, 0.95));
    animation: anime-glint 2.2s ease-in-out infinite alternate;
  }
  .eyes.theme-anime i::before {
    content: "";
    position: absolute;
    bottom: 2px;
    left: 2px;
    width: 2px;
    height: 2px;
    border-radius: 50%;
    background: #ffffff;
    opacity: 0.85;
  }
  @keyframes anime-glint {
    0% {
      transform: scale(0.75) rotate(0deg);
      opacity: 0.6;
    }
    50% {
      transform: scale(1.3) rotate(45deg);
      opacity: 1;
    }
    100% {
      transform: scale(0.85) rotate(90deg);
      opacity: 0.75;
    }
  }
  .eyes.theme-anime.happy i::after,
  .eyes.theme-anime.happy i::before {
    display: none;
  }

  /* 🎨 Theme: Retro 8-Bit Pixel 👾 */
  .eyes.theme-pixel i {
    width: 8px;
    height: 12px;
    border-radius: 0 !important;
    box-shadow:
      0 0 0 1px rgba(0, 0, 0, 0.5),
      0 0 6px 1px rgba(255, 255, 255, 0.5);
    image-rendering: pixelated;
  }
  .eyes.theme-pixel.happy i {
    width: 12px;
    height: 6px;
    border: 2px solid #fff;
    border-bottom: 0;
    border-radius: 0;
  }

  /* 🎨 Theme: Cat Eyes 🐱 */
  .eyes.theme-cat i {
    width: 10px;
    height: 13px;
    border-radius: 50% 50% 50% 50% / 60% 60% 60% 60%;
    background: #ffd166;
    box-shadow:
      0 0 6px 1px rgba(255, 209, 102, 0.6),
      0 0 12px 3px rgba(255, 209, 102, 0.2);
    overflow: hidden;
  }
  .eyes.theme-cat i::after {
    content: "";
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(calc(-50% + var(--lx, 0) * 2.5px), -50%);
    width: calc(2.2px + var(--near, 0) * 3px);
    height: 9.5px;
    border-radius: 50%;
    background: #0d0f17;
    transition:
      transform 0.18s var(--spring, cubic-bezier(0.34, 1.32, 0.5, 1)),
      width 0.15s ease;
  }
  .eyes.theme-cat.shocked i::after {
    width: 1.5px;
    transform: translate(-50%, -50%);
  }
  .eyes.theme-cat.happy i {
    background: transparent;
    border: 2.2px solid #ffd166;
    border-bottom: 0;
  }
  .eyes.theme-cat.happy i::after {
    display: none;
  }

  /* 🎨 Theme: Cartoon Pupil 👀 */
  .eyes.theme-cartoon i {
    width: 10px;
    height: 13.5px;
    border-radius: 50%;
    background: #ffffff;
    box-shadow:
      0 0 6px 1px rgba(255, 255, 255, 0.55),
      0 0 12px 3px rgba(255, 255, 255, 0.15);
    overflow: hidden;
    position: relative;
  }
  /* Specular Catchlight (Refleksi pantulan cahaya alami di kornea) */
  .eyes.theme-cartoon i::before {
    content: "";
    position: absolute;
    top: 2px;
    right: 2px;
    width: 2.2px;
    height: 2.2px;
    border-radius: 50%;
    background: #ffffff;
    opacity: 0.95;
    z-index: 3;
    pointer-events: none;
    box-shadow: 0 0 1.5px rgba(255, 255, 255, 0.85);
  }
  /* Active Gliding Pupil (Dual-layer Parallax Gaze & Dilation) */
  .eyes.theme-cartoon i::after {
    content: "";
    position: absolute;
    top: 50%;
    left: 50%;
    width: 5.4px;
    height: 6.4px;
    border-radius: 50%;
    background: #0f172a;
    transform: translate(
        calc(-50% + var(--lx, 0) * 2.8px),
        calc(-50% + var(--ly, 0) * 1.8px)
      )
      scale(calc(1 + var(--near, 0) * 0.25));
    transition:
      transform 0.18s var(--spring, cubic-bezier(0.34, 1.32, 0.5, 1)),
      width 0.15s ease,
      height 0.15s ease;
    z-index: 2;
  }
  /* Cartoon Happy Eyes */
  .eyes.theme-cartoon.happy i {
    background: transparent;
    border: 2.4px solid #fff;
    border-bottom: 0;
    box-shadow: none;
    border-radius: 12px 12px 0 0;
  }
  .eyes.theme-cartoon.happy i::before,
  .eyes.theme-cartoon.happy i::after {
    display: none;
  }
  /* Cartoon Shocked Pinpoint Pupil (😲 Pinpoint Shock Effect) */
  .eyes.theme-cartoon.shocked i::after {
    transform: translate(-50%, -50%) scale(0.38);
    transition: transform 0.08s ease;
  }

  /* 😲 Shocked / Kaget Reaction */
  .eyes.shocked i {
    width: 12.5px;
    height: 12.5px;
    border-radius: 50% !important;
    transform: scale(1.18);
    box-shadow:
      0 0 10px 2px rgba(255, 255, 255, 0.85),
      0 0 18px 5px rgba(255, 255, 255, 0.35);
  }

  /* 🔋 Charging Mode Glow (Static to eliminate repaint overhead) */
  .eyes.charging:not(.sleepy):not(.dizzy):not(.happy) i {
    box-shadow:
      0 0 6px 1.5px rgba(52, 211, 153, 0.6),
      0 0 13px 3px rgba(52, 211, 153, 0.22);
  }
  .eyes.charging.sleepy i {
    box-shadow:
      0 0 5px 1px rgba(52, 211, 153, 0.4),
      0 0 10px 2px rgba(52, 211, 153, 0.15);
  }
  .eyes.charging.happy i {
    border-color: #34d399;
    box-shadow: 0 0 8px 1.5px rgba(52, 211, 153, 0.6);
  }

  /* Expressions */
  .eyes.sleepy i {
    transform: scaleY(0.4);
  }
  .eyes.blink i {
    transform: scaleY(0.1);
  }
  .eyes.wink-left i:first-child {
    transform: scaleY(0.1);
  }
  .eyes.wink-left i:last-child {
    transform: scale(1.15);
  }
  .eyes.wink-right i:last-child {
    transform: scaleY(0.1);
  }
  .eyes.wink-right i:first-child {
    transform: scale(1.15);
  }
  .eyes.happy i {
    width: 12px;
    height: 7px;
    border: 2.4px solid #fff;
    border-bottom: 0;
    border-radius: 12px 12px 0 0;
    background: none;
    box-shadow: none;
  }
  .eyes.dizzy {
    animation: dizzy-wobble 0.25s ease-in-out infinite alternate;
  }
  .eyes.dizzy i {
    background: #ffe066;
    box-shadow: 0 0 8px 2px rgba(255, 224, 102, 0.55);
    border-radius: 40% 60% 70% 30% / 40% 50% 60% 50%;
    animation: eye-spin 0.65s linear infinite;
  }
  @keyframes dizzy-wobble {
    0% {
      transform: translate(calc(var(--lx, 0) * 9px - 2px), calc(var(--ly, 0) * 4px - 1px)) rotate(-6deg);
    }
    100% {
      transform: translate(calc(var(--lx, 0) * 9px + 2px), calc(var(--ly, 0) * 4px + 1px)) rotate(6deg);
    }
  }
  @keyframes eye-spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }
  

  /* Floating Particles */
  .zzz {
    position: absolute;
    right: -16px;
    top: -6px;
    font-size: 8px;
    font-weight: 800;
    line-height: 1;
    color: rgba(255, 255, 255, 0.75);
    pointer-events: none;
    user-select: none;
    animation: float-zzz 2.2s cubic-bezier(0.2, 0.6, 0.35, 1) infinite;
  }
  .zzz.z2 {
    right: -22px;
    top: -12px;
    font-size: 10.5px;
    animation-delay: 1.1s;
  }
  @keyframes float-zzz {
    0% {
      opacity: 0;
      transform: translate(0, 4px) scale(0.5);
    }
    25% {
      opacity: 0.9;
      transform: translate(3px, -2px) scale(0.85);
    }
    75% {
      opacity: 0.45;
      transform: translate(6px, -8px) scale(1.05);
    }
    100% {
      opacity: 0;
      transform: translate(9px, -14px) scale(1.2);
    }
  }

  /* Giggle Sparkle Particles */
  .giggle-sparkle {
    position: absolute;
    font-size: 9px;
    font-weight: 800;
    pointer-events: none;
    user-select: none;
    animation: float-giggle 1.1s ease-out infinite;
  }
  .giggle-sparkle.g1 {
    left: -14px;
    top: -8px;
    color: #ff70a6;
  }
  .giggle-sparkle.g2 {
    right: -16px;
    top: -10px;
    color: #ffd166;
    animation-delay: 0.35s;
  }
  @keyframes float-giggle {
    0% {
      opacity: 0;
      transform: translateY(2px) scale(0.5);
    }
    40% {
      opacity: 1;
      transform: translateY(-4px) scale(1);
    }
    100% {
      opacity: 0;
      transform: translateY(-12px) scale(1.2);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .eyes-wrapper.music-bop,
    .eyes-wrapper.tickle-shake,
    .eyes.dizzy,
    .eyes.theme-anime i::after,
    .zzz,
    .giggle-sparkle {
      animation: none !important;
    }
  }
</style>
