//! Transport-independent message codecs and the provisional reading adapter.

#[cfg(feature = "nmea")]
pub mod nmea0183;
#[cfg(feature = "nmea")]
pub mod nmea2000;

#[cfg(feature = "nmea")]
pub use nmea0183::{Nmea0183Codec, Nmea0183Message};
#[cfg(feature = "nmea")]
pub use nmea2000::Nmea2000Codec;

/// Decode one complete protocol message. Transport framing/reassembly happens
/// before this boundary; implementations retain protocol-specific message types.
pub trait ProtocolDecoder<Input: ?Sized> {
    type Message;

    fn decode(&mut self, input: &Input) -> DeviceResult<Self::Message>;
}

/// Encode a protocol-specific message/request without sending it to equipment.
pub trait ProtocolEncoder<Message: ?Sized> {
    type Output;

    fn encode(&mut self, message: &Message) -> DeviceResult<Self::Output>;
}

use crate::types::*;

/// Implement for each supported instrument protocol once its format is known.
/// No Heart Interface protocol is assumed or implemented.
pub trait DeviceProtocolAdapter {
    fn supports(&self, _device: &DeviceDescriptor) -> DeviceResult<bool> {
        Err(DeviceError::NotImplemented)
    }

    /// A frame may eventually yield zero, one, or multiple readings.
    fn decode(&mut self, _frame: &ReceivedFrame) -> DeviceResult<Vec<DecodedReading>> {
        Err(DeviceError::NotImplemented)
    }
}
