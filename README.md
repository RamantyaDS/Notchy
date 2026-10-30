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

🎵 **Music from any player.** Spotify, YouTube in the browser, anything that shows up in Windows' media controls. Cover, a live equalizer, play/pause/skip, and the cover's color tints the notch. Hover the progress bar to see the time, hover a long title to read all of it.

🎤 **Live lyrics.** Synced lyrics appear line by line, even in the small notch while you work.

🔔 **Notifications, in the notch.** WhatsApp, Teams, Outlook… mirrored within half a second. Click one and it opens *that* chat or email, just like the real notification.

📞 **Calls and meetings.** A call pill with mute for Teams, WhatsApp, Zoom and Meet; a meeting reminder with a quick camera check and a **Join** button.

🧩 **Two things at once.** Music, a timer, a call or a download can sit side by side in the small notch; swipe to switch which one leads.

⏱️ **Focus timer**, 💧 **water reminder**, ✅ **to-do list** with reminders, 📅 **calendar** reminders from your calendar link.

📁 **Shelf** to park files you'll drag somewhere else, 📋 **clipboard history** (text and images), ⬇️ **download progress** from your browser, ⚡ **shortcuts** page (lock, sleep, screenshot, your own apps and links), and drop a file on the notch to **share** it.

🔊 **Quick status:** volume, brightness, Caps Lock, keyboard language, charging, Wi-Fi, headphones and earbuds battery, camera and mic in use, mic mute, screenshot preview, weather, one-click audio output switch, and live **network speed** (↓/↑) in the open notch, optionally also in the small notch during heavy downloads or uploads.

🖱️ **Gestures:** scroll on the notch for volume, swipe sideways to switch, press and hold to keep it open.

🎛️ **Only what you use.** Turn pages (music, timer, water, to-do, Shelf, clipboard, shortcuts) on or off in Settings.

🫥 **Stays out of the way.** It opens only when the cursor stops on it (not when you sweep past to a browser tab), tucks into a thin line when there's a tab or button right under it, hides for full-screen video and games, and dozes off when you're idle.

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

