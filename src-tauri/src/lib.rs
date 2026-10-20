use std::sync::atomic::{AtomicBool, AtomicI32, AtomicIsize, Ordering};
use std::sync::{mpsc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, WebviewWindow};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::Foundation::POINT;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED,
};
use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, SetWinEventHook, HWINEVENTHOOK, UIA_ButtonControlTypeId,
    UIA_CheckBoxControlTypeId, UIA_ComboBoxControlTypeId, UIA_CONTROLTYPE_ID,
    UIA_EditControlTypeId, UIA_HyperlinkControlTypeId, UIA_ListItemControlTypeId,
    UIA_MenuItemControlTypeId, UIA_RadioButtonControlTypeId, UIA_SplitButtonControlTypeId,
    UIA_TabItemControlTypeId,
};
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::Shell::{
    SHQueryUserNotificationState, QUNS_BUSY, QUNS_PRESENTATION_MODE, QUNS_RUNNING_D3D_FULL_SCREEN,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetClassNameW, GetForegroundWindow, GetMessageW, GetWindowLongPtrW,
    GetWindowRect, GetWindowThreadProcessId, IsIconic, IsWindowVisible, SetForegroundWindow,
    SetWindowLongPtrW, SetWindowsHookExW, ShowWindow,
    EVENT_OBJECT_LOCATIONCHANGE, EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_MINIMIZEEND,
    EVENT_SYSTEM_MINIMIZESTART, GWL_EXSTYLE, MSG, MSLLHOOKSTRUCT, OBJID_WINDOW, SW_HIDE,
    SW_SHOWNOACTIVATE, WH_MOUSE_LL, WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS, WM_LBUTTONUP,
    WM_MOUSEMOVE,
    WS_EX_TRANSPARENT,
};

mod audio;
mod health;
mod media;
mod notifications;
mod settings;
mod store;
mod system;
mod update;

/// Briefly shows a status in the notch (volume, charger, Wi-Fi, ...).
/// `data` must be a JSON object; its `kind` tells the frontend which view to use.
pub(crate) fn peek(app: &AppHandle, kind: &str, mut data: serde_json::Value) {
    data["kind"] = kind.into();
    let _ = app.emit("peek", data);
}

static APP: OnceLock<AppHandle> = OnceLock::new();
static NOTCH_HWND: AtomicIsize = AtomicIsize::new(0);
static HOVER_TX: OnceLock<mpsc::Sender<bool>> = OnceLock::new();
static EYES_TX: OnceLock<mpsc::Sender<()>> = OnceLock::new();
static EYES_PENDING: AtomicBool = AtomicBool::new(false);
static CURSOR_X: AtomicI32 = AtomicI32::new(0);
static CURSOR_Y: AtomicI32 = AtomicI32::new(0);
static WATCH_TX: OnceLock<mpsc::Sender<()>> = OnceLock::new();
static TUCKED: AtomicBool = AtomicBool::new(false);
static CLICKED: AtomicBool = AtomicBool::new(false);
static TYPING: AtomicBool = AtomicBool::new(false);
static PREV_FOREGROUND: AtomicIsize = AtomicIsize::new(0);
static HIT: Mutex<Hit> = Mutex::new(Hit { size: (0.0, 0.0), rect: (0, 0, 0, 0), inside: false });

/// Area of the window the notch currently occupies. Everything else is click-through.
struct Hit {
    /// Notch size in logical px, as reported by the frontend.
    size: (f64, f64),
    /// Same area in physical screen px: left, top, right, bottom.
    rect: (i32, i32, i32, i32),
    inside: bool,
}

fn main_window() -> Option<WebviewWindow> {
    APP.get()?.get_webview_window("main")
}

fn hwnd(window: &WebviewWindow) -> HWND {
    HWND(window.hwnd().unwrap().0)
}

/// The frontend reports the notch size whenever it changes; the notch is
/// always centered at the top of the window.
#[tauri::command]
fn set_notch_size(w: f64, h: f64) {
    HIT.lock().unwrap().size = (w, h);
    update_hit_rect();
}

