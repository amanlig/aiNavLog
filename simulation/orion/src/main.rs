use ainavlog_device_interface::{
    monitor::{MonitorReport, MonitorState, VictronMonitor},
    store::ObservationStore,
    DeviceError,
};
use ainavlog_orion_simulation::{advertisement, Model, Readings, KEY_HEX, SOURCE_ID};
use std::{
    path::PathBuf,
    time::{Duration, Instant, SystemTime},
};

fn show(report: Option<MonitorReport>) {
    if let Some(r) = report {
        println!(
            "SIMULATION {:?}: {} sample={:?}",
            r.state, r.detail, r.sample_id
        );
        for o in r.observations {
            println!("  {} = {:?} {}", o.quantity, o.value, o.unit);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|s| s == "--help" || s == "-h") {
        println!("Orion synthetic telemetry simulator\n--model tr|xs (default tr)\n--scenario normal|faults (default faults)\n--steps 1..65535 (default 24)\n--interval-ms 1..60000 (default 1000)\n--database PATH (must not exist; default simulation/orion/data/run-<timestamp>.sqlite3)\nNo Bluetooth or real key required. Ctrl-C stops.");
        return Ok(());
    }
    let mut model = Model::Tr;
    let mut faults = true;
    let mut steps = 24u16;
    let mut interval_ms = 1000u64;
    let stamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_nanos();
    let mut database =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("data/run-{stamp}.sqlite3"));
    let mut seen = std::collections::HashSet::new();
    for pair in args.chunks(2) {
        if pair.len() != 2 || !seen.insert(&pair[0]) {
            return Err("Options require unique names and values".into());
        }
        match pair[0].as_str() {
            "--model" => {
                model = match pair[1].as_str() {
                    "tr" => Model::Tr,
                    "xs" => Model::Xs,
                    _ => return Err("Model must be tr or xs".into()),
                }
            }
            "--scenario" => {
                faults = match pair[1].as_str() {
                    "normal" => false,
                    "faults" => true,
                    _ => return Err("Scenario must be normal or faults".into()),
                }
            }
            "--steps" => {
                steps = pair[1].parse()?;
                if steps == 0 {
                    return Err("Steps must be positive".into());
                }
            }
            "--interval-ms" => {
                interval_ms = pair[1].parse()?;
                if !(1..=60000).contains(&interval_ms) {
                    return Err("Interval must be 1..60000 ms".into());
                }
            }
            "--database" => database = PathBuf::from(&pair[1]),
            _ => return Err(format!("Unknown option: {}", pair[0]).into()),
        }
    }
    if let Some(parent) = database.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    // Atomically reserve a fresh file: never append synthetic observations to live history.
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&database)?;
    let interval = Duration::from_millis(interval_ms);
    let mut monitor = VictronMonitor::new(
        SOURCE_ID.into(),
        KEY_HEX,
        interval * 3,
        ObservationStore::open(&database)?,
        Instant::now(),
    )?;
    println!(
        "SIMULATION ONLY — {model:?}; database: {}",
        database.display()
    );
    show(monitor.transition(
        MonitorState::Waiting,
        "Synthetic run started",
        SystemTime::now(),
    )?);
    let shutdown = tokio::signal::ctrl_c();
    tokio::pin!(shutdown);
    let mut previous: Option<ainavlog_device_interface::vendor::victron::VictronAdvertisement> =
        None;
    let outcome: Result<(), DeviceError> = async {
        for step in 0..steps {
            tokio::select! {
                result = &mut shutdown => { result.map_err(|e| DeviceError::Transport(e.to_string()))?; break; },
                _ = tokio::time::sleep(interval) => {}
            }
            let now = Instant::now();
            let wall = SystemTime::now();
            show(monitor.tick(now, wall)?);
            if faults && (8..=12).contains(&step) { println!("SIMULATION signal absent"); continue; }
            if faults && step == 16 { show(monitor.replace_key(KEY_HEX, now, wall)?); }
            let mut r = Readings::sample(step);
            if faults && step == 5 { r.input_centivolts = None; r.output_centivolts = None; r.input_deciamps = None; r.output_deciamps = None; }
            if faults && step == 18 { r.state = 0; r.error = 1; r.off_reason = 1; }
            let mut packet = advertisement(model, step, &r, wall);
            if faults && step == 3 { if let Some(p) = &previous { packet = p.clone(); } }
            if faults && step == 6 { packet.payload.truncate(9); }
            if faults && step == 14 { packet.payload[7] = 255; }
            previous = Some(packet.clone());
            match monitor.ingest(packet, now) {
                Ok(report) => show(report),
                Err(DeviceError::Decode(error)) => println!("SIMULATION rejected packet: {error}"),
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }.await;
    let stopped = monitor.transition(
        MonitorState::Stopped,
        "Synthetic run stopped",
        SystemTime::now(),
    );
    show(stopped?);
    outcome?;
    println!("Synthetic history saved to {}", database.display());
    Ok(())
}
