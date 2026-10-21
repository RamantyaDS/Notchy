# Notchy

Notch ala Dynamic Island untuk laptop Windows. Notch kecil di tengah atas layar yang menampilkan hal yang sedang terjadi (musik, timer, notifikasi, status sistem), membesar saat di-hover, lalu menyingkir lagi.

---

## 1. Goal

**Notchy harus nyaman dipakai setiap hari di laptop sendiri: cantik, tidak mengganggu, dan tidak memberatkan PC.**

Notchy dianggap berhasil kalau:
- Menyala otomatis saat Windows start, dan bisa dibiarkan seharian tanpa perlu di-restart.
- Tidak pernah menghalangi klik ke aplikasi di bawahnya, kecuali di area notch itu sendiri.
- Hilang sendiri saat ada aplikasi full-screen (game, video, presentasi).
- Saat diam (tidak ada animasi), CPU-nya ~0% di Task Manager. *Kenyataan terukur: ~1% satu core saat diam (sebagian besar WebView2), ~4% saat musik diputar dengan animasi piringan/equalizer.*
- Semua fitur di bagian 5 berjalan.

**Pengguna:** saya sendiri dan beberapa teman (installer dibagikan manual, lihat bagian 9). Tidak dirilis publik.

**Status:** v0.1.5 (21 Oktober 2026). Kode di GitHub `RamantyaDS/Notchy` (publik sejak 20 Okt 2026; riwayat diringkas jadi satu commit, kata "Nusa" dibersihkan). Installer: halaman Releases repo itu.

---

## 2. Keputusan yang sudah final

