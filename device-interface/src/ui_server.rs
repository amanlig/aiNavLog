//! Read-only local UI bridge. Instrument reception remains Bluetooth-only.
use ainavlog_device_interface::store::{ObservationStore, StoredObservation};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::Path,
    time::Duration,
};

pub const USAGE: &str = "serve-ui <database-path> <bmv-device-id> [port] [ui-origin] [--simulated]\nDefaults: 8787, http://localhost:8081. Listens only on 127.0.0.1.";

fn sample_json(rows: &[StoredObservation], simulated: bool) -> String {
    let Some(first) = rows.first() else {
        return "null".into();
    };
    let value = |quantity: &str, unit: &str| {
        rows.iter()
            .find(|o| o.quantity == quantity && o.unit == unit && o.quality == "Good")
            .and_then(|o| o.value)
            .filter(|v| v.is_finite())
            .map_or_else(|| "null".into(), |v| v.to_string())
    };
    let source = if simulated { "simulated" } else { "live" };
    format!("{{\"source\":\"{source}\",\"receivedAt\":{},\"houseVoltage\":{},\"houseCurrent\":{},\"houseCharge\":{},\"starterVoltage\":{}}}",
        first.received_at_ms, value("battery_voltage", "V"), value("battery_current", "A"),
        value("state_of_charge", "%"), value("auxiliary_voltage", "V"))
}

fn respond(stream: &mut TcpStream, status: &str, origin: &str, body: &str) -> std::io::Result<()> {
    write!(stream, "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: {origin}\r\nVary: Origin\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n{body}", body.len())
}
fn handle(
    mut stream: TcpStream,
    path: &Path,
    source: &str,
    origin: &str,
    simulated: bool,
) -> std::io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    let mut request = Vec::new();
    let mut chunk = [0; 1024];
    while !request.windows(4).any(|w| w == b"\r\n\r\n") {
        let count = stream.read(&mut chunk)?;
        if count == 0 {
            return Ok(());
        }
        request.extend_from_slice(&chunk[..count]);
        if request.len() > 8192 {
            return respond(
                &mut stream,
                "431 Request Header Fields Too Large",
                origin,
                "{}",
            );
        }
    }
    let request = String::from_utf8_lossy(&request);
    if request.lines().next() != Some("GET /electrical HTTP/1.1") {
        return respond(&mut stream, "404 Not Found", origin, "{}");
    }
    match ObservationStore::read_latest_bmv(path, source) {
        Ok(rows) => respond(
            &mut stream,
            "200 OK",
            origin,
            &sample_json(&rows, simulated),
        ),
        Err(_) => respond(
            &mut stream,
            "503 Service Unavailable",
            origin,
            "{\"error\":\"Observation store unavailable\"}",
        ),
    }
}
pub fn run(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if !(2..=5).contains(&args.len()) || args[1].trim().is_empty() {
        return Err("Expected database path and BMV device ID".into());
    }
    let port = args
        .get(2)
        .map(|s| s.parse::<u16>())
        .transpose()?
        .unwrap_or(8787);
    if port == 0 {
        return Err("Port must be 1..=65535".into());
    }
    let origin = args
        .get(3)
        .map(String::as_str)
        .unwrap_or("http://localhost:8081");
    if !(origin.starts_with("http://") || origin.starts_with("https://"))
        || origin.bytes().any(|b| b.is_ascii_control() || b == b' ')
    {
        return Err("Invalid UI origin".into());
    }
    let simulated = args.get(4).is_some_and(|s| s == "--simulated");
    if args.get(4).is_some() && !simulated {
        return Err("Expected --simulated".into());
    }
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    println!(
        "UI bridge: http://127.0.0.1:{port}/electrical (device {})",
        args[1]
    );
    for stream in listener.incoming() {
        if let Err(error) = handle(stream?, Path::new(&args[0]), &args[1], origin, simulated) {
            eprintln!("UI request failed: {error}");
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_and_bad_quality_are_null_and_timestamp_is_preserved() {
        assert_eq!(sample_json(&[], false), "null");
        let row = StoredObservation {
            sample_id: 1,
            source_id: "bmv".into(),
            received_at_ms: 1234,
            quantity: "battery_voltage".into(),
            unit: "V".into(),
            value: Some(12.8),
            quality: "Good".into(),
        };
        assert!(sample_json(std::slice::from_ref(&row), false).contains("\"houseVoltage\":12.8"));
        let bad = StoredObservation {
            quality: "Invalid".into(),
            ..row
        };
        assert_eq!(sample_json(&[bad], false), "{\"source\":\"live\",\"receivedAt\":1234,\"houseVoltage\":null,\"houseCurrent\":null,\"houseCharge\":null,\"starterVoltage\":null}");
    }
}