1. Download `Notchy_x.y.z_x64-setup.exe` from the [latest release](https://github.com/RamantyaDS/Notchy/releases/latest). Only download it from there.
2. Run it. Windows may warn you, because Notchy is new and not code-signed:
   - **In the browser** (*"isn't commonly downloaded"*): open your downloads, click **…** next to the file → **Keep** → **Show more** → **Keep anyway**.
   - **When you open it** (*"Windows protected your PC"*): click **More info** → **Run anyway**.
3. That's it. No admin rights, no account. Notchy **updates itself** from now on.

### Portable (no install)

Can't run installers, e.g. on a work laptop? Download `Notchy_x.y.z_x64-portable.zip` from the same release, unzip it anywhere and run `notchy.exe`. Settings and data still live in `%APPDATA%\Notchy`.

The portable copy **doesn't update itself**: when a new version is out, the notch tells you, and clicking it opens the release page. To remove it, untick **Start with Windows** in the tray menu, exit, and delete the folder.

**Tip:** turn on Windows' *Do not disturb* (there's a button in Notchy's Settings) so notifications pop up only in the notch, not twice.

<details>
<summary>Check that your download is the real one</summary>

Every release page lists the **SHA-256** fingerprint of the installer and the portable zip. In PowerShell:

```bash
Get-FileHash .\Notchy_x.y.z_x64-setup.exe -Algorithm SHA256
```

(or `.\Notchy_x.y.z_x64-portable.zip`)

If the hash doesn't match the one on the release page, don't run the file.
</details>

## FAQ

<details><summary>How do I update?</summary>

You don't have to: Notchy checks when it starts, every hour and after your laptop wakes up, then installs the new version. To check right now, right-click the tray icon → **Cek update**. The portable copy only tells you a new version is out; download the new zip from the release page.
</details>

<details><summary>The installer won't run on my work laptop</summary>

Company laptops often block programs that aren't on an approved list, with a message like *"Windows cannot access the specified device, path, or file"*. Try the **portable zip**. If that's blocked too, ask your IT team to allow Notchy; the SHA-256 on the release page lets them check the exact file.
</details>

<details><summary>Notifications show up twice</summary>

Windows still shows its own banner. Turn on **Do not disturb** (Settings → Notifications, or the button in Notchy's Settings); notifications then appear only in the notch and still land in the notification center.
</details>

<details><summary>A song has no lyrics, or the lyrics are off</summary>

Lyrics come from [LRCLIB](https://lrclib.net), a free community database. Some songs aren't there yet, and versions with a different length (live, remix) may not line up.
</details>

<details><summary>Which music players work?</summary>

Anything that shows up in Windows' media controls (the box above the volume slider): Spotify, YouTube and other sites in Edge/Chrome, Apple Music, the Media Player app and more.
</details>

<details><summary>My laptop has a thick top bezel / I don't like the eyes</summary>

Right-click the tray icon: **Notch minimal** turns the idle notch into a thin black line, and **Mata (maskot)** turns the eyes off. Things like music, alerts and hover still work.
</details>

<details><summary>How do I quit, or stop it starting with Windows?</summary>

Right-click the tray icon → **Exit**, or untick **Start with Windows**. Uninstall anytime from Windows Settings → Apps.
</details>

<details><summary>My antivirus flagged it</summary>

New, unsigned apps sometimes get flagged. Check the SHA-256 against the release page (above). Notchy listens only for volume, brightness, Caps Lock and language keys to show their overlays; it never records what you type. If you'd rather it didn't watch the keyboard at all, turn off the Caps Lock and keyboard-language peeks in Settings → **Keyboard & sistem** (applies after restarting Notchy).
</details>

## Feedback

Found a bug or have an idea? [Open an issue](https://github.com/RamantyaDS/Notchy/issues/new). If something went wrong, attaching `%APPDATA%\Notchy\notchy.log` helps a lot (it only has start/stop times and errors).

## Light and private

- **~1–2% CPU at rest**, a few % while music animates.
- **No account, no analytics.** Your to-dos, water log and settings stay in `%APPDATA%\Notchy`.
- **Careful with clicks.** Programs and scripts on the Shelf or in Downloads are shown in Explorer, never run; links from notifications open only for web, mail and the usual chat/meeting apps.
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

**Laptop Windows-mu nggak punya notch? Sekarang punya.** Notchy adalah pulau hitam kecil di atas layar yang menampilkan musik beserta lirik yang ikut jalan, notifikasi (klik langsung membuka chat-nya), panggilan dan meeting (mute, cek kamera, Join), timer fokus, pengingat minum, to-do, kalender, Shelf file, riwayat clipboard, progres unduhan, halaman pintasan, kecepatan internet, dan status sistem. Halaman yang tidak dipakai bisa dimatikan di Settings. Membesar saat kursor berhenti di atasnya, menyingkir saat menghalangi.

Unduh installer di [Releases](https://github.com/RamantyaDS/Notchy/releases/latest). Kalau muncul peringatan SmartScreen, klik **More info → Run anyway**. Setelah terpasang, Notchy update sendiri. Data tetap di laptopmu, tanpa akun.

Tidak bisa menjalankan installer, misalnya di laptop kantor? Unduh **`Notchy_x.y.z_x64-portable.zip`** dari rilis yang sama, ekstrak, lalu jalankan `notchy.exe`. Versi portable tidak update sendiri: kalau ada versi baru, notch memberi tahu dan klik membuka halaman rilis.

Ada bug atau ide? [Buat issue](https://github.com/RamantyaDS/Notchy/issues/new), dan kalau bisa lampirkan `%APPDATA%\Notchy\notchy.log`.
</details>
