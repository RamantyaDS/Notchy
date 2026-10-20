//! Small JSON files in %APPDATA%\Notchy for tasks, water log and shelf, plus
//! what the shelf needs: opening files and letting the notch preview them.

use std::path::PathBuf;

use tauri::{AppHandle, Manager};

fn file(name: &str) -> Result<PathBuf, String> {
    // Only plain names, so nothing outside the Notchy folder can be touched.
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err("invalid name".into());
    }
    Ok(PathBuf::from(std::env::var("APPDATA").unwrap_or_default()).join("Notchy").join(format!("{name}.json")))
}

#[tauri::command]
pub fn load_data(name: String) -> Result<serde_json::Value, String> {
    let path = file(&name)?;
    Ok(std::fs::read_to_string(path).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(serde_json::Value::Null))
}

#[tauri::command]
pub fn save_data(name: String, value: serde_json::Value) -> Result<(), String> {
    let path = file(&name)?;
    std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    std::fs::write(path, serde_json::to_string(&value).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

/// Lets the notch show previews of files on the shelf, and returns their sizes in bytes.
#[tauri::command]
pub fn shelf_add(app: AppHandle, paths: Vec<String>) -> Vec<Option<u64>> {
    paths
        .iter()
        .map(|p| {
            let meta = std::fs::metadata(p).ok()?;
            let _ = app.asset_protocol_scope().allow_file(p);
            Some(if meta.is_dir() { 0 } else { meta.len() })
        })
        .collect()
}

/// Opens a file or folder the user put on the shelf.
#[tauri::command]
pub fn open_path(path: String) {
    if std::path::Path::new(&path).exists() {
        let _ = std::process::Command::new("explorer").arg(&path).spawn();
    }
}
