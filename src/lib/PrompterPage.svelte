<script lang="ts">
  // Teleprompter: a script that scrolls slowly right under the webcam, so you
  // read while looking at the camera. Kept in %APPDATA%\Notchy\prompter.json.
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { t } from "./i18n";

  let { onTyping }: { onTyping: (on: boolean) => void } = $props();
  let text = $state("");
  let speed = $state(3);
  let editing = $state(false);
  let playing = $state(false);
  let box = $state<HTMLDivElement>();

  onMount(() => {
    invoke<{ text?: string; speed?: number } | null>("load_data", { name: "prompter" })
      .then((d) => {
        text = typeof d?.text === "string" ? d.text : "";
        speed = typeof d?.speed === "number" ? d.speed : 3;
      })
      .catch(() => {});
    return () => onTyping(false);
  });
  const persist = () => invoke("save_data", { name: "prompter", value: { text, speed } });

  // Scrolls `speed` × 12 px per second while playing.
  $effect(() => {
    if (!playing || !box) return;
    let last = performance.now();
    let carry = 0;
    let frame = requestAnimationFrame(function step(now) {
      carry += ((now - last) / 1000) * speed * 12;
      last = now;
      const whole = Math.floor(carry);
      carry -= whole;
      box!.scrollTop += whole;
      if (box!.scrollTop + box!.clientHeight >= box!.scrollHeight - 1) playing = false;
      else frame = requestAnimationFrame(step);
    });
    return () => cancelAnimationFrame(frame);
  });

  function edit() {
    playing = false;
    editing = true;
    onTyping(true);
  }
  function done() {
    editing = false;
    onTyping(false);
    persist();
  }
  function setSpeed(d: number) {
    speed = Math.max(1, Math.min(10, speed + d));
    persist();
  }
  function restart() {
    if (box) box.scrollTop = 0;
  }
</script>

<div class="page">
  <div class="header">
    <span class="title">{t("prompter.title")}</span>
    <span class="hint">{t("prompter.hint")}</span>
    {#if editing}
      <button class="chip on" onclick={done}>{t("prompter.done")}</button>
    {:else}
      <button class="chip" onclick={() => setSpeed(-1)} aria-label="−">−</button>
      <span class="speed">{speed}×</span>
      <button class="chip" onclick={() => setSpeed(1)} aria-label="+">+</button>
      <button class="chip" onclick={restart} title={t("prompter.restart")}>↺</button>
      <button class="chip" onclick={edit}>{t("prompter.edit")}</button>
      <button class="chip on" disabled={!text.trim()} onclick={() => (playing = !playing)}>{playing ? t("prompter.pause") : t("prompter.play")}</button>
    {/if}
  </div>

  {#if editing}
    <!-- svelte-ignore a11y_autofocus -->
    <textarea bind:value={text} placeholder={t("prompter.placeholder")} autofocus maxlength="20000"></textarea>
  {:else if !text.trim()}
    <div class="empty">{t("prompter.empty")}</div>
  {:else}
    <div class="script scroll" bind:this={box}>
      <p>{text}</p>
    </div>
  {/if}
</div>

<style>
  .page {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    padding: 10px 40px 22px;
  }
  .header {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .title {
    font-size: 14px;
    font-weight: 600;
  }
  .hint {
    flex: 1;
    font-size: 11px;
    color: rgba(255, 255, 255, 0.45);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .chip {
    border: 0;
    padding: 2px 9px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.1);
    color: #fff;
    font: inherit;
    font-size: 11px;
    cursor: pointer;
  }
  .chip:hover {
    background: rgba(255, 255, 255, 0.2);
  }
  .chip.on {
    background: #fff;
    color: #000;
    font-weight: 600;
  }
  .chip:disabled {
    opacity: 0.4;
  }
  .speed {
    font-size: 11px;
    min-width: 20px;
    text-align: center;
    color: rgba(255, 255, 255, 0.7);
  }
  textarea {
    flex: 1;
    margin-top: 6px;
    padding: 6px 10px;
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 10px;
    background: rgba(255, 255, 255, 0.07);
    color: #fff;
    font: inherit;
    font-size: 12px;
    resize: none;
    outline: none;
  }
  .empty {
    flex: 1;
    display: grid;
    place-items: center;
    font-size: 12px;
    color: rgba(255, 255, 255, 0.5);
  }
  .script {
    flex: 1;
    margin-top: 6px;
    overflow-y: auto;
    scrollbar-width: none;
    mask-image: linear-gradient(transparent, #000 18%, #000 82%, transparent);
  }
  .script p {
    margin: 26px 0 60px;
    font-size: 17px;
    font-weight: 600;
    line-height: 1.45;
    text-align: center;
    white-space: pre-wrap;
  }
</style>
