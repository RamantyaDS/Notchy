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
    "settings.savedTooltip": "Changes saved successfully",

    // Display & Layout
    "settings.monitorTitle": "Multi-Monitor Placement",
    "settings.monitorDesc": "Choose on which monitor Notchy appears automatically.",
    "settings.monitorPrimary": "Primary Display (Default)",
    "settings.monitorSecondary": "Secondary Display",
    "settings.monitorFollow": "Follow Active Mouse / Cursor",

    "settings.springTitle": "Organic Spring Physics Animation",
    "settings.springDesc": "Natural spring physics curve for Dynamic Island expansions and transitions.",
    "settings.springOrganic": "Adaptive Spring (iOS / macOS Style)",
    "settings.springSnappy": "Snappy Spring (Fast & Responsive)",
    "settings.springSmooth": "Smooth Ease (Standard Ease-Out)",
    "settings.springOff": "Off (Instant / No Physics)",

    "settings.accentTitle": "Windows Accent Color Sync (DWM)",
    "settings.accentDesc": "Synchronize border glow with Windows Accent Color from DWM registry during active notifications.",
    "settings.accentActive": "Windows Accent Sync Active",
    "settings.accentDefault": "Default Neutral Glow",

    // System & Indicators
    "settings.bluetoothTitle": "Bluetooth Device Battery Status",
    "settings.bluetoothDesc": "Show battery percentage in Dynamic Island when TWS, mouse, or keyboard connects or runs low.",
    "settings.bluetoothLowThreshold": "Low battery alert threshold:",

    "settings.netSpeedTitle": "Dynamic Island Network Speed",
    "settings.netSpeedDesc": "Show real-time download and upload speeds during high bandwidth usage.",
    "settings.netStatusTitle": "Status Bar Network Speed",
    "settings.netStatusDesc": "Show network transfer rate in the top bar when notch is opened.",
    "settings.netDownThreshold": "Minimum download threshold:",
    "settings.netUpThreshold": "Minimum upload threshold:",

    "settings.mascotTitle": "Mascot Eyes",
    "settings.mascotDesc": "Notchy looks towards your cursor and plays animations during idle time.",
    "settings.minimalTitle": "Minimal Notch",
    "settings.minimalDesc": "Notch collapses into a very slim bar when idle, ideal for small bezels.",
    "settings.mirrorTitle": "Windows Notifications",
    "settings.mirrorDesc": "Show incoming Windows app notifications inside Dynamic Island.",
    "settings.capsLockTitle": "Caps Lock & Language Peek",
    "settings.capsLockDesc": "Briefly pop up when Caps Lock or keyboard input layout changes.",

    // Productivity
    "settings.activePagesTitle": "Active Island Pages",
    "settings.activePagesDesc": "Choose which pages and tools appear when opening the notch.",
    "settings.pomodoroTitle": "Pomodoro Focus Timer",
    "settings.pomodoroDesc": "Time management tool with focus sessions and rest intervals.",
    "settings.focusSession": "Focus Session:",
    "settings.shortBreak": "Short Break:",
    "settings.waterTitle": "Hydration & Water Reminder",
    "settings.waterDesc": "Track daily water consumption with gentle reminders.",
    "settings.waterTarget": "Daily Target:",
    "settings.glassSize": "Glass Size:",
    "settings.remindBetween": "Active hours:",
    "settings.remindTo": "until:",
    "settings.remindInterval": "Remind every:",
    "settings.minutes": "minutes",
    "settings.minutesOff": "minutes (0 = off)",

    // Media & Weather
    "settings.lyricsTitle": "Synced Lyrics Peek",
    "settings.lyricsDesc": "Show current lyrics line in the collapsed notch during playback.",
    "settings.weatherTitle": "Weather Location",
    "settings.weatherDesc": "Leave empty to use automatic Windows location.",
    "settings.searchCity": "Search city (e.g. Jakarta, London, Tokyo)...",
    "settings.useWindowsLocation": "Use Windows Location (Auto)",
    "settings.calendarTitle": "Calendar Synchronization",
    "settings.calendarDesc": "Enter secret ICS calendar URL (Google / Outlook / Apple).",

    // Pages & Tooltips
    "pages.music": "Music Player",
    "pages.pomodoro": "Pomodoro Focus",
    "pages.water": "Water Tracker",
    "pages.todo": "To-Do Notes",
    "pages.shelf": "File Shelf",
    "pages.clipboard": "Clipboard",
    "pages.shortcuts": "Shortcuts",

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
    "settings.savedTooltip": "Pengaturan berhasil disimpan",

    // Display & Layout
    "settings.monitorTitle": "Penempatan Multi-Monitor",
    "settings.monitorDesc": "Tentukan di layar mana Notchy ditempatkan secara otomatis.",
    "settings.monitorPrimary": "Layar Utama (Default)",
    "settings.monitorSecondary": "Layar Sekunder",
    "settings.monitorFollow": "Ikuti Kursor / Mouse Aktif",

    "settings.springTitle": "Animasi Fisika Pegas Organik",
    "settings.springDesc": "Transisi animasi ekspansi Dynamic Island berbasis kurva fisika pegas alami.",
    "settings.springOrganic": "Pegas Adaptif (Gaya iOS / macOS)",
    "settings.springSnappy": "Pegas Cepat (Responsif & Ringan)",
    "settings.springSmooth": "Transisi Halus (Standar Ease-Out)",
    "settings.springOff": "Nonaktif (Instan / Tanpa Fisika)",

    "settings.accentTitle": "Sinkronisasi Warna Aksen Windows (DWM)",
    "settings.accentDesc": "Sinkronkan kilau border dengan Warna Aksen Windows dari registry DWM saat ada notifikasi aktif.",
    "settings.accentActive": "Sinkronisasi Windows Accent Aktif",
    "settings.accentDefault": "Warna Default Notchy",

    // System & Indicators
    "settings.bluetoothTitle": "Status Baterai Perangkat Bluetooth",
    "settings.bluetoothDesc": "Tampilkan persentase baterai di Dynamic Island saat TWS, mouse, atau keyboard terhubung atau baterai menipis.",
    "settings.bluetoothLowThreshold": "Ambang batas baterai rendah (Low Battery Alert):",

    "settings.netSpeedTitle": "Kecepatan Internet Dynamic Island",
    "settings.netSpeedDesc": "Tampilkan kecepatan unduh dan unggah di notch saat ada pemakaian data besar.",
    "settings.netStatusTitle": "Kecepatan Internet di Status Bar",
    "settings.netStatusDesc": "Tampilkan kecepatan unduh dan unggah di baris atas saat notch dibuka.",
    "settings.netDownThreshold": "Batas unduh (download) minimum:",
    "settings.netUpThreshold": "Batas unggah (upload) minimum:",

    "settings.mascotTitle": "Mata Maskot",
    "settings.mascotDesc": "Notchy melirik mengikuti kursor dan berkedip saat layar sedang santai.",
    "settings.minimalTitle": "Notch Minimal",
    "settings.minimalDesc": "Notch menyusut menjadi garis hitam sangat tipis saat diam, cocok untuk bezel tipis.",
    "settings.mirrorTitle": "Notifikasi Windows",
    "settings.mirrorDesc": "Tampilkan notifikasi Windows yang masuk di dalam Dynamic Island.",
    "settings.capsLockTitle": "Peek Caps Lock & Input Bahasa",
    "settings.capsLockDesc": "Muncul sekilas saat Caps Lock atau tata letak bahasa keyboard berganti.",

    // Productivity
    "settings.activePagesTitle": "Halaman Dynamic Island Aktif",
    "settings.activePagesDesc": "Pilih halaman apa saja yang muncul saat notch dibuka.",
    "settings.pomodoroTitle": "Fokus Pomodoro",
    "settings.pomodoroDesc": "Metode manajemen waktu dengan interval fokus dan istirahat santai.",
    "settings.focusSession": "Sesi Fokus:",
    "settings.shortBreak": "Istirahat Singkat:",
    "settings.waterTitle": "Pengingat Minum Air",
    "settings.waterDesc": "Pantau konsumsi air harian dengan pengingat ramah secara berkala.",
    "settings.waterTarget": "Target Harian:",
    "settings.glassSize": "Ukuran Gelas:",
    "settings.remindBetween": "Jam aktif pengingat:",
    "settings.remindTo": "sampai jam:",
    "settings.remindInterval": "Ingatkan tiap:",
    "settings.minutes": "menit",
    "settings.minutesOff": "menit (0 = mati)",

    // Media & Weather
    "settings.lyricsTitle": "Lirik Musik Berjalan",
    "settings.lyricsDesc": "Tampilkan bait lirik yang sedang dinyanyikan di bilah notch kecil.",
    "settings.weatherTitle": "Lokasi Cuaca",
    "settings.weatherDesc": "Kosongkan untuk memakai lokasi bawaan Windows secara otomatis.",
    "settings.searchCity": "Cari kota (contoh: Jakarta, Bandung, Surabaya)...",
    "settings.useWindowsLocation": "Gunakan Lokasi Windows (Otomatis)",
    "settings.calendarTitle": "Sinkronisasi Kalender",
    "settings.calendarDesc": "Masukkan alamat rahasia format iCal (.ics) dari Google atau Outlook.",

    // Pages & Tooltips
    "pages.music": "Pemutar Musik",
    "pages.pomodoro": "Fokus Pomodoro",
    "pages.water": "Minum Air",
    "pages.todo": "Catatan To-Do",
    "pages.shelf": "Laci File",
    "pages.clipboard": "Papan Klip",
    "pages.shortcuts": "Pintasan",

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
  },
} as const;

export type TranslationKey = keyof typeof translations.en;

let activeLocale: Locale = "en";

export function setLocale(locale: Locale | string | undefined) {
  if (locale === "id") {
    activeLocale = "id";
  } else {
    activeLocale = "en";
  }
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
