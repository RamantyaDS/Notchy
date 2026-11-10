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
    /// Off: no water reminders and no water page in the notch.
    water_enabled: bool,
    /// Water reminders on/off (the switch on the water page).
    water_reminders: bool,
    /// The clock gives way to the eyes after a quiet while.
    pub mascot: bool,
    /// Mascot tracking mode: "idle" (wait 120s idle) or "always" (continuous follow)
    pub mascot_mode: String,
    /// Mascot eye style theme: "classic", "anime", "pixel", "cat", "cartoon"
    pub mascot_theme: String,
    /// Show mascot in music island when lyrics are empty (replaces clock, default false)
    pub music_mascot: bool,
    /// A closed, idle notch is just a thin black line (for thick top bezels).
    pub minimal_notch: bool,
    /// The notch floats a little below the top edge as a rounded pill.
    pub floating_notch: bool,
    /// Thin ring in the closed notch: "off", "battery" or "water".
    pub status_ring: String,
    /// Show new Windows notifications in the notch.
    pub mirror_notifications: bool,
    /// Secret ICS address of a calendar (Google/Outlook), or empty.
    calendar_url: String,
    /// Show the line being sung in the small notch. Looks up lyrics for every
    /// song played (sends title and artist to LRCLIB).
    small_lyrics: bool,
    /// Buttons on the shortcuts page.
    pub shortcuts: Vec<Shortcut>,
    /// Caps Lock and keyboard-language peeks (one keyboard hook, read at start).
    pub caps_lock_peek: bool,
    /// Notchy handles the volume keys so only its own volume peek shows (read at start).
    pub hide_volume_osd: bool,
    /// Page toggles for the open notch.
    pub music_enabled: bool,
    pub pomodoro_enabled: bool,
    pub todo_enabled: bool,
    pub shelf_enabled: bool,
    pub clipboard_enabled: bool,
    pub shortcuts_enabled: bool,
    pub today_enabled: bool,
    /// Network speed indicators on/off and thresholds (in KB/s)
    pub net_speed_island: bool,
    pub net_speed_status: bool,
    pub net_speed_threshold_down_kb: u32,
    pub net_speed_threshold_up_kb: u32,
    /// Bluetooth battery monitor toggle & threshold
    pub bluetooth_peek: bool,
    pub bluetooth_low_threshold: u32,
    /// Monitor placement: "primary", "secondary", "follow-cursor"
    pub monitor_placement: String,
    /// Spring animation preset: "spring", "snappy", "smooth", "off"
    pub spring_physics: String,
    /// false: Default Notchy neutral ambient glow; true: Windows DWM accent sync
    pub windows_accent_glow: bool,
    /// Prayer Times & Adhan Reminder
    pub prayer_enabled: bool,
    pub prayer_pre_alert: bool,
    pub prayer_method: String,
    pub prayer_fajr: bool,
    pub prayer_dhuhr: bool,
    pub prayer_asr: bool,
    pub prayer_maghrib: bool,
    pub prayer_isha: bool,
    pub prayer_sound: bool,
    /// Language preference: "en" or "id"
    #[serde(default = "legacy_language")]
    pub language: String,
    /// Newer pages: per-app volume, saved text snippets, teleprompter.
    pub sound_enabled: bool,
    pub snippets_enabled: bool,
    pub prompter_enabled: bool,
    /// `notchy --peek`, `notchy://` links (registered for this user while on).
    pub external_peeks: bool,
    /// Claude Code hooks (`notchy --agent ...`) show the agent's state.
    pub agent_status: bool,
    /// Global hotkeys, e.g. "Win+Alt+N"; empty for none.
    pub open_hotkey: String,
    pub ocr_hotkey: String,
    /// Horizontal swipe on the music page skips tracks.
    pub swipe_tracks: bool,
    /// Warn ~30 minutes before rain (Open-Meteo, the weather city).
    pub rain_alert: bool,
    /// Remind to unplug at this battery percent (and to plug in at 20%); 0 = off.
    pub charge_reminder: u32,
    /// After a presentation or full-screen app, sum up notifications held back.
    pub presentation_summary: bool,
    /// 20-20-20: look away for 20 s after 20 minutes at the screen.
    pub eye_break: bool,
    /// Peek when the internet connection drops or comes back.
    pub connection_peek: bool,
    /// Camera and microphone dots and peeks.
    pub privacy_indicators: bool,
    /// Dates counted down on the Today page.
    pub countdowns: Vec<Countdown>,
}

