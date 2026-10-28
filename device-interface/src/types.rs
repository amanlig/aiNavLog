//! Device-specific connection contracts and shared application type aliases.
pub use ainavlog_data_model::{
    AppError as DeviceError, AppResult as DeviceResult, DecodedReading, Observation, QualityStatus,
};
use std::time::{Duration, SystemTime};
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
