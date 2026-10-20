//! Victron Instant Readout decoding for BMV battery monitors and Orion chargers.
//!
//! Input is the manufacturer-data value under company ID 0x02e1, WITHOUT the
//! company-ID bytes or BLE AD length/type bytes. One decoder/key per source.
//! AES-CTR broadcasts are not authenticated: a matching key-check byte does not
//! establish that the full key is correct, or that the sender/data is authentic.
//! See vendor/README.md for sources, wire layout, supported records and limits.

use aes::cipher::{KeyIvInit, StreamCipher};
use std::{fmt, time::SystemTime};
use zeroize::Zeroizing;

use crate::{DeviceError, DeviceResult, ProtocolDecoder};

pub const VICTRON_COMPANY_ID: u16 = 0x02e1;

/// Manufacturer data from a single BLE advertisement. Source IDs are opaque OS
/// peripheral IDs; callers must select the same source used to configure the key.
#[derive(Debug, Clone)]
pub struct VictronAdvertisement {
    pub source_id: String,
    pub received_at: SystemTime,
    pub company_id: u16,
    pub payload: Vec<u8>,
}

/// Header metadata and interpreted Orion data; timestamp is receipt time, not a
/// time reported by the charger. The counter wraps and is not a timestamp.
#[derive(Debug, Clone, PartialEq)]
pub struct VictronMessage {
    pub source_id: String,
    pub received_at: SystemTime,
    pub product_id: u16,
    pub data_counter: u16,
    pub record: OrionRecord,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OrionRecord {
    /// Published record 0x02 (BMV-712 / compatible battery monitors).
    BatteryMonitor(BatteryMonitorReadings),
    /// Published record 0x04 (DC/DC converter, including Orion-Tr Smart).
    DcDc(DcDcReadings),
    /// Published record 0x0f (Orion XS).
    OrionXs(OrionXsReadings),
}

/// General name for the shared record enum; OrionRecord remains compatible.
pub type VictronRecord = OrionRecord;

#[derive(Debug, Clone, PartialEq)]
pub enum BatteryAuxiliary {
    Voltage(Option<f64>),
    MidpointVoltage(Option<f64>),
    TemperatureKelvin(Option<f64>),
    Disabled,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatteryMonitorReadings {
    pub time_to_go_minutes: Option<u16>,
    pub battery_voltage_v: Option<f64>,
    pub battery_current_a: Option<f64>,
    pub consumed_ah: Option<f64>,
    pub state_of_charge_percent: Option<f64>,
    pub alarm_reason: u16,
    pub auxiliary: BatteryAuxiliary,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DcDcReadings {
    /// Raw state/error codes; unknown codes are preserved, 0xff is unavailable.
    pub device_state: Option<u8>,
    pub charger_error: Option<u8>,
    pub input_voltage_v: Option<f64>,
    pub output_voltage_v: Option<f64>,
    /// Bitmask, not a single enum. No unavailable sentinel is specified.
    pub off_reason: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrionXsReadings {
    /// The XS layout gives the full byte range, without an NA sentinel.
    pub device_state: u8,
    pub charger_error: u8,
    pub output_voltage_v: Option<f64>,
    pub output_current_a: Option<f64>,
    pub input_voltage_v: Option<f64>,
    pub input_current_a: Option<f64>,
    pub off_reason: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VictronError {
    InvalidKey,
    WrongSource,
    WrongCompany,
    UnsupportedAdvertisement(u8),
    UnsupportedRecord(u8),
    Truncated { required: usize, actual: usize },
    KeyMismatch,
    KeyRequired,
    CipherFailure,
}

impl fmt::Display for VictronError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidKey => {
                f.write_str("expected a 32-character hexadecimal advertisement key")
            }
            Self::WrongSource => f.write_str("advertisement source does not match this decoder"),
            Self::WrongCompany => f.write_str("not Victron manufacturer data"),
            Self::UnsupportedAdvertisement(kind) => {
                write!(f, "unsupported advertisement type {kind:#04x}")
            }
            Self::UnsupportedRecord(kind) => write!(f, "unsupported Victron record {kind:#04x}"),
            Self::Truncated { required, actual } => write!(
                f,
                "truncated manufacturer data: need {required} bytes, got {actual}"
            ),
            Self::KeyMismatch => {
                f.write_str("advertisement key check failed; key discarded, supply a fresh key")
            }
            Self::KeyRequired => f.write_str("advertisement key required"),
            Self::CipherFailure => f.write_str("advertisement decryption failed"),
        }
    }
}
impl std::error::Error for VictronError {}

/// Device-bound, decode-only implementation. Keys are redacted from Debug and
/// zeroized on replacement/drop. Caller-owned input key strings remain the
/// caller's responsibility. A header key mismatch discards this decoder's key.
pub struct VictronAdvertisementDecoder {
    source_id: String,
    key: Option<Zeroizing<[u8; 16]>>,
}

impl fmt::Debug for VictronAdvertisementDecoder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VictronAdvertisementDecoder")
            .field("source_id", &self.source_id)
            .field("key_configured", &self.key.is_some())
            .finish_non_exhaustive()
    }
}

