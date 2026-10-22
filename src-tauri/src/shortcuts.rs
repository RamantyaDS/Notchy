//! The shortcuts page: lock, sleep, screenshot, Downloads, or anything the
//! user set up in Settings (an app, file, folder, link or script).

use windows::core::HSTRING;
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

use crate::settings::{get_settings, Shortcut};

/// What a shortcut does, decided from its settings entry. Kept apart from
/// running it so the mapping can be tested.
#[derive(Debug, PartialEq)]
pub enum Plan {
    Lock,
    Sleep,
    /// Opens with the shell: an app, file, folder, link, or ms-screenclip:.
    Open(String),
    Nothing,
}

pub fn plan(shortcut: &Shortcut, downloads: Option<String>) -> Plan {
    match shortcut.action.as_str() {
        "lock" => Plan::Lock,
        "sleep" => Plan::Sleep,
        "screenshot" => Plan::Open("ms-screenclip:".into()),
        "downloads" => downloads.map(Plan::Open).unwrap_or(Plan::Nothing),
        "open" => {
            let target = shortcut.target.trim().trim_matches('"').to_string();
            if target.is_empty() { Plan::Nothing } else { Plan::Open(target) }
        }
        _ => Plan::Nothing,
    }
}

#[tauri::command]
pub fn run_shortcut(index: usize) -> Result<(), String> {
    let settings = get_settings();
    let shortcut = settings.shortcuts.get(index).ok_or("Pintasan tidak ada")?;
    let downloads = crate::downloads::downloads_dir().map(|d| d.to_string_lossy().to_string());
    match plan(shortcut, downloads) {
        Plan::Lock => unsafe { windows::Win32::System::Shutdown::LockWorkStation() }.map_err(|e| e.to_string()),
        Plan::Sleep => {
            // Let the click finish before the machine goes down.
            std::thread::spawn(|| {
                std::thread::sleep(std::time::Duration::from_millis(400));
                unsafe {
                    let _ = windows::Win32::System::Power::SetSuspendState(false, false, false);
                }
            });
            Ok(())
        }
        Plan::Open(target) => {
            let result = unsafe {
                ShellExecuteW(None, &HSTRING::from("open"), &HSTRING::from(target.as_str()), None, None, SW_SHOWNORMAL)
            };
            // ShellExecute reports success as a value above 32.
            if result.0 as usize > 32 { Ok(()) } else { Err(format!("Tidak bisa membuka {target}")) }
        }
        Plan::Nothing => Err("Pintasan belum diatur".into()),
    }
}

/// Opens a meeting link from the calendar ("Join"). Web links only.
#[tauri::command]
pub fn run_link(url: String) -> Result<(), String> {
    if !is_web_link(&url) {
        return Err("Bukan link web".into());
    }
    let result =
        unsafe { ShellExecuteW(None, &HSTRING::from("open"), &HSTRING::from(url.as_str()), None, None, SW_SHOWNORMAL) };
    if result.0 as usize > 32 { Ok(()) } else { Err("Link tidak bisa dibuka".into()) }
}

fn is_web_link(url: &str) -> bool {
    url.starts_with("https://") && !url.chars().any(|c| c.is_whitespace() || c == '"')
}

#[cfg(test)]
mod tests {
    #[test]
    fn only_web_links_are_opened() {
        assert!(super::is_web_link("https://meet.google.com/abc-defg-hij"));
        assert!(!super::is_web_link("file:///C:/Windows/System32/cmd.exe"));
        assert!(!super::is_web_link("https://x.com/a b"));
        assert!(!super::is_web_link("ms-settings:"));
    }
}

#[cfg(test)]
mod plan_tests {
    use super::*;

    fn s(action: &str, target: &str) -> Shortcut {
        Shortcut { name: "x".into(), action: action.into(), target: target.into() }
    }

    #[test]
    fn maps_actions() {
        assert_eq!(plan(&s("lock", ""), None), Plan::Lock);
        assert_eq!(plan(&s("sleep", ""), None), Plan::Sleep);
        assert_eq!(plan(&s("screenshot", ""), None), Plan::Open("ms-screenclip:".into()));
        assert_eq!(plan(&s("downloads", ""), Some(r"C:\Users\me\Downloads".into())), Plan::Open(r"C:\Users\me\Downloads".into()));
        assert_eq!(plan(&s("open", r#" "C:\Tools\backup.bat" "#), None), Plan::Open(r"C:\Tools\backup.bat".into()));
        assert_eq!(plan(&s("open", "https://mail.google.com"), None), Plan::Open("https://mail.google.com".into()));
    }

    #[test]
    fn empty_or_unknown_does_nothing() {
        assert_eq!(plan(&s("open", "  "), None), Plan::Nothing);
        assert_eq!(plan(&s("format-c", ""), None), Plan::Nothing);
        assert_eq!(plan(&s("downloads", ""), None), Plan::Nothing);
    }

    #[test]
    fn defaults_are_the_four_buttons() {
        let names: Vec<_> = crate::settings::default_shortcuts().into_iter().map(|s| s.action).collect();
        assert_eq!(names, ["lock", "sleep", "screenshot", "downloads", "camera"]);
    }
}
