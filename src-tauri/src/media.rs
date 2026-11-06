//! Now playing, from the Windows media overlay: any player that shows up there
//! (Spotify, YouTube in a browser, ...). Changes arrive as WinRT events.

use std::sync::{Mutex, OnceLock};

use base64::Engine;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use windows::Foundation::TypedEventHandler;
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession as Session,
    GlobalSystemMediaTransportControlsSessionManager as Manager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus as Status,
};
use windows::Storage::Streams::{DataReader, IRandomAccessStreamReference};
use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};

static MANAGER: OnceLock<Manager> = OnceLock::new();
/// The session we listen to, with its event tokens so they can be removed when it changes.
static CURRENT: Mutex<Option<(Session, [i64; 3])>> = Mutex::new(None);
/// Player the user picked by clicking the cover (app id), instead of the one
/// Windows considers current. Cleared when Windows switches by itself.
static CHOSEN: Mutex<Option<String>> = Mutex::new(None);
/// Album art is costly to read, so the last one is kept: (track key, data URL).
static ART: Mutex<Option<(String, String)>> = Mutex::new(None);

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NowPlaying {
    title: String,
    artist: String,
    album: String,
    /// Album art as a data URL.
    art: Option<String>,
    /// App id of the player, e.g. "Spotify.exe" or "MSEdge".
    source: String,
    playing: bool,
    /// How many players Windows lists; above 1 the cover switches between them.
    sources: u32,
    /// Seconds.
    position: f64,
    duration: f64,
    /// Unix ms at which `position` was true.
    updated_at: f64,
}

pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        // This thread stays alive as the MTA that WinRT callbacks and async commands rely on.
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        }
        let Ok(manager) = Manager::RequestAsync().and_then(|op| op.get()) else { return };
        let handle = app.clone();
        let _ = manager.CurrentSessionChanged(&TypedEventHandler::new(move |_, _| {
            *CHOSEN.lock().unwrap() = None;
            watch_current(&handle);
            Ok(())
        }));
        let handle = app.clone();
        let _ = manager.SessionsChanged(&TypedEventHandler::new(move |_, _| {
            watch_current(&handle);
            Ok(())
        }));
        MANAGER.set(manager).ok();
        watch_current(&app);
        loop {
            std::thread::park();
        }
    });
}

/// Moves our event handlers to whichever session Windows now considers current.
fn watch_current(app: &AppHandle) {
    if MANAGER.get().is_none() {
        return;
    }
    let mut current = CURRENT.lock().unwrap();
    if let Some((session, [props, playback, timeline])) = current.take() {
        let _ = session.RemoveMediaPropertiesChanged(props);
        let _ = session.RemovePlaybackInfoChanged(playback);
        let _ = session.RemoveTimelinePropertiesChanged(timeline);
    }
    if let Some(session) = session() {
        let (a, b, c) = (app.clone(), app.clone(), app.clone());
        let tokens = [
            session
                .MediaPropertiesChanged(&TypedEventHandler::new(move |_, _| Ok(publish(&a))))
                .unwrap_or(0),
            session
                .PlaybackInfoChanged(&TypedEventHandler::new(move |_, _| Ok(publish(&b))))
                .unwrap_or(0),
            session
                .TimelinePropertiesChanged(&TypedEventHandler::new(move |_, _| Ok(publish(&c))))
                .unwrap_or(0),
        ];
        *current = Some((session, tokens));
    }
    drop(current);
    publish(app);
}

fn publish(app: &AppHandle) {
    let _ = app.emit("media", now_playing());
}

fn sessions() -> Vec<Session> {
    MANAGER.get().and_then(|m| m.GetSessions().ok()).map(|list| list.into_iter().collect()).unwrap_or_default()
}

fn app_id(session: &Session) -> String {
    session.SourceAppUserModelId().map(|s| s.to_string()).unwrap_or_default()
}

/// The chosen player while it's still there, otherwise Windows' current one.
fn session() -> Option<Session> {
    let chosen = CHOSEN.lock().unwrap().clone();
    if let Some(id) = chosen {
        if let Some(s) = sessions().into_iter().find(|s| app_id(s) == id) {
            return Some(s);
        }
        *CHOSEN.lock().unwrap() = None;
    }
    MANAGER.get()?.GetCurrentSession().ok()
}

