<script lang="ts">
  // Camera check before a call: a small live preview in the notch. The camera
  // is opened only while this is shown and released (light off) when it closes.
  import { onDestroy, onMount } from "svelte";

  let { title = "", link = "", onDone, onJoin }: { title?: string; link?: string; onDone: () => void; onJoin: () => void } = $props();

  let video: HTMLVideoElement;
  let stream: MediaStream | null = null;
  let error = $state("");

  onMount(async () => {
    try {
      stream = await navigator.mediaDevices.getUserMedia({ video: { width: 640, height: 360 }, audio: false });
      video.srcObject = stream;
    } catch (e) {
      error = (e as DOMException)?.name === "NotAllowedError" ? "Akses kamera ditolak" : "Kamera tidak bisa dibuka";
    }
  });
  onDestroy(() => stream?.getTracks().forEach((t) => t.stop()));
</script>

<div class="mirror">
  <div class="cam">
    <video bind:this={video} autoplay muted playsinline></video>
    {#if error}<span class="err">{error}</span>{/if}
  </div>
  <div class="side">
    <div class="title"><span class="live"></span>Cek kamera</div>
    <div class="detail">{title || "Pastikan wajah terlihat jelas"}</div>
    <div class="actions">
      <button class="primary" onclick={onDone}>Selesai</button>
      {#if link}<button onclick={onJoin}>Join</button>{/if}
    </div>
  </div>
</div>

<style>
  .mirror {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 12px 18px 14px;
  }
  .cam {
    position: relative;
    flex: none;
    width: 176px;
    height: 99px;
    border-radius: 14px;
    overflow: hidden;
    background: #111;
  }
  video {
    width: 100%;
    height: 100%;
    object-fit: cover;
    /* Like a mirror. */
    transform: scaleX(-1);
  }
  .err {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    font-size: 11.5px;
    color: rgba(255, 255, 255, 0.6);
  }
  .side {
    flex: 1;
    min-width: 0;
  }
  .title {
    font-size: 13px;
    font-weight: 600;
  }
  .live {
    display: inline-block;
    width: 7px;
    height: 7px;
    margin-right: 6px;
    border-radius: 50%;
    background: #ef4444;
  }
  .detail {
    margin-top: 3px;
    font-size: 11.5px;
    color: rgba(255, 255, 255, 0.55);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .actions {
    display: flex;
    gap: 6px;
    margin-top: 12px;
  }
  button {
    padding: 5px 12px;
    border: 0;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.14);
    color: #fff;
    font: inherit;
    font-size: 12px;
    cursor: pointer;
  }
  button.primary {
    background: #fff;
    color: #000;
    font-weight: 600;
  }
</style>
