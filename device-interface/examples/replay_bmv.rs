//! Replay encrypted packets captured from the existing Windows BMV simulator.
//! Bypasses the radio only; uses production decoding, normalization and SQLite.
use ainavlog_device_interface::{
    monitor::VictronMonitor,
    store::ObservationStore,
    vendor::victron::{VictronAdvertisement, VICTRON_COMPANY_ID},
};
use std::{
    path::Path,
    time::{Duration, Instant, SystemTime},
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err("Usage: cargo run --example replay_bmv -- <database> <packet-log>".into());
    }
    let packets: Vec<Vec<u8>> = std::fs::read_to_string(&args[1])?
        .lines()
        .filter_map(|line| {
            line.split_once("payload=")
                .map(|(_, payload)| payload.trim())
        })
        .map(|hex| {
            hex.split('-')
                .map(|byte| u8::from_str_radix(byte, 16))
                .collect()
        })
        .collect::<Result<_, _>>()?;
    if packets.is_empty() {
        return Err("No simulator packets found".into());
    }
    let mut monitor = VictronMonitor::new(
        "bmv-simulator".into(),
        "000102030405060708090a0b0c0d0e0f",
        Duration::from_secs(5),
        ObservationStore::open(Path::new(&args[0]))?,
        Instant::now(),
    )?;
    println!("SIMULATED packet replay: production Rust decoder → SQLite. Radio bypassed.");
    for payload in packets {
        let report = monitor.ingest(
            VictronAdvertisement {
                source_id: "bmv-simulator".into(),
                received_at: SystemTime::now(),
                company_id: VICTRON_COMPANY_ID,
                payload,
            },
            Instant::now(),
        )?;
        if let Some(report) = report {
            println!("Stored sample {:?}", report.sample_id);
        }
        std::thread::sleep(Duration::from_secs(1));
    }
    Ok(())
}
