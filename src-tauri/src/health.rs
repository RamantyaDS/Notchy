//! Keeps the notch page itself alive. Notchy can be running while its page has
//! stopped drawing (seen 20 Oct 2026: process fine, window visible, nothing on
//! screen), which neither the watchdog nor the panic log notices.
//!
//! The page sends a heartbeat every 5 s from an animation frame, which only runs
//! while the page is alive *and* rendering. If the window is showing and no beat
//! came for a while, the page is reloaded; if a reload doesn't help, Notchy
//! restarts itself. Page errors go to notchy.log too.

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, Manager};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, IsWindowVisible};

use crate::{log_line, NOTCH_HWND};

/// When the last beat came, in ms since 1970.
static LAST_BEAT: AtomicU64 = AtomicU64::new(0);
/// Whether the page was already reloaded since the last beat (then the next step is a restart, after 60 s).
static RELOADS: AtomicU32 = AtomicU32::new(0);

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

#[tauri::command]
pub fn heartbeat() {
    LAST_BEAT.store(now_ms(), Ordering::Relaxed);
    RELOADS.store(0, Ordering::Relaxed);
}

/// An uncaught error or rejected promise in the page.
#[tauri::command]
pub fn log_js(msg: String) {
    static LAST: Mutex<String> = Mutex::new(String::new());
    let msg: String = msg.chars().take(400).collect();
    let mut last = LAST.lock().unwrap();
    if *last != msg {
        log_line(&format!("page error: {msg}"));
        *last = msg;
    }
}

/// Reloads the notch page (also the tray's "Reload" item).
pub fn reload(app: &AppHandle, why: &str) {
    log_line(&format!("reloading the notch page ({why})"));
    LAST_BEAT.store(now_ms(), Ordering::Relaxed);
    if let Some(window) = app.get_webview_window("main") {
        if let Err(e) = window.reload() {
            log_line(&format!("reload failed: {e}"));
        }
    }
}

pub fn start(app: AppHandle) {
    LAST_BEAT.store(now_ms(), Ordering::Relaxed);
    std::thread::spawn(move || {
        let mut last_check = Instant::now();
        loop {
            std::thread::sleep(Duration::from_secs(5));
            let slept = last_check.elapsed() > Duration::from_secs(20);
            last_check = Instant::now();
            let hwnd = HWND(NOTCH_HWND.load(Ordering::Relaxed) as _);
            // Hidden (full screen), locked (no foreground window) or just woken up:
            // a page may rightly skip frames then, so the clock starts over.
            let showing = unsafe { IsWindowVisible(hwnd).as_bool() && !GetForegroundWindow().is_invalid() };
            if !showing || slept {
                LAST_BEAT.store(now_ms(), Ordering::Relaxed);
                continue;
            }
            let quiet = now_ms().saturating_sub(LAST_BEAT.load(Ordering::Relaxed));
            let reloads = RELOADS.load(Ordering::Relaxed);
            if quiet > 30_000 << reloads {
                if reloads >= 1 {
                    // Reloading didn't bring it back (a hung renderer can stay hung): start over.
                    log_line("page still silent after a reload, restarting Notchy");
                    app.restart();
                }
                RELOADS.store(reloads + 1, Ordering::Relaxed);
                reload(&app, &format!("no heartbeat for {} s", quiet / 1000));
            }
        }
    });
}
