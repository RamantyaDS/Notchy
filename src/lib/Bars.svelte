<script lang="ts">
  // Equalizer bars. With `levels` (0..1 each, from the audio meter) they follow the
  // music; without, they fall back to a canned animation. Only when `live`, so a
  // closed notch stays idle.
  let { live = false, levels = null }: { live?: boolean; levels?: number[] | null } = $props();
  const metered = $derived(live && !!levels);
</script>

<span class="bars" class:live={live && !metered} class:metered>
  {#each [0, 1, 2, 3] as i}
    <i style:transform={metered ? `scaleY(${Math.max(0.12, levels![i] ?? 0)})` : null}></i>
  {/each}
</span>

<style>
  .bars {
    display: inline-flex;
    align-items: flex-end;
    gap: 2px;
    height: 13px;
  }
  i {
    width: 3px;
    border-radius: 2px;
    background: rgba(255, 255, 255, 0.9);
    transform-origin: bottom;
  }
  i:nth-child(1) { height: 45%; }
  i:nth-child(2) { height: 90%; }
  i:nth-child(3) { height: 60%; }
  i:nth-child(4) { height: 100%; }
  /* Metered: full height, scaled by the level (updates ~8 times a second, no tweening). */
  .metered i {
    height: 100%;
  }

  /* Stepped at 0.2 s and offset by whole steps, so every bar (and the spinning
     disc, same rhythm) changes on the same frame: ~5 repaints a second instead
     of 60, which keeps the transparent window cheap to draw. */
  .live i {
    animation: eq 1.2s steps(6) infinite alternate;
  }
  .live i:nth-child(2) { animation-delay: -0.4s; }
  .live i:nth-child(3) { animation-delay: -1s; }
  .live i:nth-child(4) { animation-delay: -0.6s; }

  @keyframes eq {
    from { transform: scaleY(0.35); }
    to { transform: scaleY(1); }
  }
</style>
