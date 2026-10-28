//! Desktop application composition: receiver supervision + capture + local storage.
use ainavlog_data_capture::CaptureService;
use ainavlog_data_model::ObservationSink;
use ainavlog_device_interface::{
    monitor::{
        AdvertisementSource, BluetoothAdvertisementSource, MonitorReport, MonitorState,
        VictronMonitor,
    },
    DeviceError, DeviceResult,
};
use ainavlog_local_storage::ObservationStore;
use std::{
    future::Future,
    io::Read,
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime},
};
use zeroize::Zeroizing;
#[derive(Debug, Clone)]
pub struct MonitorConfig {
    pub source_id: String,
    pub key_file: PathBuf,
    pub database: PathBuf,
    pub adapter_index: usize,
    pub stale_after: Duration,
    pub retry_delay: Duration,
    /// Restart a silent scanner periodically to recover backends that stop
    /// emitting events without a state-change notification.
    pub restart_after: Duration,
}
impl MonitorConfig {
    pub fn validate(&self) -> DeviceResult<()> {
        if self.source_id.trim().is_empty()
            || self.stale_after.is_zero()
            || self.retry_delay.is_zero()
            || self.restart_after < self.stale_after
        {
            return Err(DeviceError::Configuration("source ID and positive timeouts required; restart timeout must be >= stale timeout".into()));
        }
        Ok(())
    }
}

/// Keys are provided through a private local file, never command-line values.
/// On Unix require owner-only permissions; Windows ACLs are managed by the user.
#[cfg(feature = "desktop")]
pub fn read_key_file(path: &Path) -> DeviceResult<Zeroizing<String>> {
    let mut file = std::fs::File::open(path)
        .map_err(|e| DeviceError::Configuration(format!("Cannot open key file: {e}")))?;
    let metadata = file
        .metadata()
        .map_err(|_| DeviceError::Configuration("Cannot inspect key file".into()))?;
    if !metadata.is_file() {
        return Err(DeviceError::Configuration(
            "Key path must be a regular file".into(),
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err(DeviceError::Configuration(
                "Key file must have owner-only permissions (chmod 600)".into(),
            ));
        }
    }
    let mut text = Zeroizing::new(String::new());
    (&mut file)
        .take(129)
        .read_to_string(&mut text)
        .map_err(|_| DeviceError::Configuration("Cannot read key file".into()))?;
    let key = text.trim();
    if key.len() != 32 || !key.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(DeviceError::Configuration(
            "Key file must contain exactly 32 hex characters, optionally followed by a newline"
                .into(),
        ));
    }
    Ok(Zeroizing::new(key.to_ascii_lowercase()))
}

/// Run until shutdown, retrying scan failures. Storage failures are fatal: we do
/// not display a sample as persisted when its transaction failed. The caller
/// must await this future for scan cleanup; forcibly aborting it is not graceful.
#[cfg(feature = "desktop")]
pub async fn run_victron_monitor(
    config: MonitorConfig,
    shutdown: impl Future<Output = DeviceResult<()>>,
    report: impl FnMut(&MonitorReport),
) -> DeviceResult<()> {
    let source = BluetoothAdvertisementSource::new(config.adapter_index);
    run_with_source(config, source, shutdown, report).await
}