impl VictronAdvertisementDecoder {
    pub fn new(source_id: impl Into<String>, key_hex: &str) -> Result<Self, VictronError> {
        Ok(Self {
            source_id: source_id.into(),
            key: Some(parse_key(key_hex)?),
        })
    }

    /// Replace an invalidated/rotated key obtained from VictronConnect.
    pub fn replace_key(&mut self, key_hex: &str) -> Result<(), VictronError> {
        self.key = Some(parse_key(key_hex)?);
        Ok(())
    }

    /// Decode with structured vendor errors. The ProtocolDecoder implementation
    /// below adapts these into the application's DeviceError contract.
    pub fn decode_advertisement(
        &mut self,
        input: &VictronAdvertisement,
    ) -> Result<VictronMessage, VictronError> {
        if input.source_id != self.source_id {
            return Err(VictronError::WrongSource);
        }
        if input.company_id != VICTRON_COMPANY_ID {
            return Err(VictronError::WrongCompany);
        }
        let data = &input.payload;
        require_len(data, 8)?;
        if data[0] != 0x10 {
            return Err(VictronError::UnsupportedAdvertisement(data[0]));
        }
        let required = match data[4] {
            0x02 => 15, // SOC ends at plaintext bit 118; reserved bits may follow
            0x04 => 10,
            0x0f => 14,
            kind => return Err(VictronError::UnsupportedRecord(kind)),
        };
        require_len(data, 8 + required)?;
        let key = self.key.as_ref().ok_or(VictronError::KeyRequired)?;
        if data[7] != key[0] {
            self.key = None;
            return Err(VictronError::KeyMismatch);
        }
        // Victron's two nonce bytes are copied in wire order to the start of
        // the AES block; the remaining big-endian counter starts at zero.
        let mut iv = [0u8; 16];
        iv[..2].copy_from_slice(&data[5..7]);
        let mut plain = Zeroizing::new(data[8..].to_vec());
        let mut cipher = ctr::Ctr128BE::<aes::Aes128>::new_from_slices(key.as_ref(), &iv)
            .map_err(|_| VictronError::CipherFailure)?;
        cipher
            .try_apply_keystream(&mut plain)
            .map_err(|_| VictronError::CipherFailure)?;
        // Minimum sizes were checked before decrypting; trailing extensions are
        // allowed by the specification and ignored until their layout is known.
        let record = if data[4] == 0x02 {
            let current = bits(&plain, 66, 22);
            let signed_current = ((current << 10) as i32) >> 10;
            let consumed = bits(&plain, 88, 20);
            let soc = bits(&plain, 108, 10);
            let aux = u16_at(&plain, 6);
            let auxiliary = match bits(&plain, 64, 2) {
                0 => BatteryAuxiliary::Voltage(signed_measurement(&plain, 6, 0.01)),
                1 => BatteryAuxiliary::MidpointVoltage(
                    (aux != 0xffff).then_some(f64::from(aux) * 0.01),
                ),
                2 => BatteryAuxiliary::TemperatureKelvin(
                    (aux != 0xffff).then_some(f64::from(aux) * 0.01),
                ),
                _ => BatteryAuxiliary::Disabled,
            };
            OrionRecord::BatteryMonitor(BatteryMonitorReadings {
                time_to_go_minutes: (u16_at(&plain, 0) != 0xffff).then_some(u16_at(&plain, 0)),
                battery_voltage_v: signed_measurement(&plain, 2, 0.01),
                battery_current_a: (current != 0x3fffff)
                    .then_some(f64::from(signed_current) * 0.001),
                consumed_ah: (consumed != 0xfffff).then_some(-f64::from(consumed) * 0.1),
                state_of_charge_percent: (soc <= 1000).then_some(f64::from(soc) * 0.1),
                alarm_reason: u16_at(&plain, 4),
                auxiliary,
            })
        } else if data[4] == 0x04 {
            OrionRecord::DcDc(DcDcReadings {
                device_state: (plain[0] != 0xff).then_some(plain[0]),
                charger_error: (plain[1] != 0xff).then_some(plain[1]),
                input_voltage_v: unsigned_measurement(&plain, 2, 0.01),
                output_voltage_v: signed_measurement(&plain, 4, 0.01),
                off_reason: u32_at(&plain, 6),
            })
        } else {
            OrionRecord::OrionXs(OrionXsReadings {
                device_state: plain[0],
                charger_error: plain[1],
                output_voltage_v: signed_measurement(&plain, 2, 0.01),
                output_current_a: signed_measurement(&plain, 4, 0.1),
                input_voltage_v: unsigned_measurement(&plain, 6, 0.01),
                input_current_a: unsigned_measurement(&plain, 8, 0.1),
                off_reason: u32_at(&plain, 10),
            })
        };
        Ok(VictronMessage {
            source_id: input.source_id.clone(),
            received_at: input.received_at,
            product_id: u16::from_le_bytes([data[2], data[3]]),
            data_counter: u16::from_le_bytes([data[5], data[6]]),
            record,
        })
    }
}

