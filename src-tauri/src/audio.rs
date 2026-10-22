//! Volume, microphone mute and output-device changes, all as COM callbacks.

use std::sync::Mutex;

use serde_json::json;
use tauri::{AppHandle, Emitter};
use serde::Serialize;
use windows::core::{implement, Interface, Result, GUID, HRESULT, HSTRING, PCWSTR};
use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
use windows::Win32::Foundation::PROPERTYKEY;
use windows::Win32::Media::Audio::Endpoints::{
    IAudioEndpointVolume, IAudioEndpointVolumeCallback, IAudioEndpointVolumeCallback_Impl,
};
use windows::Win32::Media::Audio::{
    eCapture, eCommunications, eConsole, eMultimedia, eRender, EDataFlow, ERole, IMMDevice, IMMDeviceEnumerator,
    IMMNotificationClient, IMMNotificationClient_Impl, MMDeviceEnumerator, AUDIO_VOLUME_NOTIFICATION_DATA,
    DEVICE_STATE, DEVICE_STATE_ACTIVE, PKEY_AudioEndpoint_FormFactor,
};
use windows::Win32::System::Com::StructuredStorage::{PropVariantToStringAlloc, PropVariantToUInt32};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoTaskMemFree, CLSCTX_ALL, COINIT_MULTITHREADED, STGM_READ,
};

use crate::peek;

/// What we are registered on, so callbacks can be moved when the default device changes.
struct Endpoints {
    enumerator: IMMDeviceEnumerator,
    output: Option<(IAudioEndpointVolume, IAudioEndpointVolumeCallback)>,
    mic: Option<(IAudioEndpointVolume, IAudioEndpointVolumeCallback)>,
}
// The COM objects are free-threaded (MTA), so they may move between threads.
unsafe impl Send for Endpoints {}

static ENDPOINTS: Mutex<Option<Endpoints>> = Mutex::new(None);

#[implement(IAudioEndpointVolumeCallback)]
struct VolumeCallback {
    app: AppHandle,
    mic: bool,
}

impl IAudioEndpointVolumeCallback_Impl for VolumeCallback_Impl {
    fn OnNotify(&self, data: *mut AUDIO_VOLUME_NOTIFICATION_DATA) -> Result<()> {
        let data = unsafe { &*data };
        let muted = data.bMuted.as_bool();
        if self.mic {
            let _ = self.app.emit("mic-muted", muted);
            peek(&self.app, "mic", json!({ "muted": muted }));
        } else {
            peek(&self.app, "volume", json!({ "level": data.fMasterVolume, "muted": muted }));
        }
        Ok(())
    }
}

#[implement(IMMNotificationClient)]
struct DeviceCallback {
    app: AppHandle,
}

impl IMMNotificationClient_Impl for DeviceCallback_Impl {
    fn OnDeviceStateChanged(&self, _: &PCWSTR, _: DEVICE_STATE) -> Result<()> {
        Ok(())
    }
    fn OnDeviceAdded(&self, _: &PCWSTR) -> Result<()> {
        Ok(())
    }
    fn OnDeviceRemoved(&self, _: &PCWSTR) -> Result<()> {
        Ok(())
    }
    fn OnDefaultDeviceChanged(&self, flow: EDataFlow, role: ERole, _: &PCWSTR) -> Result<()> {
        if role != eConsole {
            return Ok(());
        }
        // Calling back into the audio API from this callback can deadlock; do it elsewhere.
        let app = self.app.clone();
        std::thread::spawn(move || {
            unsafe {
                let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            }
            attach(&app, flow == eCapture, flow == eRender);
        });
        Ok(())
    }
    fn OnPropertyValueChanged(&self, _: &PCWSTR, _: &PROPERTYKEY) -> Result<()> {
        Ok(())
    }
}

pub fn start(app: AppHandle) {
    std::thread::spawn(move || unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let Ok(enumerator) = CoCreateInstance::<_, IMMDeviceEnumerator>(&MMDeviceEnumerator, None, CLSCTX_ALL) else {
            return;
        };
        let client: IMMNotificationClient = DeviceCallback { app: app.clone() }.into();
        let _ = enumerator.RegisterEndpointNotificationCallback(&client);
        *ENDPOINTS.lock().unwrap() = Some(Endpoints { enumerator, output: None, mic: None });
        attach(&app, true, false);
        attach(&app, false, false);
        let _ = app.emit("mic-muted", mic_muted());
        // Keep the MTA (and the device callback) alive.
        std::mem::forget(client);
        loop {
            std::thread::park();
        }
    });
}

