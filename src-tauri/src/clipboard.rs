//! Recent clipboard history (text and images), kept in memory only, so it is
//! gone after a restart. Content that apps mark as private (password managers
//! set "ExcludeClipboardContentFromMonitorProcessing" or
//! "CanIncludeInClipboardHistory" = 0) is never stored.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Mutex, OnceLock};

use base64::Engine;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{GlobalFree, HANDLE, HGLOBAL, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::DataExchange::{
    AddClipboardFormatListener, CloseClipboard, EmptyClipboard, GetClipboardData, IsClipboardFormatAvailable,
    OpenClipboard, RegisterClipboardFormatW, SetClipboardData,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE};
use windows::Win32::System::Ole::{CF_DIB, CF_UNICODETEXT};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, RegisterClassW, HWND_MESSAGE, MSG,
    WINDOW_EX_STYLE, WINDOW_STYLE, WM_CLIPBOARDUPDATE, WNDCLASSW,
};

const MAX_ITEMS: usize = 20;
const MAX_TEXT: usize = 20_000;
const MAX_IMAGE: usize = 32 * 1024 * 1024;

enum Data {
    Text(String),
    /// A packed DIB (BITMAPINFOHEADER + pixels), as Windows hands it out.
    Image(Vec<u8>),
}

struct Item {
    id: u32,
    data: Data,
    /// Shown in the list: the text (shortened) or a small BMP thumbnail.
    preview: String,
    size: Option<(u32, u32)>,
}

#[derive(Serialize, Clone)]
pub struct Entry {
    id: u32,
    kind: &'static str,
    preview: String,
    width: Option<u32>,
    height: Option<u32>,
}

static ITEMS: Mutex<Vec<Item>> = Mutex::new(Vec::new());
static NEXT_ID: AtomicU32 = AtomicU32::new(1);
/// Set while we put something on the clipboard ourselves, so it isn't re-added.
static SELF_COPY: AtomicBool = AtomicBool::new(false);
static APP: OnceLock<AppHandle> = OnceLock::new();

pub fn start(app: AppHandle) {
    APP.set(app).ok();
    std::thread::spawn(|| unsafe {
        let instance = GetModuleHandleW(None).unwrap_or_default();
        let class = w!("NotchyClipboard");
        RegisterClassW(&WNDCLASSW { lpfnWndProc: Some(proc), hInstance: instance.into(), lpszClassName: class, ..Default::default() });
        let Ok(hwnd) = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            class,
            PCWSTR::null(),
            WINDOW_STYLE(0),
            0,
            0,
            0,
            0,
            Some(HWND_MESSAGE),
            None,
            Some(instance.into()),
            None,
        ) else {
            return;
        };
        if AddClipboardFormatListener(hwnd).is_err() {
            return;
        }
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            DispatchMessageW(&msg);
        }
    });
}

unsafe extern "system" fn proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg == WM_CLIPBOARDUPDATE {
        if !SELF_COPY.swap(false, Ordering::Relaxed)
            && let Some(data) = unsafe { read(hwnd) }
        {
            add(data);
        }
        return LRESULT(0);
    }
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

/// Apps flag content that shouldn't be kept (passwords, one-time codes).
unsafe fn is_private() -> bool {
    unsafe {
        let exclude = RegisterClipboardFormatW(w!("ExcludeClipboardContentFromMonitorProcessing"));
        let ignore = RegisterClipboardFormatW(w!("Clipboard Viewer Ignore"));
        if IsClipboardFormatAvailable(exclude).is_ok() || IsClipboardFormatAvailable(ignore).is_ok() {
            return true;
        }
        let history = RegisterClipboardFormatW(w!("CanIncludeInClipboardHistory"));
        if IsClipboardFormatAvailable(history).is_ok()
            && let Ok(h) = GetClipboardData(history)
        {
            let p = GlobalLock(HGLOBAL(h.0)) as *const u32;
            let allowed = p.is_null() || *p != 0;
            let _ = GlobalUnlock(HGLOBAL(h.0));
            return !allowed;
        }
        false
    }
}