impl ProtocolDecoder<VictronAdvertisement> for VictronAdvertisementDecoder {
    type Message = VictronMessage;
    fn decode(&mut self, input: &VictronAdvertisement) -> DeviceResult<VictronMessage> {
        self.decode_advertisement(input)
            .map_err(|error| match error {
                VictronError::WrongCompany
                | VictronError::UnsupportedAdvertisement(_)
                | VictronError::UnsupportedRecord(_) => DeviceError::UnsupportedProtocol,
                other => DeviceError::Decode(other.to_string()),
            })
    }
}

fn parse_key(input: &str) -> Result<Zeroizing<[u8; 16]>, VictronError> {
    if input.len() != 32 || !input.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(VictronError::InvalidKey);
    }
    let mut key = Zeroizing::new([0u8; 16]);
    for (index, byte) in key.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&input[index * 2..index * 2 + 2], 16)
            .map_err(|_| VictronError::InvalidKey)?;
    }
    Ok(key)
}

fn require_len(data: &[u8], required: usize) -> Result<(), VictronError> {
    if data.len() < required {
        Err(VictronError::Truncated {
            required,
            actual: data.len(),
        })
    } else {
        Ok(())
    }
}
fn u16_at(data: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([data[offset], data[offset + 1]])
}
fn u32_at(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(
        data[offset..offset + 4]
            .try_into()
            .expect("checked record length"),
    )
}
fn unsigned_measurement(data: &[u8], offset: usize, scale: f64) -> Option<f64> {
    let raw = u16_at(data, offset);
    (raw != 0xffff).then_some(f64::from(raw) * scale)
}
fn signed_measurement(data: &[u8], offset: usize, scale: f64) -> Option<f64> {
    let raw = u16_at(data, offset);
    (raw != 0x7fff).then_some(f64::from(raw as i16) * scale)
}

// Little-endian bit fields can cross byte boundaries (BMV current, Ah and SOC).
fn bits(data: &[u8], offset: usize, width: usize) -> u32 {
    (0..width).fold(0, |value, bit| {
        value | (u32::from((data[(offset + bit) / 8] >> ((offset + bit) % 8)) & 1) << bit)
    })
}
