<script module lang="ts">
  /** One status from Rust's `peek()`: `kind` plus kind-specific fields. */
  export type PeekData = { kind: string; [key: string]: any };
</script>

<script lang="ts">
  import { convertFileSrc, invoke } from "@tauri-apps/api/core";

  // onOpen: the peek was clicked through (opens the app / closes the peek).
  // onAction: one of its buttons was clicked (e.g. "camera", "join").
  let { peek, onOpen, onAction }: { peek: PeekData; onOpen?: () => void; onAction?: (action: string) => void } = $props();

  // Stroke icons, 24×24.
  const ICONS: Record<string, string> = {
    volume: "M11 5 6 9H3v6h3l5 4zM15.5 8.5a5 5 0 0 1 0 7M18.5 5.5a9 9 0 0 1 0 13",
    muted: "M11 5 6 9H3v6h3l5 4zM22 9l-6 6M16 9l6 6",
    sun: "M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8zM12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4",
    mic: "M12 3a3 3 0 0 0-3 3v6a3 3 0 0 0 6 0V6a3 3 0 0 0-3-3zM19 11a7 7 0 0 1-14 0M12 18v3",
    micOff: "M9 9v3a3 3 0 0 0 5.1 2.1M15 9.3V6a3 3 0 0 0-5.7-1.3M19 11a7 7 0 0 1-1.2 3.9M5 11a7 7 0 0 0 10.6 6M12 18v3M3 3l18 18",
    caps: "M12 4 4 13h4v5h8v-5h4zM8 21h8",
    keyboard: "M3 6h18v12H3zM7 10h.01M11 10h.01M15 10h.01M7 14h10",
    bolt: "M13 2 4 14h7l-1 8 9-12h-7z",
    battery: "M3 8h15v8H3zM21 11v2M6 11v2",
    wifi: "M5 12.5a10 10 0 0 1 14 0M8.5 16a5 5 0 0 1 7 0M2 9a15 15 0 0 1 20 0M12 19.5h.01",
    wifiOff: "M2 2l20 20M8.5 16a5 5 0 0 1 7 0M5 12.5a10 10 0 0 1 4.3-2.6M19 12.5a10 10 0 0 0-2.9-2M2 9a15 15 0 0 1 4.6-3M22 9a15 15 0 0 0-11-4M12 19.5h.01",
    headphones: "M3 14v-2a9 9 0 0 1 18 0v2M3 14h4v6H3zM17 14h4v6h-4z",
    speaker: "M6 3h12v18H6zM12 14a2.5 2.5 0 1 0 0 .01M12 7h.01",
    camera: "M15 10l5-3v10l-5-3zM3 7h12v10H3z",
    timer: "M12 8v5l3 2M9 2h6M12 21a8 8 0 1 0 0-16 8 8 0 0 0 0 16z",
    drop: "M12 3s6 6.5 6 11a6 6 0 0 1-12 0c0-4.5 6-11 6-11z",
    check: "M9 11l3 3 8-8M20 12v7H4V5h11",
    calendar: "M4 6h16v14H4zM4 10h16M8 3v4M16 3v4",
    download: "M12 3v12M7 10l5 5 5-5M4 20h16",
    done: "M5 12l5 5L20 7",
    sync: "M20 11a8 8 0 0 0-14.9-3M4 13a8 8 0 0 0 14.9 3M4 4v4h4M20 20v-4h-4",
    alert: "M12 9v4M12 17h.01M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z",
    code: "M8 9l-3 3 3 3M16 9l3 3-3 3M13 6l-2 12",
  };

  type View = { icon: string; tone?: string; title: string; detail?: string; value?: string; level?: number; battery?: number };

  const view = $derived.by((): View | null => {
    const p = peek;
    switch (p.kind) {
      case "volume":
        return { icon: p.muted ? "muted" : "volume", title: p.muted ? "Suara dimatikan" : "Volume", level: p.muted ? 0 : p.level };
      case "brightness":
        return { icon: "sun", tone: "#fbbf24", title: "Kecerahan", level: p.level };
      case "mic":
        return p.muted
          ? { icon: "micOff", tone: "#f87171", title: "Mikrofon dimatikan" }
          : { icon: "mic", tone: "#34d399", title: "Mikrofon aktif" };
      case "caps":
        return { icon: "caps", tone: p.on ? "#38bdf8" : undefined, title: "Caps Lock", value: p.on ? "ON" : "OFF" };
      case "language":
        return { icon: "keyboard", title: "Bahasa keyboard", detail: p.locale, value: String(p.locale).split("-")[0].toUpperCase() };
      case "power":
        return p.charging
          ? { icon: "bolt", tone: "#34d399", title: "Mengisi daya", detail: "Charger tersambung", value: `${p.percent}%` }
          : { icon: "battery", title: "Memakai baterai", detail: "Charger dicabut", value: `${p.percent}%` };
      case "network":
        return p.online
          ? { icon: "wifi", tone: "#38bdf8", title: p.wifi ? "Wi-Fi tersambung" : "Internet tersambung", detail: p.name }
          : { icon: "wifiOff", tone: "#f87171", title: "Koneksi terputus", detail: "Tidak ada internet" };
      case "output":
        return p.headphones
          ? {
              icon: "headphones",
              tone: "#a78bfa",
              title: p.battery != null ? p.name : "Headphone tersambung",
              detail: p.battery != null ? "Tersambung" : p.name,
              battery: p.battery ?? undefined,
            }
          : { icon: "speaker", title: "Audio keluar lewat", detail: p.name };
      case "earbuds-low":
        return { icon: "headphones", tone: "#fb923c", title: `Baterai tinggal ${p.battery}%`, detail: p.name, battery: p.battery };
      case "update-info":
        return p.state === "checking"
          ? { icon: "sync", tone: "#38bdf8", title: "Mengecek update…", detail: `Notchy v${p.version}` }
          : p.state === "latest"
            ? { icon: "done", tone: "#34d399", title: "Sudah versi terbaru", detail: `Notchy v${p.version}` }
            : p.state === "dev"
              ? { icon: "code", title: "Versi pengembang", detail: "Update lewat build, bukan updater" }
              : { icon: "alert", tone: "#f87171", title: "Gagal mengecek update", detail: "Cek koneksi internet" };
      case "camera":
        return { icon: "camera", tone: "#34d399", title: "Kamera dipakai", detail: p.app };
      case "mic-in-use":
        return { icon: "mic", tone: "#fb923c", title: "Mikrofon dipakai", detail: p.app };
      case "timer":
        return p.phase === "break"
          ? { icon: "timer", tone: "#34d399", title: "Waktunya istirahat", detail: "Sesi fokus selesai" }
          : { icon: "timer", tone: "#fb7185", title: "Istirahat selesai", detail: "Siap fokus lagi?" };
      case "water":
        return {
          icon: "drop",
          tone: "#38bdf8",
          title: "Waktunya minum",
          detail: `${(p.today / 1000).toLocaleString("id-ID")} dari ${(p.goal / 1000).toLocaleString("id-ID")} L hari ini`,
        };
      case "todo":
        return { icon: "check", tone: "#fb923c", title: p.text, detail: `Pengingat ${p.time}` };
      case "calendar": {
        const at = new Date(p.start).toLocaleTimeString("id-ID", { hour: "2-digit", minute: "2-digit" }).replace(".", ":");
        return { icon: "calendar", tone: "#fb923c", title: p.title, detail: p.before ? `Mulai ${p.before} menit lagi · ${at}` : `Dimulai sekarang · ${at}` };
      }
      default:
        return null;
    }
  });

  const open = (reveal: boolean) => {
    invoke("open_screenshot", { path: peek.path, reveal });
    onOpen?.();
  };