#[cfg(feature = "desktop")]
pub async fn run_with_source<S: AdvertisementSource>(
    config: MonitorConfig,
    mut source: S,
    shutdown: impl Future<Output = DeviceResult<()>>,
    mut report: impl FnMut(&MonitorReport),
) -> DeviceResult<()> {
    config.validate()?;
    let mut key = read_key_file(&config.key_file)?;
    let capture = CaptureService::new(ObservationStore::open(&config.database)?);
    let mut monitor = VictronMonitor::new(
        config.source_id.clone(),
        &key,
        config.stale_after,
        capture,
        monotonic_now(),
    )?;
    monitor.sink().record_event(
        &config.source_id,
        SystemTime::now(),
        "Waiting",
        "Monitoring started",
    )?;
    report(&MonitorReport {
        state: MonitorState::Waiting,
        detail: "Waiting for Victron advertisements".into(),
        sample_id: None,
        observations: Vec::new(),
    });
    tokio::pin!(shutdown);
    let mut ticker = tokio::time::interval(Duration::from_secs(1));
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut running = false;
    let mut retry_at = tokio::time::Instant::now();
    let mut failures = 0u32;
    let mut last_packet = monotonic_now();
    let mut last_warning = String::new();
    // Keep all early errors inside this async block so explicit cleanup always runs.
    let outcome:DeviceResult<()>=async {
        loop {
            tokio::select! {
                biased;
                result=&mut shutdown=>break result,
                _=tokio::time::sleep_until(retry_at), if !running=> {
                    let started=tokio::select! {
                        biased;
                        result=&mut shutdown=>break result,
                        result=tokio::time::timeout(Duration::from_secs(25),source.start())=>
                            result.unwrap_or_else(|_|Err(DeviceError::Transport("Scan initialization timed out".into()))),
                    };
                    match started {
                        Ok(())=> {
                            running=true; last_packet=monotonic_now();
                            if monitor.state()!=MonitorState::KeyRequired {
                                if let Some(update)=monitor.transition(MonitorState::Waiting,"Scanning; waiting for selected device",SystemTime::now())? {report(&update);}
                            }
                        }
                        Err(error)=> {
                            let cleanup=tokio::time::timeout(Duration::from_secs(21),source.stop()).await;
                            if !matches!(cleanup,Ok(Ok(()))) {return Err(DeviceError::Transport("Failed to clean up scan after startup failure".into()));}
                            failures=failures.saturating_add(1);
                            retry_at=tokio::time::Instant::now()+config.retry_delay.saturating_mul(1<<failures.saturating_sub(1).min(4)).min(Duration::from_secs(60));
                            let detail=format!("Scan failed; retrying: {error}");
                            if monitor.state()!=MonitorState::KeyRequired {
                                if let Some(update)=monitor.transition(MonitorState::Recovering,&detail,SystemTime::now())? {report(&update);}
                            }
                            warn_once(&mut monitor,&mut last_warning,&detail,&mut report)?;
                        }
                    }
                }
                _=ticker.tick()=> {
                    match read_key_file(&config.key_file) {
                        Ok(new_key) if *new_key!=*key=> {
                            if let Some(update)=monitor.replace_key(&new_key,monotonic_now(),SystemTime::now())? {report(&update);}
                            key=new_key;last_warning.clear();
                        }
                        Ok(_)=>{},
                        Err(error)=>warn_once(&mut monitor,&mut last_warning,&format!("Key reload failed; existing in-memory key unchanged: {error}"),&mut report)?,
                    }
                    if let Some(update)=monitor.tick(monotonic_now(),SystemTime::now())? {report(&update);}
                    if running && monotonic_now().saturating_duration_since(last_packet)>=config.restart_after {
                        tokio::time::timeout(Duration::from_secs(21),source.stop()).await
                            .map_err(|_|DeviceError::Transport("Scan cleanup timed out".into()))??;
                        running=false;retry_at=tokio::time::Instant::now()+config.retry_delay;
                        if monitor.state()!=MonitorState::KeyRequired {
                            if let Some(update)=monitor.transition(MonitorState::Recovering,"Selected device silent; restarting scan",SystemTime::now())? {report(&update);}
                        }
                    }
                }
                result=source.next(), if running=> {
                    match result {
                        Ok(input)=> {
                            if input.source_id.eq_ignore_ascii_case(&config.source_id) {last_packet=monotonic_now();}
                            match monitor.ingest(input,monotonic_now()) {
                                Ok(Some(update))=>{failures=0;last_warning.clear();report(&update);},
                                Ok(None)=>{},
                                Err(DeviceError::Decode(error))=>warn_once(&mut monitor,&mut last_warning,&format!("Packet rejected: {error}"),&mut report)?,
                                Err(error)=>return Err(error),
                            }
                        }
                        Err(error)=> {
                            tokio::time::timeout(Duration::from_secs(21),source.stop()).await
                                .map_err(|_|DeviceError::Transport("Scan cleanup timed out".into()))??;
                            running=false;failures=failures.saturating_add(1);
                            retry_at=tokio::time::Instant::now()+config.retry_delay.saturating_mul(1<<failures.saturating_sub(1).min(4)).min(Duration::from_secs(60));
                            let detail=format!("Reception failed; retrying: {error}");
                            if monitor.state()!=MonitorState::KeyRequired {
                                if let Some(update)=monitor.transition(MonitorState::Recovering,&detail,SystemTime::now())? {report(&update);}
                            }
                            warn_once(&mut monitor,&mut last_warning,&detail,&mut report)?;
                        }
                    }
                }
            }
        }
    }.await;
    let cleanup = tokio::time::timeout(Duration::from_secs(21), source.stop())
        .await
        .map_err(|_| DeviceError::Transport("Scan cleanup timed out".into()))
        .and_then(|r| r);
    let stopped = monitor.transition(
        MonitorState::Stopped,
        "Monitoring stopped",
        SystemTime::now(),
    );
    if let Ok(Some(update)) = &stopped {
        report(update);
    }
    // Report cleanup errors even when another failure already exists.
    if let Err(error) = &cleanup {
        report(&MonitorReport {
            state: MonitorState::Stopped,
            detail: format!("Scan cleanup failed: {error}"),
            sample_id: None,
            observations: Vec::new(),
        });
    }
    outcome.and(cleanup).and(stopped.map(|_| ()))
}
#[cfg(feature = "desktop")]
fn warn_once<S: ObservationSink>(
    monitor: &mut VictronMonitor<S>,
    last: &mut String,
    detail: &str,
    report: &mut impl FnMut(&MonitorReport),
) -> DeviceResult<()> {
    if last != detail {
        monitor
            .sink()
            .record_event(monitor.source_id(), SystemTime::now(), "Warning", detail)?;
        report(&MonitorReport {
            state: monitor.state(),
            detail: detail.into(),
            sample_id: None,
            observations: Vec::new(),
        });
        *last = detail.into();
    }
    Ok(())
}

#[cfg(feature = "desktop")]
fn monotonic_now() -> Instant {
    tokio::time::Instant::now().into_std()
}
