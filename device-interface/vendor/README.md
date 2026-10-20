# Vendor protocol implementations

This directory contains application-owned vendor integrations, not downloaded
third-party source. It is exposed as `ainavlog_device_interface::vendor` through
an explicit module path in `src/lib.rs`.

## Victron Instant Readout

`victron::VictronAdvertisementDecoder` implements
`ProtocolDecoder<VictronAdvertisement>`. It decrypts and interprets **Orion-Tr
Smart/DC-DC record 0x04** and **Orion XS record 0x0f**. Other Victron record types
return an unsupported-protocol error; this is not an implementation of every
product listed in Victron's specification.

The implementation uses RustCrypto `aes` and `ctr` plus `zeroize`. It has no
encoder, equipment-control operations, Bluetooth connection management or live
scan loop. The existing NMEA codecs are separate protocols and are not involved.

### Usage

Enable Instant Readout and obtain the advertisement key using VictronConnect.
Create one decoder per selected peripheral using its opaque OS ID and a
32-character hexadecimal key. Keep the decoder for that source across updates.

This executable example uses a public synthetic fixture, not a real device key:

```rust
use ainavlog_device_interface::{
    ProtocolDecoder,
    vendor::victron::{
        OrionRecord, VictronAdvertisement, VictronAdvertisementDecoder,
        VICTRON_COMPANY_ID,
    },
};
use std::time::SystemTime;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut decoder = VictronAdvertisementDecoder::new(
        "example-device", "000102030405060708090a0b0c0d0e0f",
    )?;
    let input = VictronAdvertisement {
        source_id: "example-device".into(),
        received_at: SystemTime::now(),
        company_id: VICTRON_COMPANY_ID,
        payload: vec![
            0x10, 0x02, 0x34, 0x12, 0x04, 0x34, 0x12, 0x00,
            0x2a, 0x60, 0x6e, 0x0e, 0x66, 0x04, 0x89, 0xca, 0xae, 0x60,
        ],
    };
    let message = decoder.decode(&input)?;
    if let OrionRecord::DcDc(readings) = message.record {
        assert!((readings.input_voltage_v.unwrap() - 13.2).abs() < 0.001);
        println!("Output voltage: {:?} V", readings.output_voltage_v);
    }
    Ok(())
}
```

The live monitor selects the value under manufacturer company
ID `0x02e1` from `CentralEvent::ManufacturerDataAdvertisement.manufacturer_data`. Supply that
value as `payload`; cached properties are not treated as fresh updates.
It already excludes the company ID bytes: do not prepend them or feed an entire
BLE advertising packet. Do not filter only by a GATT service UUID: this path
consumes manufacturer advertisements without establishing a GATT connection.

### Packet boundary and decoding

Offsets below are relative to the manufacturer-data value supplied by btleplug:

| Byte offset | Content |
| --- | --- |
| 0 | Product-advertisement marker `0x10` |
| 1 | Outer advertisement byte, not interpreted by this decoder |
| 2–3 | Little-endian product ID, preserved without guessing a model name |
| 4 | Extra-data record type (`0x04` or `0x0f`) |
| 5–6 | Nonce/data-counter bytes, preserved as a little-endian integer |
| 7 | First-byte key check |
| 8 onward | AES-128-CTR encrypted record body |

Copy bytes 5–6 unchanged into the first two bytes of a zero-filled AES IV. The
remaining counter starts at zero and increments in big-endian order. Decrypt
without padding. Fields are little-endian. The 2022 specification's field offsets
include the four-byte extra-record header; the XS update's offsets start at the
decrypted body. The implementation accounts for that difference.

DC/DC decoding requires ten body bytes; XS requires fourteen. Extra trailing
bytes are accepted for forward compatibility. No measurements are invented for
short bodies. The DC/DC record contains no current measurement. XS includes
signed output current and unsigned input current. Voltages use 0.01 V units;
XS currents use 0.1 A units. Published unavailable measurement sentinels become
`None`. State/error codes and off-reason bits are preserved for later interpretation.