fn now_playing() -> Option<NowPlaying> {
    let session = session()?;
    let props = session.TryGetMediaPropertiesAsync().ok()?.get().ok()?;
    let title = props.Title().ok()?.to_string();
    if title.is_empty() {
        return None;
    }
    let artist = props.Artist().map(|s| s.to_string()).unwrap_or_default();
    let album = props.AlbumTitle().map(|s| s.to_string()).unwrap_or_default();
    let playing = session
        .GetPlaybackInfo()
        .and_then(|info| info.PlaybackStatus())
        .is_ok_and(|status| status == Status::Playing);

    let secs = |ticks: i64| ticks as f64 / 10_000_000.0;
    let (position, duration, updated_at) = match session.GetTimelineProperties() {
        Ok(t) => (
            t.Position().map(|p| secs(p.Duration)).unwrap_or(0.0),
            t.EndTime().map(|p| secs(p.Duration)).unwrap_or(0.0),
            // DateTime counts 100ns ticks since 1601.
            t.LastUpdatedTime()
                .map(|d| (d.UniversalTime - 116_444_736_000_000_000) as f64 / 10_000.0)
                .unwrap_or(0.0),
        ),
        Err(_) => (0.0, 0.0, 0.0),
    };

    let key = format!("{title}\u{1}{artist}");
    let cached = ART.lock().unwrap().as_ref().filter(|(k, _)| *k == key).map(|(_, a)| a.clone());
    let art = cached.or_else(|| {
        let art = read_art(props.Thumbnail().ok()?)?;
        *ART.lock().unwrap() = Some((key, art.clone()));
        Some(art)
    });

    Some(NowPlaying {
        title,
        artist,
        album,
        art,
        source: app_id(&session),
        playing,
        sources: sessions().len() as u32,
        position,
        duration,
        updated_at,
    })
}

/// Reads an image stream (album art, app logo) into a data URL.
pub(crate) fn read_art(thumbnail: IRandomAccessStreamReference) -> Option<String> {
    let stream = thumbnail.OpenReadAsync().ok()?.get().ok()?;
    let size = stream.Size().ok()?;
    // Art is a few hundred KB at most; a huge stream is not worth reading.
    if size == 0 || size > 16 * 1024 * 1024 {
        return None;
    }
    let size = size as u32;
    let reader = DataReader::CreateDataReader(&stream.GetInputStreamAt(0).ok()?).ok()?;
    reader.LoadAsync(size).ok()?.get().ok()?;
    let mut bytes = vec![0u8; size as usize];
    reader.ReadBytes(&mut bytes).ok()?;
    let mime = stream
        .ContentType()
        .map(|s| s.to_string())
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "image/png".into());
    Some(format!("data:{mime};base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes)))
}

// Commands are async so they run on a worker thread, not the UI thread, while
// they wait for the player.

#[tauri::command]
pub async fn media_now() -> Option<NowPlaying> {
    now_playing()
}

/// Clicking the cover: show the next player Windows lists.
#[tauri::command]
pub async fn media_next_source(app: AppHandle) {
    let list = sessions();
    if list.len() < 2 {
        return;
    }
    let current = session().map(|s| app_id(&s)).unwrap_or_default();
    let at = list.iter().position(|s| app_id(s) == current).unwrap_or(0);
    let next = app_id(&list[(at + 1) % list.len()]);
    *CHOSEN.lock().unwrap() = Some(next);
    watch_current(&app);
}

#[tauri::command]
pub async fn media_toggle() {
    if let Some(s) = session() {
        let _ = s.TryTogglePlayPauseAsync().and_then(|op| op.get());
    }
}

#[tauri::command]
pub async fn media_next() {
    if let Some(s) = session() {
        let _ = s.TrySkipNextAsync().and_then(|op| op.get());
    }
}

#[tauri::command]
pub async fn media_prev() {
    if let Some(s) = session() {
        let _ = s.TrySkipPreviousAsync().and_then(|op| op.get());
    }
}

#[tauri::command]
pub async fn media_seek(seconds: f64) {
    if let Some(s) = session() {
        let _ = s
            .TryChangePlaybackPositionAsync((seconds * 10_000_000.0) as i64)
            .and_then(|op| op.get());
    }
}
