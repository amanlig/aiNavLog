use ainavlog_device_interface::{
    monitor::{run_victron_monitor, MonitorConfig, MonitorReport},
    store::ObservationStore,
    DeviceError, DeviceResult,
};
use std::{path::PathBuf, time::Duration};

pub const USAGE: &str = "monitor-victron <device-id> --key-file <path> [--database <path>] [--adapter <index>] [--stale-seconds <n>] [--retry-seconds <n>] [--duration-seconds <n>]\n       observations [database-path] [row-limit]\nMonitor defaults: data/observations.sqlite3, adapter 0, stale 10s, retry 3s. Ctrl-C stops scanning. Keys are read from a private file, never a command-line value.";

#[derive(Debug)]
pub struct MonitorOptions {
    pub config: MonitorConfig,
    pub duration: Option<Duration>,
}
fn config_error(text: impl Into<String>) -> DeviceError {
    DeviceError::Configuration(text.into())
}
pub fn parse_monitor(args: &[String]) -> DeviceResult<MonitorOptions> {
    let source = args
        .first()
        .filter(|s| !s.trim().is_empty() && !s.starts_with("--"))
        .ok_or_else(|| config_error("A device ID is required"))?;
    let mut config = MonitorConfig {
        source_id: source.clone(),
        key_file: PathBuf::new(),
        database: "data/observations.sqlite3".into(),
        adapter_index: 0,
        stale_after: Duration::from_secs(10),
        retry_delay: Duration::from_secs(3),
        restart_after: Duration::from_secs(60),
    };
    let mut duration = None;
    let mut seen = std::collections::HashSet::new();
    let mut rest = args[1..].iter();
    while let Some(flag) = rest.next() {
        if !seen.insert(flag) {
            return Err(config_error("Duplicate option"));
        }
        let value = rest
            .next()
            .filter(|s| !s.is_empty() && !s.starts_with("--"))
            .ok_or_else(|| config_error("Each option requires a value"))?;
        match flag.as_str() {
            "--key-file" => config.key_file = value.into(),
            "--database" => config.database = value.into(),
            "--adapter" => {
                config.adapter_index = value
                    .parse()
                    .map_err(|_| config_error("Adapter index must be a nonnegative integer"))?
            }
            "--stale-seconds" | "--retry-seconds" | "--duration-seconds" => {
                let seconds: u64 = value
                    .parse()
                    .map_err(|_| config_error("Timeout must be an integer"))?;
                if !(1..=86400).contains(&seconds) {
                    return Err(config_error("Timeout must be between 1 and 86400 seconds"));
                }
                let time = Duration::from_secs(seconds);
                match flag.as_str() {
                    "--stale-seconds" => config.stale_after = time,
                    "--retry-seconds" => config.retry_delay = time,
                    _ => duration = Some(time),
                }
            }
            _ => return Err(config_error("Unknown monitor option")),
        }
    }
    if config.key_file.as_os_str().is_empty() {
        return Err(config_error("--key-file is required"));
    }
    config.restart_after = config
        .stale_after
        .saturating_mul(3)
        .max(Duration::from_secs(60));
    config.validate()?;
    Ok(MonitorOptions { config, duration })
}
pub async fn run_monitor(options: MonitorOptions) -> DeviceResult<()> {
    println!(
        "Monitoring {} → {}",
        options.config.source_id,
        options.config.database.display()
    );
    let shutdown = async {
        let deadline = async {
            match options.duration {
                Some(d) => tokio::time::sleep(d).await,
                None => std::future::pending::<()>().await,
            }
        };
        tokio::select! {
            signal=tokio::signal::ctrl_c()=>signal.map_err(|e|DeviceError::Transport(e.to_string())),
            _=deadline=>Ok(()),
        }
    };
    run_victron_monitor(options.config, shutdown, print_report).await
}
fn print_report(report: &MonitorReport) {
    println!("{:?}: {}", report.state, report.detail);
    if let Some(id) = report.sample_id {
        for observation in &report.observations {
            let value = observation
                .value
                .map(|v| v.to_string())
                .unwrap_or_else(|| "unavailable".into());
            println!(
                "sample={id}\t{}\t{}={} {}\t{:?}",
                observation.source_id,
                observation.quantity,
                value,
                observation.unit,
                observation.quality
            );
        }
    }
}
pub fn show_history(args: &[String]) -> DeviceResult<()> {
    if args.len() > 2 {
        return Err(config_error(
            "Expected observations [database-path] [row-limit]",
        ));
    }
    let path = args
        .first()
        .map(PathBuf::from)
        .unwrap_or_else(|| "data/observations.sqlite3".into());
    let limit = args
        .get(1)
        .map(|s| s.parse::<usize>())
        .transpose()
        .map_err(|_| config_error("Row limit must be an integer"))?
        .unwrap_or(20);
    let rows = ObservationStore::read_recent(&path, limit)?;
    println!("Historical rows (quality at ingestion; receipt time is Unix milliseconds)");
    println!("SAMPLE\tSOURCE\tRECEIVED_MS\tQUANTITY\tVALUE\tUNIT\tQUALITY");
    for row in rows {
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}",
            row.sample_id,
            row.source_id,
            row.received_at_ms,
            row.quantity,
            row.value
                .map(|v| v.to_string())
                .unwrap_or_else(|| "unavailable".into()),
            row.unit,
            row.quality
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| s.to_string()).collect()
    }
    #[test]
    fn monitor_options_require_key_path_and_bounded_values() {
        let result =
            parse_monitor(&args(&["D4:B3:CB:26:5E:75", "--key-file", "private.key"])).unwrap();
        assert_eq!(result.config.stale_after, Duration::from_secs(10));
        assert_eq!(result.config.source_id, "D4:B3:CB:26:5E:75");
        for values in [
            vec![],
            vec!["id"],
            vec!["id", "--key", "secret"],
            vec!["id", "--key-file"],
            vec!["id", "--key-file", "k", "--stale-seconds", "0"],
            vec!["id", "--key-file", "k", "--adapter", "-1"],
            vec!["id", "--key-file", "k", "--key-file", "k"],
        ] {
            assert!(parse_monitor(&args(&values)).is_err());
        }
    }
}
