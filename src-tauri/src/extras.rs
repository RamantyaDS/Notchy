//! Smaller tools: keep-awake, the keyboard-cleaning lock, text from the screen
//! (OCR), dark mode, per-app volume, CPU/RAM load, idle time, global hotkeys,
//! and peeks sent from outside (`notchy --peek`, `notchy://` links and Claude
//! Code hooks calling `notchy --agent`).

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde_json::json;
use tauri::{AppHandle, Emitter};
use windows::core::{Interface, HSTRING};
use windows::Win32::Foundation::{CloseHandle, FILETIME, LPARAM, WPARAM};
use windows::Win32::Media::Audio::{
    eMultimedia, eRender, AudioSessionStateExpired, IAudioSessionControl2, IAudioSessionManager2, IMMDeviceEnumerator,
    ISimpleAudioVolume, MMDeviceEnumerator,
};
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED};
use windows::Win32::System::Power::{
    SetThreadExecutionState, ES_CONTINUOUS, ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED,
};
use windows::Win32::System::SystemInformation::{GetTickCount, GlobalMemoryStatusEx, MEMORYSTATUSEX};
use windows::Win32::System::Threading::{
    GetSystemTimes, OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetLastInputInfo, RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, LASTINPUTINFO, MOD_ALT, MOD_CONTROL,
    MOD_NOREPEAT, MOD_SHIFT, MOD_WIN,
};
use windows::Win32::UI::WindowsAndMessaging::{
    PostThreadMessageW, SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_APP, WM_SETTINGCHANGE,
};
use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE};
use winreg::RegKey;

use crate::peek;

static APP: OnceLock<AppHandle> = OnceLock::new();

pub fn start(app: AppHandle) {
    APP.set(app.clone()).ok();
    register_link(crate::settings::get_settings().external_peeks);
    // A copy started with `--peek` etc. that became the running Notchy shows it too,
    // once the page is listening.
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| is_external_arg(a)) {
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(4));
            handle_args(&app, &args);
        });
    }
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

// ---------------------------------------------------------------- keep awake

/// 0: off. u64::MAX: until turned off. Otherwise: unix ms when it ends.
static AWAKE_UNTIL: AtomicU64 = AtomicU64::new(0);
static AWAKE_GEN: AtomicU64 = AtomicU64::new(0);

/// `minutes` < 0 stops, 0 keeps awake until stopped, > 0 for that many minutes.
/// The screen and the PC stay on; Windows' power settings are untouched, and it
/// ends with Notchy (the request belongs to this thread).
#[tauri::command]
pub fn keep_awake(app: AppHandle, minutes: i64) {
    let generation = AWAKE_GEN.fetch_add(1, Ordering::SeqCst) + 1;
    if minutes < 0 {
        AWAKE_UNTIL.store(0, Ordering::SeqCst);
        let _ = app.emit("keep-awake", 0u64);
        return;
    }
    let until = if minutes == 0 { u64::MAX } else { now_ms() + minutes as u64 * 60_000 };
    AWAKE_UNTIL.store(until, Ordering::SeqCst);
    let _ = app.emit("keep-awake", until);
    std::thread::spawn(move || {
        unsafe { SetThreadExecutionState(ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED) };
        while AWAKE_GEN.load(Ordering::SeqCst) == generation && now_ms() < until {
            std::thread::sleep(Duration::from_secs(1));
        }
        unsafe { SetThreadExecutionState(ES_CONTINUOUS) };
        if AWAKE_GEN.load(Ordering::SeqCst) == generation {
            AWAKE_UNTIL.store(0, Ordering::SeqCst);
            let _ = app.emit("keep-awake", 0u64);
        }
    });
}

#[tauri::command]
pub fn keep_awake_state() -> u64 {
    AWAKE_UNTIL.load(Ordering::SeqCst)
}

// ---------------------------------------------------------------- keyboard cleaning

/// Read by the keyboard hook: while set, every key is swallowed.
pub static KEYS_LOCKED: AtomicBool = AtomicBool::new(false);
static LOCK_GEN: AtomicU64 = AtomicU64::new(0);

