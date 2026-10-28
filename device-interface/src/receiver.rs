//! BLE event handling belongs to Device Interface. Callers supply lifecycle and
//! accepted-packet callbacks; this module has no persistence or UI dependency.
use crate::{
    bluetooth::BluetoothAdapter,
    vendor::victron::{VictronAdvertisement, VICTRON_COMPANY_ID},
    DeviceError, DeviceResult,
};
use btleplug::api::{CentralEvent, CentralState};
use std::time::{Duration, SystemTime};

pub async fn scan_victron(
    adapter_index: usize,
    silence_after: Duration,
    ready: impl FnOnce(),
    mut accept: impl FnMut(VictronAdvertisement) -> DeviceResult<bool>,
) -> DeviceResult<()> {
    let mut adapter = tokio::time::timeout(
        Duration::from_secs(20),
        BluetoothAdapter::new(adapter_index),
    )
    .await
    .map_err(|_| DeviceError::Transport("Bluetooth startup timed out".into()))??;
    let mut scan = adapter.start_advertisements().await?;
    ready();
    let mut silence = tokio::time::Instant::now();
    let outcome: DeviceResult<()> = async {
        loop {
            let remaining = silence_after.saturating_sub(silence.elapsed());
            let event = tokio::time::timeout(remaining, scan.next_event())
                .await
                .map_err(|_| DeviceError::Transport("Selected source silent".into()))??;
            match event {
                CentralEvent::ManufacturerDataAdvertisement {
                    id,
                    mut manufacturer_data,
                } => {
                    if let Some(payload) = manufacturer_data.remove(&VICTRON_COMPANY_ID) {
                        if accept(VictronAdvertisement {
                            source_id: id.to_string(),
                            company_id: VICTRON_COMPANY_ID,
                            payload,
                            received_at: SystemTime::now(),
                        })? {
                            silence = tokio::time::Instant::now();
                        }
                    }
                }
                CentralEvent::StateUpdate(CentralState::PoweredOff) => {
                    return Err(DeviceError::Transport("Bluetooth powered off".into()))
                }
                _ => {}
            }
            if silence.elapsed() >= silence_after {
                return Err(DeviceError::Transport("Selected source silent".into()));
            }
        }
    }
    .await;
    let stopped = scan.stop().await;
    outcome.and(stopped)
}