| Keputusan | Pilihan | Alasan |
|---|---|---|
| Nama | **Notchy** | â€” |
| Framework | **Tauri v2** (Rust + HTML/CSS/TS) | Tampilan dan animasi gampang dibuat cantik dengan CSS. Hasilnya tetap aplikasi desktop (.exe) yang ringan. |
| Platform | Windows 10 dan 11 saja | Banyak API yang dipakai hanya ada di Windows |
| Sumber musik | API media Windows (`GlobalSystemMediaTransportControlsSessionManager`) | Bisa membaca player apa pun: Spotify, YouTube di browser, Apple Music, dll. |
| Sumber lirik | **LRCLIB** (lrclib.net) | Gratis, tanpa login, ada lirik tersinkron. Lirik **tidak** diambil dari Spotify karena tidak ada API resminya dan endpoint internalnya melanggar ToS. |
| Frontend | **Svelte** | Ringan, dan ada `spring`/`transition` bawaan untuk animasi notch. Satu komponen per mode. |
| Cuaca | Open-Meteo | Gratis, tanpa API key |
| Lokasi cuaca | Lokasi Windows (GPS/Wi-Fi) kalau diizinkan; kalau tidak, kota yang diisi manual di Settings (dicari lewat geocoding Open-Meteo) | Tidak memakai lokasi berbasis IP: meleset dan alamat IP terkirim ke pihak ketiga |
| Jendela | **Ukuran tetap dan transparan**; yang berubah ukuran hanya elemen notch di dalamnya (CSS) | Resize jendela Windows setiap animasi akan patah-patah dan berkedip |
| Deteksi hover | Low-level mouse hook di Rust (`WH_MOUSE_LL`). Jendela click-through kecuali saat kursor berada di area notch, yang ukurannya dilaporkan frontend lewat `set_notch_size`. | Jendela yang click-through tidak menerima event mouse, jadi hover harus dideteksi dari luar; hook berbasis event, tanpa polling. Batas atas area notch sengaja tidak dicek, karena hook melihat posisi kursor sebelum dibatasi tepi layar (y bisa negatif saat mouse didorong ke atas). |
| Mengubah click-through | `set_ignore_cursor_events(true)` hanya sekali saat start; setelah itu hanya `WS_EX_TRANSPARENT` yang diubah langsung lewat `SetWindowLongPtrW` dari thread pekerja (bukan dari callback hook) | API Tauri ikut melepas/memasang `WS_EX_LAYERED`, yang membuat WebView2 menyusun ulang permukaan jendela (lag, tampilan macet). Callback hook harus cepat, kalau tidak Windows mencabut hook-nya diam-diam. |
| Tampil/sembunyi jendela | `ShowWindow` langsung (`SW_SHOWNOACTIVATE` / `SW_HIDE`), bukan `show()`/`hide()` Tauri | Tauri tidak tahu jendela sudah ditampilkan lewat `ShowWindow`, jadi `hide()`-nya diabaikan |
| Musik | `media.rs`: event WinRT (`CurrentSessionChanged`, `MediaPropertiesChanged`, `PlaybackInfoChanged`, `TimelinePropertiesChanged`) di thread MTA sendiri; posisi lagu diekstrapolasi di frontend dari `position` + `updatedAt`, ticker 250 ms hanya saat notch terbuka dan lagu berjalan | Tanpa polling; saat tertutup tidak ada yang bergerak |
| Lirik | Diambil dari frontend (`fetch` ke LRCLIB, CORS diizinkan) untuk **setiap lagu** selama opsi "baris lirik di notch kecil" aktif (default aktif; kalau dimatikan, hanya saat panel lirik dibuka). Yang ditemukan di-cache di `localStorage`, yang tidak ditemukan tidak di-cache. Judul dibersihkan dulu: buang "(Officialâ€¦)", "lyric official" tanpa kurung, "- Topic"; untuk "A - B", sisi yang sama dengan artis dari player dianggap artis (menangani "Judul - Artis" maupun "Artis - Judul"). | Privasi: judul + artis + durasi terkirim ke LRCLIB; bisa dimatikan di Settings |
| Baris lirik di notch kecil | Notch ringkas 380Ã—30 menampilkan baris yang sedang dinyanyikan; frontend tidak berdetak, tapi memasang timer tepat ke awal baris berikutnya. Saat tidak ada lirik atau bagian instrumental ("â™ª"), yang tampil jam. | Hemat CPU |
| Notch menyingkir | Hanya saat jendela aktif menutupi area notch **dan** ada elemen yang bisa diklik tepat di bawahnya (tab, tombol, kolom teks, link, item menu...), dicek lewat UI Automation di 5 titik selebar notch. Tab strip atau title bar yang kosong tidak dihitung, jadi notch tetap berukuran normal. Hasil cek dipakai ulang hingga 3 detik selama jendela tidak berubah, dan dicek ulang setelah klik mouse. Saat menyingkir, notch mengecil jadi **garis putih tipis 90Ã—6** (tidak hilang). Saat mengecas, garis itu hijau dan terisi sesuai persen baterai. Area hover = garis itu, jadi tab di bawahnya tetap bisa diklik. Dideteksi lewat `SetWinEventHook` (foreground, minimize, location change) plus cek ulang tiap 1 detik. | Permintaan pengguna |
| Status sistem | Semua lewat event: callback volume (`IAudioEndpointVolumeCallback`), perangkat audio (`IMMNotificationClient`), `RegisterPowerSettingNotification` (charger, baterai, kecerahan), keyboard hook (hanya melihat tombol Caps Lock dan perubahan layout; isi ketikan tidak dibaca), `NetworkStatusChanged`, `RegNotifyChangeKeyValue` pada ConsentStore (kamera/mic), dan pemantau folder Screenshots. Ditampilkan sebagai "peek" 310Ã—60 selama ~2 detik. | Tanpa polling. Overlay volume bawaan Windows masih ikut muncul; menyembunyikannya belum dikerjakan. |
| Settings | Jendela terpisah dari menu tray: mirror notifikasi (on/off + tombol ke pengaturan Do Not Disturb), baris lirik di notch kecil, link kalender ICS, durasi Pomodoro, pengaturan minum air, kota cuaca. | Disimpan di `%APPDATA%\Notchy\settings.json` |
| Mengetik di notch | `begin_typing`/`end_typing`: notch sementara dibuat focusable dan diambil alih fokusnya lewat `AttachThreadInput` + `SetForegroundWindow`; setelah selesai, fokus dikembalikan ke aplikasi sebelumnya | Windows hanya mengizinkan aplikasi yang menerima input terakhir mengambil fokus, sedangkan klik di notch diterima proses WebView2 |
| Data produktivitas | `todo.json`, `water.json` (7 hari terakhir), `shelf.json` di `%APPDATA%\Notchy`; pengingat memakai timer di frontend | Tanpa server |
| Mirror notifikasi | `UserNotificationListener` (baca saja). Pemicu: Windows menulis `LastNotificationAddedTime` per aplikasi di `HKCU\...\Notifications\Settings` ~75â€“125 ms setelah notifikasi masuk; nilai itu dibaca tiap 200 ms (handle key disimpan, struktur dipindai ulang tiap 10 s; AUMID aplikasi desktop berupa key bertingkat), dan daftar notifikasi baru diambil kalau nilainya berubah. Delay di notch ~0,3â€“0,5 s. Notifikasi beruntun diantre; peek notifikasi/screenshot tertahan saat di-hover dan bisa diklik. Klik = klik notifikasi asli: XML toast diambil dari database notifikasi Windows (`%LOCALAPPDATA%\Microsoft\Windows\Notifications\wpndatabase.db`, baca saja, lewat `rusqlite`) berdasarkan Id yang sama dengan listener (kalau toast-nya sudah diganti, pakai toast terbaru dari aplikasi yang sama). Argumen `launch` diteruskan ke `INotificationActivationCallback::Activate` milik aplikasinya, dengan CLSID dari `ToastActivatorCLSID` di AppxManifest (folder paket, atau `WindowsApps\<paket>` untuk Edge) atau dari `CustomActivator` di registry. Pengirim yang dipakai adalah `PrimaryId` di database (mis. `Edge!https://web.whatsapp.com/`, bukan `Edge!App` dari listener). Toast `activationType="protocol"` dibuka sebagai link. Setelah berhasil, toast dihapus dari pusat notifikasi. Kalau gagal, alasannya ditulis di log dan Notchy membuka aplikasinya lewat `shell:AppsFolder` (cara lama). Teruji: WA Web langsung membuka chat-nya. Supaya tidak double: Do Not Disturb Windows (tombol di Settings). | Event `NotificationChanged` ditolak untuk aplikasi non-MSIX; `RegNotifyChangeKeyValue` tidak terpicu untuk key ini; menulis `ShowBanner` ke registry tidak berlaku langsung; banner Windows 11 bukan jendela tersendiri, jadi tidak bisa disembunyikan |
| Ganti output audio | `IPolicyConfig::SetDefaultEndpoint` (COM tidak resmi, dipakai juga oleh EarTrumpet/SoundSwitch); tombol di notch terbuka berpindah ke perangkat output berikutnya | Tidak ada API resmi |
| Kalender | Link ICS rahasia di Settings, diunduh lewat `Windows.Web.Http.HttpClient` di Rust (server kalender tidak mengizinkan CORS), di-parse di frontend; event berikutnya tampil di tampilan jam; pengingat 10 menit sebelum dan saat mulai; diambil ulang tiap 15 menit | Tanpa login OAuth |
| Shelf | Drop pakai event drag-drop Tauri; drag keluar pakai `tauri-plugin-drag` (CrabNebula) | Drag native Windows tidak tersedia di Tauri inti |
| Tidak merebut fokus | Opsi `focusable: false` di `tauri.conf.json` | Tauri menyusun ulang style jendela, jadi `WS_EX_NOACTIVATE` yang dipasang manual akan terhapus |
| Deteksi full-screen | `SHQueryUserNotificationState` **atau** jendela aktif menutupi seluruh monitor, dicek setiap 1 detik | Shell Windows tidak mendeteksi semua jendela full-screen (misalnya browser F11) |
| Penyimpanan data | File JSON di `%APPDATA%\Notchy\` | Tanpa akun, tanpa server, data tetap di laptop |
| Penjaga (watchdog) | Task Scheduler **"Notchy Watchdog"** (hanya di laptop pengguna, belum ikut installer): saat login + tiap 2 menit menjalankan `notchy.exe --watchdog`, tanpa batas waktu, tetap jalan di baterai, `MultipleInstances IgnoreNew`. Kalau Notchy hidup, salinan baru langsung keluar (single-instance); kalau mati, salinan itu yang mengambil alih (log: `start (restarted by watchdog)`). Exit dari tray menulis `%APPDATA%\Notchy\exited-by-user`, sehingga penjaga membiarkannya mati; dibuka normal (atau autostart) menghapus penanda itu. | Notchy sempat hilang tanpa jejak. Autostart (Run key) hanya berlaku saat login; Notchy bertahan melewati sleep. |
| Update otomatis | `tauri-plugin-updater` dari GitHub Releases repo ini (repo publik): `releases/latest/download/latest.json`. Dicek saat Notchy menyala dan tiap 6 jam. Saat baru menyala, update langsung dipasang (installer NSIS mode passive, Notchy mulai ulang sendiri); setelahnya hanya diunduh lalu muncul peek "Notchy vX siap, klik untuk pasang", jadi tidak pernah restart saat dipakai. Installer ditandatangani: kunci privat di `%USERPROFILE%\.tauri\notchy.key` (tanpa password, **jangan di-commit, wajib dibackup**; kalau hilang, pengguna harus pasang ulang manual), kunci publik di `tauri.conf.json`. Salinan yang jalan dari folder `target\` (milik pengembang) tidak pernah update sendiri. | Pilihan pengguna: tarik dari repo yang sama (dijadikan publik); riwayat git diringkas jadi satu commit supaya bersih |
| Kesehatan tampilan | Halaman notch mengirim detak jantung tiap 5 detik dari `requestAnimationFrame` (hanya jalan kalau halaman hidup **dan** menggambar). Kalau jendela sedang tampil dan 30 detik tanpa detak: halaman dimuat ulang; 60 detik kemudian masih diam: Notchy restart sendiri. Penghitung direset saat jendela tersembunyi, layar terkunci, atau baru bangun dari sleep. Error JS dan promise yang gagal ditulis ke `notchy.log` ("page error"). Tray punya "Muat ulang notch". Teruji dengan membekukan renderer: pulih dalam ~1,5 menit. | 20 Okt 2026 Notchy "hilang" padahal prosesnya hidup dan jendelanya tampil, hanya tidak menggambar apa pun; penjaga dan log tidak menangkapnya |
| Halaman GitHub | `README.md` berbahasa Inggris (plus ringkasan Bahasa Indonesia) dengan banner, GIF demo, dan potongan panel di `docs/`. Sumbernya di `D:\Projects\notchy-promo\readme` (`banner.html` dirender oleh `shot.mjs`; GIF dan potongan dari video promo lewat ffmpeg). Deskripsi repo + 14 topik diisi lewat `gh repo edit`. Gambar social preview harus diunggah manual (Settings → General → Social preview, pakai `docs/banner.png`). Tidak menyebut Nusa. | Permintaan pengguna: repo menarik dilihat orang |
| Tetap paling atas | Tiap detik (di loop watcher) dicek apakah ada jendela biasa yang terlihat di atas notch dalam urutan z; kalau ada, `SetWindowPos(HWND_TOPMOST, SWP_NOMOVE/NOSIZE/NOACTIVATE)` dan dicatat di log (paling sering sekali per 10 menit). Teruji: didorong ke bawah 18 jendela, kembali ke atas < 2 detik. | 21 Okt 2026: notch "hilang" saat Outlook di depan; jendelanya masih bertanda topmost tapi posisinya di bawah jendela biasa (urutan ke-74). Kemungkinan ini juga penyebab "hilang" 20 Okt dan muat ulang otomatis malamnya: WebView2 berhenti menggambar saat tertutup |
| Log "kotak hitam" | `%APPDATA%\Notchy\notchy.log`: `start vX`, `exit (normal)`, dan `PANIC â€¦` (thread, file:baris, pesan). Release memakai `panic = "unwind"`, sehingga panic di thread latar hanya mematikan thread itu. | Notchy dua kali "hilang" tanpa jejak crash (19 Okt 2026). Ada baris PANIC berarti bug kode; tidak ada baris sama sekali berarti dimatikan dari luar (curiga: Trend Micro Apex One, karena Notchy memakai keyboard hook). |
| Distribusi | Installer bawaan Tauri (NSIS, per-user, tanpa admin), dipasang manual sekali; sesudahnya update otomatis (baris "Update otomatis"). Data tetap saat update. Cara membuat rilis: bagian 9. | Hanya untuk diri sendiri dan teman. Tanpa code signing. Windows akan menampilkan peringatan SmartScreen; teman cukup klik "Run anyway". |

---

## 3. Keputusan yang masih terbuka

- [ ] Shortcut keyboard untuk membuka notch.
- [ ] Ikon aplikasi: masih ikon bawaan Tauri. Kandidat: wordmark "notchy" + pil notch dari video promo.
- [ ] Overlay volume bawaan Windows masih ikut muncul bersama peek volume Notchy.

---

## 4. Prinsip desain dan perilaku

**Jendela**
- Tanpa border, transparan, selalu di atas, tidak muncul di taskbar, dan tidak merebut fokus saat diklik.
- Area transparan di luar bentuk notch harus **click-through**.
- Posisi di tengah atas monitor utama, dengan ukuran yang tetap benar di skala 100% sampai 200%.
- Saat aplikasi full-screen berjalan, notch disembunyikan.

**Interaksi**
- Hover membuka notch dan kursor menjauh menutupnya. Tidak ada yang perlu di-pin atau diatur.
- Event penting (charger dicolok, mic dipakai, screenshot baru) memunculkan notch sebentar dalam bentuk ringkas, lalu kembali sendiri.
- Bisa juga dibuka dengan shortcut keyboard (shortcut-nya ditentukan nanti).

**Ukuran notch** (piksel logis, otomatis ikut skala Windows). Dirancang untuk lebar logis ~1500px (laptop 1920px di skala 125%).

| Mode | Lebar Ã— tinggi | Radius sudut bawah | Lengkung "telinga" |
|---|---|---|---|
| Tertutup / istirahat | 150 Ã— 30 | 14 | 10 |
| Menyingkir: ada yang bisa diklik di bawah notch (garis putih; hijau dan terisi sesuai baterai saat mengecas) | 90 Ã— 6 | 4 | 4 |
| Ringkas: musik diputar (cover kiri, equalizer kanan) | 200 Ã— 30 | 14 | 10 |
| Ringkas: mengecas (cincin baterai + petir, jam) | 200 Ã— 30 | 14 | 10 |
| Ringkas: musik + baris lirik (piringan berputar, lirik, equalizer) | 380 Ã— 30 | 14 | 10 |
| Ringkas: timer Pomodoro berjalan (cincin + sisa waktu) | 200 Ã— 30 | 14 | 10 |
| Screenshot baru (thumbnail + tombol) | 360 Ã— 96 | 24 | 16 |
| Notifikasi Windows (ikon app, nama app, judul, isi) | 380 Ã— 72 | 22 | 14 |
| Notifikasi ringkas (charger, headphone, dsb.) | 310 Ã— 60 | 19 | 13 |
| Terbuka (musik, timer, air, shelf, to-do) | 420 Ã— 140 | 32 | 22 |
| Lirik (terbelah) | 520 Ã— 140 | 32 | 22 |

- Notch menempel ke tepi atas layar: sudut atas lurus, hanya sudut bawah yang membulat.
- "Telinga" adalah lengkung cekung kecil di kiri dan kanan atas, supaya notch terlihat menyatu dengan tepi layar.
- Animasi ukuran: `width`/`height` 0.62 detik dengan `cubic-bezier(0.34, 1.32, 0.5, 1)` (sedikit membal), `border-radius` 0.45 detik.
- Jendela Tauri dibuat tetap ~600 Ã— 200 supaya mode terbesar plus bayangan muat; sisanya transparan dan click-through.
- Angka-angka ini titik awal dan boleh disesuaikan setelah dicoba di layar asli.

**Visual**
- Musik: layout player: cover besar sejajar header, progress di bawahnya, tombol lirik di bawah cover, transport di tengah. Warna dasar notch mengambil warna utama cover album selama tampilan musik. Animasi piringan dan equalizer dibuat bertahap (5 kali/detik, serentak) karena animasi 60 fps di jendela transparan memakan ~12% satu core; versi bertahap ~4%.
- Mode istirahat: cuaca di kiri, jam di tengah; setelah 2 menit tanpa gerakan mouse, jam diganti mata.
- **Hitam polos** (dipilih pengguna, menggantikan glow gradien): latar hitam pekat dengan garis tepi putih tipis (16%) yang ikut melengkung di "telinga", plus sorotan putih sangat halus di dasar. Warna hanya dipakai untuk status: hijau mengecas, merah timer fokus, oranye mic dipakai, biru air, dan warna cover album saat musik. Sudut membulat, animasi spring, tipografi bersih.
- Notch terbuka punya 5 halaman (Musik/Jam, Timer, Air, To-do, Shelf): titik di bawah, panah kiri/kanan di sisi, dan scroll mouse (kecuali di atas daftar yang bisa di-scroll). Tombol mic dan output audio ada di halaman Jam.
- Mata: berkedip setiap 3,5â€“6,5 detik, melirik ke arah kursor, dan membesar sedikit saat kursor mendekat (dalam ~250px). Arah pandang dihitung di Rust, maksimal ~60 update/detik, dan hanya dikirim saat berubah.

**Performa**
- Hanya menggambar saat ada yang bergerak. Saat diam, animasi CSS harus berhenti total.
- Pakai event dari Windows, bukan polling cepat. Kalau terpaksa polling, maksimal setiap 1 detik. Pengecualian yang disengaja: nilai registry notifikasi dibaca tiap 200 ms (bacaan nilai saja, ~0,2% CPU) karena tidak ada event yang bisa dipakai dan delay notifikasi harus kecil.
- Animasi yang berjalan terus (piringan, equalizer) dibuat bertahap 5 kali/detik dan serentak; animasi 60 fps di jendela transparan WebView2 mahal (~12% satu core).

---

## 5. Daftar fitur (semuanya masuk scope v1)

Setiap fitur baru dianggap selesai kalau memenuhi **Definition of Done (DoD)**-nya.

### A. Fondasi
- **Jendela notch**: transparan, click-through, posisi benar, animasi buka/tutup saat hover.
  DoD: klik di luar notch tembus ke aplikasi di bawahnya. Tampilan benar di skala 100%, 125%, dan 150%.
- **Tray icon**: menu Start with Windows, Personality (mata on/off), Settings, dan Exit.
- **Autostart** saat Windows menyala.
- **Sembunyi saat full-screen**.
  DoD: notch hilang saat YouTube di-full-screen atau game berjalan, dan muncul lagi setelahnya.
- **Settings** sederhana yang disimpan ke JSON.
  Dibuat bersama fitur pertama yang butuh setting (misalnya cuaca atau air minum). Item "Personality" di tray juga ditambahkan bersamaan dengan fitur mata. Status autostart sudah tersimpan di registry Windows, jadi untuk saat ini belum ada yang perlu disimpan ke JSON.

### B. Musik dan lirik
- **Now playing**: cover, judul, artis, progress bar, tombol play/pause/next/prev, dan seek.
  DoD: berjalan untuk aplikasi Spotify dan YouTube di Chrome/Edge.
- **Lirik tersinkron** dari LRCLIB: notch terbelah dua, baris yang sedang dinyanyikan menyala, dan klik pada sebuah baris akan lompat ke bagian itu.
  - Judul dibersihkan dulu sebelum dicari, misalnya dengan membuang "(Official Video)", "[Lyrics]", "- Topic".
  - Kalau player tidak mengirim posisi detik, lirik tetap tampil tanpa sorotan.
  - Kalau lirik tidak ditemukan, tampilkan pesan singkat dan jangan error.
  - Hasil pencarian di-cache supaya lagu yang sama tidak dicari berulang kali.

### C. Produktivitas
- **Pomodoro**: fokus 25 menit dan istirahat 5 menit (bisa diatur), countdown tampil di notch, ada bunyi chime.
- **To-do dengan pengingat**: tambah, edit, dan centang task, opsional dengan jam. Notch akan "menepuk" saat waktunya tiba.
- **Pengingat minum air**: ukuran botol, target harian, jam aktif, dan grafik minum hari ini.
- **Shelf file**: drag file ke notch untuk disimpan sementara, drag keluar ke chat/email/folder, dan klik untuk membuka.

### D. Status sistem
- **Overlay volume dan brightness**.
- **Caps Lock dan perubahan bahasa keyboard**.
- **Baterai**: notifikasi saat charger dicolok/dicabut, plus persentase.
- **Wi-Fi**: notifikasi saat koneksi putus atau tersambung kembali.
- **Headphone Bluetooth**: notifikasi saat tersambung, plus baterai kalau perangkatnya melaporkannya.

### E. Fitur tambahan Notchy
1. **Indikator kamera dan mikrofon**: titik warna di notch saat ada aplikasi yang memakai kamera/mic.
   Sumber data: registry `CapabilityAccessManager\ConsentStore\{webcam,microphone}`.
2. **Mute mic global**: satu klik di notch untuk mute/unmute mic default, dengan status yang selalu terlihat.
3. **Preview screenshot**: setelah Win+Shift+S, thumbnail muncul di notch, bisa di-drag atau disimpan, dan otomatis masuk Shelf.
4. **Cuaca**: suhu dan ikon di mode istirahat, dari Open-Meteo.
5. **Ganti output audio** (speaker â†” headphone) dari notch.
   âš ï¸ Memakai API Windows yang tidak resmi (`IPolicyConfig`).
6. **Mirror notifikasi Windows**, misalnya pesan WhatsApp Desktop tampil di notch, bisa diklik untuk membuka aplikasinya. Dengan Do Not Disturb Windows aktif, notifikasi hanya muncul di notch (tidak double).
   Hasil riset: membaca notifikasi tidak butuh MSIX; yang butuh MSIX hanya event perubahannya (lihat bagian 2). Klik hanya membuka aplikasinya, bukan chat/email tertentu, dan tombol aksi (Balas, Terima) tidak bisa dipakai, karena aksi itu hanya bisa dijalankan Windows sendiri.
7. **Event kalender berikutnya** dan countdown meeting, lewat link ICS rahasia dari Google/Outlook Calendar (tanpa login OAuth).

---

## 6. Urutan pengerjaan

Semua fitur di atas masuk v1. Urutan ini ada karena sebagian fitur bergantung pada fitur lain, bukan karena sebagian ditunda.

1. **Fondasi (A)**. Semua fitur lain bergantung pada jendela notch ini. âœ…
2. **Musik dan lirik (B)**, fitur utama sekaligus uji coba sistem "tampilan per mode". âœ… Sudah dites dengan YouTube; aplikasi Spotify belum dites.
3. **Status sistem (D)** dan **E1â€“E4**. Semuanya event ringan yang memakai pola "muncul sebentar lalu kembali" yang sama. âœ… Sudah dites: volume, kecerahan, Caps Lock, screenshot, mode mengecas, mute mic, cuaca. Belum dites: ganti bahasa keyboard, Wi-Fi putus/sambung, headphone, kamera/mic dipakai, jendela Settings. Baterai headphone belum dibuat. Screenshot belum bisa di-drag (menunggu Shelf).
4. **Produktivitas (C)**. âœ… Notch terbuka sekarang punya 5 halaman (Musik/Jam, Timer, Air, To-do, Shelf) dengan titik dan panah kiri/kanan, plus scroll mouse. Sudah dites: timer jalan dan tampil di notch kecil, catat minum, tambah to-do dengan mengetik di notch, screenshot masuk Shelf, navigasi. Belum dites: drag file dari Explorer ke notch, drag file keluar dari Shelf, bunyi chime, dan pengingat to-do/minum saat waktunya tiba.
5. **E5â€“E7**. âœ… Riset E6: membaca notifikasi **tidak** butuh MSIX (`UserNotificationListener` mengizinkan aplikasi biasa), tapi event perubahannya butuh MSIX. Sudah dites: notifikasi uji tampil di notch, tombol output audio tampil, parser kalender (zona waktu, event berulang, EXDATE, dibatalkan, seharian). Belum dites: berpindah output (laptop ini hanya punya satu output), kalender dengan link ICS asli, jendela Settings.
6. **Penyempurnaan setelah v1** âœ…
   - Tampilan hitam polos; garis menyingkir hanya saat ada yang bisa diklik di bawah notch (UI Automation).
   - Musik: layout player, warna dari cover, piringan berputar, equalizer bergerak, baris lirik di notch kecil (jam saat tidak ada lirik), jam di tengah saat istirahat.
   - Notifikasi: delay ~0,3â€“0,5 s (sebelumnya sampai 3 s), antrean, bisa diklik, panduan Do Not Disturb. Sudah dites: delay, antrean, dan tanpa banner saat DND aktif. Belum dites: klik notifikasi untuk membuka aplikasinya.

Setiap langkah selesai = DoD-nya terpenuhi + dites sendiri di laptop.

---

## 7. Non-goals (sengaja tidak dikerjakan)

- Versi macOS atau Linux.
- Akun, login, sinkronisasi cloud, atau analytics.
- Code signing Windows (peringatan SmartScreen tetap muncul saat pasang manual).
- Lirik dari Spotify atau sumber lain yang butuh login/scraping.
- Jadwal sholat.
- Tema yang bisa dikustomisasi bebas (cukup satu gaya yang rapi dulu).

---

## 8. Catatan untuk setiap sesi pengerjaan

- Baca file ini dulu sebelum mulai.
- Kalau ada keputusan baru, catat di bagian 2 atau 3, jangan hanya disebut di chat.
- Kalau ada fitur yang mau ditambah atau dibuang, ubah bagian 5 dan 7 dulu, baru kerjakan kodenya.

---

## 9. Rilis

| Versi | Tanggal | Isi |
|---|---|---|
| v0.1.0 | 19 Okt 2026 | Rilis pertama untuk teman: semua fitur bagian 5, tampilan hitam polos, musik + lirik di notch kecil |
| v0.1.1 | 19 Okt 2026 | Notifikasi ~0,3â€“0,5 s, antrean, klik untuk membuka aplikasi, tombol Do Not Disturb di Settings |
| v0.1.2 | 20 Okt 2026 | Klik notifikasi = klik notifikasi asli (WA Web langsung ke chat-nya); update otomatis dari GitHub Releases; log kotak hitam, panic tidak mematikan aplikasi; perbaikan notifikasi bertumpuk dengan halaman saat di-hover. Teruji: v0.1.1 uji update sendiri ke v0.1.2 dalam ~9 detik |
| v0.1.3 | 20 Okt 2026 | Notch yang berhenti tergambar pulih sendiri (detak jantung, muat ulang, restart); error tampilan masuk log; menu "Muat ulang notch" di tray |
| v0.1.4 | 20 Okt 2026 | Baris lirik di notch kecil langsung muncul setelah ganti lagu, pause/play, geser posisi, atau kembali dari tampilan lain (sebelumnya jam internalnya basi sehingga lirik tertahan sampai ada kejadian lain, misalnya minimize semua jendela) |
| v0.1.5 | 21 Okt 2026 | Notch tidak lagi "hilang" di balik jendela lain (misalnya Outlook): kalau terdorong ke bawah, dalam 1 detik kembali paling atas |

**Cara membuat rilis baru** (sejak v0.1.2, satu perintah)
1. Pastikan semua perubahan sudah di-commit, lalu `.\scripts\release.ps1 <versi> "catatan rilis"`.
2. Skrip itu: menaikkan versi di `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`; menjeda `Notchy Watchdog` dan menutup Notchy; build installer bertanda tangan (kunci `%USERPROFILE%\.tauri\notchy.key`, mengulang otomatis kalau antivirus memicu os error 1224); menjalankan Notchy lagi dan menyalakan penjaga; menulis `latest.json`; commit, tag `v<versi>`, push; membuat GitHub Release berisi setup, `.sig`, dan `latest.json`.
   Peringatan "__TAURI_BUNDLE_TYPE variable not found in binary" saat pengemasan diulang itu aman selama `notchy.exe` **di dalam installer** memuat `__TAURI_BUNDLE_TYPE_VAR_NSS`; tanpa penanda itu updater tidak tahu jenis installernya. Cek dengan 7-Zip: `7z x Notchy_<versi>_x64-setup.exe notchy.exe -r`, lalu cari teks itu. Exe di `target\release` bisa tetap `VAR_UNK` (Tauri menandai salinan yang dikemas); itu normal. v0.1.3–v0.1.5 sudah dicek: `NSS`.
3. Salinan teman yang sudah v0.1.2 ke atas akan update sendiri (saat menyala, atau lewat peek dalam 6 jam). Yang masih v0.1.0/v0.1.1 perlu menjalankan setup terbaru sekali secara manual (ada di halaman Releases).

**Video promo** (bukan bagian repo ini): `D:\Projects\notchy-promo` â€“ Reels 9:16, 27 detik, konsep "Satu garis, semuanya"; render lewat `render-par.mjs` (batas segmen dikunci ke frame), musik `music.html`, campur VO `mix.mjs`.