/// Ignores the keyboard for `seconds` (at most 2 minutes) so it can be wiped.
/// The mouse still works to unlock early; Ctrl+Alt+Del is never blocked by Windows.
#[tauri::command]
pub fn lock_keyboard(app: AppHandle, seconds: u64) {
    let seconds = seconds.clamp(5, 120);
    let thread = HOOK_THREAD.load(Ordering::Relaxed);
    if thread != 0 {
        let _ = unsafe { PostThreadMessageW(thread, WM_KEYBOARD_HOOK, WPARAM(0), LPARAM(0)) };
    }
    let generation = LOCK_GEN.fetch_add(1, Ordering::SeqCst) + 1;
    KEYS_LOCKED.store(true, Ordering::SeqCst);
    let _ = app.emit("keyboard-lock", now_ms() + seconds * 1000);
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(seconds));
        if LOCK_GEN.load(Ordering::SeqCst) == generation {
            KEYS_LOCKED.store(false, Ordering::SeqCst);
            let _ = app.emit("keyboard-lock", 0u64);
        }
    });
}

#[tauri::command]
pub fn unlock_keyboard(app: AppHandle) {
    LOCK_GEN.fetch_add(1, Ordering::SeqCst);
    KEYS_LOCKED.store(false, Ordering::SeqCst);
    let _ = app.emit("keyboard-lock", 0u64);
}

// ---------------------------------------------------------------- text from the screen

/// When the user asked for OCR: the next image on the clipboard (from the
/// Windows snipping overlay) is read as text, within a minute.
static OCR_ASKED: Mutex<Option<Instant>> = Mutex::new(None);

/// Opens Windows' own area picker (the Win+Shift+S overlay). Its result lands on
/// the clipboard, where `clipboard_image` picks it up.
#[tauri::command]
pub fn start_ocr() -> Result<(), String> {
    *OCR_ASKED.lock().unwrap() = Some(Instant::now());
    crate::shortcuts::shell_open("ms-screenclip:")
}

/// Called by the clipboard watcher for every new image. True if it was taken for OCR.
pub fn clipboard_image(dib: &[u8]) -> bool {
    let asked = OCR_ASKED.lock().unwrap().take();
    if !asked.is_some_and(|at| at.elapsed() < Duration::from_secs(60)) {
        return false;
    }
    let dib = dib.to_vec();
    std::thread::spawn(move || {
        let _ = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        let Some(app) = APP.get() else { return };
        match ocr(&dib) {
            Ok(text) if !text.trim().is_empty() => {
                let words = text.split_whitespace().count();
                crate::clipboard::set_text(&text);
                let preview: String = text.split_whitespace().collect::<Vec<_>>().join(" ").chars().take(80).collect();
                peek(app, "ocr", json!({ "words": words, "text": preview }));
            }
            _ => peek(app, "ocr", json!({ "words": 0 })),
        }
    });
    true
}

/// Turns a clipboard DIB into a BMP file in memory and runs Windows' built-in
/// OCR on it (offline, in the languages installed in Windows).
fn ocr(dib: &[u8]) -> windows::core::Result<String> {
    use windows::Graphics::Imaging::BitmapDecoder;
    use windows::Media::Ocr::OcrEngine;
    use windows::Storage::Streams::{DataWriter, InMemoryRandomAccessStream};

    let file = bmp_file(dib).ok_or_else(|| windows::core::Error::from(windows::Win32::Foundation::E_INVALIDARG))?;
    let stream = InMemoryRandomAccessStream::new()?;
    let writer = DataWriter::CreateDataWriter(&stream)?;
    writer.WriteBytes(&file)?;
    writer.StoreAsync()?.get()?;
    writer.FlushAsync()?.get()?;
    writer.DetachStream()?;
    stream.Seek(0)?;
    let decoder = BitmapDecoder::CreateAsync(&stream)?.get()?;
    let bitmap = decoder.GetSoftwareBitmapAsync()?.get()?;
    let engine = OcrEngine::TryCreateFromUserProfileLanguages()?;
    let result = engine.RecognizeAsync(&bitmap)?.get()?;
    let lines = result.Lines()?;
    let mut out = Vec::new();
    for i in 0..lines.Size()? {
        out.push(lines.GetAt(i)?.Text()?.to_string());
    }
    Ok(out.join("\n"))
}