fn update_hit_rect() {
    let Some(window) = main_window() else { return };
    let geometry = match (window.is_visible(), window.outer_position(), window.outer_size()) {
        (Ok(true), Ok(pos), Ok(size)) => Some((pos, size, window.scale_factor().unwrap_or(1.0))),
        _ => None,
    };
    let mut hit = HIT.lock().unwrap();
    hit.rect = match geometry {
        Some((pos, size, scale)) => {
            let w = (hit.size.0 * scale) as i32;
            let h = (hit.size.1 * scale) as i32;
            let left = pos.x + (size.width as i32 - w) / 2;
            (left, pos.y, left + w, pos.y + h)
        }
        None => (0, 0, 0, 0),
    };
}

fn on_mouse_move(x: i32, y: i32) {
    CURSOR_X.store(x, Ordering::Relaxed);
    CURSOR_Y.store(y, Ordering::Relaxed);
    if !EYES_PENDING.swap(true, Ordering::Relaxed) {
        if let Some(tx) = EYES_TX.get() {
            let _ = tx.send(());
        }
    }

    let changed = {
        let mut hit = HIT.lock().unwrap();
        let (l, _, r, b) = hit.rect;
        // No top check: the hook sees the point before it's clamped to the
        // screen, so pushing against the top edge reports y < 0.
        let inside = x >= l && x < r && y < b;
        let changed = inside != hit.inside;
        hit.inside = inside;
        changed.then_some(inside)
    };
    if let Some(inside) = changed {
        if let Some(tx) = HOVER_TX.get() {
            let _ = tx.send(inside);
        }
    }
}

/// Flips only WS_EX_TRANSPARENT, directly. Tauri's `set_ignore_cursor_events`
/// also toggles WS_EX_LAYERED, which makes WebView2 rebuild its surface (lag).
fn set_click_through(on: bool) {
    let h = HWND(NOTCH_HWND.load(Ordering::Relaxed) as _);
    unsafe {
        let style = GetWindowLongPtrW(h, GWL_EXSTYLE);
        let t = WS_EX_TRANSPARENT.0 as isize;
        SetWindowLongPtrW(h, GWL_EXSTYLE, if on { style | t } else { style & !t });
    }
}

/// The notch normally never takes keyboard focus. To type (e.g. a new task),
/// the frontend asks for it here, and gives it back when done. While typing the
/// window stays clickable even when the cursor leaves it.
#[tauri::command]
fn begin_typing(window: WebviewWindow) {
    TYPING.store(true, Ordering::Relaxed);
    let _ = window.set_focusable(true);
    // Queued after set_focusable, which rewrites the window style on the UI thread.
    let w = window.clone();
    let _ = window.run_on_main_thread(move || unsafe {
        set_click_through(false);
        // Windows only lets the app that got the last input take the foreground,
        // and the click went to WebView2's process. Sharing the active window's
        // input queue for a moment makes the switch allowed.
        let fg = GetForegroundWindow();
        PREV_FOREGROUND.store(fg.0 as isize, Ordering::Relaxed);
        let fg_thread = GetWindowThreadProcessId(fg, None);
        let me = GetCurrentThreadId();
        let attached = fg_thread != me && AttachThreadInput(me, fg_thread, true).as_bool();
        let _ = SetForegroundWindow(hwnd(&w));
        let _ = w.set_focus();
        if attached {
            let _ = AttachThreadInput(me, fg_thread, false);
        }
    });
}

#[tauri::command]
fn end_typing(window: WebviewWindow) {
    TYPING.store(false, Ordering::Relaxed);
    let _ = window.set_focusable(false);
    let _ = window.run_on_main_thread(|| unsafe {
        set_click_through(!HIT.lock().unwrap().inside);
        // Hand the keyboard back to whatever was active before.
        let prev = HWND(PREV_FOREGROUND.swap(0, Ordering::Relaxed) as _);
        if !prev.is_invalid() {
            let _ = SetForegroundWindow(prev);
        }
    });
}

/// The hook callback must return fast or Windows drops the hook, so the style
/// change (which waits on the UI thread) and the frontend event run here instead.
fn start_hover_emitter() {
    let (tx, rx) = mpsc::channel::<bool>();
    HOVER_TX.set(tx).ok();
    std::thread::spawn(move || {
        for inside in rx {
            set_click_through(!inside && !TYPING.load(Ordering::Relaxed));
            if let Some(window) = main_window() {
                let _ = window.emit("notch-hover", inside);
            }
        }
    });
}

