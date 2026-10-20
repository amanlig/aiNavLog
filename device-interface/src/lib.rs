//! Device Interface contracts and an async btleplug BLE evaluation adapter.

#[cfg(feature = "desktop")]
pub mod bluetooth;
pub mod connection;
pub mod monitor;
pub mod normalizer;
pub mod protocol;
pub mod store;
#[cfg(feature = "desktop")]
pub mod transport;
pub mod types;

#[path = "../vendor/mod.rs"]
pub mod vendor;

pub use connection::ConnectionManager;
pub use normalizer::ObservationNormalizer;
pub use protocol::{DeviceProtocolAdapter, ProtocolDecoder, ProtocolEncoder};
#[cfg(feature = "desktop")]
pub use transport::{BluetoothAdapter, TransportAdapter};
pub use types::*;

#[cfg(feature = "nmea")]
pub use protocol::{Nmea0183Codec, Nmea0183Message, Nmea2000Codec};