/// A BMP file header in front of a packed DIB.
pub fn bmp_file(dib: &[u8]) -> Option<Vec<u8>> {
    if dib.len() < 40 {
        return None;
    }
    let u32_at = |i: usize| u32::from_le_bytes(dib[i..i + 4].try_into().unwrap());
    let header = u32_at(0) as usize;
    let bits = u16::from_le_bytes(dib[14..16].try_into().unwrap()) as usize;
    let compression = u32_at(16);
    let used = u32_at(32) as usize;
    let palette = if bits <= 8 { if used > 0 { used } else { 1 << bits } } else { used };
    let masks = if compression == 3 && header == 40 { 12 } else { 0 };
    let offset = 14 + header + palette * 4 + masks;
    if offset > 14 + dib.len() {
        return None;
    }
    let mut file = Vec::with_capacity(14 + dib.len());
    file.extend_from_slice(b"BM");
    file.extend_from_slice(&((14 + dib.len()) as u32).to_le_bytes());
    file.extend_from_slice(&[0; 4]);
    file.extend_from_slice(&(offset as u32).to_le_bytes());
    file.extend_from_slice(dib);
    Some(file)
}

// ---------------------------------------------------------------- dark mode

const PERSONALIZE: &str = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";

fn dark_mode() -> bool {
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(PERSONALIZE)
        .and_then(|k| k.get_value::<u32, _>("AppsUseLightTheme"))
        .is_ok_and(|v| v == 0)
}

/// Flips Windows between light and dark (apps and the taskbar). Returns the new state.
pub fn toggle_dark_mode() -> Result<bool, String> {
    let dark = !dark_mode();
    let key = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(PERSONALIZE, KEY_READ | KEY_SET_VALUE)
        .map_err(|e| e.to_string())?;
    let light = u32::from(!dark);
    key.set_value("AppsUseLightTheme", &light).map_err(|e| e.to_string())?;
    key.set_value("SystemUsesLightTheme", &light).map_err(|e| e.to_string())?;
    // Tell running apps; some don't answer, so never wait on them here.
    std::thread::spawn(|| unsafe {
        let name = HSTRING::from("ImmersiveColorSet");
        let _ = SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            WPARAM(0),
            LPARAM(name.as_ptr() as isize),
            SMTO_ABORTIFHUNG,
            500,
            None,
        );
    });
    if let Some(app) = APP.get() {
        peek(app, "darkmode", json!({ "dark": dark }));
    }
    Ok(dark)
}

// ---------------------------------------------------------------- per-app volume

#[derive(Serialize, Clone, PartialEq, Debug)]
pub struct AppVolume {
    /// Program name, e.g. "Spotify"; also the key for changes (one app can have several sessions).
    name: String,
    volume: f32,
    muted: bool,
}

fn process_name(pid: u32) -> Option<String> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 512];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, windows::core::PWSTR(buf.as_mut_ptr()), &mut len);
        let _ = CloseHandle(handle);
        ok.ok()?;
        let path = String::from_utf16_lossy(&buf[..len as usize]);
        let file = path.rsplit('\\').next()?.to_string();
        Some(pretty_name(&file))
    }
}

/// "spotify.exe" -> "Spotify", "msedge.exe" -> "Msedge".
pub fn pretty_name(file: &str) -> String {
    let stem = file.strip_suffix(".exe").or_else(|| file.strip_suffix(".EXE")).unwrap_or(file);
    let mut c = stem.chars();
    match c.next() {
        Some(f) => f.to_uppercase().chain(c).collect(),
        None => String::new(),
    }
}

/// Every live audio session on the default output, as (app name, volume control).
fn sessions() -> Vec<(String, ISimpleAudioVolume)> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let Ok(enumerator) = CoCreateInstance::<_, IMMDeviceEnumerator>(&MMDeviceEnumerator, None, CLSCTX_ALL) else {
            return vec![];
        };
        let Ok(device) = enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia) else { return vec![] };
        let Ok(manager) = device.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None) else { return vec![] };
        let Ok(list) = manager.GetSessionEnumerator() else { return vec![] };
        let mut found = vec![];
        for i in 0..list.GetCount().unwrap_or(0) {
            let Ok(control) = list.GetSession(i) else { continue };
            if control.GetState().is_ok_and(|s| s == AudioSessionStateExpired) {
                continue;
            }
            let Ok(control2) = control.cast::<IAudioSessionControl2>() else { continue };
            let Ok(volume) = control.cast::<ISimpleAudioVolume>() else { continue };
            let name = if control2.IsSystemSoundsSession().0 == 0 {
                "Windows".to_string()
            } else {
                let Ok(pid) = control2.GetProcessId() else { continue };
                match process_name(pid) {
                    Some(n) if pid != 0 => n,
                    _ => continue,
                }
            };
            found.push((name, volume));
        }
        found
    }
}

