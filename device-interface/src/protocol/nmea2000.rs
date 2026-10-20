//! NMEA 2000 PGN codec backed by CANboat's embedded database.
//!
//! `Frame` holds a COMPLETE PGN payload and its priority/source/destination,
//! not an arbitrary eight-byte CAN fragment. Gateway parsing, fast-packet/ISO-TP
//! reassembly and outbound fragmentation belong to the transport layer. This
//! codec opens no ports, claims no address and transmits nothing.

pub use canboat::{ids, DecodedPgn, EncodeValue, FieldValue, Frame, PgnBuilder, Units};

use super::{ProtocolDecoder, ProtocolEncoder};
use crate::{DeviceError, DeviceResult};

pub struct Nmea2000Codec {
    database: &'static canboat::Database,
}

impl Default for Nmea2000Codec {
    fn default() -> Self {
        Self::new(Units::Metric)
    }
}

impl Nmea2000Codec {
    pub fn new(units: Units) -> Self {
        Self {
            database: canboat::Database::embedded(units),
        }
    }

    /// Start an encoding request using a CANboat schema identifier, e.g.
    /// `windData`. Use the builder for fields, repeating sets and CAN metadata.
    /// Unset fields use CANboat's schema defaults (including unavailable values).
    pub fn message(&self, schema_id: &str) -> DeviceResult<PgnBuilder> {
        self.database
            .encode(schema_id)
            .map_err(|error| DeviceError::Encode(error.to_string()))
    }
}

fn validate(frame: &Frame) -> Result<(), String> {
    if frame.prio > 7 {
        return Err("CAN priority must be in 0..=7".into());
    }
    if frame.pgn > 0x3ffff {
        return Err("PGN must fit in 18 bits".into());
    }
    if frame.data.len() > canboat::FRAME_MAX_SIZE {
        return Err("PGN payload exceeds the transport maximum".into());
    }
    Ok(())
}

impl ProtocolDecoder<Frame> for Nmea2000Codec {
    type Message = DecodedPgn;

    fn decode(&mut self, input: &Frame) -> DeviceResult<DecodedPgn> {
        validate(input).map_err(DeviceError::Decode)?;
        if let Some(info) = self.database.pick_variant(input) {
            if let Some(minimum) = info.min_length.or(info.length) {
                if input.data.len() < minimum as usize {
                    return Err(DeviceError::Decode(format!(
                        "PGN {} requires at least {minimum} bytes, got {}",
                        input.pgn,
                        input.data.len()
                    )));
                }
            }
        }
        self.database
            .decode(input)
            .map_err(|error| DeviceError::Decode(error.to_string()))
    }
}

impl ProtocolEncoder<PgnBuilder> for Nmea2000Codec {
    type Output = Frame;

    fn encode(&mut self, message: &PgnBuilder) -> DeviceResult<Frame> {
        let frame = message
            .build()
            .map_err(|error| DeviceError::Encode(error.to_string()))?;
        validate(&frame).map_err(DeviceError::Encode)?;
        Ok(frame)
    }
}
