# aiNavLog Device Interface

Rust Device Interface library and BLE evaluation CLI for architecture section 5.4.3.
See the [UML class diagram](../docs/device-interface-class-diagram.md) for current
components, relationships, and data contracts.

Bluetooth uses [btleplug](https://github.com/deviceplug/btleplug) with Tokio.
This is Bluetooth Low Energy (BLE) central support, not Bluetooth Classic serial.
Beta instrument connectivity is Bluetooth-only. Wi-Fi support is shelved for
post-beta consideration in the [backlog](../docs/backlog.md#wi-fi-instrument-connectivity).
Both NMEA codecs remain in beta scope.

## Discover and connect

From this directory:

```sh
cargo run -- discover
cargo run -- discover 10
cargo run -- connect '<device-id>'
```

Discovery scans for five seconds by default and prints opaque device IDs and
names. Copy an exact ID into `connect`; quote it to preserve special characters.
Unnamed devices are included. Results can include OS-cached devices and do not
guarantee that a device is currently reachable or connectable.

```sh
cargo run -- discover 10 0
cargo run -- connect '<device-id>' 10 0
cargo run -- --help
```

Optional arguments are scan duration (1–300 seconds) and zero-based local adapter
index (default 0). Connection performs a fresh scan, connects only to the selected
ID, and holds the connection until Ctrl-C, then explicitly disconnects. On Windows,
if the ID is absent from the scan, it attempts the explicitly supplied Bluetooth
address directly; an advertisement is not required to start that attempt. Device
IDs are matched without case sensitivity. Connection
and disconnection operations have 20-second timeouts. Failures exit nonzero;
no arguments display help without accessing hardware. An existing connection not
owned by this adapter is rejected. A failed connection attempts cleanup.

The BLE CLI writes no characteristics, decodes no NMEA data, and sends no equipment
control commands. NMEA codecs are available separately through the library API. Pairing flows, automatic reconnect, notification subscriptions,
and monitoring unexpected remote disconnections are not implemented yet.

## Platform setup

- **Linux:** Requires an accessible Bluetooth controller, BlueZ and the system
  D-Bus service. The Linux dependency enables vendored libdbus, so building needs
  a C compiler but not system D-Bus development headers or `pkg-config`.
- **WSL:** A working Rust build does not establish Bluetooth access. The Linux
  environment must expose a controller and run the required services. If it does
  not, run the CLI natively on Windows with Windows Rust tooling and Bluetooth enabled.
- **macOS:** Grant Bluetooth permission to the terminal/application; packaged
  applications need the Bluetooth usage description described by btleplug.
- **iOS/Android:** Native application packaging, permissions, and a React Native
  bridge are still required. This CLI does not implement the mobile bridge.

See [btleplug's platform setup](https://github.com/deviceplug/btleplug#buildinstallation-notes-for-specific-platforms)
and the [library evaluation notes](../docs/dependencies/rust-bluetooth-libraries.md).

## Library API and remaining stubs

`BluetoothAdapter::default()` creates an inert adapter without accessing hardware.
Use its async methods inside a Tokio runtime:

- `BluetoothAdapter::new(index).await`: select an OS adapter.
- `discover(Duration).await`: scan and return device descriptors; lazy initialization
  uses adapter 0 when constructed with `default()`.
- `connect(&descriptor).await`: connect to a previously discovered device.
- `connect_by_id(id).await`: resolve a discovered ID or, on Windows, register and
  attempt the explicitly supplied Bluetooth address even if it was not in the scan.
- `disconnect().await`: release the owned connection; callers should invoke this
  even after a failed connection to clean up a possible partial connection.

The Bluetooth adapter has async inherent methods and does not implement the
provisional synchronous `TransportAdapter` trait. That trait, `ConnectionManager`, `DeviceProtocolAdapter` remain
stubs returning `DeviceError::NotImplemented`. The CLI calls Bluetooth directly.
The unified transport/connection-manager contract remains future integration work.
`ObservationNormalizer` now validates and preserves unit-labelled readings. The
Victron monitor maps vendor records to observations and persists them locally.

Discovery should be allowed to finish so it can stop the OS scan. Dropping/cancelling
its future is not a substitute for cleanup. Likewise, callers own disconnection;
there is no async cleanup on drop.

## NMEA decoding and encoding

See the [NMEA library usage guide](../docs/dependencies/rust-nmea-libraries.md) for complete
examples, message boundaries, units, error handling and integration limitations.

Requires Rust **1.96+** (CANboat's minimum). `ProtocolDecoder<Input>` and
`ProtocolEncoder<Message>` are separate interfaces with associated output types.
`protocol::nmea0183::Nmea0183Codec` uses **nmea-kit 0.8.9**;
`protocol::nmea2000::Nmea2000Codec` uses **canboat 8.3.0**, with only its `decode`
feature enabled (which includes encoding). Neither codec accesses hardware.

```rust
use ainavlog_device_interface::{
    Nmea0183Codec, Nmea0183Message, Nmea2000Codec, ProtocolDecoder, ProtocolEncoder,
    protocol::{nmea0183::nmea::Hdt, nmea2000::ids::field::wind_data as wd},
};

let mut sentences = Nmea0183Codec;
let heading = sentences.decode("$GPHDT,123.456,T*32")?;
println!("{:?}", heading.sentence());
let outbound = Nmea0183Message::from_sentence("GP", &Hdt {
    heading_true: Some(90.0),
})?;
let line = sentences.encode(&outbound)?; // checksum + CRLF

let mut pgns = Nmea2000Codec::default();
let mut request = pgns.message("windData")?.source(42).destination(255);
request.push(wd::WIND_SPEED, 5.23)?;
let frame = pgns.encode(&request)?; // complete PGN payload + CAN metadata
let wind = pgns.decode(&frame)?;
println!("{:?}", wind.field(wd::WIND_SPEED));
# Ok::<(), Box<dyn std::error::Error>>(())
```

- **0183 input:** one complete ASCII sentence, optionally including CRLF and an
  IEC tag block. Checksums are checked when supplied; checksumless input is
  accepted. Encoding always emits a checksum and CRLF. Talker, raw field
  precision, unknown/proprietary sentences and existing tag blocks are preserved
  when forwarding. Typed missing/malformed values follow nmea-kit and become
  `None`; non-finite outbound numbers become empty fields. AIS envelopes are preserved but AIS payload decoding/reassembly is
  not implemented by this wrapper.
- **2000 input/output:** CANboat `Frame`, containing a **complete PGN payload**,
  priority, source, destination and optional timestamp. The caller must handle
  gateway framing, incoming CAN fast-packet/ISO-TP reassembly and outgoing
  fragmentation. CANboat builders expose schema fields, enums and repeating sets;
  omitted fields use schema defaults, including unavailable sentinels. Unknown
  PGNs follow CANboat's fallback definitions or return a decode error. Payloads
  shorter than the schema minimum are rejected.
- `Nmea2000Codec::default()` uses CANboat's Metric units; `new(Units::Si)` selects
  SI. Consult field units or `as_f64_in(...)` when normalizing. The interfaces keep
  protocol-specific messages instead of forcing a lossy common measurement model.
- Device discovery, BLE notification subscriptions, address claiming, actual
  transmission, and mapping decoded messages into `DecodedReading` remain separate
  integration work. The existing `DeviceProtocolAdapter` is still a provisional
  reading/normalization boundary; these codecs do not claim to implement it.

## Victron vendor decoder

The [vendor implementation guide](vendor/README.md) documents the application-owned
`device-interface/vendor/victron` module. `VictronAdvertisementDecoder` implements
the shared decoding interface for encrypted Orion-Tr Smart/DC-DC and Orion XS
Instant Readout advertisements. It requires a device-specific advertisement key.
Live advertisement reception, key-file loading/reloading, freshness/retry handling
and SQLite persistence are implemented. See the
[live monitoring guide](../docs/victron-live-monitoring.md) for commands using the
recorded device ID, database queries and physical-validation steps.

## Live monitoring and local history

```sh
cargo run --locked -- monitor-victron 'D4:B3:CB:26:5E:75' --key-file /path/to/orion.key
cargo run --locked -- observations data/observations.sqlite3 25
```

Use a private key file (Unix: `chmod 600`); do not pass the key itself on the command
line. Monitoring scans for broadcasts and requires no GATT connection. Ctrl-C
stops it. Data is stored locally in SQLite; no data or key is uploaded. The
[live guide](../docs/victron-live-monitoring.md) covers Windows, recovery behavior,
missing values, and remaining mobile/hardware validation.

## Validation

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```

Tests validate CLI arguments, Bluetooth request rejection without hardware,
remaining stub contracts, NMEA wire bytes/round trips, metadata, unavailable values,
and malformed input. Real discovery and connection require a BLE peripheral;
these tests do not claim radio interoperability or NMEA gateway compatibility.

No application license has been selected. The package is marked `publish = false`.

## Windows executable

The Windows x64 release executable is built at:

```text
target/x86_64-pc-windows-msvc/release/ainavlog-device-interface.exe
```

Run from Windows PowerShell in this directory:

```powershell
.\target\x86_64-pc-windows-msvc\release\ainavlog-device-interface.exe discover 10
.\target\x86_64-pc-windows-msvc\release\ainavlog-device-interface.exe connect '<device-id>'
```

This uses Windows Bluetooth directly, even when the executable is stored in a
WSL project directory. Bluetooth must be enabled in Windows.

To rebuild natively, install Rust for Windows with the MSVC toolchain and Visual
Studio C++ Build Tools/Windows SDK, then run `./build-windows.ps1` in PowerShell.
The script uses the committed Cargo lockfile. Release binaries are generated
artifacts under `target`, not committed source files.

Cross-building from Linux is also supported using
[cargo-xwin](https://github.com/rust-cross/cargo-xwin), the Windows Rust target,
and LLVM's `lld-link` (Rust's bundled `rust-lld` can provide that linker):

```sh
rustup target add x86_64-pc-windows-msvc
cargo xwin build --release --locked --target x86_64-pc-windows-msvc
```

The cross-build downloads Microsoft SDK/CRT link libraries on first use. Native
C/C++ dependencies, if added later, may additionally require clang tooling.

### Device appears in discovery but will not connect

Try the Windows executable with the exact device address:

```powershell
.\target\x86_64-pc-windows-msvc\release\ainavlog-device-interface.exe connect 'D4:B3:CB:26:5E:75' 10
```

The CLI now prints when it is attempting an address that was not seen in the
current scan. The result distinguishes a missing scan result from a connection
failure returned by Windows. This fallback cannot make an offline or
non-connectable advertiser accept a BLE connection. Confirm the device model,
BLE/GATT support, required pairing/connection mode, and whether another app or
phone is using its connection. Some devices change advertising addresses; verify
identity with the device's documentation rather than guessing from similar IDs.

## Third-party licenses

See the [library license inventory](../docs/dependencies/library-licenses.md) for direct and
transitive dependencies, browser libraries, and redistribution requirements.

The beta deployment targets are **iOS and Android**. Windows/Linux CLI builds are
evaluation tools. See the [mobile beta license assessment](../docs/dependencies/library-licenses.md#mobile-beta-distribution-policy)
for target-specific dependency coverage and the remaining mobile packaging review.

## Local UI bridge

In another terminal alongside `monitor-victron`, run:

```sh
cargo run --locked -- serve-ui data/observations.sqlite3 '<bmv-device-id>'
```

`GET http://127.0.0.1:8787/electrical` returns the latest complete BMV packet as
JSON, or `null` when no BMV packet exists for that device. Missing/poor-quality
measurements are `null`; timestamps are original receipt times in Unix milliseconds.
An unreadable database returns HTTP 503. Reads do not create or modify the database.
The bridge is loopback-only, read-only, and allows the browser origin
`http://localhost:8081` by default. Optional arguments are port and UI origin.
See [UI setup](../user-interface/README.md#connect-the-rust-device-interface-locally).
This development bridge does not implement native mobile integration or add a
Wi-Fi instrument transport. Advertisement keys and raw packets are never returned.

## Shared Rust core for native integration

The default build enables `desktop` and `nmea`, preserving existing CLI commands,
Bluetooth scanning and NMEA codecs. A native wrapper can depend on the BMV core
without those optional integrations:

```toml
ainavlog-device-interface = { path = "../device-interface", default-features = false }
```

The core includes the shared Victron decoder, `VictronMonitor`, normalization and
SQLite storage. A platform scanner supplies `VictronAdvertisement` with its selected
source ID, company ID, manufacturer payload and original receipt time. The payload
excludes the Bluetooth company-ID prefix, as documented by the decoder. Call
`VictronMonitor::ingest` from a serialized owner with both wall-clock receipt time
and monotonic time; storage commits before freshness is advanced. `tick` handles
freshness transitions, and `replace_key` clears prior freshness and duplicates.

Platform permission/scanning lifecycle and protected key storage belong to the
native adapter. The desktop file-key loader, asynchronous scanner and scan recovery
runner require `desktop`. NMEA codecs can be enabled separately with `features =
["nmea"]`; their separation does not remove NMEA from the project scope.

```sh
cargo build --locked --lib --no-default-features
cargo test --locked --no-default-features
cargo clippy --locked --no-default-features --all-targets -- -D warnings
cargo tree --locked --no-default-features --edges normal
```

These host checks verify reuse without btleplug, D-Bus, Tokio or NMEA in the normal
core dependency graph. They do not verify iOS/Android cross-compilation or a phone install.

The [native bridge](native-bridge/README.md) now provides a host-tested C ABI for
session lifecycle, advertisement ingestion and persisted electrical readings.
Its C integration test uses captured simulator ciphertext. Target packaging,
Swift/Kotlin Expo modules and phone Bluetooth reception remain the next steps.
