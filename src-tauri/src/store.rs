//! Small JSON files in %APPDATA%\Notchy for tasks, water log and shelf, plus
//! what the shelf needs: opening files and letting the notch preview them.

use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};

fn file(name: &str) -> Result<PathBuf, String> {
    // Only plain names, so nothing outside the Notchy folder can be touched.
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err("invalid name".into());
    }
    Ok(PathBuf::from(std::env::var("APPDATA").unwrap_or_default()).join("Notchy").join(format!("{name}.json")))
}

/// Writes next to the target first and then swaps it in, so a crash or power
/// loss mid-write leaves the old file instead of a cut-off one.
pub fn write_safely(path: &Path, text: &str) -> Result<(), String> {
    let dir = path.parent().ok_or("no parent folder")?;
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    let tmp = dir.join(format!(".{name}.tmp"));
    std::fs::write(&tmp, text).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn load_data(name: String) -> Result<serde_json::Value, String> {
    let path = file(&name)?;
    Ok(std::fs::read_to_string(path).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(serde_json::Value::Null))
}

#[tauri::command]
pub fn save_data(name: String, value: serde_json::Value) -> Result<(), String> {
    let path = file(&name)?;
    write_safely(&path, &serde_json::to_string(&value).map_err(|e| e.to_string())?)
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

/// Opens a file or folder the user put on the shelf (programs are only shown in Explorer).
#[tauri::command]
pub fn open_path(path: String) {
    // A leading '-' or '/' would be read by explorer as a switch.
    if path.starts_with('-') || path.starts_with('/') {
        return;
    }
    let p = Path::new(&path);
    if p.exists() {
        crate::downloads::open_or_reveal(p);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_file_names() {
        assert!(file("todo").is_ok());
        assert!(file("water-log").is_ok());
        assert!(file("../evil").is_err());
        assert!(file("test/sub").is_err());
        assert!(file("").is_err());
        assert!(file("with space").is_err());
    }

    #[test]
    fn safe_write_replaces_the_file() {
        let dir = std::env::temp_dir().join(format!("notchy-test-{}", std::process::id()));
        let path = dir.join("data.json");
        write_safely(&path, "one").unwrap();
        write_safely(&path, "two").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "two");
        assert!(!dir.join(".data.json.tmp").exists());
        let _ = std::fs::remove_dir_all(dir);
    }
}
