//! Receiver behaviour can be exercised without any database implementation.
use ainavlog_data_model::{AppError, AppResult, Observation, ObservationSink, SampleMetadata};
use ainavlog_device_interface::{
    monitor::{MonitorState, VictronMonitor},
    vendor::victron::VictronAdvertisement,
};
use std::time::{Duration, Instant, SystemTime};
struct MemorySink {
    fail_once: bool,
    saved: usize,
}
impl ObservationSink for MemorySink {
    fn capture(
        &mut self,
        sample: &SampleMetadata,
        payload: &[u8],
        observations: &[Observation],
    ) -> AppResult<i64> {
        assert_eq!(sample.record_type, 2);
        assert_eq!(sample.source_id, "test-bmv");
        assert!(!payload.is_empty());
        assert!(observations
            .iter()
            .all(|o| o.source_id == sample.source_id && o.received_at == sample.received_at));
        if self.fail_once {
            self.fail_once = false;
            return Err(AppError::Storage("injected failure".into()));
        }
        self.saved += 1;
        Ok(self.saved as i64)
    }
    fn record_event(
        &self,
        _source: &str,
        _time: SystemTime,
        _state: &str,
        _detail: &str,
    ) -> AppResult<()> {
        Ok(())
    }
}
#[test]
fn failed_capture_does_not_mark_live_or_suppress_retry_of_same_packet() {
    let now = Instant::now();
    let mut monitor = VictronMonitor::new(
        "test-bmv".into(),
        "000102030405060708090a0b0c0d0e0f",
        Duration::from_secs(5),
        MemorySink {
            fail_once: true,
            saved: 0,
        },
        now,
    )
    .unwrap();
    let packet = VictronAdvertisement {
        source_id: "test-bmv".into(),
        company_id: 0x02e1,
        payload: include_bytes!("fixtures/victron/bmv_starter.bin").to_vec(),
        received_at: SystemTime::UNIX_EPOCH + Duration::from_secs(123),
    };
    assert!(monitor.ingest(packet.clone(), now).is_err());
    assert_eq!(monitor.state(), MonitorState::Waiting);
    assert_eq!(monitor.sink().saved, 0);
    let report = monitor
        .ingest(packet.clone(), now + Duration::from_secs(1))
        .unwrap()
        .unwrap();
    assert_eq!(report.sample_id, Some(1));
    assert_eq!(monitor.state(), MonitorState::Live);
    assert!(monitor
        .ingest(packet, now + Duration::from_secs(2))
        .unwrap()
        .is_none());
    assert_eq!(monitor.sink().saved, 1);
    assert_eq!(
        monitor
            .tick(now + Duration::from_secs(6), SystemTime::now())
            .unwrap()
            .unwrap()
            .state,
        MonitorState::Stale
    );
}
