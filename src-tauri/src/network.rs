//! Background monitor for real-time network download & upload speeds.
//! Uses Windows IP Helper API (GetIfTable2) to query network interface octets.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use windows::Win32::NetworkManagement::IpHelper::{FreeMibTable, GetIfTable2, MIB_IF_TABLE2};
use windows::Win32::NetworkManagement::Ndis::{IfOperStatusUp, IF_OPER_STATUS};

static RUNNING: AtomicBool = AtomicBool::new(false);
static MONITORING_ACTIVE: AtomicBool = AtomicBool::new(false);

#[derive(serde::Serialize, Clone, Copy, Debug, PartialEq)]
pub struct NetworkSpeed {
    pub down: u64,
    pub up: u64,
}

/// Enables or disables polling when network speed display is toggled in Settings.
pub fn set_monitoring_active(active: bool) {
    MONITORING_ACTIVE.store(active, Ordering::Relaxed);
}

/// Determines whether an interface row is a physical or primary data-link adapter
/// that should be counted, excluding software loopbacks, down interfaces, and NDIS
/// lightweight filter drivers (firewalls, antivirus, packet schedulers).
pub(crate) fn is_valid_interface(row_type: u32, flags_bitfield: u8, oper_status: IF_OPER_STATUS) -> bool {
    // Skip software loopback (IF_TYPE_SOFTWARE_LOOPBACK = 24)
    if row_type == 24 {
        return false;
    }

    // Skip NDIS lightweight filter drivers (e.g. antivirus, WFP, QoS Packet Scheduler).
    // Bit 1 in InterfaceAndOperStatusFlags is FilterInterface.
    let is_filter = (flags_bitfield & 0x02) != 0;
    if is_filter {
        return false;
    }

    // Only count interfaces that are operationally UP
    oper_status == IfOperStatusUp
}

/// Rounds bytes/s to a bucket matching human-readable speed steps (KB or 0.1 MB)
/// so that insignificant sub-kilobyte fluctuations do not wake WebView2 when idle.
pub(crate) fn round_rate(bytes: u64) -> u64 {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * 1024;

    if bytes < KB {
        0
    } else if bytes < MB {
        // Nearest 1 KB
        ((bytes + KB / 2) / KB) * KB
    } else {
        // Nearest 0.1 MB
        let tenths = ((bytes as f64 / MB as f64) * 10.0).round() as u64;
        (tenths * MB) / 10
    }
}

/// Reads the current cumulative (InOctets, OutOctets) across active non-loopback, non-filter interfaces.
fn read_total_octets() -> Option<(u64, u64)> {
    unsafe {
        let mut table: *mut MIB_IF_TABLE2 = std::ptr::null_mut();
        if GetIfTable2(&mut table).is_err() || table.is_null() {
            return None;
        }

        let num_entries = (*table).NumEntries as usize;
        let rows = std::slice::from_raw_parts((*table).Table.as_ptr(), num_entries);

        let mut total_in = 0u64;
        let mut total_out = 0u64;

        for row in rows {
            if is_valid_interface(row.Type, row.InterfaceAndOperStatusFlags._bitfield, row.OperStatus) {
                total_in = total_in.saturating_add(row.InOctets);
                total_out = total_out.saturating_add(row.OutOctets);
            }
        }

        FreeMibTable(table as _);
        Some((total_in, total_out))
    }
}

pub fn start(app: AppHandle) {
    if RUNNING.swap(true, Ordering::SeqCst) {
        return;
    }

    std::thread::spawn(move || {
        let mut last_sample: Option<(u64, u64)> = None;
        let mut last_time = Instant::now();
        let mut last_emitted_rounded: Option<(u64, u64)> = None;

        loop {
            // When both toggles are off in Settings, sleep without querying Windows networking APIs.
            if !MONITORING_ACTIVE.load(Ordering::Relaxed) {
                last_sample = None;
                last_emitted_rounded = None;
                std::thread::sleep(Duration::from_millis(1500));
                continue;
            }

            std::thread::sleep(Duration::from_millis(1000));

            let now = Instant::now();
            let current = read_total_octets();

            if let (Some((prev_in, prev_out)), Some((curr_in, curr_out))) = (last_sample, current) {
                let elapsed = now.duration_since(last_time).as_secs_f64();
                if elapsed > 0.1 {
                    let down_bytes = curr_in.saturating_sub(prev_in);
                    let up_bytes = curr_out.saturating_sub(prev_out);

                    let down_rate = (down_bytes as f64 / elapsed).round() as u64;
                    let up_rate = (up_bytes as f64 / elapsed).round() as u64;

                    let rounded = (round_rate(down_rate), round_rate(up_rate));
                    let notch_open = crate::is_notch_open();

                    // Idle stays idle: only emit if the rounded rate actually changed or if the notch is open
                    if last_emitted_rounded != Some(rounded) || notch_open {
                        let speed = NetworkSpeed {
                            down: down_rate,
                            up: up_rate,
                        };
                        let _ = app.emit("network-speed", speed);
                        last_emitted_rounded = Some(rounded);
                    }
                }
            }

            last_sample = current;
            last_time = now;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::NetworkManagement::Ndis::IfOperStatusDown;

    #[test]
    fn test_is_valid_interface_logic() {
        // Loopback (Type 24) should always be skipped
        assert!(!is_valid_interface(24, 0x00, IfOperStatusUp));

        // NDIS Lightweight Filter Driver (Bit 1 in _bitfield set to 1) should be skipped
        assert!(!is_valid_interface(6, 0x02, IfOperStatusUp));
        assert!(!is_valid_interface(71, 0x02, IfOperStatusUp));
        assert!(!is_valid_interface(6, 0x07, IfOperStatusUp));

        // Inactive / disconnected interface should be skipped
        assert!(!is_valid_interface(6, 0x00, IfOperStatusDown));
        assert!(!is_valid_interface(71, 0x00, IfOperStatusDown));

        // Active Ethernet (Type 6, not a filter, OperStatusUp) should be accepted
        assert!(is_valid_interface(6, 0x05, IfOperStatusUp));
        assert!(is_valid_interface(6, 0x01, IfOperStatusUp));

        // Active Wi-Fi (Type 71, not a filter, OperStatusUp) should be accepted
        assert!(is_valid_interface(71, 0x05, IfOperStatusUp));
        assert!(is_valid_interface(71, 0x01, IfOperStatusUp));
    }

    #[test]
    fn test_round_rate_bucketing() {
        // Sub-kilobyte traffic rounds to 0 (idle)
        assert_eq!(round_rate(0), 0);
        assert_eq!(round_rate(400), 0);
        assert_eq!(round_rate(1023), 0);

        // Kilobyte range rounds to nearest KB
        assert_eq!(round_rate(1024), 1024);
        assert_eq!(round_rate(1500), 1024);
        assert_eq!(round_rate(1600), 2048);

        // Megabyte range rounds to nearest 100 KB (0.1 MB)
        let mb = 1024 * 1024;
        assert_eq!(round_rate(mb), mb);
        assert_eq!(round_rate(mb + 10_000), mb);
        assert_eq!(round_rate(mb + 60_000), (11 * mb) / 10);
    }
}
