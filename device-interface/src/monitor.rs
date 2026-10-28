//! Foreground reception, decoding and freshness; persistence uses an injected capture sink.
#[cfg(feature = "desktop")]
use crate::{bluetooth::AdvertisementScan, BluetoothAdapter};
use crate::{
    vendor::victron::{
        OrionRecord, VictronAdvertisement, VictronAdvertisementDecoder, VictronError,
        VictronMessage, VICTRON_COMPANY_ID,
    },
    DecodedReading, DeviceError, DeviceResult, Observation, ObservationNormalizer, QualityStatus,
};
use ainavlog_data_model::ObservationSink;
#[cfg(feature = "desktop")]
use btleplug::api::{CentralEvent, CentralState};
#[cfg(feature = "desktop")]
use std::future::Future;
use std::{
    collections::VecDeque,
    time::{Duration, Instant, SystemTime},
};

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
pub struct VictronMonitor<S: ObservationSink> {
    source_id: String,
    decoder: VictronAdvertisementDecoder,
    sink: S,
    stale_after: Duration,
    started: Instant,
    last_sample: Option<Instant>,
    recent: VecDeque<(Instant, Vec<u8>)>,
    state: MonitorState,
}
impl<S: ObservationSink> VictronMonitor<S> {
    pub fn new(
        source_id: String,
        key: &str,
        stale_after: Duration,
        sink: S,
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
            sink,
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
    pub fn sink(&self) -> &S {
        &self.sink
    }
    pub fn source_id(&self) -> &str {
        &self.source_id
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
        self.sink
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
        let id = self
            .sink
            .capture(&message.metadata(), &input.payload, &observations)?;
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