/// A date counted down on the Today page; `repeat` is "none", "monthly" or "yearly".
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Countdown {
    pub name: String,
    /// "YYYY-MM-DD"
    pub date: String,
    pub repeat: String,
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

fn legacy_language() -> String {
    "id".into()
}

fn default_language() -> String {
    unsafe {
        let langid = windows::Win32::Globalization::GetUserDefaultUILanguage();
        // Primary language ID in low 10 bits: LANG_ENGLISH = 0x09
        if (langid & 0x03FF) == 0x09 {
            "en".into()
        } else {
            "id".into() // Default fallback to Indonesian as requested by Ramantya
        }
    }
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
            water_enabled: true,
            water_reminders: true,
            mascot: true,
            mascot_mode: "idle".into(),
            mascot_theme: "classic".into(),
            music_mascot: false,
            minimal_notch: false,
            floating_notch: false,
            status_ring: "off".into(),
            mirror_notifications: true,
            calendar_url: String::new(),
            small_lyrics: true,
            shortcuts: default_shortcuts(),
            caps_lock_peek: true,
            hide_volume_osd: false,
            music_enabled: true,
            pomodoro_enabled: true,
            todo_enabled: true,
            shelf_enabled: true,
            clipboard_enabled: true,
            shortcuts_enabled: true,
            today_enabled: true,
            net_speed_island: false,
            net_speed_status: true,
            net_speed_threshold_down_kb: 500,
            net_speed_threshold_up_kb: 250,
            bluetooth_peek: false,
            bluetooth_low_threshold: 20,
            monitor_placement: "primary".into(),
            spring_physics: "spring".into(),
            windows_accent_glow: false,
            prayer_enabled: false,
            prayer_pre_alert: true,
            prayer_method: "kemenag".into(),
            prayer_fajr: true,
            prayer_dhuhr: true,
            prayer_asr: true,
            prayer_maghrib: true,
            prayer_isha: true,
            prayer_sound: true,
            language: default_language(),
            sound_enabled: true,
            snippets_enabled: false,
            prompter_enabled: false,
            external_peeks: true,
            agent_status: true,
            open_hotkey: "Ctrl+Alt+N".into(),
            ocr_hotkey: String::new(),
            swipe_tracks: true,
            rain_alert: true,
            charge_reminder: 0,
            presentation_summary: true,
            eye_break: false,
            connection_peek: true,
            privacy_indicators: true,
            countdowns: vec![],
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
    let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    crate::store::write_safely(&path(), &json)?;
    crate::network::update_settings(&settings);
    crate::extras::hotkeys_changed();
    crate::extras::register_link(settings.external_peeks);
    crate::place_window_for_settings(&app);
    let _ = app.emit("settings", &settings);
    Ok(())
}

/// Changes a few settings by name (camelCase, as the page and Settings window use
/// them), e.g. `{"waterReminders": false}` from the water page's switch.
#[tauri::command]
pub fn patch_settings(app: AppHandle, patch: serde_json::Value) -> Result<Settings, String> {
    let mut all = serde_json::to_value(get_settings()).map_err(|e| e.to_string())?;
    if let (Some(all), Some(patch)) = (all.as_object_mut(), patch.as_object()) {
        for (k, v) in patch {
            all.insert(k.clone(), v.clone());
        }
    }
    let settings: Settings = serde_json::from_value(all).map_err(|e| e.to_string())?;
    save_settings(app, settings.clone())?;
    Ok(settings)
}

/// The water page's "Atur" button.
#[tauri::command]
pub fn open_settings(app: AppHandle) {
    open_window(&app);
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
        .inner_size(780.0, 560.0)
        .theme(Some(tauri::Theme::Dark))
        .resizable(false)
        .center()
        .build();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_default_thresholds() {
        let s = Settings::default();
        assert_eq!(s.net_speed_threshold_down_kb, 500);
        assert_eq!(s.net_speed_threshold_up_kb, 250);
        assert!(!s.hide_volume_osd);
        assert!(!s.bluetooth_peek);
        assert_eq!(s.bluetooth_low_threshold, 20);
        assert_eq!(s.monitor_placement, "primary");
        assert_eq!(s.spring_physics, "spring");
        assert!(!s.windows_accent_glow);
        assert!(!s.floating_notch);
        assert_eq!(s.status_ring, "off");
        assert!(s.today_enabled);
        assert_eq!(s.net_speed_island, false);
        assert!(s.net_speed_status);
        assert!(s.language == "en" || s.language == "id");
        assert!(s.sound_enabled && !s.snippets_enabled && !s.prompter_enabled);
        assert_eq!(s.open_hotkey, "Ctrl+Alt+N");
        assert_eq!(s.charge_reminder, 0);
        assert!(!s.eye_break && s.rain_alert && s.privacy_indicators);
    }

    #[test]
    fn test_settings_deserialize_backward_compatibility() {
        // Deserializing older settings JSON without threshold fields uses defaults
        let old_json = r#"{
            "focusMinutes": 30,
            "netSpeedIsland": true,
            "netSpeedStatus": true
        }"#;
        let s: Settings = serde_json::from_str(old_json).expect("deserialize old json");
        assert_eq!(s.focus_minutes, 30);
        assert!(!s.hide_volume_osd);
        assert!(s.net_speed_island);
        assert!(s.net_speed_status);
        assert_eq!(s.net_speed_threshold_down_kb, 500);
        assert_eq!(s.net_speed_threshold_up_kb, 250);
        assert!(!s.bluetooth_peek);
        assert_eq!(s.bluetooth_low_threshold, 20);
        assert_eq!(s.monitor_placement, "primary");
        assert_eq!(s.spring_physics, "spring");
        assert!(!s.windows_accent_glow);
        assert_eq!(s.language, "id");
    }

    #[test]
    fn test_settings_deserialize_custom_thresholds() {
        let json = r#"{
            "netSpeedThresholdDownKb": 1200,
            "netSpeedThresholdUpKb": 600
        }"#;
        let s: Settings = serde_json::from_str(json).expect("deserialize custom thresholds");
        assert_eq!(s.net_speed_threshold_down_kb, 1200);
        assert_eq!(s.net_speed_threshold_up_kb, 600);
    }
}
