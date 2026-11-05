//! Background watcher for Bluetooth device battery and connection status.
//! Watches Windows AEP (Association Endpoint) Bluetooth devices for connect events
//! and low battery thresholds (< 20%) without CPU polling.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use serde_json::json;
use tauri::AppHandle;
use windows::core::{HSTRING, Interface};
use windows::Devices::Enumeration::{DeviceInformation, DeviceInformationUpdate};
use windows::Foundation::{IPropertyValue, TypedEventHandler};
use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};

use crate::peek;

#[derive(Default)]
struct DeviceState {
    name: String,
    battery: Option<u32>,
    connected: bool,
    category: String,
}

static DEVICES: std::sync::OnceLock<Arc<Mutex<HashMap<String, DeviceState>>>> = std::sync::OnceLock::new();

pub fn start(app: AppHandle) {
    let devices = DEVICES.get_or_init(|| Arc::new(Mutex::new(HashMap::new()))).clone();

    std::thread::spawn(move || {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        }

        // Bluetooth AEP protocol GUID query
        let aqs = HSTRING::from(r#"System.Devices.Aep.ProtocolId:="{E0CBF06C-CD8B-4647-BB8A-263B43F0F974}""#);

        let props: windows_collections::IIterable<HSTRING> = vec![
            HSTRING::from("System.Devices.Aep.IsConnected"),
            HSTRING::from("System.Devices.BatteryLifePercent"),
            HSTRING::from("System.Devices.Aep.Bluetooth.Cod.Major"),
        ].into();

        let Ok(watcher) = DeviceInformation::CreateWatcherAqsFilterAndAdditionalProperties(&aqs, &props) else {
            return;
        };

        let enumerated = Arc::new(AtomicBool::new(false));
        let enum_added = enumerated.clone();
        let enum_done = enumerated.clone();

        let _ = watcher.EnumerationCompleted(&TypedEventHandler::new(move |_, _| {
            enum_done.store(true, Ordering::SeqCst);
            Ok(())
        }));

        let app_added = app.clone();
        let dev_added = devices.clone();
        let _ = watcher.Added(&TypedEventHandler::new(move |_, info: windows_core::Ref<'_, DeviceInformation>| {
            let Some(info) = info.as_ref() else { return Ok(()) };
            let Ok(id) = info.Id() else { return Ok(()) };
            let name = info.Name().map(|h| h.to_string()).unwrap_or_else(|_| "Perangkat Bluetooth".into());

            let mut connected = false;
            let mut battery = None;
            let mut category = "bluetooth".to_string();

            if let Ok(props) = info.Properties() {
                if let Ok(c) = props.Lookup(&HSTRING::from("System.Devices.Aep.IsConnected")) {
                    if let Ok(b) = c.cast::<IPropertyValue>().and_then(|pv| pv.GetBoolean()) {
                        connected = b;
                    }
                }
                if let Ok(b) = props.Lookup(&HSTRING::from("System.Devices.BatteryLifePercent")) {
                    if let Ok(pv) = b.cast::<IPropertyValue>() {
                        if let Ok(pct) = pv.GetUInt32() {
                            battery = Some(pct);
                        } else if let Ok(pct) = pv.GetUInt8() {
                            battery = Some(pct as u32);
                        }
                    }
                }
                if let Ok(cat) = props.Lookup(&HSTRING::from("System.Devices.Aep.Bluetooth.Cod.Major")) {
                    if let Ok(m) = cat.cast::<IPropertyValue>().and_then(|pv| pv.GetUInt32()) {
                        category = match m {
                            4 => "audio".to_string(), // Audio / Video (TWS, Headphone)
                            5 => "hid".to_string(),   // Peripheral (Mouse, Keyboard)
                            _ => "bluetooth".to_string(),
                        };
                    }
                }
            }

            let mut state_map = dev_added.lock().unwrap();
            let entry = state_map.entry(id.to_string()).or_default();
            entry.name = name.clone();
            entry.connected = connected;
            entry.battery = battery;
            entry.category = category.clone();

            // Ignore Added events during initial boot enumeration to avoid burst of peeks
            if connected && enum_added.load(Ordering::SeqCst) {
                let settings = crate::settings::get_settings();
                if settings.bluetooth_peek {
                    peek(
                        &app_added,
                        "bluetooth",
                        json!({
                            "name": name,
                            "battery": battery,
                            "category": category,
                            "low": battery.is_some_and(|b| b <= settings.bluetooth_low_threshold),
                            "action": "connected"
                        }),
                    );
                }
            }

            Ok(())
        }));

        let app_updated = app.clone();
        let dev_updated = devices.clone();
        let _ = watcher.Updated(&TypedEventHandler::new(move |_, update: windows_core::Ref<'_, DeviceInformationUpdate>| {
            let Some(update) = update.as_ref() else { return Ok(()) };
            let Ok(id) = update.Id() else { return Ok(()) };
            let id_str = id.to_string();

            let mut new_conn = None;
            let mut new_bat = None;

            if let Ok(props) = update.Properties() {
                if let Ok(c) = props.Lookup(&HSTRING::from("System.Devices.Aep.IsConnected")) {
                    if let Ok(b) = c.cast::<IPropertyValue>().and_then(|pv| pv.GetBoolean()) {
                        new_conn = Some(b);
                    }
                }
                if let Ok(b) = props.Lookup(&HSTRING::from("System.Devices.BatteryLifePercent")) {
                    if let Ok(pv) = b.cast::<IPropertyValue>() {
                        if let Ok(pct) = pv.GetUInt32() {
                            new_bat = Some(pct);
                        } else if let Ok(pct) = pv.GetUInt8() {
                            new_bat = Some(pct as u32);
                        }
                    }
                }
            }

            let mut state_map = dev_updated.lock().unwrap();
            let Some(entry) = state_map.get_mut(&id_str) else { return Ok(()) };

            let was_conn = entry.connected;
            let prev_bat = entry.battery;

            if let Some(c) = new_conn {
                entry.connected = c;
            }
            if let Some(b) = new_bat {
                entry.battery = Some(b);
            }

            let settings = crate::settings::get_settings();
            if !settings.bluetooth_peek {
                return Ok(());
            }

            let just_connected = !was_conn && entry.connected;
            let battery_dropped_low = entry.battery.is_some_and(|b| {
                b <= settings.bluetooth_low_threshold && prev_bat.is_none_or(|p| p > settings.bluetooth_low_threshold)
            });

            if just_connected || battery_dropped_low {
                peek(
                    &app_updated,
                    "bluetooth",
                    json!({
                        "name": entry.name,
                        "battery": entry.battery,
                        "category": entry.category,
                        "low": entry.battery.is_some_and(|b| b <= settings.bluetooth_low_threshold),
                        "action": if just_connected { "connected" } else { "low_battery" }
                    }),
                );
            }

            Ok(())
        }));

        let _ = watcher.Start();

        // Keep thread alive for background events
        loop {
            std::thread::sleep(std::time::Duration::from_secs(3600));
        }
    });
}