/// (Re)registers the volume callback on the current default mic or output device.
/// With `announce`, a new output device is shown in the notch.
fn attach(app: &AppHandle, mic: bool, announce: bool) {
    let mut guard = ENDPOINTS.lock().unwrap();
    let Some(endpoints) = guard.as_mut() else { return };
    let slot = if mic { &mut endpoints.mic } else { &mut endpoints.output };
    if let Some((volume, callback)) = slot.take() {
        unsafe {
            let _ = volume.UnregisterControlChangeNotify(&callback);
        }
    }
    let flow = if mic { eCapture } else { eRender };
    let Ok(device) = (unsafe { endpoints.enumerator.GetDefaultAudioEndpoint(flow, eConsole) }) else { return };
    if let Ok(volume) = unsafe { device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None) } {
        let callback: IAudioEndpointVolumeCallback = VolumeCallback { app: app.clone(), mic }.into();
        if unsafe { volume.RegisterControlChangeNotify(&callback) }.is_ok() {
            *slot = Some((volume, callback));
        }
    }
    drop(guard);
    if announce {
        let (name, headphones) = describe(&device);
        let battery = if headphones { battery_of(&device) } else { None };
        peek(app, "output", json!({ "name": name, "headphones": headphones, "battery": battery }));
        if headphones && battery.is_some() {
            watch_battery(app.clone(), device_id(&device));
        }
    }
}

// ---------------------------------------------------------------- earbuds battery

/// PKEY_Device_ContainerId: ties the audio endpoint to the physical device.
const PKEY_CONTAINER_ID: PROPERTYKEY = PROPERTYKEY {
    fmtid: GUID::from_u128(0x8c7ed206_3f8a_4827_b3ab_ae9e1faefc6c),
    pid: 2,
};
/// Battery level that Bluetooth headsets report (hands-free profile or BLE battery service).
const BATTERY_KEY: &str = "{104EA319-6EE2-4701-BD47-8DDBF425BBE5} 2";

/// Battery percent of a wireless headset, when the device reports one.
fn battery_of(device: &IMMDevice) -> Option<u8> {
    use windows::core::IInspectable;
    use windows::Devices::Enumeration::{DeviceInformation, DeviceInformationKind};
    use windows_collections::IIterable;
    use windows::Foundation::IReference;
    use windows::Win32::System::Com::StructuredStorage::PropVariantToGUID;
    let container = unsafe {
        let store = device.OpenPropertyStore(STGM_READ).ok()?;
        PropVariantToGUID(&store.GetValue(&PKEY_CONTAINER_ID).ok()?).ok()?
    };
    let aqs = HSTRING::from(format!("System.Devices.ContainerId:=\"{{{container:?}}}\""));
    let props: IIterable<HSTRING> = vec![HSTRING::from(BATTERY_KEY)].into();
    let found = DeviceInformation::FindAllAsyncWithKindAqsFilterAndAdditionalProperties(&aqs, &props, DeviceInformationKind::Device)
        .ok()?
        .get()
        .ok()?;
    found.into_iter().find_map(|info| {
        let value: IInspectable = info.Properties().ok()?.Lookup(&HSTRING::from(BATTERY_KEY)).ok()?;
        value.cast::<IReference<u8>>().ok()?.Value().ok()
    })
}

/// Warns once when the headset in use drops below 15%. Checks every 5 minutes
/// while it stays the default output.
fn watch_battery(app: AppHandle, id: String) {
    std::thread::spawn(move || unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let mut warned = false;
        loop {
            std::thread::sleep(std::time::Duration::from_secs(300));
            let device = {
                let guard = ENDPOINTS.lock().unwrap();
                let Some(e) = guard.as_ref() else { return };
                match e.enumerator.GetDefaultAudioEndpoint(eRender, eConsole) {
                    Ok(d) if device_id(&d) == id => d,
                    _ => return,
                }
            };
            let Some(level) = battery_of(&device) else { return };
            if level < 15 && !warned {
                warned = true;
                let (name, _) = describe(&device);
                peek(&app, "earbuds-low", json!({ "name": name, "battery": level }));
            }
        }
    });
}

// ---------------------------------------------------------------- volume by scrolling

