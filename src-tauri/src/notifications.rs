//! Mirrors new Windows notifications (WhatsApp, Teams, mail...) into the notch.
//!
//! The official listener lets unpackaged apps read notifications, but its
//! change event needs an MSIX package (subscribing fails without one).
//! Instead we watch Windows' per-app notification settings in the registry:
//! Windows bumps `LastNotificationAddedTime` there ~75 ms after a notification
//! arrives. That key sends no change notifications (RegNotifyChangeKeyValue
//! never fires for it), so we read those values every 200 ms, which is cheap,
//! and fetch the notification list only when one moved. A full look every 30 s
//! covers anything that slips through. (The notification database file is
//! written lazily, so it's no use as a trigger either.)

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use serde_json::json;
use tauri::AppHandle;
use winreg::enums::HKEY_CURRENT_USER;
use winreg::RegKey;
use windows::core::Interface;
use windows::Foundation::Size;
use windows::Storage::Streams::IRandomAccessStreamReference;
use windows::UI::Notifications::Management::{UserNotificationListener, UserNotificationListenerAccessStatus};
use windows::UI::Notifications::{KnownNotificationBindings, NotificationKinds, UserNotification};
use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};

use crate::{media, peek, settings};

const SETTINGS_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Notifications\Settings";

pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        }
        let Ok(listener) = UserNotificationListener::Current() else { return };
        if listener.GetAccessStatus().ok() != Some(UserNotificationListenerAccessStatus::Allowed) {
            let _ = listener.RequestAccessAsync().and_then(|op| op.get());
        }

        // Whatever is already in the notification center is old news.
        let mut seen = ids(&list(&listener));
        let mut logos = HashMap::new();

        let mut stamps = Stamps::new();
        let mut stamp = stamps.latest();
        let mut last_full = Instant::now();

        loop {
            std::thread::sleep(Duration::from_millis(200));
            let now = stamps.latest();
            let woken = now != stamp;
            // A full look only when Windows logged a new notification (or every 30 s, just in case).
            if !woken && last_full.elapsed() < Duration::from_secs(30) {
                continue;
            }
            stamp = now;
            last_full = Instant::now();
            let mut fresh = new_ones(&listener, &mut seen);
            if woken && fresh.is_empty() {
                // The registry can be a hair ahead of the list; look once more.
                std::thread::sleep(Duration::from_millis(120));
                fresh = new_ones(&listener, &mut seen);
            }
            if fresh.is_empty() || !settings::get_settings().mirror_notifications {
                continue;
            }
            // Oldest first; a burst (e.g. after waking up) is capped so it doesn't flood.
            for n in fresh.iter().rev().take(4).rev() {
                if let Some(data) = describe(n, &mut logos) {
                    peek(&app, "notification", data);
                }
            }
        }
    });
}

/// The per-app keys that carry `LastNotificationAddedTime`, kept open so each
/// 200 ms check is just ~40 value reads. Desktop apps' ids contain backslashes
/// ("{guid}\path\app.exe"), which Windows stores as nested keys, so finding
/// them walks a few levels deep; that walk is redone every 10 s for new apps.
struct Stamps {
    keys: Vec<RegKey>,
    scanned: Instant,
}

impl Stamps {
    fn new() -> Self {
        Self { keys: Self::scan(), scanned: Instant::now() }
    }

    fn scan() -> Vec<RegKey> {
        fn walk(key: RegKey, depth: u8, out: &mut Vec<RegKey>) {
            if depth > 0 {
                for name in key.enum_keys().flatten() {
                    if let Ok(sub) = key.open_subkey(&name) {
                        walk(sub, depth - 1, out);
                    }
                }
            }
            if key.get_value::<u64, _>("LastNotificationAddedTime").is_ok() {
                out.push(key);
            }
        }
        let mut out = vec![];
        if let Ok(root) = RegKey::predef(HKEY_CURRENT_USER).open_subkey(SETTINGS_KEY) {
            walk(root, 6, &mut out);
        }
        out
    }

    /// Newest notification time over all apps.
    fn latest(&mut self) -> u64 {
        if self.scanned.elapsed() > Duration::from_secs(10) {
            self.keys = Self::scan();
            self.scanned = Instant::now();
        }
        self.keys.iter().filter_map(|k| k.get_value::<u64, _>("LastNotificationAddedTime").ok()).max().unwrap_or(0)
    }
}

