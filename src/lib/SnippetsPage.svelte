<script lang="ts">
  // Text typed often (an address, a phone number, an email): one click copies it.
  // Kept in %APPDATA%\Notchy\snippets.json.
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { t } from "./i18n";

  let { onTyping }: { onTyping: (on: boolean) => void } = $props();
  let items = $state<string[]>([]);
  let draft = $state("");
  let adding = $state(false);
  let copied = $state<number | null>(null);

  onMount(async () => {
    const saved = await invoke<string[] | null>("load_data", { name: "snippets" }).catch(() => null);
    items = Array.isArray(saved) ? saved.filter((s) => typeof s === "string") : [];
  });
  const persist = () => invoke("save_data", { name: "snippets", value: items });

  async function copy(i: number) {
    if (await invoke<boolean>("clipboard_set_text", { text: items[i] })) {
      copied = i;
      setTimeout(() => (copied = null), 900);
    }
  }
  function add() {
    const text = draft.trim();
    if (text) {
      items = [...items, text].slice(-30);
      persist();
    }
    draft = "";
    adding = false;
    onTyping(false);
  }
  function remove(i: number) {
    items = items.filter((_, j) => j !== i);
    persist();
  }
  function startAdding() {
    adding = true;
    onTyping(true);
  }
</script>

<div class="page">
  <div class="header">
    <span class="title">{t("snip.title")}</span>
    <span class="hint">{t("snip.hint")}</span>
    {#if !adding}<button class="add" onclick={startAdding}>+ {t("snip.add")}</button>{/if}
  </div>

  {#if adding}
    <form class="new" onsubmit={(e) => (e.preventDefault(), add())}>
      <!-- svelte-ignore a11y_autofocus -->
      <input bind:value={draft} placeholder={t("snip.placeholder")} autofocus maxlength="300"
        onkeydown={(e) => e.key === "Escape" && ((draft = ""), add())} onblur={add} />
    </form>
  {/if}

  {#if items.length === 0 && !adding}
    <div class="empty">{t("snip.empty")}</div>
  {:else}
    <div class="list scroll">
      {#each items as text, i (i + text)}
        <div class="item" class:done={copied === i}>
          <button class="text" onclick={() => copy(i)} title={text}>{text}</button>
          {#if copied === i}<span class="ok">{t("peek.copied")}</span>{/if}
          <button class="remove" onclick={() => remove(i)} aria-label={t("snip.remove")}>×</button>
        </div>
      {/each}
    </div>
  {/if}
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
    flex: 1;
    font-size: 11px;
    color: rgba(255, 255, 255, 0.45);
  }
  button {
    border: 0;
    background: none;
    color: #fff;
    font: inherit;
    cursor: pointer;
  }
  .add {
    font-size: 11.5px;
    color: rgba(255, 255, 255, 0.7);
  }
  .add:hover {
    color: #fff;
  }
  .new input {
    width: 100%;
    margin-top: 6px;
    padding: 5px 10px;
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 9px;
    background: rgba(255, 255, 255, 0.08);
    color: #fff;
    font: inherit;
    font-size: 12px;
    outline: none;
    box-sizing: border-box;
  }
  .empty {
    flex: 1;
    display: grid;
    place-items: center;
    text-align: center;
    padding: 0 20px;
    font-size: 12px;
    line-height: 1.5;
    color: rgba(255, 255, 255, 0.5);
  }
  .list {
    flex: 1;
    margin-top: 6px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    overflow-y: auto;
    scrollbar-width: none;
  }
  .item {
    flex: none;
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0 6px 0 0;
    border-radius: 9px;
    background: rgba(255, 255, 255, 0.07);
  }
  .item:hover {
    background: rgba(255, 255, 255, 0.14);
  }
  .item.done {
    background: rgba(52, 211, 153, 0.18);
  }
  .text {
    flex: 1;
    min-width: 0;
    padding: 5px 10px;
    text-align: left;
    font-size: 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .remove {
    opacity: 0;
    font-size: 14px;
    color: rgba(255, 255, 255, 0.5);
  }
  .item:hover .remove {
    opacity: 1;
  }
  .ok {
    font-size: 11px;
    color: #34d399;
    font-weight: 600;
  }
</style>
