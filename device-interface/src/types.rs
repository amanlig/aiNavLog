//! Provisional public data contracts; device-specific formats remain TBD.

use std::fmt;
use std::time::{Duration, SystemTime};

pub type DeviceResult<T> = Result<T, DeviceError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceError {
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

impl fmt::Display for DeviceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for DeviceError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportKind {
    Bluetooth,
}

#[derive(Debug, Clone)]
pub struct DeviceDescriptor {
    /// Opaque adapter-owned identity; not necessarily a MAC address.
    pub id: String,
    pub name: Option<String>,
    pub transport: TransportKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Failed,
}

#[derive(Debug, Clone)]
pub struct ConnectionStatus {
    pub device: Option<DeviceDescriptor>,
    pub state: ConnectionState,
    pub last_observation_at: Option<SystemTime>,
    pub error: Option<DeviceError>,
}

#[derive(Debug, Clone)]
pub struct ReconnectPolicy {
    /// Zero disables automatic retries. No default policy is assumed.
    pub max_attempts: u32,
    pub retry_delay: Duration,
}

#[derive(Debug, Clone)]
pub struct ReceivedFrame {
    pub source_id: String,
    pub received_at: SystemTime,
    pub payload: Vec<u8>,
}

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
