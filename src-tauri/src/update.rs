//! Updates itself from the GitHub releases (latest.json + a signed installer).
//!
//! Checked at start and every 6 hours. Right after start nobody is using the notch
//! yet, so a new version is installed at once; later on it is only downloaded and
//! offered as a peek ("click to install"), so Notchy never restarts mid-use.
//! The copy run from the build folder (the developer's own) never updates itself.

use std::sync::Mutex;
use std::time::Duration;

use serde_json::json;
use tauri::AppHandle;
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::{log_line, peek};

/// A downloaded update waiting for its peek to be clicked.
static READY: Mutex<Option<(Update, Vec<u8>)>> = Mutex::new(None);

pub fn start(app: AppHandle) {
    let exe = std::env::current_exe().unwrap_or_default();
    if exe.to_string_lossy().to_lowercase().contains(r"\target\") {
        return;
    }
    std::thread::spawn(move || {
        let mut just_started = true;
        loop {
            if let Some((update, bytes)) = tauri::async_runtime::block_on(fetch(&app)) {
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

async fn fetch(app: &AppHandle) -> Option<(Update, Vec<u8>)> {
    let update = match app.updater().ok()?.check().await {
        Ok(found) => found?,
        Err(e) => {
            log_line(&format!("update check failed: {e}"));
            return None;
        }
    };
    match update.download(|_, _| {}, || {}).await {
        Ok(bytes) => Some((update, bytes)),
        Err(e) => {
            log_line(&format!("update download failed: {e}"));
            None
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