/// Apps with sound sessions, one row per app.
#[tauri::command]
pub async fn mixer_list() -> Vec<AppVolume> {
    let mut rows: Vec<AppVolume> = vec![];
    for (name, control) in sessions() {
        let volume = unsafe { control.GetMasterVolume() }.unwrap_or(1.0);
        let muted = unsafe { control.GetMute() }.is_ok_and(|m| m.as_bool());
        match rows.iter_mut().find(|r| r.name == name) {
            Some(row) => row.volume = row.volume.max(volume),
            None => rows.push(AppVolume { name, volume, muted }),
        }
    }
    rows.sort_by_key(|r| (r.name == "Windows", r.name.to_lowercase()));
    rows
}

#[tauri::command]
pub async fn mixer_set(name: String, volume: Option<f32>, muted: Option<bool>) {
    for (n, control) in sessions() {
        if n != name {
            continue;
        }
        unsafe {
            if let Some(v) = volume {
                let _ = control.SetMasterVolume(v.clamp(0.0, 1.0), std::ptr::null());
            }
            if let Some(m) = muted {
                let _ = control.SetMute(m, std::ptr::null());
            }
        }
    }
}

// ---------------------------------------------------------------- CPU, RAM, idle

static LAST_CPU: Mutex<Option<(u64, u64)>> = Mutex::new(None);

fn ticks(t: FILETIME) -> u64 {
    ((t.dwHighDateTime as u64) << 32) | t.dwLowDateTime as u64
}

/// CPU and memory use in percent. CPU is measured since the previous call.
#[tauri::command]
pub fn system_load() -> (u32, u32) {
    let (mut idle, mut kernel, mut user) = (FILETIME::default(), FILETIME::default(), FILETIME::default());
    let cpu = if unsafe { GetSystemTimes(Some(&mut idle), Some(&mut kernel), Some(&mut user)) }.is_ok() {
        // Kernel time includes idle time.
        let now = (ticks(idle), ticks(kernel) + ticks(user));
        let prev = LAST_CPU.lock().unwrap().replace(now);
        match prev {
            Some(p) if now.1 > p.1 => {
                let busy = 1.0 - (now.0 - p.0) as f64 / (now.1 - p.1) as f64;
                (busy.clamp(0.0, 1.0) * 100.0).round() as u32
            }
            _ => 0,
        }
    } else {
        0
    };
    let mut mem = MEMORYSTATUSEX { dwLength: size_of::<MEMORYSTATUSEX>() as u32, ..Default::default() };
    let ram = if unsafe { GlobalMemoryStatusEx(&mut mem) }.is_ok() { mem.dwMemoryLoad } else { 0 };
    (cpu, ram)
}

/// Seconds since the last keyboard or mouse input (for the eye-break reminder).
#[tauri::command]
pub fn idle_seconds() -> u32 {
    let mut info = LASTINPUTINFO { cbSize: size_of::<LASTINPUTINFO>() as u32, dwTime: 0 };
    if unsafe { GetLastInputInfo(&mut info) }.as_bool() {
        unsafe { GetTickCount() }.wrapping_sub(info.dwTime) / 1000
    } else {
        0
    }
}

// ---------------------------------------------------------------- hotkeys

const HOTKEY_OPEN: i32 = 1;
const HOTKEY_OCR: i32 = 2;
/// Posted to the hook thread to register the hotkeys again after a settings change.
pub const WM_REREGISTER: u32 = WM_APP + 7;
/// Posted to the hook thread to install the keyboard hook (for the cleaning lock).
pub const WM_KEYBOARD_HOOK: u32 = WM_APP + 8;
static HOOK_THREAD: AtomicU32 = AtomicU32::new(0);

/// "Win+Alt+N" -> modifiers and virtual key. Letters, digits and F1–F24.
pub fn parse_hotkey(text: &str) -> Option<(HOT_KEY_MODIFIERS, u32)> {
    let mut mods = HOT_KEY_MODIFIERS(0);
    let mut key = None;
    for part in text.split('+').map(|p| p.trim()).filter(|p| !p.is_empty()) {
        match part.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => mods |= MOD_CONTROL,
            "alt" => mods |= MOD_ALT,
            "shift" => mods |= MOD_SHIFT,
            "win" | "windows" => mods |= MOD_WIN,
            k if k.len() == 1 && k.chars().all(|c| c.is_ascii_alphanumeric()) => {
                key = Some(k.to_ascii_uppercase().chars().next().unwrap() as u32)
            }
            k if k.starts_with('f') && k[1..].parse::<u32>().is_ok_and(|n| (1..=24).contains(&n)) => {
                key = Some(0x70 + k[1..].parse::<u32>().unwrap() - 1)
            }
            _ => return None,
        }
    }
    // A bare letter would steal normal typing.
    if mods.0 == 0 {
        return None;
    }
    key.map(|k| (mods | MOD_NOREPEAT, k))
}