unsafe fn read(hwnd: HWND) -> Option<Data> {
    unsafe {
        // Another app may hold the clipboard for a moment.
        let mut opened = false;
        for _ in 0..10 {
            if OpenClipboard(Some(hwnd)).is_ok() {
                opened = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(30));
        }
        if !opened {
            return None;
        }
        let data = (|| {
            if is_private() {
                return None;
            }
            if IsClipboardFormatAvailable(CF_UNICODETEXT.0 as u32).is_ok() {
                let h = GetClipboardData(CF_UNICODETEXT.0 as u32).ok()?;
                let p = GlobalLock(HGLOBAL(h.0)) as *const u16;
                if p.is_null() {
                    return None;
                }
                let len = (0..).take_while(|&i| *p.add(i) != 0).count();
                let text = String::from_utf16_lossy(std::slice::from_raw_parts(p, len));
                let _ = GlobalUnlock(HGLOBAL(h.0));
                return (!text.trim().is_empty()).then(|| Data::Text(text.chars().take(MAX_TEXT).collect()));
            }
            if IsClipboardFormatAvailable(CF_DIB.0 as u32).is_ok() {
                let h = GetClipboardData(CF_DIB.0 as u32).ok()?;
                let size = GlobalSize(HGLOBAL(h.0));
                let p = GlobalLock(HGLOBAL(h.0)) as *const u8;
                if p.is_null() || size == 0 || size > MAX_IMAGE {
                    let _ = GlobalUnlock(HGLOBAL(h.0));
                    return None;
                }
                let bytes = std::slice::from_raw_parts(p, size).to_vec();
                let _ = GlobalUnlock(HGLOBAL(h.0));
                return Some(Data::Image(bytes));
            }
            None
        })();
        let _ = CloseClipboard();
        data
    }
}

fn add(data: Data) {
    let (preview, size) = match &data {
        Data::Text(t) => (t.chars().take(200).collect::<String>(), None),
        Data::Image(dib) => match thumbnail(dib, 72) {
            Some((bmp, w, h)) => (format!("data:image/bmp;base64,{}", base64::engine::general_purpose::STANDARD.encode(bmp)), Some((w, h))),
            None => return,
        },
    };
    let mut items = ITEMS.lock().unwrap();
    // The same text or picture copied again moves to the top instead of repeating
    // (some apps also announce one image copy twice).
    match &data {
        Data::Text(t) => items.retain(|i| !matches!(&i.data, Data::Text(old) if old == t)),
        Data::Image(dib) => items.retain(|i| !matches!(&i.data, Data::Image(old) if old == dib)),
    }
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    items.insert(0, Item { id, data, preview, size });
    items.truncate(MAX_ITEMS);
    drop(items);
    notify();
}

fn notify() {
    if let Some(app) = APP.get() {
        let _ = app.emit("clipboard", clipboard_list());
    }
}

#[tauri::command]
pub fn clipboard_list() -> Vec<Entry> {
    ITEMS
        .lock()
        .unwrap()
        .iter()
        .map(|i| Entry {
            id: i.id,
            kind: if matches!(i.data, Data::Text(_)) { "text" } else { "image" },
            preview: i.preview.clone(),
            width: i.size.map(|s| s.0),
            height: i.size.map(|s| s.1),
        })
        .collect()
}

/// Puts an item back on the clipboard and moves it to the top.
#[tauri::command]
pub fn clipboard_copy(id: u32) -> bool {
    let mut items = ITEMS.lock().unwrap();
    let Some(pos) = items.iter().position(|i| i.id == id) else { return false };
    let item = items.remove(pos);
    let ok = unsafe { write(&item.data) };
    items.insert(0, item);
    drop(items);
    notify();
    ok
}

#[tauri::command]
pub fn clipboard_clear() {
    ITEMS.lock().unwrap().clear();
    notify();
}

unsafe fn write(data: &Data) -> bool {
    unsafe {
        let (format, bytes): (u32, Vec<u8>) = match data {
            Data::Text(t) => (CF_UNICODETEXT.0 as u32, t.encode_utf16().chain([0]).flat_map(|u| u.to_le_bytes()).collect()),
            Data::Image(dib) => (CF_DIB.0 as u32, dib.clone()),
        };
        let Ok(mem) = GlobalAlloc(GMEM_MOVEABLE, bytes.len()) else { return false };
        let p = GlobalLock(mem) as *mut u8;
        if p.is_null() {
            let _ = GlobalFree(Some(mem));
            return false;
        }
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), p, bytes.len());
        let _ = GlobalUnlock(mem);
        if OpenClipboard(None).is_err() {
            let _ = GlobalFree(Some(mem));
            return false;
        }
        SELF_COPY.store(true, Ordering::Relaxed);
        let _ = EmptyClipboard();
        let ok = SetClipboardData(format, Some(HANDLE(mem.0))).is_ok();
        let _ = CloseClipboard();
        // The clipboard owns the memory only once SetClipboardData succeeds.
        if !ok {
            SELF_COPY.store(false, Ordering::Relaxed);
            let _ = GlobalFree(Some(mem));
        }
        ok
    }
}

