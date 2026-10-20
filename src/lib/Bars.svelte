<script lang="ts">
  // Equalizer bars. They only move when `live`, so a closed notch stays idle.
  let { live = false }: { live?: boolean } = $props();
</script>

<span class="bars" class:live><i></i><i></i><i></i><i></i></span>

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
  }
  i:nth-child(1) { height: 45%; }
  i:nth-child(2) { height: 90%; }
  i:nth-child(3) { height: 60%; }
  i:nth-child(4) { height: 100%; }

  /* Stepped at 0.2 s and offset by whole steps, so every bar (and the spinning
     disc, same rhythm) changes on the same frame: ~5 repaints a second instead
     of 60, which keeps the transparent window cheap to draw. */
  .live i {
    animation: eq 1.2s steps(6) infinite alternate;
    transform-origin: bottom;
  }
  .live i:nth-child(2) { animation-delay: -0.4s; }
  .live i:nth-child(3) { animation-delay: -1s; }
  .live i:nth-child(4) { animation-delay: -0.6s; }

  @keyframes eq {
    from { transform: scaleY(0.35); }
    to { transform: scaleY(1); }
  }
</style>