### Keys, errors and observation semantics

- The decoder binds a key to one source ID. It rejects other sources and companies
  before using the key. Source IDs are routing identifiers, not cryptographic identity.
- A first-byte key mismatch discards the stored key and stops decoding until
  `replace_key()` succeeds. Callers must also invalidate their persisted key.
- The key and temporary plaintext are zeroized on drop. Decoder `Debug` output
  only reports whether a key is configured. Caller-owned key strings and persistent
  storage remain the caller's responsibility; store real keys in platform secure storage.
- AES-CTR does **not authenticate** these messages. A matching first byte cannot
  detect a wrong key with the same first byte, corrupted ciphertext or a forged
  advertisement. This implementation does not claim authenticated measurements.
- `decode_advertisement()` returns structured `VictronError` values. The shared
  `decode()` interface maps unsupported formats to `DeviceError::UnsupportedProtocol`
  and other failures to `DeviceError::Decode`.
- `received_at` is the local receipt time. The data counter wraps; it is not a
  timestamp. Duplicates are returned unchanged so the receive pipeline can decide
  freshness/deduplication policy. Missing values remain optional.

The [foreground monitor](../../docs/victron-live-monitoring.md) implements live
advertisement reception, private key-file loading/reloading, normalization,
freshness/retry handling and SQLite storage. Mobile secure key provisioning and
background behavior still require app integration. Physical Orion interoperability
has not been validated in this environment.

### Verification and sources

Run from `device-interface`:

```sh
cargo test --locked --test victron_decoder
cargo test --locked --doc
```

Tests use independently generated OpenSSL ciphertexts, documented under
`tests/fixtures/victron/README.md`, and cover metadata, byte order, signed/scaled
values, sentinels, truncated inputs, extensions, device routing and key lifecycle.

Protocol sources:

- [Victron's integration announcement](https://communityarchive.victronenergy.com/questions/187303/victron-bluetooth-advertising-protocol.html)
- [Published extra manufacturer-data specification, 2022-12-14](https://communityarchive.victronenergy.com/storage/attachments/extra-manufacturer-data-2022-12-14.pdf)
- [Orion XS layout published by Victron staff, 2024-09-05](https://community.victronenergy.com/t/orion-xs-12v-12v-50a-bluetooth-advertising-data/2183/2)
- [Manufacturer-envelope reference implementation](https://github.com/keshavdv/victron-ble/blob/main/victron_ble/devices/base.py)

The implementation is original code based on the protocol descriptions. It does
not copy the reference project's source or bundle Victron's specification PDF.

## BMV-712 / Battery Monitor record 0x02

The decoder also accepts the published battery-monitor record. It exposes battery
voltage, signed 22-bit current (A), consumed charge (negative Ah), SOC (%),
time-to-go (minutes), alarm bits and a typed auxiliary value (second voltage,
midpoint voltage, temperature in kelvin, or disabled). NA sentinels become None;
SOC outside 0..100% becomes unavailable. At least 15 encrypted bytes are needed;
trailing reserved bytes/extensions are accepted. `VictronRecord` aliases the
existing `OrionRecord` type for compatibility.

The monitor normalizes these to battery_voltage, battery_current, state_of_charge,
consumed_charge, time_to_go, alarm_reason and the selected auxiliary quantity.
SQLite records battery-monitor samples as record_type 2. Auxiliary voltage is
mapped to the starter battery by installation configuration; the decoder does not
invent starter current or SOC. The beta auxiliary setting is second-battery voltage.

Regression coverage includes the user-transcribed iPhone packet, independent
OpenSSL charge/discharge fixtures, unavailable values, starter auxiliary voltage,
truncation/extension handling and monitor persistence/duplicate suppression.
The tests use synthetic keys and packets, not real hardware captures.
