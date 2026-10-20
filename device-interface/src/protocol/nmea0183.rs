//! NMEA 0183 sentence codec backed by `nmea-kit`.
//!
//! Input is one complete ASCII sentence, with optional CRLF and tag block.
//! Checksums are validated when present; checksumless input is accepted, matching
//! nmea-kit. Missing/malformed measurement fields become `None`. Unknown sentences
//! and AIS envelopes retain their raw fields; AIS payload reassembly is not done.

pub use nmea_kit::{nmea, NmeaEncodable, NmeaFrame, NmeaSentence};

use super::{ProtocolDecoder, ProtocolEncoder};
use crate::{DeviceError, DeviceResult};

/// An owned, validated sentence. Keeping the original fields avoids losing
/// unknown/proprietary content or precision when forwarding a decoded message.
#[derive(Debug, Clone, PartialEq)]
pub struct Nmea0183Message {
    line: String,
    sentence: NmeaSentence,
}

impl Nmea0183Message {
    /// Parsed measurements. Unknown sentence types are retained as `Unknown`.
    pub fn sentence(&self) -> &NmeaSentence {
        &self.sentence
    }

    /// Original validated sentence, including any tag block and checksum.
    pub fn as_str(&self) -> &str {
        &self.line
    }

    /// Access talker, prefix, fields and optional tag metadata.
    pub fn frame(&self) -> NmeaFrame<'_> {
        // The only constructors validate the private, immutable line.
        nmea_kit::parse_frame(&self.line).expect("validated NMEA sentence")
    }

    /// Create an outbound message from any nmea-kit typed sentence.
    pub fn from_sentence<T: NmeaEncodable>(talker: &str, sentence: &T) -> DeviceResult<Self> {
        if !T::PROPRIETARY
            && (talker.len() != 2
                || !talker
                    .bytes()
                    .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit()))
        {
            return Err(DeviceError::Encode(
                "talker must be two uppercase ASCII letters/digits".into(),
            ));
        }
        let line = sentence
            .to_sentence(talker)
            .map_err(|error| DeviceError::Encode(error.to_string()))?;
        Nmea0183Codec
            .decode(&line)
            .map_err(|error| DeviceError::Encode(error.to_string()))
    }
}

#[derive(Debug, Default)]
pub struct Nmea0183Codec;

impl ProtocolDecoder<str> for Nmea0183Codec {
    type Message = Nmea0183Message;

    fn decode(&mut self, input: &str) -> DeviceResult<Self::Message> {
        let line = input.trim_end_matches(['\r', '\n']);
        if !line.is_ascii() || line.contains(['\r', '\n']) {
            return Err(DeviceError::Decode(
                "expected one ASCII NMEA sentence".into(),
            ));
        }
        let frame =
            nmea_kit::parse_frame(line).map_err(|error| DeviceError::Decode(error.to_string()))?;
        Ok(Nmea0183Message {
            sentence: NmeaSentence::parse(&frame),
            line: line.to_owned(),
        })
    }
}

impl ProtocolEncoder<Nmea0183Message> for Nmea0183Codec {
    type Output = String;

    /// Emit a checksum and CRLF, preserving the original tag block if present.
    fn encode(&mut self, message: &Nmea0183Message) -> DeviceResult<String> {
        let frame = message.frame();
        let sentence = nmea_kit::encode_frame(
            frame.prefix,
            frame.talker,
            frame.sentence_type,
            &frame.fields,
        )
        .map_err(|error| DeviceError::Encode(error.to_string()))?;
        if frame.tag_block.is_some() {
            let line = message.line.trim();
            // parse_frame already verified the opening/closing tag delimiters.
            let end = line[1..].find('\\').expect("validated tag block") + 2;
            Ok(format!("{}{sentence}", &line[..end]))
        } else {
            Ok(sentence)
        }
    }
}
