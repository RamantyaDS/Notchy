//! Updates itself from the GitHub releases (latest.json + a signed installer).
//!
//! Checked at start and every 6 hours. Right after start nobody is using the notch
//! yet, so a new version is installed at once; later on it is only downloaded and
//! offered as a peek ("click to install"), so Notchy never restarts mid-use.
//! The tray's "Cek update" checks right away and installs whatever it finds.
//! The copy run from the build folder (the developer's own) never updates itself.

use std::sync::Mutex;
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

pub fn start(app: AppHandle) {
    if is_dev_build() {
        return;
    }
    std::thread::spawn(move || {
        let mut just_started = true;
        loop {
            if let Ok(Some((update, bytes))) = tauri::async_runtime::block_on(fetch(&app)) {
                if just_started {
                    install(update, bytes);
                } else {
                    peek(&app, "update", json!({ "version": update.version }));
                    *READY.lock().unwrap() = Some((update, bytes));
                }
            }
            just_started = false;
            std::thread::sleep(Duration::from_secs(6 * 60 * 60));
        }
    });
}

/// The update peek was clicked.
#[tauri::command]
pub fn install_update() {
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
        match tauri::async_runtime::block_on(fetch(&app)) {
            Ok(Some((update, bytes))) => install(update, bytes),
            Ok(None) => peek(&app, "update-info", json!({ "state": "latest", "version": version })),
            Err(()) => peek(&app, "update-info", json!({ "state": "failed", "version": version })),
        }
    });
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
