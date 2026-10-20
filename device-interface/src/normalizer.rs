use crate::types::*;

/// Converts decoded readings into the common Data capture contract.
#[derive(Default)]
pub struct ObservationNormalizer;

impl ObservationNormalizer {
    pub fn new() -> Self {
        Self
    }

    /// Validate and preserve an already unit-labelled reading. Missing or invalid
    /// values are never promoted to good quality; no unit conversion is guessed.
    pub fn normalize(&self, reading: &DecodedReading) -> DeviceResult<Observation> {
        if reading.source_id.is_empty()
            || reading.quantity.is_empty()
            || reading.unit.is_empty()
            || reading.value.is_some_and(|v| !v.is_finite())
        {
            return Err(DeviceError::InvalidReading);
        }
        let quality = if reading.value.is_none() && reading.quality == QualityStatus::Good {
            QualityStatus::Unknown
        } else {
            reading.quality.clone()
        };
        Ok(Observation {
            source_id: reading.source_id.clone(),
            observed_at: reading.observed_at,
            received_at: reading.received_at,
            quantity: reading.quantity.clone(),
            unit: reading.unit.clone(),
            value: reading.value,
            quality,
        })
    }
}
