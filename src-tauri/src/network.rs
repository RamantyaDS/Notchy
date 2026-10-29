//! Background monitor for real-time network download & upload speeds.
//! Uses Windows IP Helper API (GetIfTable2) to query network interface octets.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use windows::Win32::NetworkManagement::IpHelper::{FreeMibTable, GetIfTable2, MIB_IF_TABLE2};

static RUNNING: AtomicBool = AtomicBool::new(false);

#[derive(serde::Serialize, Clone, Copy, Debug, PartialEq)]
pub struct NetworkSpeed {
    pub down: u64,
    pub up: u64,
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
            // Skip software loopback (IF_TYPE_SOFTWARE_LOOPBACK = 24)
            if row.Type == 24 {
                continue;
            }

            // Skip NDIS lightweight filter drivers (e.g. antivirus, WFP, QoS Packet Scheduler).
            // Bit 1 in InterfaceAndOperStatusFlags is FilterInterface.
            let is_filter = (row.InterfaceAndOperStatusFlags._bitfield & 0x02) != 0;
            if is_filter {
                continue;
            }

            // Only count interfaces that are operationally UP
            let is_up = row.OperStatus == windows::Win32::NetworkManagement::Ndis::IfOperStatusUp;
            if !is_up {
                continue;
            }

            total_in = total_in.saturating_add(row.InOctets);
            total_out = total_out.saturating_add(row.OutOctets);
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
        let mut last_sample = read_total_octets();
        let mut last_time = Instant::now();

        loop {
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

                    let speed = NetworkSpeed {
                        down: down_rate,
                        up: up_rate,
                    };

                    let _ = app.emit("network-speed", speed);
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
    use windows::Win32::NetworkManagement::Ndis::IfOperStatusUp;

    #[test]
    fn test_filter_real_interfaces() {
        unsafe {
            let mut table: *mut MIB_IF_TABLE2 = std::ptr::null_mut();
            if GetIfTable2(&mut table).is_err() || table.is_null() {
                return;
            }
            let num = (*table).NumEntries as usize;
            let rows = std::slice::from_raw_parts((*table).Table.as_ptr(), num);

            let mut filter_count = 0;
            let mut non_filter_active = Vec::new();

            for row in rows {
                let is_filter = (row.InterfaceAndOperStatusFlags._bitfield & 0x02) != 0;
                let is_up = row.OperStatus == IfOperStatusUp;
                let is_loopback = row.Type == 24;

                if is_filter {
                    filter_count += 1;
                } else if !is_loopback && is_up {
                    let alias = String::from_utf16_lossy(&row.Alias);
                    let alias = alias.trim_matches('\0').to_string();
                    non_filter_active.push((row.InterfaceIndex, alias, row.InOctets, row.OutOctets));
                }
            }
            FreeMibTable(table as _);

            // On Windows systems with antivirus/VPN/firewall, filter drivers should be identified and skipped
            assert!(filter_count > 0, "Expected filter interfaces to be present and detected");
            // There should be active non-filter interfaces detected
            assert!(!non_filter_active.is_empty(), "Expected at least one active non-filter interface");
        }
    }
}
