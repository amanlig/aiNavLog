use crate::types::*;

/// Connection lifecycle and reconnection-policy boundary.
/// Adapter wiring and state management are not implemented.
pub struct ConnectionManager;

impl ConnectionManager {
    pub fn new() -> Self {
        Self
    }

    pub fn connect(&mut self, _device: &DeviceDescriptor) -> DeviceResult<()> {
        Err(DeviceError::NotImplemented)
    }

    pub fn disconnect(&mut self) -> DeviceResult<()> {
        Err(DeviceError::NotImplemented)
    }

    pub fn status(&self) -> DeviceResult<ConnectionStatus> {
        Err(DeviceError::NotImplemented)
    }

    pub fn set_reconnect_policy(&mut self, _policy: ReconnectPolicy) -> DeviceResult<()> {
        Err(DeviceError::NotImplemented)
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}
