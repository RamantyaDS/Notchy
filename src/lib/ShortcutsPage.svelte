<script lang="ts">
  // Buttons for quick actions; the list is set in Settings.
  import { invoke } from "@tauri-apps/api/core";
  import { t } from "./i18n";

  function getShortcutLabel(name: string, action: string): string {
    const defaultMap: Record<string, string> = {
      lock: "shortcuts.lock",
      sleep: "shortcuts.sleep",
      screenshot: "shortcuts.screenshot",
      downloads: "shortcuts.downloads",
      camera: "shortcuts.camera",
    };
    if (defaultMap[action] && (["Kunci", "Lock", "Sleep", "Screenshot", "Downloads", "Kamera", "Camera"].includes(name))) {
      return t(defaultMap[action]);
    }
    // Newer actions show their own name until one is typed.
    if (!name.trim() && action !== "open") return t(`actions.${action}`).replace(/ \(Settings\)$/, "");
    return name;
  }

  export type Shortcut = { name: string; action: string; target: string };

  // onCamera: the "camera" action opens the notch's own camera check.
  // keepAwake: 0 off, otherwise when it ends (Number.MAX_SAFE_INTEGER-ish: until turned off).
  let {
    shortcuts,
    onCamera,
    keepAwake = 0,
    now = Date.now(),
  }: { shortcuts: Shortcut[]; onCamera: () => void; keepAwake?: number; now?: number } = $props();

  // Each click on "Keep awake" moves to the next length, then off.
  const AWAKE_STEPS = [30, 60, 120, 0];
  let awakeStep = $state(-1);
  function cycleAwake() {
    awakeStep = keepAwake ? awakeStep + 1 : 0;
    const minutes = awakeStep < AWAKE_STEPS.length ? AWAKE_STEPS[awakeStep] : -1;
    if (minutes < 0) awakeStep = -1;
    invoke("keep_awake", { minutes });
  }
  const awakeLabel = $derived(
    !keepAwake ? "" : keepAwake > 4e15 ? t("ka.0") : `${Math.max(1, Math.round((keepAwake - now) / 60_000))} min`,
  );
  let failed = $state<number | null>(null);

  const ICON: Record<string, string> = {
    lock: "M6 11h12v10H6zM8 11V7a4 4 0 0 1 8 0v4",
    sleep: "M20 14A8 8 0 1 1 10 4a6 6 0 0 0 10 10z",
    screenshot: "M4 8V5h3M17 5h3v3M20 16v3h-3M7 19H4v-3",
    downloads: "M3 7h6l2 2h10v10H3z",
    camera: "M15 10l5-3v10l-5-3zM3 7h12v10H3z",
    open: "M14 4h6v6M20 4l-9 9M18 14v6H4V6h6",
    keepawake: "M4 9h13v5a5 5 0 0 1-5 5H9a5 5 0 0 1-5-5zM17 11h1.5a2.5 2.5 0 0 1 0 5H17M8 3v3M12 3v3",
    keyboardclean: "M3 6h18v12H3zM7 10h.01M11 10h.01M15 10h.01M7 14h10",
    ocr: "M4 8V5h3M17 5h3v3M20 16v3h-3M7 19H4v-3M8 10h8M8 14h5",
    darkmode: "M12 3a9 9 0 1 0 9 9 7 7 0 0 1-9-9z",
    nightlight: "M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8zM12 2v2M12 20v2M2 12h2M20 12h2",
    batterysaver: "M7 4h10v17H7zM10 2h4M10 14l2-4 2 4",
    focus: "M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9M3 3l18 18",
  };

  async function run(i: number) {
    if (shortcuts[i]?.action === "camera") return onCamera();
    if (shortcuts[i]?.action === "keepawake") return cycleAwake();
    if (shortcuts[i]?.action === "keyboardclean") return invoke("lock_keyboard", { seconds: 60 });
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
    <span class="title">{t("shortcuts.title")}</span>
    <span class="hint">{t("shortcuts.hint")}</span>
  </div>
  <!-- One row for up to five, two rows of four beyond that. -->
  <div class="grid" style:--cols={shortcuts.length <= 5 ? Math.max(shortcuts.length, 1) : 4}>
    {#each shortcuts.slice(0, 8) as s, i}
      {@const awake = s.action === "keepawake" && !!keepAwake}
      <button class="sc" class:failed={failed === i} class:on={awake} onclick={() => run(i)} title={s.target || s.name}>
        <svg viewBox="0 0 24 24"><path d={ICON[s.action] ?? ICON.open} /></svg>
        <span>{failed === i ? t("shortcuts.failed") : getShortcutLabel(s.name, s.action)}</span>
        {#if awake}<small>{awakeLabel}</small>{/if}
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
  .sc.on {
    background: rgba(52, 211, 153, 0.18);
    color: #34d399;
    box-shadow: inset 0 0 0 1px rgba(52, 211, 153, 0.4);
  }
  .sc small {
    font-size: 9.5px;
    opacity: 0.85;
  }
  .sc.failed {
    background: rgba(248, 113, 113, 0.2);
  }
  .sc span {
    max-width: 100%;
    overflow: hidden;
    /* Up to two lines, so names like "Copy text from screen" still fit. */
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    text-align: center;
    line-height: 1.15;
    font-size: 11px;
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
