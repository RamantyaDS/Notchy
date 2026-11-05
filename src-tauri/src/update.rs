//! Updates itself from the GitHub releases (latest.json + a signed installer).
//!
//! Checked at start, every hour, and shortly after the laptop wakes from sleep.
//! Right after start nobody is using the notch yet, so a new version is installed
//! at once; later on it is only downloaded and offered as a peek ("click to
//! install", once per version), so Notchy never restarts mid-use.
//! The tray's "Cek update" checks right away and installs whatever it finds.
//! The copy run from the build folder (the developer's own) never updates itself.
//! The portable copy (zip, no installer) can't install one either: it only says
//! a new version is out, and clicking the peek opens the releases page.

use std::sync::{mpsc, Mutex, OnceLock};
use std::time::Duration;

use serde_json::json;
use tauri::AppHandle;
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::{log_line, peek};

/// A downloaded update waiting for its peek to be clicked.
static READY: Mutex<Option<(Update, Vec<u8>)>> = Mutex::new(None);

fn is_dev_build() -> bool {
    let exe = std::env::current_exe().unwrap_or_default();
    exe.to_string_lossy().to_lowercase().contains(r"\target\")
}

/// Only the copy inside the installer is marked as installed (see notchy.md §9).
fn is_portable() -> bool {
    tauri::utils::platform::bundle_type().is_none()
}

const RELEASES: &str = "https://github.com/RamantyaDS/Notchy/releases/latest";

static WAKE: OnceLock<mpsc::Sender<()>> = OnceLock::new();

/// The laptop woke from sleep: look for an update once the network is back.
pub fn woke() {
    if let Some(tx) = WAKE.get() {
        let _ = tx.send(());
    }
}

pub fn start(app: AppHandle) {
    if is_dev_build() {
        return;
    }
    let (tx, rx) = mpsc::channel::<()>();
    WAKE.set(tx).ok();
    std::thread::spawn(move || {
        let mut just_started = true;
        let mut offered: Option<String> = None;
        loop {
            if is_portable() {
                if let Ok(Some(version)) = tauri::async_runtime::block_on(newer(&app)) {
                    if offered.as_deref() != Some(version.as_str()) {
                        peek(&app, "update", json!({ "version": version, "portable": true }));
                        offered = Some(version);
                    }
                }
            } else if let Ok(Some((update, bytes))) = tauri::async_runtime::block_on(fetch(&app)) {
                if just_started {
                    install(update, bytes);
                } else {
                    if offered.as_deref() != Some(update.version.as_str()) {
                        peek(&app, "update", json!({ "version": update.version }));
                        offered = Some(update.version.clone());
                    }
                    *READY.lock().unwrap() = Some((update, bytes));
                }
            }
            just_started = false;
            // Hourly, or soon after a wake-up (Wi-Fi needs a moment to reconnect).
            if rx.recv_timeout(Duration::from_secs(60 * 60)).is_ok() {
                std::thread::sleep(Duration::from_secs(30));
                while rx.try_recv().is_ok() {}
            }
        }
    });
}

/// The update peek was clicked.
#[tauri::command]
pub fn install_update() {
    if is_portable() {
        let _ = std::process::Command::new("explorer").arg(RELEASES).spawn();
        return;
    }
    if let Some((update, bytes)) = READY.lock().unwrap().take() {
        install(update, bytes);
    }
}

/// Tray "Cek update": installs a newer version now, or says why not.
pub fn check_now(app: AppHandle) {
    let version = env!("CARGO_PKG_VERSION");
    if is_dev_build() {
        peek(&app, "update-info", json!({ "state": "dev", "version": version }));
        return;
    }
    std::thread::spawn(move || {
        if let Some((update, bytes)) = READY.lock().unwrap().take() {
            install(update, bytes);
            return;
        }
        peek(&app, "update-info", json!({ "state": "checking", "version": version }));
        if is_portable() {
            match tauri::async_runtime::block_on(newer(&app)) {
                Ok(Some(new)) => peek(&app, "update", json!({ "version": new, "portable": true })),
                Ok(None) => peek(&app, "update-info", json!({ "state": "latest", "version": version })),
                Err(()) => peek(&app, "update-info", json!({ "state": "failed", "version": version })),
            }
            return;
        }
        match tauri::async_runtime::block_on(fetch(&app)) {
            Ok(Some((update, bytes))) => install(update, bytes),
            Ok(None) => peek(&app, "update-info", json!({ "state": "latest", "version": version })),
            Err(()) => peek(&app, "update-info", json!({ "state": "failed", "version": version })),
        }
    });
}

/// The newer version's number, without downloading it.
async fn newer(app: &AppHandle) -> Result<Option<String>, ()> {
    match app.updater().map_err(|_| ())?.check().await {
        Ok(update) => Ok(update.map(|u| u.version)),
        Err(e) => {
            log_line(&format!("update check failed: {e}"));
            Err(())
        }
    }
}

/// Ok(None) when already up to date; Err when the check or download failed.
async fn fetch(app: &AppHandle) -> Result<Option<(Update, Vec<u8>)>, ()> {
    let update = match app.updater().map_err(|_| ())?.check().await {
        Ok(Some(update)) => update,
        Ok(None) => return Ok(None),
        Err(e) => {
            log_line(&format!("update check failed: {e}"));
            return Err(());
        }
    };
    match update.download(|_, _| {}, || {}).await {
        Ok(bytes) => Ok(Some((update, bytes))),
        Err(e) => {
            log_line(&format!("update download failed: {e}"));
            Err(())
        }
    }
}

/// Runs the installer; it closes Notchy and starts the new version.
fn install(update: Update, bytes: Vec<u8>) {
    log_line(&format!("updating to v{}", update.version));
    if let Err(e) = update.install(bytes) {
        log_line(&format!("update install failed: {e}"));
    }
}
