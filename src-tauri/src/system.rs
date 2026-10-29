//! System status shown in the notch: power and battery, brightness, Caps Lock,
//! keyboard language, network, camera/mic use and new screenshots.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Mutex, OnceLock};
use std::time::Duration;

use serde::Serialize;
use serde_json::json;
use tauri::{AppHandle, Emitter};
use windows::core::{w, GUID, PCWSTR};
use windows::Networking::Connectivity::{
    NetworkConnectivityLevel, NetworkInformation, NetworkStatusChangedEventHandler,
};
use windows::Win32::Foundation::{HANDLE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Globalization::LCIDToLocaleName;
use windows::Win32::System::Com::{CoInitializeEx, CoTaskMemFree, COINIT_MULTITHREADED};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Power::{
    GetSystemPowerStatus, RegisterPowerSettingNotification, POWERBROADCAST_SETTING, SYSTEM_POWER_STATUS,
};
use windows::Win32::System::SystemServices::{
    GUID_ACDC_POWER_SOURCE, GUID_BATTERY_PERCENTAGE_REMAINING, GUID_VIDEO_CURRENT_MONITOR_BRIGHTNESS,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyState, GetKeyboardLayout, VK_CAPITAL};
use windows::Win32::UI::Shell::{SHGetKnownFolderPath, FOLDERID_Screenshots, KF_FLAG_DEFAULT};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, CreateWindowExW, DefWindowProcW, GetForegroundWindow, GetWindowThreadProcessId,
    RegisterClassW, SetWindowsHookExW, DEVICE_NOTIFY_WINDOW_HANDLE, HWND_MESSAGE, KBDLLHOOKSTRUCT,
    PBT_POWERSETTINGCHANGE, WH_KEYBOARD_LL, WINDOW_EX_STYLE, WINDOW_STYLE, WM_KEYUP, WM_POWERBROADCAST,
    WM_SYSKEYUP, WNDCLASSW,
};
use winreg::enums::HKEY_CURRENT_USER;
use winreg::RegKey;

use crate::peek;

static APP: OnceLock<AppHandle> = OnceLock::new();

pub fn start(app: AppHandle) {
    APP.set(app.clone()).ok();
    start_network(app.clone());
    start_privacy(app.clone());
    start_screenshots(app);
    start_keyboard_worker();
}

/// Runs on the input-hook thread, which has a message loop: a hidden window
/// for power notifications, and a keyboard hook for Caps Lock and language.
pub fn install_on_hook_thread() {
    unsafe {
        let instance = GetModuleHandleW(None).unwrap_or_default();
        let class = w!("NotchyPower");
        RegisterClassW(&WNDCLASSW {
            lpfnWndProc: Some(power_proc),
            hInstance: instance.into(),
            lpszClassName: class,
            ..Default::default()
        });
        let Ok(hwnd) = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            class,
            PCWSTR::null(),
            WINDOW_STYLE(0),
            0,
            0,
            0,
            0,
            Some(HWND_MESSAGE),
            None,
            Some(instance.into()),
            None,
        ) else {
            return;
        };
        for guid in [&GUID_ACDC_POWER_SOURCE, &GUID_BATTERY_PERCENTAGE_REMAINING, &GUID_VIDEO_CURRENT_MONITOR_BRIGHTNESS] {
            let _ = RegisterPowerSettingNotification(HANDLE(hwnd.0), guid, DEVICE_NOTIFY_WINDOW_HANDLE);
        }
        if crate::settings::get_settings().caps_lock_peek {
            let _ = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), None, 0);
        }
    }
}

// ---------------------------------------------------------------- power

#[derive(Serialize, Clone, Copy, PartialEq)]
pub struct Power {
    charging: bool,
    percent: u8,
    /// False on desktops without a battery.
    battery: bool,
}

fn read_power() -> Power {
    let mut s = SYSTEM_POWER_STATUS::default();
    let _ = unsafe { GetSystemPowerStatus(&mut s) };
    Power {
        charging: s.ACLineStatus == 1,
        percent: if s.BatteryLifePercent <= 100 { s.BatteryLifePercent } else { 100 },
        battery: s.BatteryFlag != 128,
    }
}

