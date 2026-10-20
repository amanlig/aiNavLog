# Device Interface UML class diagram

This diagram describes the Rust code in `device-interface/src`, reviewed on
2026-10-05. Rust structs are shown as classes and traits as interfaces. It reflects
the current implementation rather than the proposed architecture. The vendor
view focuses on the selected BMV-712 beta battery monitor; charger-specific
record types are omitted from this document.

Beta instrument connectivity is Bluetooth-only. Wi-Fi has been removed from the
active interfaces and diagrams and [shelved in the backlog](backlog.md#wi-fi-instrument-connectivity).

## Navigate to code

Click a class or interface to open its Rust definition. Project links are relative
to this document and include GitHub-style line anchors; editor previews may open
the file without jumping to the line. External nodes link to the pinned library
source or API documentation.

Diagram clicks require a Markdown viewer that permits
[Mermaid links](https://mermaid.js.org/syntax/classDiagram.html#interaction).
Viewers using Mermaid's strict security mode can disable them. The source index
below provides ordinary Markdown links as a fallback.

## Implementation status

Colors describe the code as of **2026-10-05**, within each component's stated
scope. Green means **100% implemented for that scope**, not complete support for
every protocol feature, platform or live device. Interfaces and data contracts
can be complete as definitions while their consuming pipeline remains unfinished.

| Color | Status | Components and scope |
| --- | --- | --- |
| 🟩 Green | 100% implemented within stated scope | `Nmea0183Codec`, `Nmea0183Message`, `Nmea2000Codec`: complete-message encoding/decoding; `VictronAdvertisementDecoder`: BMV-712 battery-monitor advertisement decoding; `ProtocolDecoder`, `ProtocolEncoder`: implemented interface definitions; `CLI`: discovery, connection, foreground monitoring and history commands; `VictronMonitor`, `ObservationNormalizer`, `ObservationStore`: tested processing, validation and SQLite persistence; data structs/enums: existing field and variant definitions. |
| 🟨 Amber | In progress | `BluetoothAdapter`, `AdvertisementScan`, `BluetoothAdvertisementSource`: scanning and reception implemented, pending hardware/mobile validation; GATT notifications and unified transport integration remain unfinished. |
| 🟥 Red | Not implemented | `TransportAdapter`, `ConnectionManager`, `DeviceProtocolAdapter`: operational methods remain `NotImplemented` stubs. Constructors and type declarations do not count as operational implementation. |
| ⬜ Gray | External dependency | btleplug handles, nmea-kit, CANboat and their types. These are outside the project's implementation-status assessment. |

The same colors apply in all diagrams below. The table supplies text equivalents
so status does not depend on color alone. Data contracts remain provisional even
though their current definitions are implemented. Codec status excludes transport
framing, CAN fragmentation/reassembly, AIS payload assembly, normalization and
hardware integration; see the [NMEA usage guide](dependencies/rust-nmea-libraries.md) and
[Victron vendor guide](../device-interface/vendor/README.md).

## Components and dependencies

```mermaid
classDiagram
    direction LR

    class BluetoothAdapter {
        <<struct>>
        -Option~Adapter~ adapter
        -BTreeMap discovered
        -Option~Peripheral~ connected
        +new(index: usize) DeviceResult~Self~
        +discover(duration: Duration) DeviceResult
        +connect_by_id(id: str) DeviceResult
        +connect(device: DeviceDescriptor) DeviceResult
        +disconnect() DeviceResult
        +start_advertisements() DeviceResult~AdvertisementScan~
        -register_known_address(id: str) DeviceResult~String~
    }
    class TransportAdapter {
        <<interface>>
        +discover() DeviceResult
        +connect(device: DeviceDescriptor) DeviceResult
        +disconnect() DeviceResult
        +try_receive() DeviceResult
    }
    class ConnectionManager {
        <<struct>>
        +new() Self
        +connect(device: DeviceDescriptor) DeviceResult
        +disconnect() DeviceResult
        +status() DeviceResult~ConnectionStatus~
        +set_reconnect_policy(policy: ReconnectPolicy) DeviceResult
    }
    class DeviceProtocolAdapter {
        <<interface>>
        +supports(device: DeviceDescriptor) DeviceResult~bool~
        +decode(frame: ReceivedFrame) DeviceResult
    }
    class ObservationNormalizer {
        <<struct>>
        +new() Self
        +normalize(reading: DecodedReading) DeviceResult~Observation~
    }
    class BtleplugAdapter {
        <<external>>
    }
    class BtleplugPeripheral {
        <<external>>
    }
    class CLI {
        <<binary>>
    }
    class ProtocolDecoder {
        <<interface>>
        +decode(input) DeviceResult~Message~
    }
    class ProtocolEncoder {
        <<interface>>
        +encode(message) DeviceResult~Output~
    }
    class Nmea0183Codec {
        <<struct>>
    }
    class Nmea2000Codec {
        <<struct>>
    }

    ProtocolDecoder <|.. Nmea0183Codec
    ProtocolEncoder <|.. Nmea0183Codec
    ProtocolDecoder <|.. Nmea2000Codec
    ProtocolEncoder <|.. Nmea2000Codec

    BluetoothAdapter --> "0..1" BtleplugAdapter : adapter handle
    BluetoothAdapter --> "0..*" BtleplugPeripheral : discovered handles by ID
    BluetoothAdapter --> "0..1" BtleplugPeripheral : selected connection handle
    CLI ..> BluetoothAdapter : awaits methods directly
    BluetoothAdapter ..> DeviceDescriptor : accepts and returns
    TransportAdapter ..> DeviceDescriptor : accepts and returns
    TransportAdapter ..> ReceivedFrame : returns optional frame
    ConnectionManager ..> DeviceDescriptor : accepts
    ConnectionManager ..> ConnectionStatus : returns
    ConnectionManager ..> ReconnectPolicy : accepts
    DeviceProtocolAdapter ..> DeviceDescriptor : checks support
    DeviceProtocolAdapter ..> ReceivedFrame : accepts
    DeviceProtocolAdapter ..> DecodedReading : returns list
    ObservationNormalizer ..> DecodedReading : accepts
    ObservationNormalizer ..> Observation : returns

    note for BluetoothAdapter "In progress: async discovery/connect/disconnect and manufacturer event reception implemented; physical validation pending. Notification reception and TransportAdapter integration remain unfinished."
    note for ConnectionManager "Stub: no adapter wiring or connection state storage yet."
    note for TransportAdapter "Synchronous trait. Default operations return NotImplemented."
    note for DeviceProtocolAdapter "Not implemented: reading adapter is a stub. NMEA codecs exist separately; no mapping to DecodedReading yet."
    note for ObservationNormalizer "Validates identifiers and finite values; preserves units and marks unavailable readings Unknown."

    style CLI fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    style ProtocolDecoder fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    style ProtocolEncoder fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    style Nmea0183Codec fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    style Nmea2000Codec fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    style DeviceDescriptor fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    style ReceivedFrame fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    style ConnectionStatus fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    style ReconnectPolicy fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    style DecodedReading fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    style Observation fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    style BluetoothAdapter fill:#fef3c7,stroke:#92400e,color:#78350f,stroke-width:2px
    style TransportAdapter fill:#fee2e2,stroke:#991b1b,color:#7f1d1d,stroke-width:2px
    style ConnectionManager fill:#fee2e2,stroke:#991b1b,color:#7f1d1d,stroke-width:2px
    style DeviceProtocolAdapter fill:#fee2e2,stroke:#991b1b,color:#7f1d1d,stroke-width:2px
    style ObservationNormalizer fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    style BtleplugAdapter fill:#f1f5f9,stroke:#64748b,color:#334155,stroke-width:1px
    style BtleplugPeripheral fill:#f1f5f9,stroke:#64748b,color:#334155,stroke-width:1px

    click CLI href "../device-interface/src/main.rs#L96" "Open CLI source or API"
    click ProtocolDecoder href "../device-interface/src/protocol.rs#L11" "Open ProtocolDecoder source or API"
    click ProtocolEncoder href "../device-interface/src/protocol.rs#L18" "Open ProtocolEncoder source or API"
    click Nmea0183Codec href "../device-interface/src/protocol/nmea0183.rs#L60" "Open Nmea0183Codec source or API"
    click Nmea2000Codec href "../device-interface/src/protocol/nmea2000.rs#L13" "Open Nmea2000Codec source or API"
    click DeviceDescriptor href "../device-interface/src/types.rs#L36" "Open DeviceDescriptor source or API"
    click ReceivedFrame href "../device-interface/src/types.rs#L68" "Open ReceivedFrame source or API"
    click ConnectionStatus href "../device-interface/src/types.rs#L53" "Open ConnectionStatus source or API"
    click ReconnectPolicy href "../device-interface/src/types.rs#L61" "Open ReconnectPolicy source or API"
    click DecodedReading href "../device-interface/src/types.rs#L83" "Open DecodedReading source or API"
    click Observation href "../device-interface/src/types.rs#L95" "Open Observation source or API"
    click BluetoothAdapter href "../device-interface/src/bluetooth.rs#L17" "Open BluetoothAdapter source or API"
    click TransportAdapter href "../device-interface/src/transport.rs#L6" "Open TransportAdapter source or API"
    click ConnectionManager href "../device-interface/src/connection.rs#L5" "Open ConnectionManager source or API"
    click DeviceProtocolAdapter href "../device-interface/src/protocol.rs#L28" "Open DeviceProtocolAdapter source or API"
    click ObservationNormalizer href "../device-interface/src/normalizer.rs#L5" "Open ObservationNormalizer source or API"
    click BtleplugAdapter href "https://docs.rs/btleplug/0.13.3/btleplug/platform/struct.Adapter.html" "Open BtleplugAdapter source or API"
    click BtleplugPeripheral href "https://docs.rs/btleplug/0.13.3/btleplug/platform/struct.Peripheral.html" "Open BtleplugPeripheral source or API"
```

Solid arrows represent stored handles; dotted arrows represent use through method
arguments or results. The hollow triangle denotes trait implementation. `btleplug`
handles are shared/clonable, so they are not shown as exclusively owned UML
compositions. A selected connection handle can be retained after a failed attempt
for cleanup; its presence alone does not establish a live connection.

All operations returning `DeviceResult` use the alias
`DeviceResult<T> = Result<T, DeviceError>`. To keep signatures readable, Rust
borrows and some nested generic return types are abbreviated:

| Method | Successful result |
| --- | --- |
| Both `discover` methods | `Vec<DeviceDescriptor>` |
| `TransportAdapter::try_receive` | `Option<ReceivedFrame>` |
| `DeviceProtocolAdapter::decode` | `Vec<DecodedReading>` |
| Connect, disconnect, and set-reconnect-policy operations | `()` |

`BluetoothAdapter::register_known_address` is private and platform-specific:
Windows can register an explicit Bluetooth address with btleplug; other platforms
currently return an error for an ID absent from discovery. The CLI uses
`connect_by_id` after scanning.

## Data contracts

```mermaid
classDiagram
    direction LR

    class DeviceDescriptor {
        <<struct>>
        +String id
        +Option~String~ name
        +TransportKind transport
    }
    class TransportKind {
        <<enumeration>>
        Bluetooth
    }
    class ConnectionStatus {
        <<struct>>
        +Option~DeviceDescriptor~ device
        +ConnectionState state
        +Option~SystemTime~ last_observation_at
        +Option~DeviceError~ error
    }
    class ConnectionState {
        <<enumeration>>
        Disconnected
        Connecting
        Connected
        Reconnecting
        Failed
    }
    class ReconnectPolicy {
        <<struct>>
        +u32 max_attempts
        +Duration retry_delay
    }
    class ReceivedFrame {
        <<struct>>
        +String source_id
        +SystemTime received_at
        +Vec~u8~ payload
    }
    class DecodedReading {
        <<struct>>
        +String source_id
        +Option~SystemTime~ observed_at
        +SystemTime received_at
        +String quantity
        +String unit
        +Option~f64~ value
        +QualityStatus quality
    }
    class Observation {
        <<struct>>
        +String source_id
        +Option~SystemTime~ observed_at
        +SystemTime received_at
        +String quantity
        +String unit
        +Option~f64~ value
        +QualityStatus quality
    }
    class QualityStatus {
        <<enumeration>>
        Good
        Stale
        Invalid
        Unknown
    }
    class DeviceError {
        <<enumeration>>
        NotImplemented
        UnsupportedProtocol
        PermissionDenied
        Disconnected
        InvalidReading
        Transport(String)
        Decode(String)
        Encode(String)
        Storage(String)
        Configuration(String)
    }

    DeviceDescriptor --> TransportKind : transport
    ConnectionStatus --> "0..1" DeviceDescriptor : device
    ConnectionStatus --> ConnectionState : state
    ConnectionStatus --> "0..1" DeviceError : error
    DecodedReading --> QualityStatus : quality
    Observation --> QualityStatus : quality

    style DeviceDescriptor fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    style TransportKind fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    style ConnectionStatus fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    style ConnectionState fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    style ReconnectPolicy fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    style ReceivedFrame fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    style DecodedReading fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    style Observation fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    style QualityStatus fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    style DeviceError fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px

    click DeviceDescriptor href "../device-interface/src/types.rs#L36" "Open DeviceDescriptor source or API"
    click TransportKind href "../device-interface/src/types.rs#L31" "Open TransportKind source or API"
    click ConnectionStatus href "../device-interface/src/types.rs#L53" "Open ConnectionStatus source or API"
    click ConnectionState href "../device-interface/src/types.rs#L44" "Open ConnectionState source or API"
    click ReconnectPolicy href "../device-interface/src/types.rs#L61" "Open ReconnectPolicy source or API"
    click ReceivedFrame href "../device-interface/src/types.rs#L68" "Open ReceivedFrame source or API"
    click DecodedReading href "../device-interface/src/types.rs#L83" "Open DecodedReading source or API"
    click Observation href "../device-interface/src/types.rs#L95" "Open Observation source or API"
    click QualityStatus href "../device-interface/src/types.rs#L75" "Open QualityStatus source or API"
    click DeviceError href "../device-interface/src/types.rs#L9" "Open DeviceError source or API"
```

`DecodedReading` and `Observation` are independent structs with matching fields;
neither inherits from the other. Missing timestamps and measurements remain
explicit through `Option`. Device IDs are opaque identifiers, not universally MAC
addresses.

## Source references

These links mirror every clickable diagram node.

| Class / interface | Definition or dependency source |
| --- | --- |
| `BluetoothAdapter` | [BluetoothAdapter](../device-interface/src/bluetooth.rs#L17) |
| `BtleplugAdapter` | [BtleplugAdapter](https://docs.rs/btleplug/0.13.3/btleplug/platform/struct.Adapter.html) |
| `BtleplugPeripheral` | [BtleplugPeripheral](https://docs.rs/btleplug/0.13.3/btleplug/platform/struct.Peripheral.html) |
| `CLI` | [CLI](../device-interface/src/main.rs#L96) |
| `ConnectionManager` | [ConnectionManager](../device-interface/src/connection.rs#L5) |
| `ConnectionState` | [ConnectionState](../device-interface/src/types.rs#L44) |
| `ConnectionStatus` | [ConnectionStatus](../device-interface/src/types.rs#L53) |
| `DecodedPgn` | [DecodedPgn](https://docs.rs/crate/canboat/8.3.0/source/src/engine/decode.rs) |
| `DecodedReading` | [DecodedReading](../device-interface/src/types.rs#L83) |
| `DeviceDescriptor` | [DeviceDescriptor](../device-interface/src/types.rs#L36) |
| `DeviceError` | [DeviceError](../device-interface/src/types.rs#L9) |
| `DeviceProtocolAdapter` | [DeviceProtocolAdapter](../device-interface/src/protocol.rs#L28) |
| `Frame` | [Frame](https://docs.rs/crate/canboat/8.3.0/source/src/engine/frame.rs) |
| `Nmea0183Codec` | [Nmea0183Codec](../device-interface/src/protocol/nmea0183.rs#L60) |
| `Nmea0183Message` | [Nmea0183Message](../device-interface/src/protocol/nmea0183.rs#L16) |
| `Nmea2000Codec` | [Nmea2000Codec](../device-interface/src/protocol/nmea2000.rs#L13) |
| `Observation` | [Observation](../device-interface/src/types.rs#L95) |
| `ObservationNormalizer` | [ObservationNormalizer](../device-interface/src/normalizer.rs#L5) |
| `PgnBuilder` | [PgnBuilder](https://docs.rs/crate/canboat/8.3.0/source/src/engine/encode.rs) |
| `ProtocolDecoder` | [ProtocolDecoder](../device-interface/src/protocol.rs#L11) |
| `ProtocolEncoder` | [ProtocolEncoder](../device-interface/src/protocol.rs#L18) |
| `QualityStatus` | [QualityStatus](../device-interface/src/types.rs#L75) |
| `ReceivedFrame` | [ReceivedFrame](../device-interface/src/types.rs#L68) |
| `ReconnectPolicy` | [ReconnectPolicy](../device-interface/src/types.rs#L61) |
| `TransportAdapter` | [TransportAdapter](../device-interface/src/transport.rs#L6) |
| `TransportKind` | [TransportKind](../device-interface/src/types.rs#L31) |
| `canboat` | [canboat](https://docs.rs/crate/canboat/8.3.0/source/src/lib.rs) |
| `nmea_kit` | [nmea_kit](https://docs.rs/crate/nmea-kit/0.8.9/source/src/lib.rs) |
| `VictronAdvertisementDecoder` | [VictronAdvertisementDecoder](../device-interface/vendor/victron/mod.rs#L135) |
| `VictronAdvertisement` | [VictronAdvertisement](../device-interface/vendor/victron/mod.rs#L20) |
| `VictronMessage` | [VictronMessage](../device-interface/vendor/victron/mod.rs#L30) |
| `VictronRecord` | [VictronRecord](../device-interface/vendor/victron/mod.rs#L49) |
| `BatteryMonitorReadings` | [BatteryMonitorReadings](../device-interface/vendor/victron/mod.rs#L60) |
| `BatteryAuxiliary` | [BatteryAuxiliary](../device-interface/vendor/victron/mod.rs#L52) |
| `VictronError` | [VictronError](../device-interface/vendor/victron/mod.rs#L94) |

- [Codec behavior tests](../device-interface/tests/nmea_codecs.rs)
- [Build and usage instructions](../device-interface/README.md)

The operational BLE adapter is not yet wired into the synchronous transport trait
or `ConnectionManager`. Notification reception, integration of the implemented
NMEA codecs into the transport pipeline, and the React Native bridge remain future
work. Victron advertisement monitoring, normalization and local storage are implemented.

## NMEA codec interfaces

See the [NMEA library usage guide](dependencies/rust-nmea-libraries.md) for examples and library
configuration.

These transport-independent codecs operate on complete protocol messages, before
conversion into `DecodedReading`. The existing `DeviceProtocolAdapter` stub is a
separate normalization integration point.

```mermaid
classDiagram
    direction LR
    class ProtocolDecoder {
        <<interface>>
        +decode(input) DeviceResult~Message~
    }
    class ProtocolEncoder {
        <<interface>>
        +encode(message) DeviceResult~Output~
    }
    class Nmea0183Codec {
        <<struct>>
        +decode(input: str) DeviceResult~Nmea0183Message~
        +encode(message: Nmea0183Message) DeviceResult~String~
    }
    class Nmea0183Message {
        <<struct>>
        -String line
        -NmeaSentence sentence
        +sentence() NmeaSentence
        +frame() NmeaFrame
        +as_str() str
        +from_sentence(talker: str, sentence: T) DeviceResult~Self~
    }
    class Nmea2000Codec {
        <<struct>>
        -Database database
        +new(units: Units) Self
        +message(schema_id: str) DeviceResult~PgnBuilder~
        +decode(input: Frame) DeviceResult~DecodedPgn~
        +encode(message: PgnBuilder) DeviceResult~Frame~
    }
    class nmea_kit {
        <<external>>
    }
    class canboat {
        <<external>>
    }
    class Frame {
        <<external>>
        +Option~String~ timestamp
        +u8 prio
        +u32 pgn
        +u8 src
        +u8 dst
        +SmallVec data
    }
    class PgnBuilder {
        <<external>>
    }
    class DecodedPgn {
        <<external>>
    }
    ProtocolDecoder <|.. Nmea0183Codec
    ProtocolEncoder <|.. Nmea0183Codec
    ProtocolDecoder <|.. Nmea2000Codec
    ProtocolEncoder <|.. Nmea2000Codec
    Nmea0183Codec ..> Nmea0183Message : decodes and encodes
    Nmea0183Codec ..> nmea_kit : parses and formats
    Nmea0183Message ..> nmea_kit : typed sentence and frame view
    Nmea2000Codec ..> canboat : embedded database
    Nmea2000Codec ..> Frame : decodes and returns
    Nmea2000Codec ..> PgnBuilder : creates and encodes
    Nmea2000Codec ..> DecodedPgn : returns

    note for Nmea0183Codec "Complete: whole-sentence codec. AIS payload assembly is outside scope."
    note for Nmea2000Codec "Complete: whole-PGN codec. CAN transport and fragmentation are outside scope."

    style ProtocolDecoder fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    style ProtocolEncoder fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    style Nmea0183Codec fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    style Nmea0183Message fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    style Nmea2000Codec fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    style nmea_kit fill:#f1f5f9,stroke:#64748b,color:#334155,stroke-width:1px
    style canboat fill:#f1f5f9,stroke:#64748b,color:#334155,stroke-width:1px
    style Frame fill:#f1f5f9,stroke:#64748b,color:#334155,stroke-width:1px
    style PgnBuilder fill:#f1f5f9,stroke:#64748b,color:#334155,stroke-width:1px
    style DecodedPgn fill:#f1f5f9,stroke:#64748b,color:#334155,stroke-width:1px

    click ProtocolDecoder href "../device-interface/src/protocol.rs#L11" "Open ProtocolDecoder source or API"
    click ProtocolEncoder href "../device-interface/src/protocol.rs#L18" "Open ProtocolEncoder source or API"
    click Nmea0183Codec href "../device-interface/src/protocol/nmea0183.rs#L60" "Open Nmea0183Codec source or API"
    click Nmea0183Message href "../device-interface/src/protocol/nmea0183.rs#L16" "Open Nmea0183Message source or API"
    click Nmea2000Codec href "../device-interface/src/protocol/nmea2000.rs#L13" "Open Nmea2000Codec source or API"
    click nmea_kit href "https://docs.rs/crate/nmea-kit/0.8.9/source/src/lib.rs" "Open nmea_kit source or API"
    click canboat href "https://docs.rs/crate/canboat/8.3.0/source/src/lib.rs" "Open canboat source or API"
    click Frame href "https://docs.rs/crate/canboat/8.3.0/source/src/engine/frame.rs" "Open Frame source or API"
    click PgnBuilder href "https://docs.rs/crate/canboat/8.3.0/source/src/engine/encode.rs" "Open PgnBuilder source or API"
    click DecodedPgn href "https://docs.rs/crate/canboat/8.3.0/source/src/engine/decode.rs" "Open DecodedPgn source or API"
```

0183 decodes `str` into an owned `Nmea0183Message` and encodes that message to a
checksummed CRLF string. Typed sentences construct outbound messages through
`Nmea0183Message::from_sentence`. 2000 decodes a complete CANboat `Frame` into
`DecodedPgn` and encodes a `PgnBuilder` into a complete `Frame`. Transport framing,
CAN reassembly/fragmentation and device I/O are outside both interfaces.

## Victron vendor decoding

The implementation is located directly under `device-interface/vendor/victron`.
The decoder consumes manufacturer advertisements for one configured source, with
its VictronConnect advertisement key. This is a separate protocol from NMEA.
Green indicates completed decoding for battery-monitor record 0x02. The foreground
monitor receives advertisements, reloads a private key file, normalizes readings
and persists them to SQLite. Mobile secure storage and application integration
remain pending. No live-device interoperability has been verified.

```mermaid
classDiagram
    direction LR
    class ProtocolDecoder {
        <<interface>>
        +decode(input) DeviceResult~Message~
    }
    class VictronAdvertisementDecoder {
        <<struct>>
        -String source_id
        -Option key
        +new(source_id, key_hex) Result~Self~
        +replace_key(key_hex) Result
        +decode_advertisement(input) Result~VictronMessage~
        +decode(input) DeviceResult~VictronMessage~
    }
    class VictronAdvertisement {
        <<struct>>
        +String source_id
        +SystemTime received_at
        +u16 company_id
        +Vec~u8~ payload
    }
    class VictronMessage {
        <<struct>>
        +String source_id
        +SystemTime received_at
        +u16 product_id
        +u16 data_counter
        +VictronRecord record
    }
    class VictronRecord {
        <<type alias>>
        BatteryMonitor
    }
    class BatteryMonitorReadings {
        <<struct>>
        +Option~u16~ time_to_go_minutes
        +Option~f64~ battery_voltage_v
        +Option~f64~ battery_current_a
        +Option~f64~ consumed_ah
        +Option~f64~ state_of_charge_percent
        +u16 alarm_reason
        +BatteryAuxiliary auxiliary
    }
    class BatteryAuxiliary {
        <<enumeration>>
        Voltage Option~f64~
        MidpointVoltage Option~f64~
        TemperatureKelvin Option~f64~
        Disabled
    }
    class VictronError {
        <<enumeration>>
        InvalidKey
        WrongSource
        WrongCompany
        UnsupportedAdvertisement
        UnsupportedRecord
        Truncated
        KeyMismatch
        KeyRequired
        CipherFailure
    }
    ProtocolDecoder <|.. VictronAdvertisementDecoder
    VictronAdvertisementDecoder ..> VictronAdvertisement : consumes
    VictronAdvertisementDecoder ..> VictronMessage : returns
    VictronAdvertisementDecoder ..> VictronError : reports
    VictronMessage --> VictronRecord : record
    VictronRecord --> BatteryMonitorReadings : BatteryMonitor record 0x02
    BatteryMonitorReadings --> BatteryAuxiliary : selected auxiliary mode
    note for VictronAdvertisementDecoder "Key mismatch clears the key. AES-CTR does not authenticate readings."
    style ProtocolDecoder fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    click ProtocolDecoder href "../device-interface/src/protocol.rs#L11" "Open ProtocolDecoder source"
    style VictronAdvertisementDecoder fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    click VictronAdvertisementDecoder href "../device-interface/vendor/victron/mod.rs#L135" "Open VictronAdvertisementDecoder source"
    style VictronAdvertisement fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    click VictronAdvertisement href "../device-interface/vendor/victron/mod.rs#L20" "Open VictronAdvertisement source"
    style VictronMessage fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    click VictronMessage href "../device-interface/vendor/victron/mod.rs#L30" "Open VictronMessage source"
    style VictronRecord fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    click VictronRecord href "../device-interface/vendor/victron/mod.rs#L49" "Open record alias"
    style BatteryMonitorReadings fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    click BatteryMonitorReadings href "../device-interface/vendor/victron/mod.rs#L60" "Open battery readings"
    style BatteryAuxiliary fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    click BatteryAuxiliary href "../device-interface/vendor/victron/mod.rs#L52" "Open auxiliary enum"
    style VictronError fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    click VictronError href "../device-interface/vendor/victron/mod.rs#L94" "Open VictronError source"
```

See the [vendor guide](../device-interface/vendor/README.md) and
[fixture tests](../device-interface/tests/bmv_decoder.rs) for supported layouts,
key lifecycle, error handling and limits.

## Live monitoring and local storage

See the [monitoring guide](victron-live-monitoring.md) for commands, key setup,
recovery behavior and the physical validation checklist. Green processing and
storage components are tested in software; amber radio components await physical
validation. `MonitorRunner` represents the async supervision function.

```mermaid
classDiagram
    direction LR
    class MonitorRunner
    class MonitorConfig
    class AdvertisementSource
    class BluetoothAdvertisementSource
    class AdvertisementScan
    class VictronMonitor
    class VictronAdvertisementDecoder
    class ObservationNormalizer
    class ObservationStore
    class MonitorReport
    class MonitorState
    class StoredObservation
    MonitorRunner --> MonitorConfig
    MonitorRunner --> AdvertisementSource : supervises and retries
    AdvertisementSource <|.. BluetoothAdvertisementSource
    BluetoothAdvertisementSource --> AdvertisementScan : consumes events
    MonitorRunner --> VictronMonitor : supplies advertisements
    VictronMonitor --> VictronAdvertisementDecoder : decrypts and decodes
    VictronMonitor --> ObservationNormalizer : validates readings
    VictronMonitor --> ObservationStore : commits sample atomically
    VictronMonitor --> MonitorReport : reports transitions and samples
    MonitorReport --> MonitorState
    ObservationStore --> StoredObservation : reads history
    style MonitorRunner fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    click MonitorRunner href "../device-interface/src/monitor.rs#L359" "Open MonitorRunner source"
    style MonitorConfig fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    click MonitorConfig href "../device-interface/src/monitor.rs#L23" "Open MonitorConfig source"
    style AdvertisementSource fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    click AdvertisementSource href "../device-interface/src/monitor.rs#L266" "Open AdvertisementSource source"
    style BluetoothAdvertisementSource fill:#fef3c7,stroke:#92400e,color:#78350f,stroke-width:2px
    click BluetoothAdvertisementSource href "../device-interface/src/monitor.rs#L271" "Open BluetoothAdvertisementSource source"
    style AdvertisementScan fill:#fef3c7,stroke:#92400e,color:#78350f,stroke-width:2px
    click AdvertisementScan href "../device-interface/src/bluetooth.rs#L162" "Open AdvertisementScan source"
    style VictronMonitor fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    click VictronMonitor href "../device-interface/src/monitor.rs#L65" "Open VictronMonitor source"
    style VictronAdvertisementDecoder fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    click VictronAdvertisementDecoder href "../device-interface/vendor/victron/mod.rs#L135" "Open VictronAdvertisementDecoder source"
    style ObservationNormalizer fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    click ObservationNormalizer href "../device-interface/src/normalizer.rs#L5" "Open ObservationNormalizer source"
    style ObservationStore fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    click ObservationStore href "../device-interface/src/store.rs#L18" "Open ObservationStore source"
    style MonitorReport fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    click MonitorReport href "../device-interface/src/monitor.rs#L57" "Open MonitorReport source"
    style MonitorState fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    click MonitorState href "../device-interface/src/monitor.rs#L48" "Open MonitorState source"
    style StoredObservation fill:#dcfce7,stroke:#166534,color:#14532d,stroke-width:2px
    click StoredObservation href "../device-interface/src/store.rs#L23" "Open StoredObservation source"
```

| Component | Source |
| --- | --- |
| `MonitorRunner` | [MonitorRunner](../device-interface/src/monitor.rs#L359) |
| `MonitorConfig` | [MonitorConfig](../device-interface/src/monitor.rs#L23) |
| `AdvertisementSource` | [AdvertisementSource](../device-interface/src/monitor.rs#L266) |
| `BluetoothAdvertisementSource` | [BluetoothAdvertisementSource](../device-interface/src/monitor.rs#L271) |
| `AdvertisementScan` | [AdvertisementScan](../device-interface/src/bluetooth.rs#L162) |
| `VictronMonitor` | [VictronMonitor](../device-interface/src/monitor.rs#L65) |
| `VictronAdvertisementDecoder` | [VictronAdvertisementDecoder](../device-interface/vendor/victron/mod.rs#L135) |
| `ObservationNormalizer` | [ObservationNormalizer](../device-interface/src/normalizer.rs#L5) |
| `ObservationStore` | [ObservationStore](../device-interface/src/store.rs#L18) |
| `MonitorReport` | [MonitorReport](../device-interface/src/monitor.rs#L57) |
| `MonitorState` | [MonitorState](../device-interface/src/monitor.rs#L48) |
| `StoredObservation` | [StoredObservation](../device-interface/src/store.rs#L23) |

## BMV-712 beta battery data flow

The [Windows BMV simulator](../simulation/bmv712/README.md) broadcasts encrypted
battery-monitor advertisements. Independent charge/discharge fixtures and a
user-transcribed iPhone packet verify decoding; physical BMV validation and mobile
UI integration remain pending. The simulator now broadcasts auxiliary starter voltage; independent fixture checks
and a Windows radio run passed on October 6. Mobile UI integration remains pending.

The beta uses one permanently connected house shunt and the monitor's auxiliary
input configured for starter voltage. A planned House / Starter UI selector changes
views, not electrical connections. No starter current or SOC is inferred.

| Normalized quantity | Unit | Beta use |
| --- | --- | --- |
| battery_voltage | V | House voltage |
| battery_current | A | House net current; positive charging, negative discharging |
| state_of_charge | % | House charge percentage |
| auxiliary_voltage | V | Starter voltage when auxiliary mode is Voltage |
| consumed_charge | Ah | House consumed charge, negative value |
| time_to_go | min | Monitor time remaining estimate |
| alarm_reason | bitmask | Monitor alarms |

Missing measurements retain Unknown quality and SQLite NULL values. The decoder
also distinguishes midpoint voltage and temperature in kelvin from auxiliary
voltage. Battery-monitor samples persist with record_type 2; duplicate suppression
and key lifecycle use the existing monitor. `VictronRecord` is the public alias of
the existing shared record enum; only its BatteryMonitor variant is shown here.
