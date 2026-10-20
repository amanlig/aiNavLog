use crate::types::*;

/// OS adapter boundary. Methods describe logical operations only.
/// Async execution, permission flows, framing, and buffering remain TBD.
/// No boat-equipment control API is exposed.
pub trait TransportAdapter {
    fn discover(&mut self) -> DeviceResult<Vec<DeviceDescriptor>> {
        Err(DeviceError::NotImplemented)
    }

    fn connect(&mut self, _device: &DeviceDescriptor) -> DeviceResult<()> {
        Err(DeviceError::NotImplemented)
    }

    fn disconnect(&mut self) -> DeviceResult<()> {
        Err(DeviceError::NotImplemented)
    }

    /// Intended as a nonblocking poll: None will mean no frame is available.
    fn try_receive(&mut self) -> DeviceResult<Option<ReceivedFrame>> {
        Err(DeviceError::NotImplemented)
    }
}

pub use crate::bluetooth::BluetoothAdapter;
