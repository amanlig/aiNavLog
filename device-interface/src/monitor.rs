//! Foreground Victron advertisement reception, normalization and local persistence.
#[cfg(feature = "desktop")]
use crate::{bluetooth::AdvertisementScan, BluetoothAdapter};
use crate::{
    store::ObservationStore,
    vendor::victron::{
        OrionRecord, VictronAdvertisement, VictronAdvertisementDecoder, VictronError,
        VictronMessage, VICTRON_COMPANY_ID,
    },
    DecodedReading, DeviceError, DeviceResult, Observation, ObservationNormalizer, QualityStatus,
};
#[cfg(feature = "desktop")]
use btleplug::api::{CentralEvent, CentralState};
use std::{
    collections::VecDeque,
    path::PathBuf,
    time::{Duration, Instant, SystemTime},
};
#[cfg(feature = "desktop")]
use std::{future::Future, io::Read, path::Path};
#[cfg(feature = "desktop")]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitorState {
    Waiting,
    Live,
    Stale,
    Recovering,
    KeyRequired,
    Stopped,
}
#[derive(Debug, Clone)]
pub struct MonitorReport {
    pub state: MonitorState,
    pub detail: String,
    pub sample_id: Option<i64>,
    pub observations: Vec<Observation>,
}

/// In-memory processing and freshness state, independent of Bluetooth hardware.
pub struct VictronMonitor {
    source_id: String,
    decoder: VictronAdvertisementDecoder,
    store: ObservationStore,
    stale_after: Duration,
    started: Instant,
    last_sample: Option<Instant>,
    recent: VecDeque<(Instant, Vec<u8>)>,
    state: MonitorState,
}
impl VictronMonitor {
    pub fn new(
        source_id: String,
        key: &str,
        stale_after: Duration,
        store: ObservationStore,
        now: Instant,
    ) -> DeviceResult<Self> {
        if source_id.trim().is_empty() || stale_after.is_zero() {
            return Err(DeviceError::Configuration(
                "source and stale timeout required".into(),
            ));
        }
        let decoder = VictronAdvertisementDecoder::new(&source_id, key)
            .map_err(|e| DeviceError::Configuration(e.to_string()))?;
        Ok(Self {
            source_id,
            decoder,
            store,
            stale_after,
            started: now,
            last_sample: None,
            recent: VecDeque::new(),
            state: MonitorState::Waiting,
        })
    }
    pub fn state(&self) -> MonitorState {
        self.state
    }
    pub fn store(&self) -> &ObservationStore {
        &self.store
    }
    pub fn transition(
        &mut self,
        state: MonitorState,
        detail: &str,
        wall: SystemTime,
    ) -> DeviceResult<Option<MonitorReport>> {
        if self.state == state {
            return Ok(None);
        }
        self.store
            .record_event(&self.source_id, wall, &format!("{state:?}"), detail)?;
        self.state = state;
        Ok(Some(MonitorReport {
            state,
            detail: detail.into(),
            sample_id: None,
            observations: Vec::new(),
        }))
    }
    pub fn replace_key(
        &mut self,
        key: &str,
        now: Instant,
        wall: SystemTime,
    ) -> DeviceResult<Option<MonitorReport>> {
        self.decoder
            .replace_key(key)
            .map_err(|e| DeviceError::Configuration(e.to_string()))?;
        self.recent.clear();
        self.last_sample = None;
        self.started = now;
        self.transition(
            MonitorState::Waiting,
            "Advertisement key reloaded; waiting for a new sample",
            wall,
        )
    }
    pub fn tick(&mut self, now: Instant, wall: SystemTime) -> DeviceResult<Option<MonitorReport>> {
        if !matches!(
            self.state,
            MonitorState::KeyRequired | MonitorState::Stopped | MonitorState::Recovering
        ) && now.saturating_duration_since(self.last_sample.unwrap_or(self.started))
            >= self.stale_after
        {
            self.transition(
                MonitorState::Stale,
                "No new valid sample within the freshness window",
                wall,
            )
        } else {
            Ok(None)
        }
    }
    /// Exact retransmissions inside the freshness window do not create rows or
    /// refresh measurements. Cache expires, allowing nonce wrap/device restarts.
    pub fn ingest(
        &mut self,
        mut input: VictronAdvertisement,
        now: Instant,
    ) -> DeviceResult<Option<MonitorReport>> {
        if !input.source_id.eq_ignore_ascii_case(&self.source_id)
            || input.company_id != VICTRON_COMPANY_ID
        {
            return Ok(None);
        }
        input.source_id.clone_from(&self.source_id);
        if self.state == MonitorState::KeyRequired {
            return Ok(None);
        }
        if self
            .recent
            .back()
            .is_some_and(|(_, payload)| *payload == input.payload)
        {
            return Ok(None);
        }
        while self
            .recent
            .front()
            .is_some_and(|(seen, _)| now.saturating_duration_since(*seen) >= self.stale_after)
        {
            self.recent.pop_front();
        }
        if self
            .recent
            .iter()
            .any(|(_, payload)| *payload == input.payload)
        {
            return Ok(None);
        }
        let message=match self.decoder.decode_advertisement(&input) {
            Ok(message)=>message,
            Err(VictronError::KeyMismatch|VictronError::KeyRequired)=>return self.transition(MonitorState::KeyRequired,"Key check failed; replace the key file with a fresh VictronConnect advertisement key",input.received_at),
            Err(VictronError::WrongCompany|VictronError::WrongSource|VictronError::UnsupportedAdvertisement(_))=>return Ok(None),
            Err(error)=>return Err(DeviceError::Decode(error.to_string())),
        };
        let observations = normalize_victron(&message)?;
        let id = self.store.save(&message, &input.payload, &observations)?;
        // Update freshness/dedup only after the transaction succeeds.
        self.last_sample = Some(now);
        self.recent.push_back((now, input.payload));
        if self.recent.len() > 64 {
            self.recent.pop_front();
        }
        let _ = self.transition(
            MonitorState::Live,
            "Receiving and storing observations",
            message.received_at,
        )?;
        Ok(Some(MonitorReport {
            state: MonitorState::Live,
            detail: "Sample stored".into(),
            sample_id: Some(id),
            observations,
        }))
    }
}

