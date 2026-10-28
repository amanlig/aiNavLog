//! Platform-independent application record contracts.
use std::{fmt, time::SystemTime};
pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppError {
    NotImplemented,
    UnsupportedProtocol,
    PermissionDenied,
    Disconnected,
    InvalidReading,
    Transport(String),
    Decode(String),
    Encode(String),
    Storage(String),
    Configuration(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for AppError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QualityStatus {
    Good,
    Stale,
    Invalid,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct DecodedReading {
    pub source_id: String,
    pub observed_at: Option<SystemTime>,
    pub received_at: SystemTime,
    pub quantity: String,
    pub unit: String,
    /// None represents a missing measurement, never an invented zero.
    pub value: Option<f64>,
    pub quality: QualityStatus,
}

#[derive(Debug, Clone)]
pub struct Observation {
    pub source_id: String,
    pub observed_at: Option<SystemTime>,
    pub received_at: SystemTime,
    pub quantity: String,
    pub unit: String,
    pub value: Option<f64>,
    pub quality: QualityStatus,
}

/// Source packet metadata; no vendor Rust types or SQLite implementation leak here.
#[derive(Debug, Clone)]
pub struct SampleMetadata {
    pub source_id: String,
    pub received_at: SystemTime,
    pub product_id: u16,
    pub data_counter: u16,
    pub record_type: u8,
}

/// Injected write boundary. Device reception never opens or queries a database.
pub trait ObservationSink {
    fn capture(
        &mut self,
        sample: &SampleMetadata,
        payload: &[u8],
        observations: &[Observation],
    ) -> AppResult<i64>;
    fn record_event(
        &self,
        source: &str,
        time: SystemTime,
        state: &str,
        detail: &str,
    ) -> AppResult<()>;
}
