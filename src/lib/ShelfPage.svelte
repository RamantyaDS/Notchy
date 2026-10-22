<script lang="ts">
  import { convertFileSrc, invoke } from "@tauri-apps/api/core";
  import { startDrag } from "@crabnebula/tauri-plugin-drag";
  import type { Shelf, ShelfItem } from "./productivity.svelte";

  // dropTarget: which zone the dragged file is over ("shelf" or "share").
  let { shelf, dropping, dropTarget = null }: { shelf: Shelf; dropping: boolean; dropTarget?: string | null } = $props();

  const IMAGE = /\.(png|jpe?g|gif|webp|bmp)$/i;
  const ext = (name: string) => (name.includes(".") ? name.split(".").pop()!.slice(0, 4).toUpperCase() : "");

  /** A small file badge to show under the cursor while dragging out. */
  function dragIcon(item: ShelfItem): string {
    const c = document.createElement("canvas");
    c.width = c.height = 64;
    const g = c.getContext("2d")!;
    g.fillStyle = "#2a2e3a";
    g.beginPath();
    g.roundRect(8, 4, 48, 56, 10);
    g.fill();
    g.fillStyle = "#fff";
    g.font = "bold 14px Segoe UI";
    g.textAlign = "center";
    g.fillText(item.dir ? "📁" : ext(item.name) || "FILE", 32, 38);
    return c.toDataURL("image/png");
  }

  // Press and move: drag the file out (to a chat, an email, a folder). Press and release: open it.
  let pressed: { item: ShelfItem; x: number; y: number } | null = null;
  function down(e: PointerEvent, item: ShelfItem) {
    if (e.button === 0) pressed = { item, x: e.clientX, y: e.clientY };
  }
  function move(e: PointerEvent) {
    if (!pressed || Math.hypot(e.clientX - pressed.x, e.clientY - pressed.y) < 6) return;
    const { item } = pressed;
    pressed = null;
    startDrag({ item: [item.path], icon: dragIcon(item) });
  }
  function up() {
    if (pressed) invoke("open_path", { path: pressed.item.path });
    pressed = null;
  }

  const size = (bytes: number) =>
    bytes >= 1e6 ? `${(bytes / 1e6).toFixed(1)} MB` : bytes >= 1e3 ? `${Math.round(bytes / 1e3)} KB` : `${bytes} B`;
</script>

<svelte:window onpointermove={move} onpointerup={() => (pressed = null)} />

<div class="page" class:dropping>
  <div class="header">
    <span class="title">Shelf</span>
    <span class="hint">Tarik keluar kapan saja · klik untuk membuka</span>
    {#if shelf.items.length}<button class="clear" onclick={() => shelf.clear()}>Bersihkan</button>{/if}
  </div>

  {#if dropping}
    <!-- Two places to drop: keep it on the shelf, or hand it to Windows' Share window. -->
    <div class="zones">
      <div class="zone" class:hot={dropTarget !== "share"} data-drop="shelf">
        <span class="tray">📥</span><strong>Simpan di Shelf</strong>
      </div>
      <div class="zone" class:hot={dropTarget === "share"} data-drop="share">
        <svg viewBox="0 0 24 24"><path d="M4 12v7h16v-7M12 3v12M8 7l4-4 4 4" /></svg><strong>Bagikan</strong>
      </div>
    </div>
  {:else if shelf.items.length === 0}
    <div class="zone">
      <span class="tray">📥</span>
      <strong>Tarik file ke sini</strong>
    </div>
  {:else}
    <div class="items scroll">
      {#each shelf.items as item (item.path)}
        <div class="item" role="button" tabindex="-1" title={item.name}
          onpointerdown={(e) => down(e, item)} onpointerup={up}>
          {#if IMAGE.test(item.name)}
            <img src={convertFileSrc(item.path)} alt="" draggable="false" />
          {:else}
            <span class="badge">{item.dir ? "📁" : ext(item.name) || "FILE"}</span>
          {/if}
          <span class="name">{item.name}</span>
          {#if !item.dir}<span class="size">{size(item.size)}</span>{/if}
          <button class="remove" onpointerdown={(e) => e.stopPropagation()} onclick={() => shelf.remove(item.path)} aria-label="Hapus dari shelf">×</button>
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
  .clear {
    font-size: 11.5px;
    color: rgba(255, 255, 255, 0.6);
  }
  .clear:hover {
    color: #fff;
  }
  .zone {
    flex: 1;
    margin-top: 8px;
    border: 1.5px dashed rgba(255, 255, 255, 0.2);
    border-radius: 14px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
    font-size: 13px;
    transition:
      border-color 0.2s,
      background 0.2s;
  }
  .zones {
    flex: 1;
    display: flex;
    gap: 10px;
  }
  .zones .zone {
    flex: 1;
  }
  .zone.hot {
    border-color: #38bdf8;
    background: rgba(56, 189, 248, 0.12);
    color: #bae6fd;
  }
  .zone svg {
    width: 18px;
    height: 18px;
    fill: none;
    stroke: currentColor;
    stroke-width: 2;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .tray {
    font-size: 20px;
  }
  .items {
    flex: 1;
    display: flex;
    gap: 8px;
    margin-top: 8px;
    overflow-x: auto;
    scrollbar-width: none;
  }
  .item {
    position: relative;
    flex: none;
    width: 74px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 3px;
    padding: 6px 4px;
    border-radius: 10px;
    cursor: grab;
  }
  .item:hover {
    background: rgba(255, 255, 255, 0.08);
  }
  img,
  .badge {
    width: 44px;
    height: 44px;
    border-radius: 8px;
    object-fit: cover;
  }
  .badge {
    display: grid;
    place-items: center;
    background: #2a2e3a;
    font-size: 11px;
    font-weight: 700;
  }
  .name {
    width: 100%;
    text-align: center;
    font-size: 11px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .size {
    font-size: 10px;
    color: rgba(255, 255, 255, 0.45);
  }
  .remove {
    position: absolute;
    top: 2px;
    right: 4px;
    font-size: 13px;
    color: rgba(255, 255, 255, 0.5);
    opacity: 0;
  }
  .item:hover .remove {
    opacity: 1;
  }
</style>