static LAST_POWER: Mutex<Option<Power>> = Mutex::new(None);
static LAST_BRIGHTNESS: Mutex<Option<u32>> = Mutex::new(None);

#[tauri::command]
pub fn power_state() -> Power {
    read_power()
}

unsafe extern "system" fn power_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // Back from sleep (PBT_APMRESUMEAUTOMATIC / PBT_APMRESUMESUSPEND): a good moment to look for updates.
    if msg == WM_POWERBROADCAST && matches!(wparam.0 as u32, 0x12 | 0x7) {
        crate::update::woke();
        return LRESULT(1);
    }
    if msg == WM_POWERBROADCAST && wparam.0 as u32 == PBT_POWERSETTINGCHANGE {
        let setting = unsafe { &*(lparam.0 as *const POWERBROADCAST_SETTING) };
        let Some(app) = APP.get() else { return LRESULT(1) };
        if setting.PowerSetting == GUID_VIDEO_CURRENT_MONITOR_BRIGHTNESS {
            let level = setting.Data[0] as u32;
            // Windows sends the current value right after registering; only changes count.
            let prev = LAST_BRIGHTNESS.lock().unwrap().replace(level);
            if prev.is_some_and(|p| p != level) {
                peek(app, "brightness", json!({ "level": level as f64 / 100.0 }));
            }
        } else {
            let now = read_power();
            let prev = LAST_POWER.lock().unwrap().replace(now);
            if prev != Some(now) {
                let _ = app.emit("power", now);
            }
            if prev.is_some_and(|p| p.charging != now.charging) && now.battery {
                peek(app, "power", json!({ "charging": now.charging, "percent": now.percent }));
            }
        }
        return LRESULT(1);
    }
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

// ---------------------------------------------------------------- keyboard

static KEY_TX: OnceLock<mpsc::Sender<bool>> = OnceLock::new();
static KEY_PENDING: AtomicBool = AtomicBool::new(false);

/// Only looks at *which* key went up to spot Caps Lock; nothing typed is kept.
unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let up = wparam.0 as u32 == WM_KEYUP || wparam.0 as u32 == WM_SYSKEYUP;
    if code >= 0 && up {
        let info = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
        let caps = info.vkCode == VK_CAPITAL.0 as u32;
        if (caps || !KEY_PENDING.swap(true, Ordering::Relaxed)) && let Some(tx) = KEY_TX.get() {
            let _ = tx.send(caps);
        }
    }
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

/// Keyboard layout of the active window's thread, as a locale name like "en-US".
fn foreground_layout() -> (isize, String) {
    unsafe {
        let fg = GetForegroundWindow();
        let thread = GetWindowThreadProcessId(fg, None);
        let lang = GetKeyboardLayout(thread).0 as usize as u32 & 0xFFFF;
        let mut buf = [0u16; 85];
        let len = LCIDToLocaleName(lang, Some(&mut buf), 0).max(1) as usize - 1;
        (fg.0 as isize, String::from_utf16_lossy(&buf[..len]))
    }
}

fn start_keyboard_worker() {
    let (tx, rx) = mpsc::channel::<bool>();
    KEY_TX.set(tx).ok();
    std::thread::spawn(move || {
        let mut last = foreground_layout();
        for caps in rx {
            // Let the key take effect first.
            std::thread::sleep(Duration::from_millis(120));
            let Some(app) = APP.get() else { continue };
            if caps {
                let on = unsafe { GetKeyState(VK_CAPITAL.0 as i32) } & 1 == 1;
                peek(app, "caps", json!({ "on": on }));
            } else {
                KEY_PENDING.store(false, Ordering::Relaxed);
            }
            // A layout change in the same window means the user switched language.
            let now = foreground_layout();
            if now.0 == last.0 && now.1 != last.1 && !now.1.is_empty() {
                peek(app, "language", json!({ "locale": now.1 }));
            }
            last = now;
        }
    });
}

