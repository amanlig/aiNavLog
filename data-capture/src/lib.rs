//! Capture validation and durable writes. No Bluetooth, UI or protocol decoding.
use ainavlog_data_model::{
    AppError, AppResult, Observation, ObservationSink, QualityStatus, SampleMetadata,
};
use ainavlog_local_storage::ObservationStore;
use std::{collections::HashSet, time::SystemTime};

pub struct CaptureService {
    store: ObservationStore,
}
impl CaptureService {
    pub fn new(store: ObservationStore) -> Self {
        Self { store }
    }
    pub fn store(&self) -> &ObservationStore {
        &self.store
    }
}
impl ObservationSink for CaptureService {
    fn capture(
        &mut self,
        sample: &SampleMetadata,
        payload: &[u8],
        observations: &[Observation],
    ) -> AppResult<i64> {
        let mut quantities = HashSet::new();
        if sample.source_id.trim().is_empty()
            || observations.is_empty()
            || observations.iter().any(|o| {
                o.source_id != sample.source_id
                    || o.received_at != sample.received_at
                    || o.quantity.trim().is_empty()
                    || o.unit.trim().is_empty()
                    || !quantities.insert(o.quantity.as_str())
                    || o.value.is_some_and(|v| !v.is_finite())
                    || (o.value.is_none() && o.quality == QualityStatus::Good)
            })
        {
            return Err(AppError::InvalidReading);
        }
        self.store.save(sample, payload, observations)
    }
    fn record_event(
        &self,
        source: &str,
        time: SystemTime,
        state: &str,
        detail: &str,
    ) -> AppResult<()> {
        if source.trim().is_empty() || state.trim().is_empty() {
            return Err(AppError::InvalidReading);
        }
        self.store.record_event(source, time, state, detail)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, UNIX_EPOCH};
    fn sample() -> SampleMetadata {
        SampleMetadata {
            source_id: "house-monitor".into(),
            received_at: UNIX_EPOCH + Duration::from_secs(123),
            product_id: 4660,
            data_counter: 1,
            record_type: 2,
        }
    }
    fn reading(sample: &SampleMetadata) -> Observation {
        Observation {
            source_id: sample.source_id.clone(),
            received_at: sample.received_at,
            observed_at: None,
            quantity: "battery_voltage".into(),
            unit: "V".into(),
            value: Some(12.76),
            quality: QualityStatus::Good,
        }
    }
    #[test]
    fn capture_validates_before_commit_and_retains_packet_provenance() {
        let mut service = CaptureService::new(ObservationStore::in_memory().unwrap());
        let sample = sample();
        let valid = reading(&sample);
        let mut wrong = valid.clone();
        wrong.source_id = "other".into();
        assert!(service.capture(&sample, b"packet", &[wrong]).is_err());
        let mut missing = valid.clone();
        missing.value = None;
        assert!(service.capture(&sample, b"packet", &[missing]).is_err());
        assert!(service
            .capture(&sample, b"packet", &[valid.clone(), valid.clone()])
            .is_err());
        assert!(service.store().recent(10).unwrap().is_empty());
        let id = service.capture(&sample, b"packet", &[valid]).unwrap();
        let rows = service.store().recent(10).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].sample_id, id);
        assert_eq!(rows[0].source_id, "house-monitor");
        assert_eq!(rows[0].received_at_ms, 123000);
        assert_eq!(rows[0].value, Some(12.76));
    }
    #[test]
    fn unavailable_observations_remain_null_with_unknown_quality() {
        let mut service = CaptureService::new(ObservationStore::in_memory().unwrap());
        let sample = sample();
        let mut unavailable = reading(&sample);
        unavailable.value = None;
        unavailable.quality = QualityStatus::Unknown;
        service.capture(&sample, b"packet", &[unavailable]).unwrap();
        let rows = service.store().recent(1).unwrap();
        assert_eq!(rows[0].value, None);
        assert_eq!(rows[0].quality, "Unknown");
    }
}