/// Runs on the hook thread (it owns the message loop the hotkeys post to).
pub fn register_hotkeys() {
    HOOK_THREAD.store(unsafe { windows::Win32::System::Threading::GetCurrentThreadId() }, Ordering::Relaxed);
    let s = crate::settings::get_settings();
    unsafe {
        let _ = UnregisterHotKey(None, HOTKEY_OPEN);
        let _ = UnregisterHotKey(None, HOTKEY_OCR);
        for (id, text) in [(HOTKEY_OPEN, &s.open_hotkey), (HOTKEY_OCR, &s.ocr_hotkey)] {
            if let Some((mods, vk)) = parse_hotkey(text)
                && RegisterHotKey(None, id, mods, vk).is_err()
            {
                crate::log_line(&format!("hotkey {text} is taken by another app"));
            }
        }
    }
}

/// Settings changed: the hook thread registers the hotkeys again.
pub fn hotkeys_changed() {
    let thread = HOOK_THREAD.load(Ordering::Relaxed);
    if thread != 0 {
        let _ = unsafe { PostThreadMessageW(thread, WM_REREGISTER, WPARAM(0), LPARAM(0)) };
    }
}

pub fn on_hotkey(id: i32) {
    let Some(app) = APP.get() else { return };
    match id {
        HOTKEY_OPEN => {
            let _ = app.emit("notch-hotkey", ());
        }
        HOTKEY_OCR => {
            let _ = start_ocr();
        }
        _ => {}
    }
}

// ---------------------------------------------------------------- peeks from outside

pub fn is_external_arg(a: &str) -> bool {
    a == "--peek" || a == "--agent" || a.starts_with("notchy://")
}

/// Registers (or removes) the `notchy://` link for this user.
pub fn register_link(on: bool) {
    let root = RegKey::predef(HKEY_CURRENT_USER);
    let path = r"Software\Classes\notchy";
    if !on {
        let _ = root.delete_subkey_all(path);
        return;
    }
    let Ok(exe) = std::env::current_exe() else { return };
    if let Ok((key, _)) = root.create_subkey(path) {
        let _ = key.set_value("", &"URL:Notchy");
        let _ = key.set_value("URL Protocol", &"");
        if let Ok((cmd, _)) = key.create_subkey(r"shell\open\command") {
            let _ = cmd.set_value("", &format!("\"{}\" \"%1\"", exe.display()));
        }
    }
}

/// What an outside call asks for.
#[derive(Debug, PartialEq)]
pub enum External {
    Peek { text: String, detail: String, icon: String, sound: bool, seconds: u32 },
    Agent { state: String, app: String, text: String },
}

fn clip(s: &str, n: usize) -> String {
    s.chars().filter(|c| !c.is_control()).take(n).collect()
}

/// `--peek "text" [--detail ".."] [--icon done|error|info|bell] [--sound] [--seconds n]`,
/// `--agent working|waiting|done|idle [--text ".."]`, or a `notchy://peek?...` /
/// `notchy://agent?...` link. Text only; nothing is ever run.
pub fn parse_external(args: &[String]) -> Option<External> {
    let value = |flag: &str| args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).cloned();
    if let Some(link) = args.iter().find(|a| a.starts_with("notchy://")) {
        let rest = &link["notchy://".len()..];
        let (what, query) = rest.split_once('?').unwrap_or((rest, ""));
        let what = what.trim_end_matches('/');
        let get = |k: &str| {
            query.split('&').filter_map(|kv| kv.split_once('=')).find(|(key, _)| *key == k).map(|(_, v)| url_decode(v))
        };
        return match what {
            "peek" => Some(External::Peek {
                text: clip(&get("text")?, 120),
                detail: clip(&get("detail").unwrap_or_default(), 120),
                icon: get("icon").unwrap_or_else(|| "info".into()),
                sound: get("sound").is_some_and(|v| v == "1" || v == "true"),
                seconds: get("seconds").and_then(|v| v.parse().ok()).unwrap_or(5).clamp(2, 30),
            }),
            "agent" => Some(External::Agent {
                state: get("state").unwrap_or_else(|| "done".into()),
                app: clip(&get("app").unwrap_or_else(|| "Claude Code".into()), 40),
                text: clip(&get("text").unwrap_or_default(), 120),
            }),
            _ => None,
        };
    }
    if let Some(text) = value("--peek") {
        return Some(External::Peek {
            text: clip(&text, 120),
            detail: clip(&value("--detail").unwrap_or_default(), 120),
            icon: value("--icon").unwrap_or_else(|| "info".into()),
            sound: args.iter().any(|a| a == "--sound"),
            seconds: value("--seconds").and_then(|v| v.parse().ok()).unwrap_or(5).clamp(2, 30),
        });
    }
    value("--agent").map(|state| External::Agent {
        state,
        app: clip(&value("--app").unwrap_or_else(|| "Claude Code".into()), 40),
        text: clip(&value("--text").unwrap_or_default(), 120),
    })
}