// ---------------------------------------------------------------- network

fn start_network(app: AppHandle) {
    std::thread::spawn(move || {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        }
        let read = || -> (bool, String, bool) {
            match NetworkInformation::GetInternetConnectionProfile() {
                Ok(p) => (
                    p.GetNetworkConnectivityLevel().is_ok_and(|l| l == NetworkConnectivityLevel::InternetAccess),
                    p.ProfileName().map(|s| s.to_string()).unwrap_or_default(),
                    p.IsWlanConnectionProfile().unwrap_or(false),
                ),
                Err(_) => (false, String::new(), false),
            }
        };
        let last = Mutex::new(read());
        let handler = NetworkStatusChangedEventHandler::new(move |_| {
            // The event fires in bursts while a connection settles.
            std::thread::sleep(Duration::from_millis(800));
            let now = read();
            let mut last = last.lock().unwrap();
            if now.0 != last.0 || (now.0 && now.1 != last.1) {
                peek(&app, "network", json!({ "online": now.0, "name": now.1, "wifi": now.2 }));
            }
            *last = now;
            Ok(())
        });
        let _ = NetworkInformation::NetworkStatusChanged(&handler);
        loop {
            std::thread::park();
        }
    });
}

// ---------------------------------------------------------------- camera & mic in use

#[derive(Serialize, Clone, Default, PartialEq)]
pub struct InUse {
    /// App names currently using the device, e.g. ["Zoom"].
    camera: Vec<String>,
    mic: Vec<String>,
}

static IN_USE: Mutex<Option<InUse>> = Mutex::new(None);

const CONSENT: &str = r"Software\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore";

/// Windows records each app's camera/mic use under ConsentStore; an entry
/// whose LastUsedTimeStop is 0 is in use right now. Returns the raw entries:
/// the key name (a package id, or an exe path with '#' for '\') and when the
/// use started (FILETIME).
pub fn sessions(device: &str) -> Vec<(String, u64)> {
    let Ok(root) = RegKey::predef(HKEY_CURRENT_USER).open_subkey(format!(r"{CONSENT}\{device}")) else {
        return vec![];
    };
    let active = |key: &RegKey| {
        let start: u64 = key.get_value("LastUsedTimeStart").unwrap_or(0);
        let stop: u64 = key.get_value("LastUsedTimeStop").unwrap_or(1);
        (start != 0 && stop == 0).then_some(start)
    };
    let mut found = vec![];
    for name in root.enum_keys().flatten() {
        let Ok(key) = root.open_subkey(&name) else { continue };
        if name == "NonPackaged" {
            for exe in key.enum_keys().flatten() {
                if let Some(start) = key.open_subkey(&exe).ok().and_then(|k| active(&k)) {
                    found.push((exe, start));
                }
            }
        } else if let Some(start) = active(&key) {
            found.push((name, start));
        }
    }
    found
}

/// "C:#Program Files#Zoom#bin#Zoom.exe" -> "Zoom"; "Microsoft.WindowsCamera_8wekyb3d8bbwe" -> "WindowsCamera".
pub fn app_name(id: &str) -> String {
    if id.contains('#') {
        let file = id.rsplit('#').next().unwrap_or(id);
        return file.trim_end_matches(".exe").trim_end_matches(".EXE").to_string();
    }
    let id = id.split('_').next().unwrap_or(id);
    id.rsplit('.').next().unwrap_or(id).to_string()
}

/// While Notchy's own camera mirror runs, its WebView2 shows up as a camera
/// user; that one isn't news.
pub static MIRROR_ON: AtomicBool = AtomicBool::new(false);

fn is_ours(id: &str) -> bool {
    let lower = id.to_ascii_lowercase();
    lower.ends_with("#notchy.exe") || lower.ends_with("#msedgewebview2.exe")
}

fn apps_using(device: &str) -> Vec<String> {
    let mirror = device == "webcam" && MIRROR_ON.load(Ordering::Relaxed);
    sessions(device).iter().filter(|(id, _)| !(mirror && is_ours(id))).map(|(id, _)| app_name(id)).collect()
}

