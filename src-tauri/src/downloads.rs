//! Browser downloads in progress, from the Downloads folder: Chromium browsers
//! write "name.crdownload", Firefox "name.part", and rename to "name" when done.
//!
//! The folder watcher wakes a once-a-second pass, which runs only while such
//! files exist. Browsers don't say the final size, so there's no percentage:
//! just the size so far and the speed.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use serde::Serialize;
use serde_json::json;
use tauri::{AppHandle, Emitter};
use windows::core::GUID;
use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::UI::Shell::{FOLDERID_Downloads, SHGetKnownFolderPath, KF_FLAG_DEFAULT};

use crate::peek;

#[derive(Serialize, Clone, PartialEq, Debug)]
pub struct Download {
    /// File name as it will be, e.g. "Laporan.pdf".
    name: String,
    bytes: u64,
    /// Bytes per second over the last pass.
    speed: u64,
}

pub fn downloads_dir() -> Option<PathBuf> {
    unsafe {
        let p = SHGetKnownFolderPath(&FOLDERID_Downloads as *const GUID, KF_FLAG_DEFAULT, None).ok()?;
        let path = p.to_string().ok().map(PathBuf::from);
        CoTaskMemFree(Some(p.0 as _));
        path
    }
}

/// The finished file's name for a partial download, or None if it isn't one.
/// Chromium's first name, "Unconfirmed 123456.crdownload", has no real name yet.
pub fn final_name(partial: &str) -> Option<Option<String>> {
    let lower = partial.to_ascii_lowercase();
    let stem = [".crdownload", ".part", ".partial"]
        .iter()
        .find_map(|ext| lower.ends_with(ext).then(|| &partial[..partial.len() - ext.len()]))?;
    if stem.is_empty() {
        return None;
    }
    let unconfirmed = stem.starts_with("Unconfirmed ") && stem[12..].chars().all(|c| c.is_ascii_digit());
    Some((!unconfirmed).then(|| stem.to_string()))
}

fn partials(dir: &Path) -> HashMap<PathBuf, u64> {
    std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| final_name(&e.file_name().to_string_lossy()).is_some())
        // The folder listing keeps the size from when the file was opened; asking
        // the file itself gives what the browser has written so far.
        .filter_map(|e| Some((e.path(), std::fs::metadata(e.path()).ok()?.len())))
        .collect()
}

pub fn start(app: AppHandle) {
    use notify::{RecursiveMode, Watcher};
    let Some(dir) = downloads_dir() else { return };
    std::thread::spawn(move || {
        let (tx, rx) = mpsc::channel();
        let Ok(mut watcher) = notify::recommended_watcher(tx) else { return };
        if watcher.watch(&dir, RecursiveMode::NonRecursive).is_err() {
            return;
        }
        let mut last: HashMap<PathBuf, u64> = HashMap::new();
        let mut last_pass = Instant::now();
        let mut shown: Option<Download> = None;
        loop {
            // Sleep until something happens in the folder; while downloads run, look every second.
            let busy = !last.is_empty();
            let event = if busy { rx.recv_timeout(Duration::from_secs(1)).ok() } else { rx.recv().ok() };
            if !busy && event.is_none() {
                return;
            }
            // Writes fire events constantly; one pass a second is plenty and keeps the speed honest.
            if let Some(wait) = Duration::from_millis(950).checked_sub(last_pass.elapsed()) {
                std::thread::sleep(wait);
            }
            while rx.try_recv().is_ok() {}

            let now = partials(&dir);
            let secs = last_pass.elapsed().as_secs_f64().max(0.2);
            last_pass = Instant::now();

            // Partials that disappeared: done if the final file is there, else cancelled.
            for gone in last.keys().filter(|p| !now.contains_key(*p)) {
                let name = gone.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                if let Some(Some(final_name)) = final_name(&name) {
                    let path = dir.join(&final_name);
                    if path.is_file() {
                        peek(&app, "download-done", json!({ "name": final_name, "path": path }));
                    }
                }
            }

            // The biggest download in progress is the one shown.
            let current = now
                .iter()
                .max_by_key(|(_, size)| **size)
                .map(|(path, &bytes)| {
                    let file = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                    let name = final_name(&file).flatten().unwrap_or_else(|| "Mengunduh…".into());
                    let before = last.get(path).copied().unwrap_or(bytes);
                    Download { name, bytes, speed: (bytes.saturating_sub(before) as f64 / secs) as u64 }
                });
            if current != shown {
                let _ = app.emit("download", &current);
                shown = current;
            }
            last = now;
        }
    });
}

/// Opens a finished download. Only files in the Downloads folder.
#[tauri::command]
pub fn open_download(path: String) {
    let path = PathBuf::from(path);
    let Some(dir) = downloads_dir() else { return };
    if path.parent() == Some(dir.as_path()) && path.is_file() {
        let _ = std::process::Command::new("explorer").arg(&path).spawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_partial_downloads() {
        assert_eq!(final_name("Laporan Q3.pdf.crdownload"), Some(Some("Laporan Q3.pdf".into())));
        assert_eq!(final_name("setup.exe.part"), Some(Some("setup.exe".into())));
        assert_eq!(final_name("video.MP4.CRDOWNLOAD"), Some(Some("video.MP4".into())));
        assert_eq!(final_name("Unconfirmed 482913.crdownload"), Some(None));
        assert_eq!(final_name("Unconfirmed plan.pdf.crdownload"), Some(Some("Unconfirmed plan.pdf".into())));
        assert_eq!(final_name("report.pdf"), None);
        assert_eq!(final_name(".crdownload"), None);
    }
}
