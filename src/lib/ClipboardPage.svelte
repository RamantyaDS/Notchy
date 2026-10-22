<script lang="ts">
  // Recent copies (kept in memory by Rust, gone after a restart). Click to copy again.
  import { invoke } from "@tauri-apps/api/core";

  export type ClipEntry = { id: number; kind: "text" | "image"; preview: string; width?: number; height?: number };

  let { items }: { items: ClipEntry[] } = $props();
  let copied = $state<number | null>(null);

  async function copy(item: ClipEntry) {
    if (await invoke<boolean>("clipboard_copy", { id: item.id })) {
      copied = item.id;
      setTimeout(() => (copied = null), 1200);
    }
  }
</script>

<div class="page">
  <div class="header">
    <span class="title">Clipboard</span>
    <span class="hint">{items.length ? `${items.length} terakhir · klik untuk salin` : "Belum ada yang disalin"}</span>
    {#if items.length}<button class="clear" onclick={() => invoke("clipboard_clear")}>Bersihkan</button>{/if}
  </div>

  {#if items.length === 0}
    <div class="empty">Teks dan gambar yang kamu copy muncul di sini.<br />Isi dari password manager tidak disimpan.</div>
  {:else}
    <div class="list scroll">
      {#each items as item (item.id)}
        <button class="item" class:done={copied === item.id} onclick={() => copy(item)} title={item.kind === "text" ? item.preview : "Gambar"}>
          {#if item.kind === "image"}
            <img src={item.preview} alt="" />
            <span class="text">Gambar · {item.width} × {item.height}</span>
          {:else}
            <span class="text">{item.preview}</span>
          {/if}
          {#if copied === item.id}<span class="ok">Disalin</span>{/if}
        </button>
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
  .clear {
    font-size: 11.5px;
    color: rgba(255, 255, 255, 0.6);
  }
  .clear:hover {
    color: #fff;
  }
  .empty {
    flex: 1;
    display: grid;
    place-items: center;
    text-align: center;
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
    gap: 8px;
    padding: 5px 10px;
    border-radius: 9px;
    background: rgba(255, 255, 255, 0.07);
    text-align: left;
  }
  .item:hover {
    background: rgba(255, 255, 255, 0.14);
  }
  .item.done {
    background: rgba(52, 211, 153, 0.18);
  }
  img {
    flex: none;
    height: 18px;
    max-width: 32px;
    border-radius: 3px;
    object-fit: cover;
  }
  .text {
    flex: 1;
    min-width: 0;
    font-size: 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .ok {
    flex: none;
    font-size: 11px;
    color: #34d399;
    font-weight: 600;
  }
</style>
