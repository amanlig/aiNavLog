//! Synthetic telemetry only: no Bluetooth, real credentials or charger control.
use ainavlog_device_interface::vendor::victron::{VictronAdvertisement, VICTRON_COMPANY_ID};
use ctr::cipher::{KeyIvInit, StreamCipher};
use std::time::SystemTime;

pub const SOURCE_ID: &str = "SIMULATED-ORION";
pub const KEY_HEX: &str = "000102030405060708090a0b0c0d0e0f";
const KEY: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

#[derive(Clone, Copy, Debug)]
pub enum Model {
    Tr,
    Xs,
}

/// Wire-unit readings avoid floating-point rounding and ambiguous missing values.
#[derive(Clone, Debug)]
pub struct Readings {
    pub state: u8,
    pub error: u8,
    pub input_centivolts: Option<u16>,
    pub output_centivolts: Option<i16>,
    pub input_deciamps: Option<u16>,
    pub output_deciamps: Option<i16>,
    pub off_reason: u32,
}
impl Readings {
    pub fn sample(step: u16) -> Self {
        Self {
            state: if step < 8 {
                3
            } else if step < 16 {
                4
            } else {
                5
            },
            error: 0,
            input_centivolts: Some(1320 + step % 10),
            output_centivolts: Some(if step < 16 { 1440 } else { 1380 }),
            input_deciamps: Some(150),
            output_deciamps: Some(120 - (step % 24) as i16 * 4),
            off_reason: 0,
        }
    }
}

/// Manufacturer bytes exclude the company ID. Product ID 0x1234 is synthetic.
/// Repeated counters/keys are for local testing only, not secure transmission.
pub fn advertisement(
    model: Model,
    counter: u16,
    r: &Readings,
    received_at: SystemTime,
) -> VictronAdvertisement {
    let mut plain = vec![r.state, r.error];
    match model {
        Model::Tr => {
            plain.extend(r.input_centivolts.unwrap_or(u16::MAX).to_le_bytes());
            plain.extend(r.output_centivolts.unwrap_or(i16::MAX).to_le_bytes());
        }
        Model::Xs => {
            plain.extend(r.output_centivolts.unwrap_or(i16::MAX).to_le_bytes());
            plain.extend(r.output_deciamps.unwrap_or(i16::MAX).to_le_bytes());
            plain.extend(r.input_centivolts.unwrap_or(u16::MAX).to_le_bytes());
            plain.extend(r.input_deciamps.unwrap_or(u16::MAX).to_le_bytes());
        }
    }
    plain.extend(r.off_reason.to_le_bytes());
    let mut iv = [0; 16];
    iv[..2].copy_from_slice(&counter.to_le_bytes());
    let mut cipher = ctr::Ctr128BE::<aes::Aes128>::new_from_slices(&KEY, &iv)
        .expect("fixed AES key and IV lengths");
    cipher.apply_keystream(&mut plain);
    let mut payload = vec![
        0x10,
        0x02,
        0x34,
        0x12,
        match model {
            Model::Tr => 4,
            Model::Xs => 15,
        },
    ];
    payload.extend(counter.to_le_bytes());
    payload.push(KEY[0]);
    payload.extend(plain);
    VictronAdvertisement {
        source_id: SOURCE_ID.into(),
        received_at,
        company_id: VICTRON_COMPANY_ID,
        payload,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ainavlog_device_interface::{
        monitor::{MonitorState, VictronMonitor},
        store::ObservationStore,
    };
    use std::time::{Duration, Instant};
    #[test]
    fn encoder_matches_independent_openssl_fixtures() {
        let mut r = Readings::sample(0);
        r.off_reason = 0x80000081;
        r.output_deciamps = Some(-123);
        for (model, fixture) in [
            (
                Model::Tr,
                include_bytes!("../../../device-interface/tests/fixtures/victron/orion_tr.bin")
                    .as_slice(),
            ),
            (
                Model::Xs,
                include_bytes!("../../../device-interface/tests/fixtures/victron/orion_xs.bin")
                    .as_slice(),
            ),
        ] {
            assert_eq!(
                advertisement(model, 0x1234, &r, SystemTime::now()).payload,
                fixture
            );
        }
    }
    #[test]
    fn generated_packets_drive_storage_and_recovery() {
        let now = Instant::now();
        let wall = SystemTime::now();
        let mut monitor = VictronMonitor::new(
            SOURCE_ID.into(),
            KEY_HEX,
            Duration::from_secs(3),
            ObservationStore::in_memory().unwrap(),
            now,
        )
        .unwrap();
        let packet = advertisement(Model::Xs, 1, &Readings::sample(1), wall);
        assert!(monitor
            .ingest(packet.clone(), now)
            .unwrap()
            .unwrap()
            .sample_id
            .is_some());
        assert!(monitor.ingest(packet.clone(), now).unwrap().is_none());
        assert_eq!(
            monitor
                .tick(now + Duration::from_secs(4), wall)
                .unwrap()
                .unwrap()
                .state,
            MonitorState::Stale
        );
        let mut bad = packet;
        bad.payload[7] = 255;
        assert_eq!(
            monitor.ingest(bad, now).unwrap().unwrap().state,
            MonitorState::KeyRequired
        );
        monitor.replace_key(KEY_HEX, now, wall).unwrap();
        let mut missing = Readings::sample(2);
        missing.output_centivolts = None;
        let report = monitor
            .ingest(advertisement(Model::Xs, 2, &missing, wall), now)
            .unwrap()
            .unwrap();
        assert_eq!(report.state, MonitorState::Live);
        assert!(report.observations.iter().any(|o| o.value.is_none()));
        assert_eq!(monitor.store().recent(100).unwrap().len(), 14);
    }
}
