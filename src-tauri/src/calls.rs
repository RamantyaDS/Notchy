//! A call in progress (Teams, WhatsApp, Zoom, or a meeting in the browser),
//! told apart from other mic use by which app holds the microphone.
//!
//! The mic watcher pokes us on every change. Browsers also use the mic for
//! voice typing, so a browser only counts once it has held the mic for 10 s.

use std::sync::{mpsc, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::system;

const BROWSER_GRACE_MS: u64 = 10_000;

#[derive(Serialize, Clone, PartialEq, Debug)]
pub struct Call {
    /// "Teams", "WhatsApp", "Zoom", or "Panggilan" for a meeting in the browser.
    app: String,
    /// When the app picked up the mic, ms since 1970.
    start: u64,
}

static POKE: OnceLock<mpsc::Sender<()>> = OnceLock::new();
static CURRENT: std::sync::Mutex<Option<Call>> = std::sync::Mutex::new(None);

/// Which call app a ConsentStore entry is, and whether it's a browser.
pub fn classify(id: &str) -> Option<(&'static str, bool)> {
    let lower = id.to_ascii_lowercase();
    let exe = lower.rsplit('#').next().unwrap_or(&lower);
    if lower.starts_with("msteams_") || matches!(exe, "teams.exe" | "ms-teams.exe" | "msteams.exe") {
        Some(("Teams", false))
    } else if lower.contains("whatsapp") {
        Some(("WhatsApp", false))
    } else if matches!(exe, "zoom.exe" | "cpthost.exe") || lower.starts_with("zoom") {
        Some(("Zoom", false))
    } else if matches!(
        exe,
        "msedge.exe" | "chrome.exe" | "firefox.exe" | "brave.exe" | "opera.exe" | "vivaldi.exe" | "arc.exe"
    ) {
        Some(("Panggilan", true))
    } else {
        None
    }
}

/// FILETIME (100 ns since 1601) to ms since 1970.
fn unix_ms(filetime: u64) -> u64 {
    (filetime / 10_000).saturating_sub(11_644_473_600_000)
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

/// The call to show, from the mic users right now; plus, if a browser is still
/// inside its grace period, how long until it would count.
pub fn pick(sessions: &[(String, u64)], now: u64) -> (Option<Call>, Option<u64>) {
    let mut best: Option<(Call, bool)> = None;
    let mut recheck = None;
    for (id, start) in sessions {
        let Some((app, browser)) = classify(id) else { continue };
        let start = unix_ms(*start);
        if browser && now.saturating_sub(start) < BROWSER_GRACE_MS {
            let wait = BROWSER_GRACE_MS - now.saturating_sub(start);
            recheck = Some(recheck.map_or(wait, |w: u64| w.min(wait)));
            continue;
        }
        let call = Call { app: app.to_string(), start };
        // A real call app beats a browser; otherwise the earliest started wins.
        let better = match &best {
            None => true,
            Some((b, b_browser)) => (*b_browser && !browser) || (*b_browser == browser && start < b.start),
        };
        if better {
            best = Some((call, browser));
        }
    }
    (best.map(|(c, _)| c), recheck)
}

pub fn poke() {
    if let Some(tx) = POKE.get() {
        let _ = tx.send(());
    }
}

#[tauri::command]
pub fn call_state() -> Option<Call> {
    CURRENT.lock().unwrap().clone()
}

pub fn start(app: AppHandle) {
    let (tx, rx) = mpsc::channel::<()>();
    POKE.set(tx).ok();
    std::thread::spawn(move || {
        let mut wait = None;
        loop {
            let (call, recheck) = pick(&system::sessions("microphone"), now_ms());
            let mut current = CURRENT.lock().unwrap();
            if *current != call {
                let _ = app.emit("call", &call);
                *current = call;
            }
            drop(current);
            if recheck.is_some() {
                wait = recheck.map(|ms| Duration::from_millis(ms + 200));
            }
            match wait.take() {
                Some(d) => {
                    let _ = rx.recv_timeout(d);
                }
                None => {
                    if rx.recv().is_err() {
                        return;
                    }
                }
            }
        }
    });
    poke();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ft(unix_ms: u64) -> u64 {
        (unix_ms + 11_644_473_600_000) * 10_000
    }

    #[test]
    fn classifies_call_apps() {
        assert_eq!(classify("MSTeams_8wekyb3d8bbwe"), Some(("Teams", false)));
        assert_eq!(classify("C:#Users#me#AppData#Local#Microsoft#Teams#current#Teams.exe"), Some(("Teams", false)));
        assert_eq!(classify("5319275A.WhatsAppDesktop_cv1g1gvanyjgm"), Some(("WhatsApp", false)));
        assert_eq!(classify("C:#Users#me#AppData#Roaming#Zoom#bin#Zoom.exe"), Some(("Zoom", false)));
        assert_eq!(classify("C:#Program Files (x86)#Microsoft#Edge#Application#msedge.exe"), Some(("Panggilan", true)));
        assert_eq!(classify("C:#Program Files#Google#Chrome#Application#chrome.exe"), Some(("Panggilan", true)));
        assert_eq!(classify("Microsoft.WindowsSoundRecorder_8wekyb3d8bbwe"), None);
        assert_eq!(classify("C:#Program Files#Audacity#Audacity.exe"), None);
    }

    #[test]
    fn browser_needs_ten_seconds() {
        let now = 1_000_000_000_000;
        let edge = "C:#x#msedge.exe".to_string();
        let (call, recheck) = pick(&[(edge.clone(), ft(now - 4_000))], now);
        assert_eq!(call, None);
        assert_eq!(recheck, Some(6_000));
        let (call, recheck) = pick(&[(edge, ft(now - 12_000))], now);
        assert_eq!(call.unwrap().app, "Panggilan");
        assert_eq!(recheck, None);
    }

    #[test]
    fn call_app_beats_browser_and_keeps_its_start() {
        let now = 1_000_000_000_000;
        let sessions = [
            ("C:#x#chrome.exe".to_string(), ft(now - 60_000)),
            ("MSTeams_8wekyb3d8bbwe".to_string(), ft(now - 30_000)),
        ];
        let (call, _) = pick(&sessions, now);
        assert_eq!(call, Some(Call { app: "Teams".into(), start: now - 30_000 }));
    }

    #[test]
    fn no_call_for_other_mic_users() {
        let (call, recheck) = pick(&[("Microsoft.WindowsSoundRecorder_8wekyb3d8bbwe".to_string(), ft(5))], 10_000);
        assert_eq!((call, recheck), (None, None));
    }
}