/// Tells the frontend where the eyes should look: a unit-ish vector from the
/// notch toward the cursor, plus how close it is. At most ~60 updates/s, and
/// only when it changes.
fn start_eyes_emitter() {
    let (tx, rx) = mpsc::channel::<()>();
    EYES_TX.set(tx).ok();
    std::thread::spawn(move || {
        let mut last = (0.0f64, 0.0f64, 0.0f64);
        for _ in rx {
            std::thread::sleep(Duration::from_millis(16));
            EYES_PENDING.store(false, Ordering::Relaxed);
            let ((l, t, r, b), inside) = {
                let hit = HIT.lock().unwrap();
                (hit.rect, hit.inside)
            };
            if inside || r <= l || TUCKED.load(Ordering::Relaxed) {
                continue;
            }
            let dx = (CURSOR_X.load(Ordering::Relaxed) - (l + r) / 2) as f64;
            let dy = (CURSOR_Y.load(Ordering::Relaxed) - (t + b) / 2) as f64;
            let dist = dx.hypot(dy).max(1.0);
            // Full deflection once the cursor is ~300px away.
            let reach = (dist / 300.0).min(1.0);
            // 1 when the cursor is right at the notch, 0 from ~250px away.
            let near = 1.0 - (dist / 250.0).min(1.0);
            let look = (dx / dist * reach, dy / dist * reach, near);
            if (look.0 - last.0).abs() < 0.03
                && (look.1 - last.1).abs() < 0.03
                && (look.2 - last.2).abs() < 0.03
            {
                continue;
            }
            last = look;
            if let Some(window) = main_window() {
                let _ = window.emit("eyes-look", look);
            }
        }
    });
}

unsafe extern "system" fn mouse_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 && wparam.0 as u32 == WM_MOUSEMOVE {
        let info = unsafe { &*(lparam.0 as *const MSLLHOOKSTRUCT) };
        on_mouse_move(info.pt.x, info.pt.y);
    } else if code >= 0 && wparam.0 as u32 == WM_LBUTTONUP {
        // A click can change what's under the notch (e.g. a new tab); re-check on the next pass.
        CLICKED.store(true, Ordering::Relaxed);
    }
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

/// A low-level mouse hook tells us when the cursor enters the notch, even while
/// the window ignores the mouse. Event-driven, so no polling.
fn start_mouse_hook() {
    std::thread::spawn(|| unsafe {
        let _hook = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), None, 0).unwrap();
        // Window events for the watcher share this thread's message loop.
        let flags = WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS;
        for (min, max) in [
            (EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_FOREGROUND),
            (EVENT_SYSTEM_MINIMIZESTART, EVENT_SYSTEM_MINIMIZEEND),
            (EVENT_OBJECT_LOCATIONCHANGE, EVENT_OBJECT_LOCATIONCHANGE),
        ] {
            SetWinEventHook(min, max, None, Some(win_event_proc), 0, 0, flags);
        }
        system::install_on_hook_thread();
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {}
    });
}

/// Centers the window at the top of the primary monitor.
fn place_window(window: &WebviewWindow) {
    let (Ok(Some(monitor)), Ok(size)) = (window.primary_monitor(), window.outer_size()) else { return };
    let target = PhysicalPosition::new(
        monitor.position().x + (monitor.size().width as i32 - size.width as i32) / 2,
        monitor.position().y,
    );
    if window.outer_position().ok() != Some(target) {
        let _ = window.set_position(target);
    }
}

fn fullscreen_app_running() -> bool {
    let shell_says_busy = matches!(
        unsafe { SHQueryUserNotificationState() },
        Ok(s) if s == QUNS_BUSY || s == QUNS_RUNNING_D3D_FULL_SCREEN || s == QUNS_PRESENTATION_MODE
    );
    shell_says_busy || foreground_covers_monitor()
}

