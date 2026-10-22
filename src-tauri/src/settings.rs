//! Settings in %APPDATA%\Notchy\settings.json, edited in a small separate window
//! (the notch itself can't take keyboard focus).

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use windows::core::HSTRING;
use windows::Devices::Geolocation::Geolocator;
use windows::Foundation::{TimeSpan, Uri};
use windows::Web::Http::HttpClient;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct City {
    name: String,
    latitude: f64,
    longitude: f64,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// None: use the Windows location instead.
    weather_city: Option<City>,
    /// Pomodoro lengths, minutes.
    focus_minutes: u32,
    break_minutes: u32,
    /// Water: one "Minum" tap adds `glass_ml`; reminders between the hours
    /// `water_from`..`water_to` when nothing was logged for `remind_every` minutes.
    glass_ml: u32,
    goal_ml: u32,
    water_from: u32,
    water_to: u32,
    remind_every: u32,
    /// Show new Windows notifications in the notch.
    pub mirror_notifications: bool,
    /// Secret ICS address of a calendar (Google/Outlook), or empty.
    calendar_url: String,
    /// Show the line being sung in the small notch. Looks up lyrics for every
    /// song played (sends title and artist to LRCLIB).
    small_lyrics: bool,
    /// Buttons on the shortcuts page.
    pub shortcuts: Vec<Shortcut>,
}

/// One button on the shortcuts page. `action` is "lock", "sleep", "screenshot",
/// "downloads", "camera" (the notch's camera check, handled by the page), or
/// "open" with `target` (an app, file, folder or link the user chose in
/// Settings; scripts run through their usual handler).
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct Shortcut {
    pub name: String,
    pub action: String,
    pub target: String,
}

impl Default for Shortcut {
    fn default() -> Self {
        Self { name: String::new(), action: "open".into(), target: String::new() }
    }
}

pub fn default_shortcuts() -> Vec<Shortcut> {
    [("Kunci", "lock"), ("Sleep", "sleep"), ("Screenshot", "screenshot"), ("Downloads", "downloads"), ("Kamera", "camera")]
        .map(|(name, action)| Shortcut { name: name.into(), action: action.into(), target: String::new() })
        .to_vec()
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            weather_city: None,
            focus_minutes: 25,
            break_minutes: 5,
            glass_ml: 250,
            goal_ml: 2000,
            water_from: 8,
            water_to: 21,
            remind_every: 60,
            mirror_notifications: true,
            calendar_url: String::new(),
            small_lyrics: true,
            shortcuts: default_shortcuts(),
        }
    }
}

fn path() -> PathBuf {
    PathBuf::from(std::env::var("APPDATA").unwrap_or_default()).join("Notchy").join("settings.json")
}

#[tauri::command]
pub fn get_settings() -> Settings {
    std::fs::read_to_string(path()).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    let path = path();
    std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    let _ = app.emit("settings", &settings);
    Ok(())
}

/// Latitude and longitude from Windows location, if the user allows desktop apps to use it.
#[tauri::command]
pub async fn locate() -> Option<(f64, f64)> {
    let second = 10_000_000;
    let position = Geolocator::new()
        .ok()?
        .GetGeopositionAsyncWithAgeAndTimeout(
            TimeSpan { Duration: 30 * 60 * second },
            TimeSpan { Duration: 10 * second },
        )
        .ok()?
        .get()
        .ok()?;
    let p = position.Coordinate().ok()?.Point().ok()?.Position().ok()?;
    Some((p.Latitude, p.Longitude))
}

/// Downloads the calendar set in Settings (ICS text). Done here rather than in
/// the page because calendar servers don't allow cross-origin requests.
#[tauri::command]
pub async fn fetch_calendar() -> Result<String, String> {
    let url = get_settings().calendar_url.trim().replacen("webcal://", "https://", 1);
    if url.is_empty() {
        return Ok(String::new());
    }
    if !url.starts_with("https://") {
        return Err("Link kalender harus https:// atau webcal://".into());
    }
    let uri = Uri::CreateUri(&HSTRING::from(url)).map_err(|e| e.to_string())?;
    let client = HttpClient::new().map_err(|e| e.to_string())?;
    client
        .GetStringAsync(&uri)
        .and_then(|op| op.get())
        .map(|s| s.to_string())
        .map_err(|e| e.message().to_string())
}

pub fn open_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.unminimize();
        let _ = window.set_focus();
        return;
    }
    let _ = WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("settings".into()))
        .title("Notchy Settings")
        .inner_size(460.0, 640.0)
        .resizable(false)
        .center()
        .build();
}
