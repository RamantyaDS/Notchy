// Lightweight, reactive, type-safe i18n module for Notchy

export type Locale = "en" | "id";

export const translations = {
  en: {
    // Tabs
    "tabs.display": "Display",
    "tabs.system": "System",
    "tabs.focus": "Focus",
    "tabs.media": "Media",

    // Header & General
    "settings.title": "Notchy Settings",
    "settings.subtitle": "Fluent Experience",
    "settings.searchPlaceholder": "Search settings (screen, bluetooth, spring, pomodoro...)",
    "settings.language": "Language / Bahasa",
    "settings.languageDesc": "Choose display language for Notchy interface",
    "settings.langEn": "English (US)",
    "settings.langId": "Bahasa Indonesia",
    "settings.saved": "Saved",
    "settings.savedToast": "Settings saved automatically",
    "settings.loading": "Loading Notchy configuration...",

    // Display & Layout
    "settings.monitorTitle": "Multi-Monitor Placement",
    "settings.monitorDesc": "Choose on which monitor Notchy appears automatically.",
    "settings.monitorPrimary": "Primary Display (Default)",
    "settings.monitorSecondary": "Secondary Display",
    "settings.monitorFollow": "Follow Active Mouse / Cursor",

    "settings.springTitle": "Dynamic Island Animation (Spring Physics)",
    "settings.springDesc": "Adaptive spring physics curve as Dynamic Island expands and reacts to interactions.",
    "settings.springOrganic": "Organic Spring (Bouncy iOS / macOS Style)",
    "settings.springSnappy": "Snappy (Fast & Responsive)",
    "settings.springSmooth": "Smooth (Gentle & Elegant)",
    "settings.springOff": "Off (Instant / No Spring)",

    "settings.accentTitle": "Windows Accent Color Sync (Dynamic Accent)",
    "settings.accentDesc": "Elegant border glow highlight when there is an active notification or information peek.",
    "settings.accentCurrentMode": "Current mode:",
    "settings.accentDwmActive": "Windows Accent DWM Sync",
    "settings.accentDefaultNeutral": "Notchy Default (Neutral)",
    "settings.accentHintActive": "Border glow color automatically follows the accent color chosen in Windows Personalization.",
    "settings.accentHintDefault": "Border glow color uses Notchy's default ambient monochrome theme.",

    "settings.mascotStyleTitle": "Mascot Expressions & Notch Style",
    "settings.mascotStyleDesc": "Interactive character eyes when idle and minimal notch display options.",
    "settings.mascotTitle": "Active Mascot Eyes",
    "settings.mascotDesc": "Expressive eyes peek out and follow your cursor when laptop is idle.",
    "settings.minimalTitle": "Minimal Notch",
    "settings.minimalDesc": "Collapses to a slim 5px line when idle (great for laptops with thick top bezels).",

    // System & Indicators
    "settings.bluetoothTitle": "Bluetooth Battery Status (TWS / Mouse / Keyboard)",
    "settings.bluetoothDesc": "Dynamic Island automatically expands when Bluetooth devices connect or run low on battery.",
    "settings.bluetoothLowThreshold": "Low battery alert threshold:",

    "settings.netSpeedMonitorTitle": "Network Speed Monitor",
    "settings.netSpeedMonitorDesc": "Monitor real-time download and upload internet traffic.",
    "settings.netSpeedIslandTitle": "Dynamic Island (Closed Notch)",
    "settings.netSpeedIslandDesc": "Automatically expands when high-speed downloads or uploads occur.",
    "settings.netDownThreshold": "Minimum download threshold:",
    "settings.netUpThreshold": "Minimum upload threshold:",
    "settings.netSpeedStatusTitle": "Status Bar (Open Notch)",
    "settings.netSpeedStatusDesc": "Appears on the top row of the open notch alongside battery and clock.",

    "settings.mirrorTitle": "Windows Notifications",
    "settings.mirrorDesc": "Show Windows notification pop-ups (WhatsApp, Slack, Teams, email) in the notch.",
    "settings.mirrorTip": "Tip: Enable Do not disturb in Windows to prevent duplicate notification banners.",
    "settings.openDnd": "Open Windows DND Settings",

    "settings.capsLockTitle": "Keyboard & Caps Lock Peek",
    "settings.capsLockDesc": "Show Caps Lock status and keyboard input language switching in the notch.",

    // Productivity
    "settings.activePagesTitle": "Active Pages in Open Notch",
    "settings.activePagesDesc": "Select which pages are accessible when the notch is opened.",
    "settings.activePagesMinWarn": "(At least 1 page must be active)",

    "settings.pomodoroTitle": "Pomodoro Focus Timer",
    "settings.pomodoroDesc": "Customize productive focus work cycles and rest session lengths.",
    "settings.focusSession": "Focus Session:",
    "settings.shortBreak": "Rest Session:",
    "settings.minutes": "minutes",

    "settings.waterTitle": "Water Intake Reminder",
    "settings.waterDesc": "Daily hydration tracking with gentle reminders while staying focused on work.",
    "settings.glassSize": "Glass Size:",
    "settings.waterTarget": "Daily Target:",
    "settings.waterFrom": "Start hour:",
    "settings.waterTo": "End hour:",
    "settings.remindEvery": "Remind every:",
    "settings.minutesOff": "minutes (0 = off)",

    "settings.calendarTitle": "Calendar Synchronization (ICS)",
    "settings.calendarDesc": "Paste private iCal (.ics) URL from Google Calendar or Outlook.",
    "settings.calendarHint": "ICS address is stored locally only on this device.",

    "settings.shortcutsTitle": "Quick Shortcuts (Up to 8)",
    "settings.shortcutsDesc": "Quick launcher for apps (.exe), folders, web links, or automation scripts.",
    "settings.shortcutNamePlaceholder": "Button label",
    "settings.shortcutTargetPlaceholder": "C:\\Tools\\script.bat or https://github.com...",
    "settings.addShortcut": "+ Add Shortcut",
    "settings.restoreDefaults": "Restore Defaults",
    "settings.deleteShortcut": "Delete shortcut",

    // Media & Weather
    "settings.lyricsTitle": "Music Player & Synced Lyrics",
    "settings.lyricsDesc": "Show real-time synced lyrics from LRCLIB on Dynamic Island while music plays.",
    "settings.weatherTitle": "Weather Location",
    "settings.weatherDesc": "Location source for weather forecasts on notch and clock screen.",
    "settings.currentLocation": "Current Location:",
    "settings.windowsLocationAuto": "Windows Location (Auto)",
    "settings.switchToAutoLocation": "(Switch to Automatic Windows Location)",
    "settings.searchCityPlaceholder": "Type city name, e.g. London, New York, Tokyo...",
    "settings.searchCityBtn": "Search City",
    "settings.searchingCity": "Searching...",

    // Page items in Settings checklist
    "pageItems.music": "Music Player",
    "pageItems.musicDesc": "Clock, media player, synced lyrics",
    "pageItems.pomodoro": "Pomodoro Focus",
    "pageItems.pomodoroDesc": "Focus session and break interval timer",
    "pageItems.water": "Drink Water",
    "pageItems.waterDesc": "Daily hydration tracking & periodic reminders",
    "pageItems.todo": "To-Do Notes",
    "pageItems.todoDesc": "Simple daily to-do list in the notch",
    "pageItems.shelf": "File Shelf",
    "pageItems.shelfDesc": "Temporary drop zone for files and folders",
    "pageItems.clipboard": "Clipboard",
    "pageItems.clipboardDesc": "History of copied text and screenshots",
    "pageItems.shortcuts": "Quick Shortcuts",
    "pageItems.shortcutsDesc": "Quick action buttons and app launcher",

    // Action options in Shortcuts
    "actions.lock": "Lock screen",
    "actions.sleep": "Sleep",
    "actions.screenshot": "Screenshot",
    "actions.downloads": "Open Downloads",
    "actions.camera": "Check camera",
    "actions.open": "Open app / file / folder / link / script",

    // Pages & Tooltips
    "pages.music": "Music Player",
    "pages.pomodoro": "Pomodoro Focus",
    "pages.water": "Water Tracker",
    "pages.todo": "To-Do Notes",
    "pages.shelf": "File Shelf",
    "pages.clipboard": "Clipboard",
    "pages.shortcuts": "Shortcuts",

    // Subpage: Todo
    "todo.doneToday": "done today",
    "todo.add": "+ Add",
    "todo.placeholder": "New task…",
    "todo.timeTitle": "Remind at (optional)",
    "todo.save": "Save",
    "todo.empty": "No tasks yet. Add one, optionally with a reminder time.",
    "todo.delete": "Delete",
    "todo.markDone": "Mark complete",
    "todo.markUndone": "Mark incomplete",

    // Subpage: Water
    "water.reminders": "Reminders",
    "water.remindersTitle": "Drink reminders",
    "water.goalReached": "Today's goal reached",
    "water.todayPct": "Today · {pct}%",
    "water.drinkAmount": "💧 Drink +{amount} ml",
    "water.settingsTitle": "Water settings",
    "water.settings": "Settings",
    "water.chartLabel": "Hourly water intake today",

    // Subpage: Timer
    "timer.reset": "Reset",
    "timer.focus": "Focus",
    "timer.break": "Break",
    "timer.minutesUnit": "min",
    "timer.startBreak": "☕ Break {min}",
    "timer.startBreakTitle": "Start break now",
    "timer.startFocus": "◎ Focus {min}",
    "timer.startFocusTitle": "Start focus now",
    "timer.pause": "❚❚ Pause",
    "timer.resume": "▶ Resume",
    "timer.start": "▶ Start",

    // Subpage: Clipboard
    "clipboard.recentHint": "{count} recent · click to copy",
    "clipboard.emptyHint": "Nothing copied yet",
    "clipboard.clear": "Clear",
    "clipboard.emptyDesc": "Text and images you copy appear here.<br />Password manager contents are ignored.",
    "clipboard.copied": "✓ Copied!",
    "clipboard.text": "Text",
    "clipboard.image": "Image",

    // Subpage: Shelf
    "shelf.hint": "Drag out anytime · click to open",
    "shelf.clear": "Clear",
    "shelf.saveToShelf": "Save to Shelf",
    "shelf.share": "Share",
    "shelf.dragHere": "Drag files here",
    "shelf.removeFromShelf": "Remove from shelf",

    // Subpage: Shortcuts
    "shortcuts.title": "Shortcuts",
    "shortcuts.hint": "configure in Settings",
    "shortcuts.failed": "Failed",
    "shortcuts.lock": "Lock",
    "shortcuts.sleep": "Sleep",
    "shortcuts.screenshot": "Screenshot",
    "shortcuts.downloads": "Downloads",
    "shortcuts.camera": "Camera",

    // Subpage: Mirror
    "mirror.camDenied": "Camera access denied",
    "mirror.camError": "Could not open camera",
    "mirror.title": "Camera Check",
    "mirror.detail": "Ensure face is clearly visible",
    "mirror.done": "Done",
    "mirror.join": "Join",

    // Peek & Status Bar
    "peek.bluetoothConnected": "{device} Connected",
    "peek.bluetoothLow": "{device} Low Battery",
    "peek.charging": "Charging",
    "peek.onBattery": "On Battery",
    "peek.muted": "Muted",
    "peek.cameraInUse": "Camera in use",
    "peek.micInUse": "Microphone in use",
    "peek.timeForBreak": "Time for a break",
    "peek.focusDone": "Focus session completed",
    "peek.breakDone": "Break is over",
    "peek.readyToFocus": "Ready to focus again?",
    "peek.timeToDrink": "Time to drink water",
    "peek.battery": "Battery",
    "peek.bluetoothDevice": "Bluetooth Device",
    "peek.earbudsLow": "Earbuds battery low",
    "peek.checkingUpdate": "Checking for updates…",
    "peek.latestVersion": "You\'re up to date",
    "peek.devVersion": "Developer build",
    "peek.devVersionDetail": "Update via build, not updater",
    "peek.updateFailed": "Failed to check update",
    "peek.updateFailedDetail": "Check internet connection",
    "peek.openScreenshot": "Open screenshot",
    "peek.screenshotSaved": "Screenshot saved",
    "peek.open": "Open",
    "peek.showInFolder": "Show in folder",
    "peek.downloadDone": "Download complete · click to open",
    "peek.meetingIn2Min": "Starts in 2 minutes",
    "peek.checkCam": "Check camera",
    "peek.join": "Join",

    // Main Navigation
    "main.nextPage": "Next page",
    "main.prevPage": "Previous page",
  },
  id: {
    // Tabs
    "tabs.display": "Layar",
    "tabs.system": "Sistem",
    "tabs.focus": "Fokus",
    "tabs.media": "Media",

    // Header & General
    "settings.title": "Pengaturan Notchy",
    "settings.subtitle": "Pengalaman Fluent",
    "settings.searchPlaceholder": "Cari pengaturan (layar, bluetooth, spring, pomodoro...)",
    "settings.language": "Bahasa / Language",
    "settings.languageDesc": "Pilih bahasa tampilan untuk antarmuka Notchy",
    "settings.langEn": "English (US)",
    "settings.langId": "Bahasa Indonesia",
    "settings.saved": "Tersimpan",
    "settings.savedToast": "Pengaturan tersimpan otomatis",
    "settings.loading": "Memuat konfigurasi Notchy...",

    // Display & Layout
    "settings.monitorTitle": "Penempatan Multi-Monitor",
    "settings.monitorDesc": "Tentukan di layar mana Notchy ditempatkan secara otomatis.",
    "settings.monitorPrimary": "Layar Utama (Default)",
    "settings.monitorSecondary": "Layar Sekunder",
    "settings.monitorFollow": "Ikuti Kursor / Mouse Aktif",

    "settings.springTitle": "Animasi Dynamic Island (Spring Physics)",
    "settings.springDesc": "Kurva fisika pegas adaptif saat Dynamic Island melebar dan merespons interaksi.",
    "settings.springOrganic": "Spring Organik (Bouncy iOS / macOS Style)",
    "settings.springSnappy": "Snappy (Cepat & Responsif)",
    "settings.springSmooth": "Smooth (Halus & Elegan)",
    "settings.springOff": "Mati (Tanpa Efek Pegas)",

    "settings.accentTitle": "Sinkronisasi Warna Aksen Windows (Dynamic Accent)",
    "settings.accentDesc": "Efek highlight border glow elegan saat ada notifikasi aktif atau peek informasi.",
    "settings.accentCurrentMode": "Mode saat ini:",
    "settings.accentDwmActive": "Sinkronisasi Windows Accent DWM",
    "settings.accentDefaultNeutral": "Default Notchy (Netral)",
    "settings.accentHintActive": "Warna glow border otomatis mengikuti tema aksen yang dipilih di Personalization Windows.",
    "settings.accentHintDefault": "Warna glow border menggunakan tema monokrom ambient default Notchy.",

    "settings.mascotStyleTitle": "Ekspresi Maskot & Gaya Notch",
    "settings.mascotStyleDesc": "Karakter mata interaktif saat idle dan opsi tampilan tipis.",
    "settings.mascotTitle": "Mata Maskot Aktif",
    "settings.mascotDesc": "Mata ekspresif keluar dan melihat ke arah kursor saat laptop idle.",
    "settings.minimalTitle": "Notch Minimal",
    "settings.minimalDesc": "Bentuk garis tipis 5px saat idle (sangat cocok untuk laptop ber-bezel tebal).",

    // System & Indicators
    "settings.bluetoothTitle": "Status Baterai Bluetooth (TWS / Mouse / Keyboard)",
    "settings.bluetoothDesc": "Dynamic Island otomatis melebar saat perangkat Bluetooth tersambung atau baterai menipis.",
    "settings.bluetoothLowThreshold": "Ambang batas baterai rendah (Low Battery Alert):",

    "settings.netSpeedMonitorTitle": "Pemantau Kecepatan Jaringan",
    "settings.netSpeedMonitorDesc": "Pantau traffic unduh (download) dan unggah (upload) internet secara real-time.",
    "settings.netSpeedIslandTitle": "Dynamic Island (Notch Tertutup)",
    "settings.netSpeedIslandDesc": "Otomatis melebar saat ada unduhan/unggahan berkecepatan tinggi.",
    "settings.netDownThreshold": "Batas unduh (download) minimum:",
    "settings.netUpThreshold": "Batas unggah (upload) minimum:",
    "settings.netSpeedStatusTitle": "Baris Status (Notch Terbuka)",
    "settings.netSpeedStatusDesc": "Tampil di baris atas notch terbuka bersanding dengan baterai dan jam.",

    "settings.mirrorTitle": "Notifikasi Windows",
    "settings.mirrorDesc": "Tampilkan pop-up notifikasi Windows (WhatsApp, Slack, Teams, email) di notch.",
    "settings.mirrorTip": "Tips: Nyalakan Do not disturb di Windows agar notifikasi tidak muncul ganda.",
    "settings.openDnd": "Buka Pengaturan Windows DND",

    "settings.capsLockTitle": "Keyboard & Caps Lock Peek",
    "settings.capsLockDesc": "Tampilkan indikator Caps Lock dan pergantian layout bahasa keyboard di notch.",

    // Productivity
    "settings.activePagesTitle": "Daftar Halaman di Notch Terbuka",
    "settings.activePagesDesc": "Pilih menu yang dapat diakses saat notch dibuka.",
    "settings.activePagesMinWarn": "(Minimal 1 halaman harus aktif)",

    "settings.pomodoroTitle": "Timer Fokus Pomodoro",
    "settings.pomodoroDesc": "Sesuaikan durasi siklus kerja produktif dan istirahat.",
    "settings.focusSession": "Sesi Fokus:",
    "settings.shortBreak": "Sesi Istirahat:",
    "settings.minutes": "menit",

    "settings.waterTitle": "Pengingat Minum Air",
    "settings.waterDesc": "Pencatatan hidrasi harian dengan pengingat ramah saat fokus bekerja.",
    "settings.glassSize": "Sekali minum:",
    "settings.waterTarget": "Target harian:",
    "settings.waterFrom": "Mulai jam:",
    "settings.waterTo": "Sampai jam:",
    "settings.remindEvery": "Ingatkan tiap:",
    "settings.minutesOff": "menit (0 = mati)",

    "settings.calendarTitle": "Sinkronisasi Kalender (ICS)",
    "settings.calendarDesc": "Tempel URL rahasia format iCal (.ics) dari Google Calendar atau Outlook.",
    "settings.calendarHint": "Alamat ICS hanya disimpan lokal di komputer ini.",

    "settings.shortcutsTitle": "Pintasan Cepat (Maksimal 8)",
    "settings.shortcutsDesc": "Peluncur cepat aplikasi (.exe), folder, link web, atau file skrip otomatisasi.",
    "settings.shortcutNamePlaceholder": "Nama tombol",
    "settings.shortcutTargetPlaceholder": "C:\\Tools\\script.bat atau https://github.com...",
    "settings.addShortcut": "+ Tambah Pintasan",
    "settings.restoreDefaults": "Kembalikan ke Default",
    "settings.deleteShortcut": "Hapus pintasan",

    // Media & Weather
    "settings.lyricsTitle": "Pemutar Musik & Lirik Berjalan",
    "settings.lyricsDesc": "Tampilkan baris lirik real-time dari LRCLIB pada Dynamic Island saat lagu berputar.",
    "settings.weatherTitle": "Lokasi Cuaca",
    "settings.weatherDesc": "Sumber lokasi untuk prakiraan cuaca pada notch dan layar jam.",
    "settings.currentLocation": "Lokasi Saat Ini:",
    "settings.windowsLocationAuto": "Lokasi Windows (Otomatis)",
    "settings.switchToAutoLocation": "(Ganti ke Lokasi Windows Otomatis)",
    "settings.searchCityPlaceholder": "Ketik nama kota, contoh: Jakarta, Surabaya, Tokyo...",
    "settings.searchCityBtn": "Cari Kota",
    "settings.searchingCity": "Mencari...",

    // Page items in Settings checklist
    "pageItems.music": "Pemutar Musik",
    "pageItems.musicDesc": "Jam, pemutar media, lirik lagu berjalan",
    "pageItems.pomodoro": "Fokus Pomodoro",
    "pageItems.pomodoroDesc": "Timer sesi fokus dan istirahat",
    "pageItems.water": "Minum Air",
    "pageItems.waterDesc": "Pencatatan hidrasi harian & pengingat berkala",
    "pageItems.todo": "Catatan To-Do",
    "pageItems.todoDesc": "Daftar to-do harian sederhana di notch",
    "pageItems.shelf": "Laci File (Shelf)",
    "pageItems.shelfDesc": "Tempat menaruh file/folder sementara",
    "pageItems.clipboard": "Papan Klip",
    "pageItems.clipboardDesc": "Riwayat salinan teks dan tangkapan layar",
    "pageItems.shortcuts": "Pintasan Cepat",
    "pageItems.shortcutsDesc": "Tombol aksi cepat dan peluncur aplikasi",

    // Action options in Shortcuts
    "actions.lock": "Kunci layar",
    "actions.sleep": "Sleep",
    "actions.screenshot": "Screenshot",
    "actions.downloads": "Buka Downloads",
    "actions.camera": "Cek kamera",
    "actions.open": "Buka aplikasi / file / folder / link / skrip",

    // Pages & Tooltips
    "pages.music": "Pemutar Musik",
    "pages.pomodoro": "Fokus Pomodoro",
    "pages.water": "Minum Air",
    "pages.todo": "Catatan To-Do",
    "pages.shelf": "Laci File",
    "pages.clipboard": "Papan Klip",
    "pages.shortcuts": "Pintasan",

    // Subpage: Todo
    "todo.doneToday": "selesai hari ini",
    "todo.add": "+ Tambah",
    "todo.placeholder": "Tugas baru…",
    "todo.timeTitle": "Ingatkan jam (opsional)",
    "todo.save": "Simpan",
    "todo.empty": "Belum ada tugas. Tambah satu, bisa dengan jam pengingat.",
    "todo.delete": "Hapus",
    "todo.markDone": "Selesai",
    "todo.markUndone": "Belum selesai",

    // Subpage: Water
    "water.reminders": "Pengingat",
    "water.remindersTitle": "Pengingat minum",
    "water.goalReached": "Target hari ini tercapai",
    "water.todayPct": "Hari ini · {pct}%",
    "water.drinkAmount": "💧 Minum +{amount} ml",
    "water.settingsTitle": "Pengaturan minum air",
    "water.settings": "Atur",
    "water.chartLabel": "Minum per jam hari ini",

    // Subpage: Timer
    "timer.reset": "Ulang",
    "timer.focus": "Fokus",
    "timer.break": "Istirahat",
    "timer.minutesUnit": "menit",
    "timer.startBreak": "☕ Istirahat {min}",
    "timer.startBreakTitle": "Mulai istirahat sekarang",
    "timer.startFocus": "◎ Fokus {min}",
    "timer.startFocusTitle": "Mulai fokus sekarang",
    "timer.pause": "❚❚ Jeda",
    "timer.resume": "▶ Lanjut",
    "timer.start": "▶ Mulai",

    // Subpage: Clipboard
    "clipboard.recentHint": "{count} terakhir · klik untuk salin",
    "clipboard.emptyHint": "Belum ada yang disalin",
    "clipboard.clear": "Bersihkan",
    "clipboard.emptyDesc": "Teks dan gambar yang kamu copy muncul di sini.<br />Isi dari password manager tidak disimpan.",
    "clipboard.copied": "✓ Tersalin!",
    "clipboard.text": "Teks",
    "clipboard.image": "Gambar",

    // Subpage: Shelf
    "shelf.hint": "Tarik keluar kapan saja · klik untuk membuka",
    "shelf.clear": "Bersihkan",
    "shelf.saveToShelf": "Simpan di Shelf",
    "shelf.share": "Bagikan",
    "shelf.dragHere": "Tarik file ke sini",
    "shelf.removeFromShelf": "Hapus dari shelf",

    // Subpage: Shortcuts
    "shortcuts.title": "Pintasan",
    "shortcuts.hint": "atur di Settings",
    "shortcuts.failed": "Gagal",
    "shortcuts.lock": "Kunci",
    "shortcuts.sleep": "Sleep",
    "shortcuts.screenshot": "Screenshot",
    "shortcuts.downloads": "Downloads",
    "shortcuts.camera": "Kamera",

    // Subpage: Mirror
    "mirror.camDenied": "Akses kamera ditolak",
    "mirror.camError": "Kamera tidak bisa dibuka",
    "mirror.title": "Cek kamera",
    "mirror.detail": "Pastikan wajah terlihat jelas",
    "mirror.done": "Selesai",
    "mirror.join": "Join",

    // Peek & Status Bar
    "peek.bluetoothConnected": "{device} Terhubung",
    "peek.bluetoothLow": "Baterai {device} Lemah",
    "peek.charging": "Mengisi Daya",
    "peek.onBattery": "Baterai",
    "peek.muted": "Mute",
    "peek.cameraInUse": "Kamera dipakai",
    "peek.micInUse": "Mikrofon dipakai",
    "peek.timeForBreak": "Waktunya istirahat",
    "peek.focusDone": "Sesi fokus selesai",
    "peek.breakDone": "Istirahat selesai",
    "peek.readyToFocus": "Siap fokus lagi?",
    "peek.timeToDrink": "Waktunya minum",
    "peek.battery": "Baterai",
    "peek.bluetoothDevice": "Perangkat Bluetooth",
    "peek.earbudsLow": "Baterai earbuds lemah",
    "peek.checkingUpdate": "Mengecek update…",
    "peek.latestVersion": "Sudah versi terbaru",
    "peek.devVersion": "Versi pengembang",
    "peek.devVersionDetail": "Update lewat build, bukan updater",
    "peek.updateFailed": "Gagal mengecek update",
    "peek.updateFailedDetail": "Cek koneksi internet",
    "peek.openScreenshot": "Buka screenshot",
    "peek.screenshotSaved": "Screenshot tersimpan",
    "peek.open": "Buka",
    "peek.showInFolder": "Lihat di folder",
    "peek.downloadDone": "Selesai diunduh · klik untuk buka",
    "peek.meetingIn2Min": "Mulai 2 menit lagi",
    "peek.checkCam": "Cek kamera",
    "peek.join": "Join",

    // Main Navigation
    "main.nextPage": "Halaman berikutnya",
    "main.prevPage": "Halaman sebelumnya",
  },
} as const;

export type TranslationKey = keyof typeof translations.en;

let activeLocale: Locale = "en";
const listeners = new Set<(loc: Locale) => void>();

export function setLocale(locale: Locale | string | undefined) {
  const next: Locale = locale === "id" ? "id" : "en";
  activeLocale = next;
  for (const fn of listeners) {
    try {
      fn(next);
    } catch {}
  }
}

export function subscribeLocale(fn: (loc: Locale) => void): () => void {
  fn(activeLocale);
  listeners.add(fn);
  return () => {
    listeners.delete(fn);
  };
}

export function getLocale(): Locale {
  return activeLocale;
}

export function t(key: string, params?: Record<string, string | number>): string {
  const dict = translations[activeLocale] || translations.en;
  let text = (dict as Record<string, string>)[key] || (translations.en as Record<string, string>)[key] || key;
  if (params) {
    for (const [k, v] of Object.entries(params)) {
      text = text.replace(new RegExp(`\\{${k}\\}`, "g"), String(v));
    }
  }
  return text;
}