/// Notifications not seen before, oldest first. Forgets dismissed ones so the set stays small.
fn new_ones(listener: &UserNotificationListener, seen: &mut HashSet<u32>) -> Vec<UserNotification> {
    let current = list(listener);
    let fresh = current.iter().filter(|n| n.Id().is_ok_and(|id| !seen.contains(&id))).cloned().collect();
    *seen = ids(&current);
    fresh
}

/// Opens Windows' notification settings, where Do not disturb lives. With it on,
/// banners stop but notifications still reach the notification center, so the
/// notch becomes the only place they pop up.
#[tauri::command]
pub fn open_dnd_settings() {
    let _ = std::process::Command::new("explorer").arg("ms-settings:notifications").spawn();
}

/// Clicking a mirrored notification does what clicking the real one would:
/// opens the WhatsApp chat, the email, the Teams message. Falls back to just
/// opening the app the notification came from (its Start menu entry).
#[tauri::command]
pub fn open_app(aumid: String, id: Option<u32>) {
    // App ids are plain identifiers and paths; nothing that could smuggle in arguments.
    if aumid.is_empty() || aumid.contains('"') || aumid.starts_with('-') {
        return;
    }
    std::thread::spawn(move || {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        }
        match id.ok_or("no id".to_string()).and_then(|id| activate(&aumid, id)) {
            Ok(()) => {
                // Windows also clears a toast from the notification center once it's clicked.
                if let (Some(id), Ok(listener)) = (id, UserNotificationListener::Current()) {
                    let _ = listener.RemoveNotification(id);
                }
            }
            Err(why) => {
                crate::log_line(&format!("notification click fell back to opening the app: {why}"));
                let _ = std::process::Command::new("explorer").arg(format!("shell:AppsFolder\\{aumid}")).spawn();
            }
        }
    });
}

/// Replays the click on the original toast. The listener API doesn't expose a
/// toast's launch arguments, but Windows keeps the full toast XML in its
/// notification database; with those arguments we call the app's toast
/// activator exactly as the notification center does.
fn activate(aumid: &str, id: u32) -> Result<(), String> {
    let db = std::path::PathBuf::from(std::env::var("LOCALAPPDATA").unwrap_or_default())
        .join(r"Microsoft\Windows\Notifications\wpndatabase.db");
    let conn = rusqlite::Connection::open_with_flags(&db, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(|e| e.to_string())?;
    // Web pages often replace a toast with a fresh one (new id) moments later,
    // so if ours is gone, the app's newest toast stands in. The database also
    // knows the exact sender ("Edge!https://web.whatsapp.com/" where the
    // listener only says "Edge!App"), which is what the activator expects.
    let package = format!("{}%", aumid.split('!').next().unwrap_or(aumid));
    let (payload, aumid): (Vec<u8>, String) = conn
        .query_row(
            "select n.Payload, h.PrimaryId from Notification n join NotificationHandler h on h.RecordId = n.HandlerId
             where n.Type = 'toast' and (n.Id = ?1 or h.PrimaryId like ?2)
             order by n.Id = ?1 desc, n.ArrivalTime desc limit 1",
            rusqlite::params![id, package],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|e| format!("toast {id} not in database: {e}"))?;
    let aumid = aumid.as_str();
    let xml = String::from_utf8_lossy(&payload);
    let toast = xml.find("<toast").and_then(|at| xml[at..].find('>').map(|end| &xml[at..at + end])).ok_or("no <toast> tag")?;
    let launch = attr(toast, "launch").unwrap_or_default();

    if attr(toast, "activationType").as_deref() == Some("protocol") {
        // The toast is a link (e.g. "https://..." or "ms-teams:...").
        if !launch.contains(':') || launch.starts_with('-') {
            return Err(format!("odd protocol launch {launch:?}"));
        }
        std::process::Command::new("explorer").arg(&launch).spawn().map_err(|e| e.to_string())?;
        return Ok(());
    }

    let clsid = activator(aumid).ok_or("app has no toast activator")?;
    unsafe {
        use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_LOCAL_SERVER};
        use windows::Win32::UI::Notifications::INotificationActivationCallback;
        use windows::Win32::UI::WindowsAndMessaging::{AllowSetForegroundWindow, ASFW_ANY};
        // The app is the one being clicked, so it may bring its window to the front.
        let _ = AllowSetForegroundWindow(ASFW_ANY);
        let callback: INotificationActivationCallback = CoCreateInstance(&clsid, None, CLSCTX_LOCAL_SERVER).map_err(|e| e.to_string())?;
        callback
            .Activate(&windows::core::HSTRING::from(aumid), &windows::core::HSTRING::from(launch), &[])
            .map_err(|e| e.to_string())
    }
}