/// One step up or down on the default output (scroll over the notch). The
/// volume callback then shows the usual volume peek.
#[tauri::command]
pub async fn volume_step(up: bool) {
    let guard = ENDPOINTS.lock().unwrap();
    if let Some((volume, _)) = guard.as_ref().and_then(|e| e.output.as_ref()) {
        unsafe {
            let level = volume.GetMasterVolumeLevelScalar().unwrap_or(0.0);
            let next = (level + if up { 0.02 } else { -0.02 }).clamp(0.0, 1.0);
            if up {
                let _ = volume.SetMute(false, std::ptr::null());
            }
            let _ = volume.SetMasterVolumeLevelScalar(next, std::ptr::null());
        }
    }
}

// ---------------------------------------------------------------- level meter (equalizer)

static METERING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Turns the level meter on while the equalizer is on screen. It sends four bar
/// heights (0..1) 5 times a second; the page also turns the disc on each one, so
/// the whole island redraws once per tick (cheap in a transparent window).
#[tauri::command]
pub fn set_metering(app: AppHandle, on: bool) {
    use std::sync::atomic::Ordering;
    if !on {
        METERING.store(false, Ordering::Relaxed);
        return;
    }
    if METERING.swap(true, Ordering::Relaxed) {
        return;
    }
    std::thread::spawn(move || unsafe {
        use windows::Win32::Media::Audio::Endpoints::IAudioMeterInformation;
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let meter = {
            let guard = ENDPOINTS.lock().unwrap();
            guard
                .as_ref()
                .and_then(|e| e.enumerator.GetDefaultAudioEndpoint(eRender, eConsole).ok())
                .and_then(|d| d.Activate::<IAudioMeterInformation>(CLSCTX_ALL, None).ok())
        };
        let Some(meter) = meter else {
            METERING.store(false, Ordering::Relaxed);
            return;
        };
        let mut bars = [0f32; 4];
        let mut t = 0u32;
        while METERING.load(Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(200));
            let mut ch = [0f32; 2];
            let peak = meter.GetPeakValue().unwrap_or(0.0);
            let _ = meter.GetChannelsPeakValues(&mut ch);
            bars = next_bars(bars, peak, ch, t);
            t = t.wrapping_add(1);
            // Quantized to tenths; sent every tick, since it also turns the disc.
            let _ = app.emit("audio-level", bars.map(|b| (b * 10.0).round() / 10.0));
        }
    });
}

