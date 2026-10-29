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

/// Reads the current cumulative (InOctets, OutOctets) across active non-loopback interfaces.
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