/// The COM class an app registered to receive toast clicks: in its package
/// manifest for Store/packaged apps (Edge, new Outlook, WhatsApp), in the
/// registry for plain desktop apps.
fn activator(aumid: &str) -> Option<windows::core::GUID> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let text = hkcu
        .open_subkey(format!(r"Software\Classes\AppUserModelId\{aumid}"))
        .and_then(|k| k.get_value::<String, _>("CustomActivator"))
        .ok()
        .or_else(|| {
            // "Name_publisher!App" -> installed package "Name_version_arch__publisher".
            let (name, publisher) = aumid.split('!').next()?.rsplit_once('_')?;
            let packages = hkcu
                .open_subkey(r"Software\Classes\Local Settings\Software\Microsoft\Windows\CurrentVersion\AppModel\Repository\Packages")
                .ok()?;
            packages
                .enum_keys()
                .flatten()
                .filter(|full| full.starts_with(&format!("{name}_")) && full.ends_with(&format!("_{publisher}")))
                .flat_map(|full| {
                    // Usually the manifest sits in the package folder; Edge keeps its
                    // files elsewhere and the manifest under WindowsApps.
                    let root = packages.open_subkey(&full).and_then(|k| k.get_value::<String, _>("PackageRootFolder")).unwrap_or_default();
                    let apps = std::path::PathBuf::from(std::env::var("ProgramW6432").unwrap_or_default()).join("WindowsApps").join(&full);
                    [std::path::PathBuf::from(root), apps]
                })
                .filter_map(|dir| std::fs::read_to_string(dir.join("AppxManifest.xml")).ok())
                .find_map(|manifest| attr(&manifest, "ToastActivatorCLSID"))
        })?;
    windows::core::GUID::try_from(text.trim_matches(|c| c == '{' || c == '}')).ok()
}

/// The value of `name="..."` in an XML snippet, with entities decoded.
fn attr(xml: &str, name: &str) -> Option<String> {
    let start = xml.find(&format!(" {name}=\""))? + name.len() + 3;
    let value = &xml[start..start + xml[start..].find('"')?];
    Some(
        value
            .replace("&quot;", "\"")
            .replace("&apos;", "'")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&"),
    )
}

fn list(listener: &UserNotificationListener) -> Vec<UserNotification> {
    listener
        .GetNotificationsAsync(NotificationKinds::Toast)
        .and_then(|op| op.get())
        .map(|v| v.into_iter().collect())
        .unwrap_or_default()
}

fn ids(list: &[UserNotification]) -> HashSet<u32> {
    list.iter().filter_map(|n| n.Id().ok()).collect()
}

fn describe(n: &UserNotification, logos: &mut HashMap<String, Option<String>>) -> Option<serde_json::Value> {
    let info = n.AppInfo().ok()?;
    let display = info.DisplayInfo().ok()?;
    let app = display.DisplayName().map(|s| s.to_string()).unwrap_or_default();
    let binding = n.Notification().ok()?.Visual().ok()?.GetBinding(&KnownNotificationBindings::ToastGeneric().ok()?).ok()?;
    let texts: Vec<String> = binding
        .GetTextElements()
        .ok()?
        .into_iter()
        .filter_map(|t| t.Text().ok().map(|s| s.to_string()))
        .filter(|s| !s.trim().is_empty())
        .collect();
    let title = texts.first().cloned().unwrap_or_else(|| app.clone());
    let body = texts.get(1..).map(|rest| rest.join(" · ")).unwrap_or_default();

    let key = info.AppUserModelId().map(|s| s.to_string()).unwrap_or_else(|_| app.clone());
    let logo = logos
        .entry(key)
        .or_insert_with(|| {
            let logo = display.GetLogo(Size { Width: 64.0, Height: 64.0 }).ok()?;
            media::read_art(logo.cast::<IRandomAccessStreamReference>().ok()?)
        })
        .clone();
    let aumid = info.AppUserModelId().map(|s| s.to_string()).unwrap_or_default();
    let id = n.Id().ok();
    Some(json!({ "app": app, "title": title, "body": body, "icon": logo, "aumid": aumid, "id": id }))
}