</script>

{#if peek.kind === "notification"}
  <!-- A Windows notification, mirrored: app icon, app name, title and text. Click opens the app. -->
  <button class="note" onclick={() => onOpen?.()} title="Buka {peek.app}">
    {#if peek.icon}
      <img class="app-icon" src={peek.icon} alt="" />
    {:else}
      <span class="app-icon fallback">{String(peek.app).slice(0, 1)}</span>
    {/if}
    <div class="text">
      <div class="app">{peek.app}</div>
      <div class="title one">{peek.title}</div>
      {#if peek.body}<div class="detail">{peek.body}</div>{/if}
    </div>
  </button>
{:else if peek.kind === "screenshot"}
  <div class="shot">
    <button class="thumb" onclick={() => open(false)} aria-label="Buka screenshot">
      <img src={convertFileSrc(peek.path)} alt="" />
    </button>
    <div class="text">
      <div class="title">Screenshot tersimpan</div>
      <div class="detail">{peek.name}</div>
      <div class="actions">
        <button onclick={() => open(false)}>Buka</button>
        <button onclick={() => open(true)}>Lihat di folder</button>
      </div>
    </div>
  </div>
{:else if peek.kind === "download-done"}
  <!-- A download finished: click opens it. -->
  <button class="note" onclick={() => onOpen?.()}>
    <span class="icon" style:color="#34d399" style:background="#34d39922">
      <svg viewBox="0 0 24 24"><path d={ICONS.done} /></svg>
    </span>
    <div class="text">
      <div class="title one">{peek.name}</div>
      <div class="detail">Selesai diunduh · klik untuk buka</div>
    </div>
  </button>
{:else if peek.kind === "meeting"}
  <!-- A meeting with a link starts soon: check the camera, or join. -->
  <div class="row">
    <span class="icon" style:color="#fb923c" style:background="#fb923c22">
      <svg viewBox="0 0 24 24"><path d={ICONS.calendar} /></svg>
    </span>
    <div class="text">
      <div class="title one">{peek.title}</div>
      <div class="detail">Mulai 2 menit lagi</div>
    </div>
    <div class="actions inline">
      <button onclick={() => onAction?.("camera")}>Cek kamera</button>
      {#if peek.link}<button class="primary" onclick={() => onAction?.("join")}>Join</button>{/if}
    </div>
  </div>
{:else if peek.kind === "update"}
  <!-- A new version is downloaded; clicking installs it (Notchy restarts in a few seconds). -->
  <button class="note" onclick={() => onOpen?.()}>
    <span class="icon" style:color="#38bdf8" style:background="#38bdf822">
      <svg viewBox="0 0 24 24"><path d={ICONS.download} /></svg>
    </span>
    <div class="text">
      <div class="title">Notchy v{peek.version} siap</div>
      <div class="detail">Klik untuk pasang, Notchy mulai ulang sebentar</div>
    </div>
  </button>
{:else if view}
  <div class="row">
    <span class="icon" style:color={view.tone ?? "#fff"} style:background={view.tone ? `${view.tone}22` : "rgba(255,255,255,.1)"}>
      <svg viewBox="0 0 24 24"><path d={ICONS[view.icon]} /></svg>
    </span>
    <div class="text">
      <div class="title">{view.title}</div>
      {#if view.detail}<div class="detail">{view.detail}</div>{/if}
    </div>
    {#if view.level !== undefined}
      <div class="level"><span style:width="{Math.round(view.level * 100)}%"></span></div>
      <span class="value">{Math.round(view.level * 100)}</span>
    {:else if view.battery !== undefined}
      <div class="level short">
        <span style:width="{view.battery}%" style:background={view.battery < 15 ? "#fb923c" : "#34d399"}></span>
      </div>
      <span class="value">{view.battery}%</span>
    {:else if view.value}
      <span class="value">{view.value}</span>
    {/if}
  </div>
{/if}

<style>
  .row {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 0 22px 0 16px;
  }
  .icon {
    flex: none;
    width: 34px;
    height: 34px;
    border-radius: 50%;
    display: grid;
    place-items: center;
  }
  svg {
    width: 18px;
    height: 18px;
    fill: none;
    stroke: currentColor;
    stroke-width: 2;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .text {
    flex: 1;
    min-width: 0;
  }
  .title {
    font-size: 13px;
    font-weight: 600;
  }
  .detail {
    margin-top: 2px;
    font-size: 11.5px;
    color: rgba(255, 255, 255, 0.55);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .level {
    flex: none;
    width: 92px;
    height: 5px;
    border-radius: 3px;
    background: rgba(255, 255, 255, 0.15);
    overflow: hidden;
  }
  .level span {
    display: block;
    height: 100%;
    border-radius: 3px;
    background: #fff;
    transition: width 0.15s ease-out;
  }
  .level.short {
    width: 56px;
  }
  .actions.inline {
    flex: none;
    margin-top: 0;
  }
  .actions button.primary {
    background: #fff;
    color: #000;
    font-weight: 600;
  }
  .value {
    flex: none;
    min-width: 24px;
    text-align: right;
    font-size: 13px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }

  .note {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 0 24px 0 18px;
    border: 0;
    background: none;
    color: inherit;
    font: inherit;
    text-align: left;
    cursor: pointer;
    border-radius: inherit;
  }
  .note:hover {
    background: rgba(255, 255, 255, 0.05);
  }
  .app-icon {
    flex: none;
    width: 38px;
    height: 38px;
    border-radius: 10px;
    object-fit: contain;
  }
  .app-icon.fallback {
    display: grid;
    place-items: center;
    background: #2a2e3a;
    font-weight: 700;
  }
  .app {
    font-size: 10.5px;
    color: rgba(255, 255, 255, 0.5);
  }
  .one {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .shot {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 0 18px 4px 14px;
  }
  .thumb {
    flex: none;
    width: 116px;
    height: 68px;
    padding: 0;
    border: 0;
    border-radius: 10px;
    overflow: hidden;
    cursor: pointer;
    background: #1b1f2e;
  }
  .thumb img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }
  .actions {
    display: flex;
    gap: 6px;
    margin-top: 8px;
  }
  .actions button {
    padding: 4px 10px;
    border: 0;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.14);
    color: #fff;
    font: inherit;
    font-size: 11.5px;
    cursor: pointer;
  }
  .actions button:hover {
    background: rgba(255, 255, 255, 0.24);
  }
</style>
