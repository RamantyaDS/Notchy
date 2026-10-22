//! Files dropped on the notch's "Bagikan" zone go to the Windows Share window
//! (Nearby Share, WhatsApp, Mail...). Desktop apps reach it through
//! IDataTransferManagerInterop on their own window.

use std::sync::Mutex;

use tauri::{AppHandle, Manager};
use windows::core::{Interface, HSTRING};
use windows::ApplicationModel::DataTransfer::{DataRequestedEventArgs, DataTransferManager};
use windows::Foundation::TypedEventHandler;
use windows::Storage::{IStorageItem, StorageFile};
use windows::Win32::Foundation::HWND;
use windows::Win32::System::WinRT::RoGetActivationFactory;
use windows::Win32::UI::Shell::IDataTransferManagerInterop;
use windows_collections::IIterable;

/// Files for the next share request, and the manager whose handler serves them.
static PENDING: Mutex<Vec<String>> = Mutex::new(Vec::new());
thread_local! {
    // Lives on the UI thread, where the share UI is always opened.
    static REGISTERED: std::cell::RefCell<Option<(DataTransferManager, i64)>> = const { std::cell::RefCell::new(None) };
}

/// Only files: the share targets don't take folders.
#[tauri::command]
pub fn share_files(app: AppHandle, paths: Vec<String>) -> Result<(), String> {
    let paths: Vec<String> = paths.into_iter().filter(|p| std::path::Path::new(p).is_file()).collect();
    if paths.is_empty() {
        return Err("Tidak ada file untuk dibagikan".into());
    }
    let window = app.get_webview_window("main").ok_or("no window")?;
    let hwnd = window.hwnd().map_err(|e| e.to_string())?.0 as isize;
    *PENDING.lock().unwrap() = paths;
    // The share UI has to be opened from the window's own (UI) thread.
    window
        .run_on_main_thread(move || {
            if let Err(e) = unsafe { show(HWND(hwnd as _)) } {
                crate::log_line(&format!("share failed: {e}"));
            }
        })
        .map_err(|e| e.to_string())
}

unsafe fn show(hwnd: HWND) -> windows::core::Result<()> {
    unsafe {
        let interop: IDataTransferManagerInterop =
            RoGetActivationFactory(&HSTRING::from("Windows.ApplicationModel.DataTransfer.DataTransferManager"))?;
        if REGISTERED.with_borrow(|r| r.is_none()) {
            let manager: DataTransferManager = interop.GetForWindow(hwnd)?;
            let token = manager.DataRequested(&TypedEventHandler::new(
                |_, args: windows::core::Ref<DataRequestedEventArgs>| {
                    let Some(args) = args.as_ref() else { return Ok(()) };
                    let data = args.Request()?.Data()?;
                    let paths = PENDING.lock().unwrap().clone();
                    let names: Vec<String> = paths
                        .iter()
                        .map(|p| std::path::Path::new(p).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default())
                        .collect();
                    data.Properties()?.SetTitle(&HSTRING::from(names.join(", ")))?;
                    let items: Vec<Option<IStorageItem>> = paths
                        .iter()
                        .filter_map(|p| StorageFile::GetFileFromPathAsync(&HSTRING::from(p.as_str())).ok()?.get().ok()?.cast().ok())
                        .map(Some)
                        .collect();
                    let items: IIterable<IStorageItem> = items.into();
                    data.SetStorageItemsReadOnly(&items)?;
                    Ok(())
                },
            ))?;
            REGISTERED.set(Some((manager, token)));
        }
        interop.ShowShareUIForWindow(hwnd)
    }
}