/// The shell misses some full-screen windows (e.g. browser F11), so also check
/// whether the active window covers its whole monitor. The desktop doesn't count.
fn foreground_covers_monitor() -> bool {
    unsafe {
        let fg = GetForegroundWindow();
        if fg.is_invalid() {
            return false;
        }
        let mut class = [0u16; 32];
        let len = GetClassNameW(fg, &mut class) as usize;
        let class = String::from_utf16_lossy(&class[..len]);
        if class == "Progman" || class == "WorkerW" {
            return false;
        }
        let mut rect = RECT::default();
        let mut info = MONITORINFO { cbSize: size_of::<MONITORINFO>() as u32, ..Default::default() };
        if GetWindowRect(fg, &mut rect).is_err()
            || !GetMonitorInfoW(MonitorFromWindow(fg, MONITOR_DEFAULTTONEAREST), &mut info).as_bool()
        {
            return false;
        }
        let m = info.rcMonitor;
        rect.left <= m.left && rect.top <= m.top && rect.right >= m.right && rect.bottom >= m.bottom
    }
}

/// Whether the active window reaches into the closed notch's spot, e.g. a
/// maximized browser whose tabs sit under it.
fn foreground_covers_notch() -> bool {
    unsafe {
        let ours = HWND(NOTCH_HWND.load(Ordering::Relaxed) as _);
        let fg = GetForegroundWindow();
        if fg.is_invalid() || fg == ours || IsIconic(fg).as_bool() || !IsWindowVisible(fg).as_bool() {
            return false;
        }
        let mut class = [0u16; 32];
        let len = GetClassNameW(fg, &mut class) as usize;
        let class = String::from_utf16_lossy(&class[..len]);
        if matches!(class.as_str(), "Progman" | "WorkerW" | "Shell_TrayWnd") {
            return false;
        }
        let (mut win, mut fgr) = (RECT::default(), RECT::default());
        if GetWindowRect(ours, &mut win).is_err() || GetWindowRect(fg, &mut fgr).is_err() {
            return false;
        }
        let scale = GetDpiForWindow(ours) as f64 / 96.0;
        let cx = (win.left + win.right) / 2;
        let half = (75.0 * scale) as i32;
        let bottom = win.top + (30.0 * scale) as i32;
        fgr.left < cx + half && fgr.right > cx - half && fgr.top < bottom && fgr.bottom > win.top
    }
}

/// Whether something you'd click sits under the closed notch: a tab, button,
/// text box, link... Empty title bars and tab strips don't count, so the notch
/// only shrinks when it is actually in the way. Asks UI Automation at a few
/// points across the notch.
fn something_under_notch(uia: &IUIAutomation) -> bool {
    const CLICKABLE: [UIA_CONTROLTYPE_ID; 10] = [
        UIA_ButtonControlTypeId,
        UIA_CheckBoxControlTypeId,
        UIA_ComboBoxControlTypeId,
        UIA_EditControlTypeId,
        UIA_HyperlinkControlTypeId,
        UIA_ListItemControlTypeId,
        UIA_MenuItemControlTypeId,
        UIA_RadioButtonControlTypeId,
        UIA_SplitButtonControlTypeId,
        UIA_TabItemControlTypeId,
    ];
    unsafe {
        let ours = HWND(NOTCH_HWND.load(Ordering::Relaxed) as _);
        let mut win = RECT::default();
        if GetWindowRect(ours, &mut win).is_err() {
            return false;
        }
        let scale = GetDpiForWindow(ours) as f64 / 96.0;
        let cx = (win.left + win.right) / 2;
        let y = win.top + (15.0 * scale) as i32;
        let me = std::process::id() as i32;
        [-70.0, -35.0, 0.0, 35.0, 70.0].iter().any(|dx| {
            let pt = POINT { x: cx + (dx * scale) as i32, y };
            uia.ElementFromPoint(pt).is_ok_and(|el| {
                el.CurrentProcessId().is_ok_and(|pid| pid != me)
                    && el.CurrentControlType().is_ok_and(|t| CLICKABLE.contains(&t))
            })
        })
    }
}

#[tauri::command]
fn is_tucked() -> bool {
    TUCKED.load(Ordering::Relaxed)
}

unsafe extern "system" fn win_event_proc(
    _hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    id_object: i32,
    _id_child: i32,
    _thread: u32,
    _time: u32,
) {
    // Location changes fire for every caret and cursor too; only the active window matters.
    if event == EVENT_OBJECT_LOCATIONCHANGE
        && (id_object != OBJID_WINDOW.0 || hwnd != unsafe { GetForegroundWindow() })
    {
        return;
    }
    if let Some(tx) = WATCH_TX.get() {
        let _ = tx.send(());
    }
}