/// Four lively bars from one peak and two channel peaks: each rises at once
/// and falls back at its own pace, with a little sway so they don't move as one.
pub fn next_bars(prev: [f32; 4], peak: f32, ch: [f32; 2], t: u32) -> [f32; 4] {
    // Peaks are linear amplitude; a square root reads closer to loudness.
    let loud = |v: f32| v.clamp(0.0, 1.0).sqrt();
    let sway = |i: usize| 0.85 + 0.15 * (((t as f32) * (0.9 + i as f32 * 0.37)).sin() * 0.5 + 0.5);
    let target = [loud(ch[0]) * sway(0), loud(peak) * sway(1), loud(ch[1]) * sway(2), loud((ch[0] + ch[1]) / 2.0) * sway(3)];
    let fall = [0.55, 0.7, 0.6, 0.65];
    let mut out = [0f32; 4];
    for i in 0..4 {
        out[i] = if target[i] >= prev[i] { target[i] } else { prev[i] * fall[i] + target[i] * (1.0 - fall[i]) };
        if peak < 0.001 {
            out[i] = 0.0;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::next_bars;

    #[test]
    fn bars_are_flat_in_silence_and_rise_with_sound() {
        let quiet = next_bars([0.5; 4], 0.0, [0.0, 0.0], 3);
        assert_eq!(quiet, [0.0; 4]);
        let loud = next_bars([0.0; 4], 0.8, [0.7, 0.9], 3);
        assert!(loud.iter().all(|b| *b > 0.6 && *b <= 1.0), "{loud:?}");
    }

    #[test]
    fn bars_fall_gradually() {
        let up = next_bars([0.0; 4], 1.0, [1.0, 1.0], 0);
        let down = next_bars(up, 0.05, [0.05, 0.05], 1);
        for i in 0..4 {
            assert!(down[i] < up[i] && down[i] > 0.3, "bar {i}: {} -> {}", up[i], down[i]);
        }
    }
}

/// Friendly name, and whether it is worn (headphones or headset).
fn describe(device: &IMMDevice) -> (String, bool) {
    unsafe {
        let Ok(store) = device.OpenPropertyStore(STGM_READ) else { return (String::new(), false) };
        let name = store
            .GetValue(&PKEY_Device_FriendlyName)
            .and_then(|v| PropVariantToStringAlloc(&v))
            .map(|p| {
                let s = p.to_string().unwrap_or_default();
                CoTaskMemFree(Some(p.0 as _));
                s
            })
            .unwrap_or_default();
        // EndpointFormFactor: 3 = Headphones, 5 = Headset.
        let form = store.GetValue(&PKEY_AudioEndpoint_FormFactor).and_then(|v| PropVariantToUInt32(&v));
        (name, matches!(form, Ok(3) | Ok(5)))
    }
}

fn mic_muted() -> bool {
    let guard = ENDPOINTS.lock().unwrap();
    let Some((volume, _)) = guard.as_ref().and_then(|e| e.mic.as_ref()) else { return false };
    unsafe { volume.GetMute().is_ok_and(|m| m.as_bool()) }
}

#[tauri::command]
pub async fn mic_state() -> bool {
    mic_muted()
}

#[tauri::command]
pub async fn toggle_mic() {
    let guard = ENDPOINTS.lock().unwrap();
    if let Some((volume, _)) = guard.as_ref().and_then(|e| e.mic.as_ref()) {
        unsafe {
            let muted = volume.GetMute().is_ok_and(|m| m.as_bool());
            let _ = volume.SetMute(!muted, std::ptr::null());
        }
    }
}

// ---------------------------------------------------------------- switching the output device

/// Undocumented COM interface that every audio switcher (EarTrumpet, SoundSwitch...)
/// uses to change the default device. Only `SetDefaultEndpoint` is called; the
/// methods before it just keep the vtable layout.
#[windows::core::interface("f8679f50-850a-41cf-9c72-430f290290c8")]
unsafe trait IPolicyConfig: windows::core::IUnknown {
    fn get_mix_format(&self) -> HRESULT;
    fn get_device_format(&self) -> HRESULT;
    fn reset_device_format(&self) -> HRESULT;
    fn set_device_format(&self) -> HRESULT;
    fn get_processing_period(&self) -> HRESULT;
    fn set_processing_period(&self) -> HRESULT;
    fn get_share_mode(&self) -> HRESULT;
    fn set_share_mode(&self) -> HRESULT;
    fn get_property_value(&self) -> HRESULT;
    fn set_property_value(&self) -> HRESULT;
    fn set_default_endpoint(&self, id: PCWSTR, role: ERole) -> HRESULT;
}

const POLICY_CONFIG_CLIENT: GUID = GUID::from_u128(0x870af99c_171d_4f9e_af0d_e63df40c2bc9);

#[derive(Serialize)]
pub struct Output {
    id: String,
    name: String,
    headphones: bool,
    current: bool,
}

fn device_id(device: &IMMDevice) -> String {
    unsafe {
        device
            .GetId()
            .map(|p| {
                let s = p.to_string().unwrap_or_default();
                CoTaskMemFree(Some(p.0 as _));
                s
            })
            .unwrap_or_default()
    }
}

/// Active output devices, with the default one marked.
#[tauri::command]
pub async fn list_outputs() -> Vec<Output> {
    let guard = ENDPOINTS.lock().unwrap();
    let Some(endpoints) = guard.as_ref() else { return vec![] };
    unsafe {
        let current = endpoints
            .enumerator
            .GetDefaultAudioEndpoint(eRender, eConsole)
            .map(|d| device_id(&d))
            .unwrap_or_default();
        let Ok(all) = endpoints.enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE) else { return vec![] };
        (0..all.GetCount().unwrap_or(0))
            .filter_map(|i| all.Item(i).ok())
            .map(|device| {
                let id = device_id(&device);
                let (name, headphones) = describe(&device);
                Output { current: id == current, id, name, headphones }
            })
            .collect()
    }
}

/// Makes the next output device the default (speakers ↔ headphones, ...).
/// The device-change callback then shows the new one in the notch.
#[tauri::command]
pub async fn next_output() {
    let outputs = list_outputs().await;
    let Some(i) = outputs.iter().position(|o| o.current) else { return };
    let next = &outputs[(i + 1) % outputs.len()];
    if next.current {
        return;
    }
    unsafe {
        let Ok(policy) = CoCreateInstance::<_, IPolicyConfig>(&POLICY_CONFIG_CLIENT, None, CLSCTX_ALL) else { return };
        let id = HSTRING::from(next.id.as_str());
        for role in [eConsole, eMultimedia, eCommunications] {
            let _ = policy.set_default_endpoint(PCWSTR(id.as_ptr()), role);
        }
    }
}