/// A small BMP (nearest-neighbour, at most `max` px wide/high) from a packed
/// 24/32-bit DIB, plus the original size. Other formats are skipped.
pub fn thumbnail(dib: &[u8], max: u32) -> Option<(Vec<u8>, u32, u32)> {
    let u32_at = |o: usize| dib.get(o..o + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap()));
    let header = u32_at(0)? as usize;
    let width = i32::from_le_bytes(dib.get(4..8)?.try_into().ok()?);
    let height = i32::from_le_bytes(dib.get(8..12)?.try_into().ok()?);
    let bpp = u16::from_le_bytes(dib.get(14..16)?.try_into().ok()?) as usize;
    let compression = u32_at(16)?;
    if width <= 0 || height == 0 || !(bpp == 24 || bpp == 32) || !(compression == 0 || compression == 3) {
        return None;
    }
    let (w, h) = (width as usize, height.unsigned_abs() as usize);
    // BI_BITFIELDS with a classic 40-byte header carries 3 masks after it.
    let masks = if compression == 3 && header == 40 { 12 } else { 0 };
    let pixels = header + masks;
    let stride = (w * bpp / 8 + 3) & !3;
    let total = stride.checked_mul(h).and_then(|s| pixels.checked_add(s))?;
    if dib.len() < total {
        return None;
    }
    let scale = (w.max(h) as f64 / max as f64).max(1.0);
    let (tw, th) = (((w as f64 / scale).round() as usize).max(1), ((h as f64 / scale).round() as usize).max(1));
    let tstride = (tw * 3 + 3) & !3;
    let mut out = Vec::with_capacity(54 + tstride * th);
    let file_size = (54 + tstride * th) as u32;
    out.extend_from_slice(b"BM");
    out.extend_from_slice(&file_size.to_le_bytes());
    out.extend_from_slice(&[0; 4]);
    out.extend_from_slice(&54u32.to_le_bytes());
    out.extend_from_slice(&40u32.to_le_bytes());
    out.extend_from_slice(&(tw as i32).to_le_bytes());
    out.extend_from_slice(&(th as i32).to_le_bytes()); // bottom-up
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&24u16.to_le_bytes());
    out.extend_from_slice(&[0; 24]);
    let bottom_up = height > 0;
    for ty in 0..th {
        // Thumbnail rows are written bottom-up; pick the matching source row.
        let from_top = th - 1 - ty;
        let sy_top = ((from_top as f64 + 0.5) * scale) as usize;
        let sy_top = sy_top.min(h - 1);
        let row = if bottom_up { h - 1 - sy_top } else { sy_top };
        for tx in 0..tw {
            let sx = (((tx as f64 + 0.5) * scale) as usize).min(w - 1);
            let o = pixels + row * stride + sx * bpp / 8;
            out.extend_from_slice(&dib[o..o + 3]);
        }
        out.extend(std::iter::repeat_n(0, tstride - tw * 3));
    }
    Some((out, w as u32, h as u32))
}

#[cfg(test)]
mod tests {
    use super::thumbnail;

    /// A packed 32-bit bottom-up DIB, `w`×`h`, every pixel (b, g, r).
    fn dib(w: i32, h: i32, px: [u8; 3]) -> Vec<u8> {
        let mut d = vec![];
        d.extend_from_slice(&40u32.to_le_bytes());
        d.extend_from_slice(&w.to_le_bytes());
        d.extend_from_slice(&h.to_le_bytes());
        d.extend_from_slice(&1u16.to_le_bytes());
        d.extend_from_slice(&32u16.to_le_bytes());
        d.extend_from_slice(&[0; 24]);
        for _ in 0..(w * h.abs()) {
            d.extend_from_slice(&[px[0], px[1], px[2], 255]);
        }
        d
    }

    #[test]
    fn thumbnails_keep_aspect_and_colour() {
        let (bmp, w, h) = thumbnail(&dib(1280, 720, [10, 20, 30]), 72).unwrap();
        assert_eq!((w, h), (1280, 720));
        assert_eq!(&bmp[0..2], b"BM");
        let tw = i32::from_le_bytes(bmp[18..22].try_into().unwrap());
        let th = i32::from_le_bytes(bmp[22..26].try_into().unwrap());
        assert_eq!((tw, th), (72, 41));
        assert_eq!(&bmp[54..57], &[10, 20, 30]);
    }

    #[test]
    fn small_and_top_down_images_work() {
        let (bmp, w, h) = thumbnail(&dib(4, -3, [1, 2, 3]), 72).unwrap();
        assert_eq!((w, h), (4, 3));
        assert_eq!(i32::from_le_bytes(bmp[18..22].try_into().unwrap()), 4);
    }

    #[test]
    fn rejects_garbage() {
        assert!(thumbnail(&[0; 10], 72).is_none());
        let mut d = dib(2, 2, [0, 0, 0]);
        d[14] = 8; // 8-bit palette images aren't handled
        assert!(thumbnail(&d, 72).is_none());
    }
}
