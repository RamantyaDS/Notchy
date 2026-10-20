<p align="center">
  <img src="docs/banner.png" alt="Notchy, a Dynamic Island for Windows" width="100%" />
</p>

<p align="center">
  <a href="https://github.com/RamantyaDS/Notchy/releases/latest"><img alt="Latest release" src="https://img.shields.io/github/v/release/RamantyaDS/Notchy?style=flat-square&color=000000&label=download" /></a>
  <img alt="Windows 10 and 11" src="https://img.shields.io/badge/Windows-10%20%26%2011-000000?style=flat-square&logo=windows" />
  <img alt="Built with Tauri" src="https://img.shields.io/badge/Tauri%202-Rust%20%2B%20Svelte-000000?style=flat-square&logo=tauri" />
  <a href="https://github.com/RamantyaDS/Notchy/releases"><img alt="Downloads" src="https://img.shields.io/github/downloads/RamantyaDS/Notchy/total?style=flat-square&color=000000" /></a>
</p>

<p align="center"><b>Your Windows laptop doesn't have a notch? Now it does.</b><br/>
A small black island at the top of your screen that shows what's happening (music, lyrics, notifications, timers) and grows when you hover it.</p>

<p align="center">
  <a href="https://github.com/RamantyaDS/Notchy/releases/latest"><b>⬇️ Download for Windows</b></a>
</p>

---

<table>
<tr>
<td width="38%" valign="top"><img src="docs/demo.gif" alt="Notchy in action" width="100%" /></td>
<td valign="top">

### What it does

🎵 **Music from any player.** Spotify, YouTube in the browser, anything that shows up in Windows' media controls. Cover, progress, play/pause/skip, and the cover's color tints the notch.

🎤 **Live lyrics.** Synced lyrics appear line by line, even in the small notch while you work.

🔔 **Notifications, in the notch.** WhatsApp, Teams, Outlook… mirrored within half a second. Click one and it opens *that* chat or email, just like the real notification.

⏱️ **Focus timer**, 💧 **water reminder**, ✅ **to-do list** with reminders, 📁 **Shelf** to park files you'll drag somewhere else, 📅 **calendar** reminders from your calendar link.

🔊 **Quick status:** volume, brightness, Caps Lock, keyboard language, charging, Wi-Fi, headphones, camera and mic in use, mic mute, screenshot preview, weather, one-click audio output switch.

🫥 **Stays out of the way.** It tucks into a thin line when there's a tab or button right under it, hides for full-screen video and games, and dozes off when you're idle.

</td>
</tr>
</table>

<p align="center">
  <img src="docs/player.png" width="49%" alt="Music player with lyrics" />
  <img src="docs/timer.png" width="49%" alt="Focus timer" />
  <img src="docs/todo.png" width="49%" alt="To-do list" />
  <img src="docs/water.png" width="49%" alt="Water reminder" />
</p>

## Install

1. Download `Notchy_x.y.z_x64-setup.exe` from the [latest release](https://github.com/RamantyaDS/Notchy/releases/latest).
2. Run it. Windows may say *"Windows protected your PC"* because the app isn't code-signed: click **More info → Run anyway**.
3. That's it. No admin rights, no account. Notchy **updates itself** from now on.

**Tip:** turn on Windows' *Do not disturb* (there's a button in Notchy's Settings) so notifications pop up only in the notch, not twice.

## Light and private

- **~1–2% CPU at rest**, a few % while music animates.
- **No account, no analytics.** Your to-dos, water log and settings stay in `%APPDATA%\Notchy`.
- It only goes online for lyrics ([LRCLIB](https://lrclib.net)), weather ([Open-Meteo](https://open-meteo.com)), your own calendar link if you add one, and checking for updates here on GitHub.

## Built with

[Tauri 2](https://tauri.app) · Rust ([windows-rs](https://github.com/microsoft/windows-rs)) for everything that talks to Windows · [Svelte 5](https://svelte.dev) for the notch itself · WebView2.

<details>
<summary>Build it yourself</summary>

Needs [Node.js](https://nodejs.org), [Rust](https://rustup.rs) and the MSVC build tools.

```bash
npm install
npm run tauri dev      # run with hot reload
npm run tauri build    # installer in src-tauri/target/release/bundle/nsis
```

Design notes and every decision behind the app are in [`notchy.md`](notchy.md) (in Indonesian).
</details>

---

<details>
<summary>🇮🇩 Bahasa Indonesia</summary>

**Laptop Windows-mu nggak punya notch? Sekarang punya.** Notchy adalah pulau hitam kecil di atas layar yang menampilkan musik beserta lirik yang ikut jalan, notifikasi (klik langsung membuka chat-nya), timer fokus, pengingat minum, to-do, Shelf file, kalender, dan status sistem. Membesar saat di-hover, menyingkir saat menghalangi.

Unduh installer di [Releases](https://github.com/RamantyaDS/Notchy/releases/latest). Kalau muncul peringatan SmartScreen, klik **More info → Run anyway**. Setelah terpasang, Notchy update sendiri. Data tetap di laptopmu, tanpa akun.
</details>