#[tauri::command]
pub fn in_use_state() -> InUse {
    InUse { camera: apps_using("webcam"), mic: apps_using("microphone") }
}

fn start_privacy(app: AppHandle) {
    for device in ["webcam", "microphone"] {
        let app = app.clone();
        std::thread::spawn(move || {
            let Ok(key) = RegKey::predef(HKEY_CURRENT_USER).open_subkey(format!(r"{CONSENT}\{device}")) else { return };
            loop {
                let now = in_use_state();
                let mut last = IN_USE.lock().unwrap();
                if last.as_ref() != Some(&now) {
                    let _ = app.emit("in-use", &now);
                    // Announce a device being picked up, not put down.
                    if let Some(prev) = last.as_ref() {
                        for (kind, apps, before) in [("camera", &now.camera, &prev.camera), ("mic-in-use", &now.mic, &prev.mic)] {
                            if let Some(new) = apps.iter().find(|a| !before.contains(a)) {
                                peek(&app, kind, json!({ "app": new }));
                            }
                        }
                    }
                    *last = Some(now);
                }
                drop(last);
                if device == "microphone" {
                    crate::calls::poke();
                }
                // Blocks until something under this key changes.
                if !wait_for_change(&key) {
                    return;
                }
            }
        });
    }
}

fn wait_for_change(key: &RegKey) -> bool {
    use windows::Win32::System::Registry::{
        RegNotifyChangeKeyValue, HKEY, REG_NOTIFY_CHANGE_LAST_SET, REG_NOTIFY_CHANGE_NAME,
    };
    let hkey = HKEY(key.raw_handle() as _);
    let filter = REG_NOTIFY_CHANGE_LAST_SET | REG_NOTIFY_CHANGE_NAME;
    unsafe { RegNotifyChangeKeyValue(hkey, true, filter, None, false) }.is_ok()
}

// ---------------------------------------------------------------- screenshots

fn screenshots_dir() -> Option<PathBuf> {
    unsafe {
        let p = SHGetKnownFolderPath(&FOLDERID_Screenshots as *const GUID, KF_FLAG_DEFAULT, None).ok()?;
        let path = p.to_string().ok().map(PathBuf::from);
        CoTaskMemFree(Some(p.0 as _));
        path
    }
}

/// Win+Shift+S and Win+PrtSc save into Pictures\Screenshots; each new file is shown.
fn start_screenshots(app: AppHandle) {
    use notify::{EventKind, RecursiveMode, Watcher};
    let Some(dir) = screenshots_dir() else { return };
    std::thread::spawn(move || {
        let (tx, rx) = mpsc::channel();
        let Ok(mut watcher) = notify::recommended_watcher(tx) else { return };
        if watcher.watch(&dir, RecursiveMode::NonRecursive).is_err() {
            return;
        }
        for event in rx.into_iter().flatten() {
            if !matches!(event.kind, EventKind::Create(_)) {
                continue;
            }
            for path in event.paths {
                let image = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|e| matches!(e.to_ascii_lowercase().as_str(), "png" | "jpg" | "jpeg"));
                if image {
                    // Give the writer a moment to finish the file.
                    std::thread::sleep(Duration::from_millis(400));
                    let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                    peek(&app, "screenshot", json!({ "path": path, "name": name }));
                }
            }
        }
    });
}

/// Opens a screenshot, or shows it in Explorer. Only files in the Screenshots folder.
#[tauri::command]
pub fn open_screenshot(path: String, reveal: bool) {
    let path = PathBuf::from(path);
    let Some(dir) = screenshots_dir() else { return };
    if path.parent() != Some(dir.as_path()) || !path.is_file() {
        return;
    }
    let _ = if reveal {
        std::process::Command::new("explorer").arg(format!("/select,{}", path.display())).spawn()
    } else {
        std::process::Command::new("explorer").arg(&path).spawn()
    };
}