fn url_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => out.push(b' '),
            b'%' if i + 2 < bytes.len() => {
                match std::str::from_utf8(&bytes[i + 1..i + 3]).ok().and_then(|h| u8::from_str_radix(h, 16).ok()).ok_or(()) {
                    Ok(b) => {
                        out.push(b);
                        i += 2;
                    }
                    Err(_) => out.push(b'%'),
                }
            }
            b => out.push(b),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

pub fn handle_args(app: &AppHandle, args: &[String]) {
    let settings = crate::settings::get_settings();
    match parse_external(args) {
        Some(External::Peek { text, detail, icon, sound, seconds }) if settings.external_peeks => {
            peek(app, "custom", json!({ "text": text, "detail": detail, "icon": icon, "sound": sound, "seconds": seconds }));
        }
        Some(External::Agent { state, app: agent_app, text }) if settings.agent_status => {
            let _ = app.emit("agent", json!({ "state": state, "app": agent_app, "text": text }));
        }
        _ => {}
    }
}

/// The path to this exe, for the Claude Code hook snippet in Settings.
#[tauri::command]
pub fn exe_path() -> String {
    std::env::current_exe().map(|p| p.display().to_string()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parses_peek_flags() {
        let got = parse_external(&args(&["notchy.exe", "--peek", "Build selesai", "--icon", "done", "--sound"]));
        assert_eq!(
            got,
            Some(External::Peek { text: "Build selesai".into(), detail: "".into(), icon: "done".into(), sound: true, seconds: 5 })
        );
    }

    #[test]
    fn parses_links_and_agent() {
        let got = parse_external(&args(&["notchy.exe", "notchy://peek?text=Halo%20dunia&icon=bell&seconds=99"]));
        assert_eq!(
            got,
            Some(External::Peek { text: "Halo dunia".into(), detail: "".into(), icon: "bell".into(), sound: false, seconds: 30 })
        );
        let got = parse_external(&args(&["notchy.exe", "--agent", "waiting", "--app", "Antigravity", "--text", "Needs approval"]));
        assert_eq!(got, Some(External::Agent { state: "waiting".into(), app: "Antigravity".into(), text: "Needs approval".into() }));
        let got_default = parse_external(&args(&["notchy.exe", "--agent", "done"]));
        assert_eq!(got_default, Some(External::Agent { state: "done".into(), app: "Claude Code".into(), text: "".into() }));
        let got_url = parse_external(&args(&["notchy.exe", "notchy://agent?app=Codex&state=working"]));
        assert_eq!(got_url, Some(External::Agent { state: "working".into(), app: "Codex".into(), text: "".into() }));
        assert_eq!(parse_external(&args(&["notchy.exe", "--settings"])), None);
        assert_eq!(parse_external(&args(&["notchy.exe", "notchy://format-c"])), None);
    }

    #[test]
    fn hotkeys_need_a_modifier() {
        assert!(parse_hotkey("Win+Alt+N").is_some());
        assert!(parse_hotkey("ctrl+shift+f5").is_some());
        assert!(parse_hotkey("N").is_none());
        assert!(parse_hotkey("").is_none());
        assert!(parse_hotkey("Win+Banana").is_none());
    }

    #[test]
    fn app_names_and_bmp_header() {
        assert_eq!(pretty_name("spotify.exe"), "Spotify");
        let mut dib = vec![0u8; 40 + 4];
        dib[0] = 40;
        dib[14] = 32;
        let file = bmp_file(&dib).unwrap();
        assert_eq!(&file[..2], b"BM");
        assert_eq!(u32::from_le_bytes(file[10..14].try_into().unwrap()), 54);
        assert!(bmp_file(&[0u8; 10]).is_none());
    }
}