pub fn normalize_victron(message: &VictronMessage) -> DeviceResult<Vec<Observation>> {
    let mut fields: Vec<(&str, &str, Option<f64>)> = Vec::new();
    match &message.record {
        OrionRecord::BatteryMonitor(d) => {
            fields.extend([
                ("battery_voltage", "V", d.battery_voltage_v),
                ("battery_current", "A", d.battery_current_a),
                ("state_of_charge", "%", d.state_of_charge_percent),
                ("consumed_charge", "Ah", d.consumed_ah),
                ("time_to_go", "min", d.time_to_go_minutes.map(f64::from)),
                ("alarm_reason", "bitmask", Some(f64::from(d.alarm_reason))),
            ]);
            use crate::vendor::victron::BatteryAuxiliary;
            match &d.auxiliary {
                BatteryAuxiliary::Voltage(v) => fields.push(("auxiliary_voltage", "V", *v)),
                BatteryAuxiliary::MidpointVoltage(v) => fields.push(("midpoint_voltage", "V", *v)),
                BatteryAuxiliary::TemperatureKelvin(v) => {
                    fields.push(("battery_temperature", "K", *v))
                }
                BatteryAuxiliary::Disabled => {}
            }
        }
        OrionRecord::DcDc(d) => fields.extend([
            ("input_voltage", "V", d.input_voltage_v),
            ("output_voltage", "V", d.output_voltage_v),
            ("device_state", "code", d.device_state.map(f64::from)),
            ("charger_error", "code", d.charger_error.map(f64::from)),
            ("off_reason", "bitmask", Some(f64::from(d.off_reason))),
        ]),
        OrionRecord::OrionXs(d) => fields.extend([
            ("input_voltage", "V", d.input_voltage_v),
            ("output_voltage", "V", d.output_voltage_v),
            ("input_current", "A", d.input_current_a),
            ("output_current", "A", d.output_current_a),
            ("device_state", "code", Some(f64::from(d.device_state))),
            ("charger_error", "code", Some(f64::from(d.charger_error))),
            ("off_reason", "bitmask", Some(f64::from(d.off_reason))),
        ]),
    }
    fields
        .into_iter()
        .map(|(quantity, unit, value)| {
            ObservationNormalizer::new().normalize(&DecodedReading {
                source_id: message.source_id.clone(),
                observed_at: None,
                received_at: message.received_at,
                quantity: quantity.into(),
                unit: unit.into(),
                value,
                quality: if value.is_some() {
                    QualityStatus::Good
                } else {
                    QualityStatus::Unknown
                },
            })
        })
        .collect()
}

/// Testable source seam. stop must clean up after a failed/cancelled start too.
#[cfg(feature = "desktop")]
pub trait AdvertisementSource {
    fn start(&mut self) -> impl Future<Output = DeviceResult<()>> + Send;
    fn next(&mut self) -> impl Future<Output = DeviceResult<VictronAdvertisement>> + Send;
    fn stop(&mut self) -> impl Future<Output = DeviceResult<()>> + Send;
}
#[cfg(feature = "desktop")]
pub struct BluetoothAdvertisementSource {
    index: usize,
    scan: Option<AdvertisementScan>,
}
#[cfg(feature = "desktop")]
impl BluetoothAdvertisementSource {
    pub fn new(index: usize) -> Self {
        Self { index, scan: None }
    }
}
#[cfg(feature = "desktop")]
impl AdvertisementSource for BluetoothAdvertisementSource {
    async fn start(&mut self) -> DeviceResult<()> {
        self.stop().await?;
        let mut adapter = BluetoothAdapter::new(self.index).await?;
        self.scan = Some(adapter.start_advertisements().await?);
        Ok(())
    }
    async fn next(&mut self) -> DeviceResult<VictronAdvertisement> {
        let scan = self.scan.as_mut().ok_or(DeviceError::Disconnected)?;
        loop {
            match scan.next_event().await? {
                CentralEvent::ManufacturerDataAdvertisement {
                    id,
                    mut manufacturer_data,
                } => {
                    if let Some(payload) = manufacturer_data.remove(&VICTRON_COMPANY_ID) {
                        return Ok(VictronAdvertisement {
                            source_id: id.to_string(),
                            received_at: SystemTime::now(),
                            company_id: VICTRON_COMPANY_ID,
                            payload,
                        });
                    }
                }
                CentralEvent::StateUpdate(CentralState::PoweredOff) => {
                    return Err(DeviceError::Transport("Bluetooth powered off".into()))
                }
                _ => {}
            }
        }
    }
    async fn stop(&mut self) -> DeviceResult<()> {
        if let Some(mut scan) = self.scan.take() {
            scan.stop().await?;
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
    let store = ObservationStore::open(&config.database)?;
    let mut monitor = VictronMonitor::new(
        config.source_id.clone(),
        &key,
        config.stale_after,
        store,
        monotonic_now(),
    )?;
    monitor.store.record_event(
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
fn warn_once(
    monitor: &mut VictronMonitor,
    last: &mut String,
    detail: &str,
    report: &mut impl FnMut(&MonitorReport),
) -> DeviceResult<()> {
    if last != detail {
        monitor
            .store
            .record_event(&monitor.source_id, SystemTime::now(), "Warning", detail)?;
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