/// Re-checks on window events (activated, moved, maximized, minimized), with a
/// 1-second fallback that also follows resolution/DPI changes:
/// - full-screen app: hide the window entirely;
/// - active window with something clickable under the notch: shrink the notch to a line.
/// The 1-second pass also catches changes inside a window, like a new browser tab.
fn start_watcher(window: WebviewWindow) {
    let (tx, rx) = mpsc::channel::<()>();
    WATCH_TX.set(tx).ok();
    std::thread::spawn(move || {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        }
        let uia: Option<IUIAutomation> =
            unsafe { CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER).ok() };
        // UI Automation queries cost a little, so the last answer is reused while the
        // same window sits in the same place: up to 3 s, or 150 ms while events stream in.
        let mut last_check: Option<(isize, (i32, i32, i32, i32), bool, Instant)> = None;
        let mut under_notch = |fresh: bool| -> bool {
            let Some(uia) = uia.as_ref() else { return true };
            let fg = unsafe { GetForegroundWindow() };
            let mut r = RECT::default();
            let _ = unsafe { GetWindowRect(fg, &mut r) };
            let key = (fg.0 as isize, (r.left, r.top, r.right, r.bottom));
            let fresh = fresh || CLICKED.swap(false, Ordering::Relaxed);
            if let Some((h, rect, result, at)) = last_check {
                let max_age = if fresh { Duration::from_millis(150) } else { Duration::from_secs(3) };
                if (h, rect) == key && at.elapsed() < max_age {
                    return result;
                }
            }
            let result = something_under_notch(uia);
            last_check = Some((key.0, key.1, result, Instant::now()));
            result
        };
        loop {
            let timed_out = rx.recv_timeout(Duration::from_secs(1)).is_err();
            while rx.try_recv().is_ok() {}

            let h = hwnd(&window);
            let hide = fullscreen_app_running();
            let visible = unsafe { IsWindowVisible(h).as_bool() };
            // Both directions go through ShowWindow: Tauri's hide() is a no-op here
            // because Tauri never saw the window being shown.
            if hide && visible {
                unsafe { let _ = ShowWindow(h, SW_HIDE); }
            } else if !hide && !visible {
                unsafe { let _ = ShowWindow(h, SW_SHOWNOACTIVATE); }
            }

            let tucked = !hide && foreground_covers_notch() && under_notch(!timed_out);
            if TUCKED.swap(tucked, Ordering::Relaxed) != tucked {
                let _ = window.emit("notch-tucked", tucked);
            }

            if timed_out || hide == visible {
                place_window(&window);
                update_hit_rect();
            }
        }
    });
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let autostart = CheckMenuItem::with_id(
        app,
        "autostart",
        "Start with Windows",
        true,
        app.autolaunch().is_enabled().unwrap_or(false),
        None::<&str>,
    )?;
    let settings = MenuItem::with_id(app, "settings", "Settings…", true, None::<&str>)?;
    let reload = MenuItem::with_id(app, "reload", "Muat ulang notch", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Exit", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[&settings, &autostart, &reload, &PredefinedMenuItem::separator(app)?, &quit],
    )?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("Notchy")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "autostart" => {
                let launcher = app.autolaunch();
                let _ = if launcher.is_enabled().unwrap_or(false) {
                    launcher.disable()
                } else {
                    launcher.enable()
                };
                let _ = autostart.set_checked(launcher.is_enabled().unwrap_or(false));
            }
            "settings" => settings::open_window(app),
            "reload" => health::reload(app, "from the tray"),
            "quit" => {
                // Tells the watchdog task this was on purpose, so it doesn't bring Notchy back.
                let _ = std::fs::write(exited_flag(), "");
                app.exit(0)
            }
            _ => {}
        })
        .build(app)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // A black box for "Notchy just disappeared": start, clean exit and any panic go to
    // %APPDATA%\Notchy\notchy.log. No log line at all means something outside killed it.
    std::panic::set_hook(Box::new(|info| {
        let thread = std::thread::current().name().unwrap_or("?").to_string();
        log_line(&format!("PANIC in thread '{thread}': {info}"));
    }));

    // Watchdog: a scheduled task runs `notchy.exe --watchdog` every few minutes. If Notchy
    // is alive, the single-instance plugin ends the new copy at once; if it died, this copy
    // takes over. After Exit from the tray it stays off until Notchy is opened by hand.
    let watchdog = std::env::args().any(|a| a == "--watchdog");
    if watchdog && exited_flag().exists() {
        return;
    }
    if !watchdog {
        let _ = std::fs::remove_file(exited_flag());
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|_, _, _| {}))
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
        .plugin(tauri_plugin_drag::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            set_notch_size,
            is_tucked,
            media::media_now,
            media::media_toggle,
            media::media_next,
            media::media_prev,
            media::media_seek,
            audio::mic_state,
            audio::toggle_mic,
            audio::list_outputs,
            audio::next_output,
            system::power_state,
            system::in_use_state,
            system::open_screenshot,
            settings::get_settings,
            settings::save_settings,
            settings::locate,
            settings::fetch_calendar,
            begin_typing,
            end_typing,
            store::load_data,
            store::save_data,
            store::shelf_add,
            store::open_path,
            notifications::open_app,
            notifications::open_dnd_settings,
            update::install_update,
            health::heartbeat,
            health::log_js,
        ])
        .setup(move |app| {
            // Logged here, not earlier: a second copy (e.g. a watchdog run while Notchy is
            // alive) is ended by the single-instance plugin before setup and leaves no line.
            let how = if watchdog { "restarted by watchdog" } else { "normal" };
            log_line(&format!("start v{} ({how})", env!("CARGO_PKG_VERSION")));
            APP.set(app.handle().clone()).ok();
            let window = app.get_webview_window("main").unwrap();

            // Applied once so the window stays WS_EX_LAYERED; afterwards only
            // WS_EX_TRANSPARENT is toggled (see set_click_through).
            window.set_ignore_cursor_events(true)?;
            NOTCH_HWND.store(hwnd(&window).0 as isize, Ordering::Relaxed);
            place_window(&window);

            build_tray(app.handle())?;
            start_hover_emitter();
            start_eyes_emitter();
            start_mouse_hook();
            start_watcher(window);
            media::start(app.handle().clone());
            audio::start(app.handle().clone());
            system::start(app.handle().clone());
            notifications::start(app.handle().clone());
            update::start(app.handle().clone());
            health::start(app.handle().clone());
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_, event| {
            if let tauri::RunEvent::Exit = event {
                log_line("exit (normal)");
            }
        });
}

