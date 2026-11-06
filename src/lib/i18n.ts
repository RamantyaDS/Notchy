// Lightweight, reactive, type-safe i18n module for Notchy

export type Locale = "en" | "id";

export const translations = {
  en: {
    // Tabs
    "tabs.display": "Display",
    "tabs.system": "System",

    // Header & General
    "settings.language": "Language / Bahasa",
    "settings.saved": "Saved",
    "settings.loading": "Loading Notchy configuration...",

    // Display & Layout




    // System & Indicators




    // Productivity



    "settings.calendarHint": "ICS address is stored locally only on this device.",

    "settings.shortcutNamePlaceholder": "Button label",
    "settings.shortcutTargetPlaceholder": "C:\\Tools\\script.bat or https://github.com...",
    "settings.addShortcut": "+ Add Shortcut",
    "settings.restoreDefaults": "Restore Defaults",
    "settings.deleteShortcut": "Delete shortcut",

    // Media & Weather

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
    "nav.general": "General",
    "nav.appearance": "Appearance",
    "nav.eyes": "Eyes",
    "nav.pages": "Pages",
    "nav.music": "Music & lyrics",
    "nav.focus": "Focus timer",
    "nav.water": "Water",
    "nav.calendar": "Calendar",
    "nav.shortcuts": "Shortcuts",
    "nav.notifications": "Notifications",
    "nav.system": "Keyboard & system",
    "nav.weather": "Weather",
    "s.search": "Search",
    "s.noResults": "No settings match.",
    "s.off": "Off",
    "s.open": "Open",
    "s.min": "min",
    "s.language": "Language",
    "s.monitor": "Screen",
    "s.monitorDesc": "Where the notch sits when you have more than one screen.",
    "s.monitorPrimary": "Main screen",
    "s.monitorSecondary": "Second screen",
    "s.monitorFollow": "Follow the cursor",
    "s.spring": "Animation",
    "s.springDesc": "How the notch moves when it opens and closes.",
    "s.springBouncy": "Bouncy",
    "s.springSnappy": "Snappy",
    "s.springSmooth": "Smooth",
    "s.floating": "Floating style",
    "s.floatingDesc": "The notch floats a little below the top edge as a rounded pill.",
    "s.minimal": "Minimal notch",
    "s.minimalDesc": "A thin line when nothing is happening. Good for thick top bezels.",
    "s.ring": "Status ring",
    "s.ringDesc": "A small ring in the resting notch.",
    "s.ringBattery": "Battery",
    "s.ringWater": "Today's water",
    "s.accent": "Windows accent glow",
    "s.accentDesc": "Alerts glow in your Windows accent color instead of white.",
    "s.mascot": "Mascot eyes",
    "s.mascotDesc": "Two eyes that look at your cursor.",
    "s.mascotMode": "When to show them",
    "s.mascotIdle": "After 2 minutes idle",
    "s.mascotAlways": "All the time",
    "s.eyeStyle": "Eye style",
    "s.eyeClassic": "Classic",
    "s.eyeCat": "Cat",
    "s.eyeCartoon": "Cartoon",
    "s.musicMascot": "Eyes in the music island",
    "s.musicMascotDesc": "When a song has no lyrics, the eyes bob along instead of the clock.",
    "s.pagesDesc": "Pages you can swipe through in the open notch. At least one stays on.",
    "pageItems.today": "Today",
    "pageItems.todayDesc": "Next meeting, today's tasks and water at a glance",
    "s.lyrics": "Lyrics in the small notch",
    "s.lyricsDesc": "Shows the line being sung. Looks up each song on LRCLIB.",
    "s.coverHint": "With more than one player open, click the cover in the player to switch between them.",
    "s.focusLen": "Focus session",
    "s.breakLen": "Break",
    "s.waterOn": "Water tracking",
    "s.waterOnDesc": "Water page and gentle reminders to drink.",
    "s.glass": "One glass",
    "s.goal": "Daily goal",
    "s.from": "Remind from",
    "s.to": "Remind until",
    "s.remindEvery": "Remind every",
    "s.remindEveryDesc": "0 turns reminders off.",
    "s.calendarUrl": "Calendar link (ICS)",
    "s.calendarDesc": "The secret iCal address from Google Calendar or Outlook.",
    "s.mirror": "Show notifications in the notch",
    "s.mirrorDesc": "WhatsApp, Teams, Outlook and others.",
    "s.dnd": "Windows Do not disturb",
    "s.dndDesc": "Turn it on so notifications show only in the notch.",
    "s.keyboard": "Keyboard",
    "s.caps": "Caps Lock & keyboard language",
    "s.capsDesc": "Shows a peek when they change.",
    "s.hideOsd": "Hide Windows volume overlay",
    "s.hideOsdDesc": "Notchy handles the volume keys, so only its own volume shows.",
    "s.restartHint": "Keyboard settings apply after restarting Notchy.",
    "s.bluetooth": "Bluetooth alerts",
    "s.bluetoothDesc": "A peek when a device connects or its battery runs low.",
    "s.lowBattery": "Low battery at",
    "s.network": "Network",
    "s.netStatus": "Speed in the open notch",
    "s.netStatusDesc": "Download and upload speed in the top row.",
    "s.netIsland": "Speed in the small notch",
    "s.netIslandDesc": "Only during heavy downloads or uploads.",
    "s.netDown": "Download above",
    "s.netUp": "Upload above",
    "s.location": "Location",
    "s.locationAuto": "Windows location (automatic)",
    "s.useAuto": "Use automatic",
    "s.cityPlaceholder": "City name",
    "pages.today": "Today",
    "today.next": "Next up",
    "today.noEvent": "No more meetings today",
    "today.tasks": "Tasks · {n} open",
    "today.noTasks": "Nothing planned today",
    "today.allDone": "All done for today 🎉",
    "main.otherPlayer": "Click to switch player",
    "pages.pomodoro": "Pomodoro Focus",
    "pages.water": "Water Tracker",
    "pages.todo": "To-Do Notes",
    "pages.shelf": "File Shelf",
    "pages.clipboard": "Clipboard",
    "pages.shortcuts": "Shortcuts",

    // Subpage: Todo
    "todo.doneToday": "done today",
    "todo.doneThatDay": "done that day",
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

    // Header & General
    "settings.language": "Bahasa / Language",
    "settings.saved": "Tersimpan",
    "settings.loading": "Memuat konfigurasi Notchy...",

    // Display & Layout




    // System & Indicators




    // Productivity



    "settings.calendarHint": "Alamat ICS hanya disimpan lokal di komputer ini.",

    "settings.shortcutNamePlaceholder": "Nama tombol",
    "settings.shortcutTargetPlaceholder": "C:\\Tools\\script.bat atau https://github.com...",
    "settings.addShortcut": "+ Tambah Pintasan",
    "settings.restoreDefaults": "Kembalikan ke Default",
    "settings.deleteShortcut": "Hapus pintasan",

    // Media & Weather

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
    "nav.general": "Umum",
    "nav.appearance": "Tampilan",
    "nav.eyes": "Mata",
    "nav.pages": "Halaman",
    "nav.music": "Musik & lirik",
    "nav.focus": "Timer fokus",
    "nav.water": "Minum air",
    "nav.calendar": "Kalender",
    "nav.shortcuts": "Pintasan",
    "nav.notifications": "Notifikasi",
    "nav.system": "Keyboard & sistem",
    "nav.weather": "Cuaca",
    "s.search": "Cari",
    "s.noResults": "Tidak ada pengaturan yang cocok.",
    "s.off": "Mati",
    "s.open": "Buka",
    "s.min": "mnt",
    "s.language": "Bahasa",
    "s.monitor": "Layar",
    "s.monitorDesc": "Posisi notch kalau kamu punya lebih dari satu layar.",
    "s.monitorPrimary": "Layar utama",
    "s.monitorSecondary": "Layar kedua",
    "s.monitorFollow": "Ikuti kursor",
    "s.spring": "Animasi",
    "s.springDesc": "Cara notch bergerak saat membuka dan menutup.",
    "s.springBouncy": "Memantul",
    "s.springSnappy": "Cepat",
    "s.springSmooth": "Halus",
    "s.floating": "Gaya melayang",
    "s.floatingDesc": "Notch melayang sedikit di bawah tepi atas, berbentuk pil bulat.",
    "s.minimal": "Notch minimal",
    "s.minimalDesc": "Jadi garis tipis saat tidak ada apa-apa. Cocok untuk bezel atas yang tebal.",
    "s.ring": "Cincin status",
    "s.ringDesc": "Cincin kecil di notch saat diam.",
    "s.ringBattery": "Baterai",
    "s.ringWater": "Air minum hari ini",
    "s.accent": "Glow warna aksen Windows",
    "s.accentDesc": "Peringatan bersinar dengan warna aksen Windows, bukan putih.",
    "s.mascot": "Mata maskot",
    "s.mascotDesc": "Dua mata yang melirik kursormu.",
    "s.mascotMode": "Kapan muncul",
    "s.mascotIdle": "Setelah 2 menit diam",
    "s.mascotAlways": "Selalu",
    "s.eyeStyle": "Gaya mata",
    "s.eyeClassic": "Klasik",
    "s.eyeCat": "Kucing",
    "s.eyeCartoon": "Kartun",
    "s.musicMascot": "Mata di pulau musik",
    "s.musicMascotDesc": "Saat lagu tanpa lirik, mata mengangguk menggantikan jam.",
    "s.pagesDesc": "Halaman yang bisa digeser di notch terbuka. Minimal satu tetap nyala.",
    "pageItems.today": "Hari ini",
    "pageItems.todayDesc": "Meeting berikutnya, tugas dan air minum hari ini sekilas",
    "s.lyrics": "Lirik di notch kecil",
    "s.lyricsDesc": "Menampilkan baris yang sedang dinyanyikan. Tiap lagu dicari di LRCLIB.",
    "s.coverHint": "Kalau ada lebih dari satu pemutar, klik cover di pemutar untuk berpindah.",
    "s.focusLen": "Sesi fokus",
    "s.breakLen": "Istirahat",
    "s.waterOn": "Catat minum air",
    "s.waterOnDesc": "Halaman air dan pengingat minum.",
    "s.glass": "Satu gelas",
    "s.goal": "Target harian",
    "s.from": "Ingatkan dari",
    "s.to": "Ingatkan sampai",
    "s.remindEvery": "Ingatkan tiap",
    "s.remindEveryDesc": "0 mematikan pengingat.",
    "s.calendarUrl": "Link kalender (ICS)",
    "s.calendarDesc": "Alamat iCal rahasia dari Google Calendar atau Outlook.",
    "s.mirror": "Tampilkan notifikasi di notch",
    "s.mirrorDesc": "WhatsApp, Teams, Outlook, dan lainnya.",
    "s.dnd": "Jangan Ganggu Windows",
    "s.dndDesc": "Nyalakan supaya notifikasi hanya muncul di notch.",
    "s.keyboard": "Keyboard",
    "s.caps": "Caps Lock & bahasa keyboard",
    "s.capsDesc": "Muncul peek saat berubah.",
    "s.hideOsd": "Sembunyikan overlay volume Windows",
    "s.hideOsdDesc": "Tombol volume ditangani Notchy, jadi hanya volume Notchy yang muncul.",
    "s.restartHint": "Pengaturan keyboard berlaku setelah Notchy dibuka ulang.",
    "s.bluetooth": "Peringatan Bluetooth",
    "s.bluetoothDesc": "Peek saat perangkat tersambung atau baterainya menipis.",
    "s.lowBattery": "Baterai lemah di",
    "s.network": "Jaringan",
    "s.netStatus": "Kecepatan di notch terbuka",
    "s.netStatusDesc": "Kecepatan unduh dan unggah di baris atas.",
    "s.netIsland": "Kecepatan di notch kecil",
    "s.netIslandDesc": "Hanya saat unduhan atau unggahan besar.",
    "s.netDown": "Unduh di atas",
    "s.netUp": "Unggah di atas",
    "s.location": "Lokasi",
    "s.locationAuto": "Lokasi Windows (otomatis)",
    "s.useAuto": "Pakai otomatis",
    "s.cityPlaceholder": "Nama kota",
    "pages.today": "Hari Ini",
    "today.next": "Berikutnya",
    "today.noEvent": "Tidak ada meeting lagi hari ini",
    "today.tasks": "Tugas · {n} belum",
    "today.noTasks": "Belum ada rencana hari ini",
    "today.allDone": "Semua beres hari ini 🎉",
    "main.otherPlayer": "Klik untuk ganti pemutar",
    "pages.pomodoro": "Fokus Pomodoro",
    "pages.water": "Minum Air",
    "pages.todo": "Catatan To-Do",
    "pages.shelf": "Laci File",
    "pages.clipboard": "Papan Klip",
    "pages.shortcuts": "Pintasan",

    // Subpage: Todo
    "todo.doneToday": "selesai hari ini",
    "todo.doneThatDay": "selesai hari itu",
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

export function detectDefaultLocale(): Locale {
  if (typeof navigator !== "undefined" && navigator.language) {
    if (navigator.language.toLowerCase().startsWith("en")) return "en";
  }
  return "id"; // Default fallback to "id" as requested by Ramantya
}

let activeLocale: Locale = detectDefaultLocale();
const listeners = new Set<(loc: Locale) => void>();

export function setLocale(locale: Locale | string | undefined) {
  const next: Locale = locale === "en" ? "en" : locale === "id" ? "id" : detectDefaultLocale();
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
