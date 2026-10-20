mod monitor_cli;
mod ui_server;

use ainavlog_device_interface::{BluetoothAdapter, DeviceError};
use std::{env, process::ExitCode, time::Duration};

const USAGE: &str = "Usage: ainavlog-device-interface discover [scan-seconds] [adapter-index]\n       ainavlog-device-interface connect <device-id> [scan-seconds] [adapter-index]\nBLE only. Connect scans first, then holds the connection until Ctrl-C.\nDiscovery may include cached devices; listed devices are not guaranteed connectable.";

#[derive(Debug, PartialEq)]
struct Options {
    device: Option<String>,
    seconds: u64,
    adapter: usize,
}

fn parse(args: &[String]) -> Result<Option<Options>, String> {
    let Some(command) = args.first().map(String::as_str) else {
        return Ok(None);
    };
    if matches!(command, "--help" | "-h") {
        return Ok(None);
    }
    let (device, rest) = match command {
        "discover" => (None, &args[1..]),
        "connect" if args.get(1).is_some_and(|id| !id.is_empty()) => {
            (Some(args[1].clone()), &args[2..])
        }
        _ => return Err("Expected discover or connect <device-id>".into()),
    };
    if rest.len() > 2 {
        return Err("Too many arguments".into());
    }
    let seconds = rest
        .first()
        .map(|s| s.parse::<u64>())
        .transpose()
        .map_err(|_| "Scan seconds must be an integer")?
        .unwrap_or(5);
    if !(1..=300).contains(&seconds) {
        return Err("Scan seconds must be between 1 and 300".into());
    }
    let adapter = rest
        .get(1)
        .map(|s| s.parse::<usize>())
        .transpose()
        .map_err(|_| "Adapter index must be a nonnegative integer")?
        .unwrap_or(0);
    Ok(Some(Options {
        device,
        seconds,
        adapter,
    }))
}

async fn run(options: Options) -> Result<(), DeviceError> {
    let mut bluetooth = BluetoothAdapter::new(options.adapter).await?;
    println!("Scanning BLE devices for {} seconds…", options.seconds);
    let devices = bluetooth
        .discover(Duration::from_secs(options.seconds))
        .await?;
    println!("ID\tNAME");
    for device in &devices {
        println!(
            "{}\t{}",
            device.id,
            device.name.as_deref().unwrap_or("(unnamed)")
        );
    }
    if devices.is_empty() {
        println!("No BLE devices found. Check advertising, range, and permissions.");
    }
    if let Some(id) = options.device {
        if !devices
            .iter()
            .any(|device| device.id.eq_ignore_ascii_case(&id))
        {
            #[cfg(target_os = "windows")]
            println!("{id} was not seen in this scan; attempting the explicitly supplied Windows Bluetooth address.");
        }
        println!("Connecting to {id} (timeout: 20 seconds)…");
        if let Err(error) = bluetooth.connect_by_id(&id).await {
            if let Err(cleanup) = bluetooth.disconnect().await {
                eprintln!("Connection cleanup failed: {cleanup}");
            }
            return Err(error);
        }
        println!("Connected to {id}. Press Ctrl-C to disconnect.");
        let signal = tokio::signal::ctrl_c().await;
        bluetooth.disconnect().await?;
        signal.map_err(|error| DeviceError::Transport(error.to_string()))?;
        println!("Disconnected.");
    }
    Ok(())
}

#[tokio::main]
async fn main() -> ExitCode {
    let args: Vec<_> = env::args().skip(1).collect();
    if args.first().is_some_and(|s| s == "serve-ui") {
        return match ui_server::run(&args[1..]) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("{error}\n{}", ui_server::USAGE);
                ExitCode::FAILURE
            }
        };
    }
    if args
        .first()
        .is_some_and(|s| s == "monitor-victron" || s == "observations")
    {
        let result = if args[0] == "monitor-victron" {
            match monitor_cli::parse_monitor(&args[1..]) {
                Ok(options) => monitor_cli::run_monitor(options).await,
                Err(error) => Err(error),
            }
        } else {
            monitor_cli::show_history(&args[1..])
        };
        return match result {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("{error}\n{}", monitor_cli::USAGE);
                ExitCode::FAILURE
            }
        };
    }
    match parse(&args) {
        Ok(None) => {
            println!(
                "{USAGE}\n       {}\n       {}",
                monitor_cli::USAGE,
                ui_server::USAGE
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}\n{USAGE}");
            ExitCode::FAILURE
        }
        Ok(Some(options)) => match run(options).await {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("Bluetooth error: {error}");
                ExitCode::FAILURE
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|v| v.to_string()).collect()
    }
    #[test]
    fn validates_commands_before_accessing_hardware() {
        assert_eq!(parse(&[]).unwrap(), None);
        assert_eq!(
            parse(&args(&["discover"])).unwrap(),
            Some(Options {
                device: None,
                seconds: 5,
                adapter: 0
            })
        );
        assert_eq!(
            parse(&args(&["connect", "opaque-id", "10", "1"])).unwrap(),
            Some(Options {
                device: Some("opaque-id".into()),
                seconds: 10,
                adapter: 1
            })
        );
        for input in [
            vec!["connect"],
            vec!["discover", "0"],
            vec!["discover", "301"],
            vec!["discover", "abc"],
            vec!["discover", "5", "-1"],
            vec!["unknown"],
        ] {
            assert!(parse(&args(&input)).is_err());
        }
    }
}
