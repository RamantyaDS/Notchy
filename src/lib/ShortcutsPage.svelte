<script lang="ts">
  // Buttons for quick actions; the list is set in Settings.
  import { invoke } from "@tauri-apps/api/core";

  export type Shortcut = { name: string; action: string; target: string };

  // onCamera: the "camera" action opens the notch's own camera check.
  let { shortcuts, onCamera }: { shortcuts: Shortcut[]; onCamera: () => void } = $props();
  let failed = $state<number | null>(null);

  const ICON: Record<string, string> = {
    lock: "M6 11h12v10H6zM8 11V7a4 4 0 0 1 8 0v4",
    sleep: "M20 14A8 8 0 1 1 10 4a6 6 0 0 0 10 10z",
    screenshot: "M4 8V5h3M17 5h3v3M20 16v3h-3M7 19H4v-3",
    downloads: "M3 7h6l2 2h10v10H3z",
    camera: "M15 10l5-3v10l-5-3zM3 7h12v10H3z",
    open: "M14 4h6v6M20 4l-9 9M18 14v6H4V6h6",
  };

  async function run(i: number) {
    if (shortcuts[i]?.action === "camera") return onCamera();
    try {
      await invoke("run_shortcut", { index: i });
    } catch {
      failed = i;
      setTimeout(() => (failed = null), 1500);
    }
  }
</script>

<div class="page">
  <div class="header">
    <span class="title">Pintasan</span>
    <span class="hint">atur di Settings</span>
  </div>
  <!-- One row for up to five, two rows of four beyond that. -->
  <div class="grid" style:--cols={shortcuts.length <= 5 ? Math.max(shortcuts.length, 1) : 4}>
    {#each shortcuts.slice(0, 8) as s, i}
      <button class="sc" class:failed={failed === i} onclick={() => run(i)} title={s.target || s.name}>
        <svg viewBox="0 0 24 24"><path d={ICON[s.action] ?? ICON.open} /></svg>
        <span>{failed === i ? "Gagal" : s.name}</span>
      </button>
    {/each}
  </div>
</div>

<style>
  .page {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    padding: 12px 38px 20px;
  }
  .header {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }
  .title {
    font-size: 14px;
    font-weight: 600;
  }
  .hint {
    font-size: 11px;
    color: rgba(255, 255, 255, 0.45);
  }
  .grid {
    flex: 1;
    margin-top: 8px;
    margin-bottom: 4px;
    display: grid;
    grid-template-columns: repeat(var(--cols, 4), 1fr);
    grid-auto-rows: 1fr;
    gap: 6px;
  }
  .sc {
    min-width: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 5px;
    padding: 4px;
    border: 0;
    border-radius: 12px;
    background: rgba(255, 255, 255, 0.07);
    color: #fff;
    font: inherit;
    font-size: 11.5px;
    cursor: pointer;
  }
  .sc:hover {
    background: rgba(255, 255, 255, 0.15);
  }
  .sc.failed {
    background: rgba(248, 113, 113, 0.2);
  }
  .sc span {
    max-width: 100%;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  svg {
    width: 18px;
    height: 18px;
    fill: none;
    stroke: #fff;
    stroke-width: 1.8;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
</style>
