# Orion telemetry simulator

In-process simulation of Orion-Tr DC/DC (`0x04`) and Orion XS (`0x0f`) Instant
Readout advertisements. No Orion, real key, Bluetooth adapter or pairing is needed.
The public synthetic key and product ID match the independent test fixtures.
This models telemetry, not charger firmware, electrical dynamics or battery safety.

From the `aiNavLog/` directory:

```sh
cargo run --manifest-path simulation/orion/Cargo.toml --locked -- --model tr
cargo run --manifest-path simulation/orion/Cargo.toml --locked -- --model xs --scenario faults --interval-ms 100
cargo run --manifest-path simulation/orion/Cargo.toml --locked -- --model xs --scenario normal --steps 60
```

Default: 24 steps, one second each, fault scenario. Ctrl-C stops early. Use `--help`
for options. Relative explicit paths are relative to the working directory.
Default databases are unique files under `simulation/orion/data/`; every supplied
`--database PATH` must also be a new file. All observations use `SIMULATED-ORION`,
never the real device ID. The output prints the database path for inspection:

```sh
cargo run --manifest-path device-interface/Cargo.toml --locked -- observations /path/to/simulation.sqlite3 100
```

## Pipeline and scenario

### Windows-to-iPhone radio discovery test

`broadcast-windows.ps1` is a separate native Windows PowerShell 5.1 broadcaster.
It emits synthetic TR packets using the same public key and synthetic product ID,
with increasing counters, varying input voltage and constant 14.4 V output. It
does not run the Rust monitor, write SQLite, expose GATT services or implement the
fault scenario. The existing Rust CLI remains an in-process simulator.

From Windows PowerShell, in this directory (including via the WSL network path):

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\broadcast-windows.ps1 -CheckFixture
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\broadcast-windows.ps1 -DurationSeconds 300
```

Keep Bluetooth enabled in Windows and the adapter owned by Windows. The script
requires an adapter/driver supporting BLE advertisements. It checks publisher
startup and stops on failure; Ctrl-C or duration expiry stops broadcasting.
Packets are replaced once per second with a stop/start cycle, so gaps are expected.

On the iPhone, keep a BLE scanner such as nRF Connect or LightBlue in the foreground
and allow Bluetooth access. Scan all devices, including unnamed/nonconnectable
entries. Look for manufacturer ID `0x02E1` (Victron) and manufacturer payload
beginning `10 02 34 12 04`; scanners showing the company ID inline will prefix
`E1 02`. The following two bytes are the changing counter. Do not rely on a device
name, the real Orion MAC address, or the iPhone Bluetooth pairing screen.

Pass criteria: Windows reports `Started`, the iPhone receives the matching
manufacturer data, and repeated observations show changing counters. Stopping the
publisher should stop fresh receptions (the scanner may retain a cached entry).
Publisher startup alone does not prove iPhone reception or telemetry decoding.

Windows validation on 2026-10-05: the encoder matched the independent TR fixture
and the Intel adapter reported publisher `Started`. iPhone reception awaits user
confirmation. See [Microsoft's advertisement API](https://learn.microsoft.com/en-us/windows/apps/develop/devices-sensors/ble-beacon)
and [Apple's advertisement data API](https://developer.apple.com/documentation/corebluetooth/advertising-data).

The simulator serializes readings in protocol wire units and encrypts the payload
with AES-128 CTR. It passes the advertisement to the production `VictronMonitor`,
which decrypts, normalizes and persists through `ObservationStore`. It calls the
production freshness timer and explicitly stops the monitor at completion.
The Bluetooth adapter and scan supervisor are bypassed; this does not test radio
reception, scanner retries or mobile lifecycle behavior.

| Zero-based step | Fault scenario |
| --- | --- |
| 3 | Repeat preceding packet; no duplicate stored sample |
| 5 | Unavailable measurements become NULL/Unknown |
| 6 | Truncated encrypted packet rejected; run continues |
| 8–12 | No advertisements; stale timeout is three intervals |
| 13 | Reception resumes |
| 14 | Wrong key-check byte causes KeyRequired |
| 15 | Packet ignored while a key is required |
| 16 | Explicitly reload synthetic key and recover |
| 18 | Synthetic off/error status |

Other steps emit changing voltage/current and bulk/absorption/float state codes.
TR omits current fields. The fault sequence occurs once; `normal` omits all injected
faults. The error code and off-reason are illustrative, not a diagnosis of a real
charger. Key recovery uses the core API, not real key-file provisioning. Keys and
nonce reuse are public test data and must not be used for secure transmission.

## Validation

```sh
cargo test --manifest-path simulation/orion/Cargo.toml --locked
cargo clippy --manifest-path simulation/orion/Cargo.toml --locked --all-targets -- -D warnings
```

Encryption is checked against committed, independently generated OpenSSL fixtures
for both record formats, not just an encoder/decoder round trip. A production-monitor
test checks storage, duplicates, stale detection, key recovery and missing values.
Build requirements are inherited from `device-interface`, including a C compiler
for bundled SQLite and Linux vendored D-Bus. No system Bluetooth service is needed
to run this simulator. This development crate has its own Cargo.lock; it adds no
new library families beyond the [device-interface inventory](../../docs/dependencies/library-licenses.md).

## Components

```mermaid
classDiagram
    class Model
    class Readings
    class SimulatorCLI
    class VictronMonitor
    class ObservationStore
    SimulatorCLI --> Model
    SimulatorCLI --> Readings : builds encrypted advertisement
    SimulatorCLI --> VictronMonitor : ingest and tick
    VictronMonitor --> ObservationStore : persist
    style Model fill:#dcfce7,stroke:#166534
    style Readings fill:#dcfce7,stroke:#166534
    style SimulatorCLI fill:#dcfce7,stroke:#166534
    style VictronMonitor fill:#dcfce7,stroke:#166534
    style ObservationStore fill:#dcfce7,stroke:#166534
    click Model "src/lib.rs" "Model definition"
    click Readings "src/lib.rs" "Readings and wire encoder"
    click SimulatorCLI "src/main.rs" "Simulation driver"
    click VictronMonitor "../../device-interface/src/monitor.rs" "Production monitor"
    click ObservationStore "../../device-interface/src/store.rs" "Production store"
```

Green means implemented within this in-process scope. Physical Orion validation
remains shelved. See the [handoff](../../docs/status/status-2026-10-02.md).

The CLI integration test executes the full XS fault scenario, verifies 15 samples
(105 observations, four NULL values), and confirms that rerunning against an
existing database fails without changing its observations.
