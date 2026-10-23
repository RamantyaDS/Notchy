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
        let mut seen: HashMap<PathBuf, Seen> = HashMap::new();
        let mut last_pass = Instant::now();
        let mut shown: Option<Download> = None;
        loop {
            // Sleep until something happens in the folder; while a download grows, look every second.
            let busy = seen.values().any(|s| s.changed.elapsed() < KEEP_LOOKING);
            let event = if busy { rx.recv_timeout(Duration::from_secs(1)).ok() } else { rx.recv().ok() };
            if !busy && event.is_none() {
                return;
            }
            // Writes fire events constantly; one pass a second is plenty and keeps the speed honest.
            if let Some(wait) = Duration::from_millis(950).checked_sub(last_pass.elapsed()) {
                std::thread::sleep(wait);
            }
            while rx.try_recv().is_ok() {}

            let secs = last_pass.elapsed().as_secs_f64().max(0.2);
            last_pass = Instant::now();
            let step = step(&mut seen, partials(&dir), Instant::now());

            // Partials that disappeared while in use: done if the final file is there, else cancelled.
            for gone in step.finished {
                let name = gone.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                if let Some(Some(final_name)) = final_name(&name) {
                    let path = dir.join(&final_name);
                    if path.is_file() {
                        peek(&app, "download-done", json!({ "name": final_name, "path": path }));
                    }
                }
            }

            // The biggest download in progress is the one shown.
            let current = step.growing.iter().max_by_key(|(_, bytes, _)| *bytes).map(|(path, bytes, grew)| {
                let file = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                let name = final_name(&file).flatten().unwrap_or_else(|| "Mengunduh…".into());
                Download { name, bytes: *bytes, speed: (*grew as f64 / secs) as u64 }
            });
            if current != shown {
                let _ = app.emit("download", &current);
                shown = current;
            }
        }
    });
}

/// A partial download that hasn't grown for this long is a leftover (failed or
/// cancelled earlier), not a download in progress.
const STALE: Duration = Duration::from_secs(15);
/// How long after the last growth the folder is still looked at every second.
const KEEP_LOOKING: Duration = Duration::from_secs(60);

pub struct Seen {
    bytes: u64,
    changed: Instant,
}

pub struct Step {
    /// In progress: path, size, bytes added since the last pass.
    pub growing: Vec<(PathBuf, u64, u64)>,
    /// Gone since the last pass after being written to recently.
    pub finished: Vec<PathBuf>,
}

/// Folds one look at the folder into what we know of each partial file. A file
/// first seen counts as old until it grows, so leftovers never show.
pub fn step(seen: &mut HashMap<PathBuf, Seen>, now: HashMap<PathBuf, u64>, at: Instant) -> Step {
    let finished = seen
        .iter()
        .filter(|(p, s)| !now.contains_key(*p) && at.duration_since(s.changed) < KEEP_LOOKING)
        .map(|(p, _)| p.clone())
        .collect();
    seen.retain(|p, _| now.contains_key(p));
    let mut growing = vec![];
    for (path, bytes) in now {
        // Back-dated so a leftover is neither shown nor reported finished when it goes.
        let entry = seen.entry(path.clone()).or_insert(Seen { bytes, changed: at.checked_sub(KEEP_LOOKING).unwrap_or(at) });
        let grew = bytes.saturating_sub(entry.bytes);
        if bytes != entry.bytes {
            entry.bytes = bytes;
            entry.changed = at;
        }
        if at.duration_since(entry.changed) < STALE {
            growing.push((path, bytes, grew));
        }
    }
    Step { growing, finished }
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

    fn sizes(list: &[(&str, u64)]) -> HashMap<PathBuf, u64> {
        list.iter().map(|(p, b)| (PathBuf::from(p), *b)).collect()
    }

    #[test]
    fn leftover_partials_never_show() {
        let mut seen = HashMap::new();
        let t0 = Instant::now() + Duration::from_secs(100);
        let old = sizes(&[("big.zip.crdownload", 223_000_000)]);
        assert!(step(&mut seen, old.clone(), t0).growing.is_empty());
        assert!(step(&mut seen, old, t0 + Duration::from_secs(1)).growing.is_empty());
    }

    #[test]
    fn growing_downloads_show_and_fade_when_they_stop() {
        let mut seen = HashMap::new();
        let t0 = Instant::now() + Duration::from_secs(100);
        step(&mut seen, sizes(&[("a.pdf.crdownload", 0)]), t0);
        let s = step(&mut seen, sizes(&[("a.pdf.crdownload", 3_000_000)]), t0 + Duration::from_secs(1));
        assert_eq!(s.growing, vec![(PathBuf::from("a.pdf.crdownload"), 3_000_000, 3_000_000)]);
        // Stuck for longer than STALE: no longer shown.
        let s = step(&mut seen, sizes(&[("a.pdf.crdownload", 3_000_000)]), t0 + Duration::from_secs(20));
        assert!(s.growing.is_empty());
    }

    #[test]
    fn finished_only_for_recent_downloads() {
        let mut seen = HashMap::new();
        let t0 = Instant::now() + Duration::from_secs(100);
        step(&mut seen, sizes(&[("a.pdf.crdownload", 0), ("old.zip.part", 5)]), t0);
        step(&mut seen, sizes(&[("a.pdf.crdownload", 9), ("old.zip.part", 5)]), t0 + Duration::from_secs(1));
        let s = step(&mut seen, sizes(&[]), t0 + Duration::from_secs(2));
        assert_eq!(s.finished, vec![PathBuf::from("a.pdf.crdownload")]);
        assert!(seen.is_empty());
    }

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