fn data_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var("APPDATA").unwrap_or_default()).join("Notchy")
}

/// Present while the user has quit Notchy from the tray; the watchdog leaves it off.
fn exited_flag() -> std::path::PathBuf {
    data_dir().join("exited-by-user")
}

/// Appends a timestamped line to %APPDATA%\Notchy\notchy.log, starting over past 512 KB.
fn log_line(msg: &str) {
    use std::io::Write;
    let dir = data_dir();
    let path = dir.join("notchy.log");
    let _ = std::fs::create_dir_all(&dir);
    if std::fs::metadata(&path).is_ok_and(|m| m.len() > 512 * 1024) {
        let _ = std::fs::remove_file(&path);
    }
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(f, "[{}] {msg}", unix_to_local(secs));
    }
}

/// "2026-10-19 16:25:02" in local time, without pulling in a date crate.
fn unix_to_local(secs: u64) -> String {
    use windows::Win32::Foundation::{FILETIME, SYSTEMTIME};
    use windows::Win32::System::Time::{FileTimeToSystemTime, SystemTimeToTzSpecificLocalTime};
    let ticks = (secs + 11_644_473_600) * 10_000_000;
    let ft = FILETIME { dwLowDateTime: ticks as u32, dwHighDateTime: (ticks >> 32) as u32 };
    let (mut utc, mut local) = (SYSTEMTIME::default(), SYSTEMTIME::default());
    unsafe {
        if FileTimeToSystemTime(&ft, &mut utc).is_err() || SystemTimeToTzSpecificLocalTime(None, &utc, &mut local).is_err() {
            return secs.to_string();
        }
    }
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", local.wYear, local.wMonth, local.wDay, local.wHour, local.wMinute, local.wSecond)
}
