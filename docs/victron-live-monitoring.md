# Live Victron monitoring and local storage

The foreground monitoring path is implemented for Orion-Tr Smart/DC-DC and Orion
XS Instant Readout advertisements. It uses Bluetooth scanning, the vendor decoder,
observation normalization and SQLite. It does not connect to the charger or send
control commands. NMEA streaming remains a separate integration.

> **2026-10-02:** Orion key setup and physical validation are shelved at the
> user's request until the device is nearby. The instructions below are retained
> for resuming later; see the [current handoff](status/status-2026-10-02.md).

## Run against the recorded device

The device ID recorded in this project is **`D4:B3:CB:26:5E:75`**. IDs are local to
the Bluetooth backend; use `discover` if a different OS exposes another ID.

1. Enable Instant Readout in VictronConnect and obtain the device's advertisement
   key. This is not the Bluetooth PIN.
2. Put the 32 hexadecimal characters in a private local text file outside the
   repository, for example `orion.key` in your private configuration directory.
   Do not put the key itself in a command argument or commit it. On Linux/macOS,
   apply `chmod 600 /path/to/orion.key`; on Windows restrict the file's ACL to your
   user. This CLI uses a key file; mobile Keychain/Keystore integration is not built.
3. From `device-interface`, start the foreground monitor:

```sh
cargo run --locked -- monitor-victron 'D4:B3:CB:26:5E:75' \
  --key-file /path/to/orion.key \
  --database data/observations.sqlite3
```

For a native Windows build, in PowerShell:

```powershell
.\build-windows.ps1
.\target\x86_64-pc-windows-msvc\release\ainavlog-device-interface.exe monitor-victron 'D4:B3:CB:26:5E:75' --key-file "$env:LOCALAPPDATA\aiNavLog\orion.key" --database '.\data\observations.sqlite3'
```

The PowerShell example assumes you have created the private key file. Building on
Windows requires Rust MSVC and Visual Studio C++ Build Tools/Windows SDK; bundled
SQLite needs the native C compiler. Do not use an old executable built before the
monitor command was added.

Optional arguments:

| Option | Default | Meaning |
| --- | --- | --- |
| `--adapter` | `0` | Local adapter index |
| `--stale-seconds` | `10` | Time without a new valid sample before stale status |
| `--retry-seconds` | `3` | Initial retry delay; repeated failures back off to 60 seconds |
| `--duration-seconds` | Unlimited | Stop automatically after this foreground duration |
| `--database` | `data/observations.sqlite3` | Local SQLite file; parent directories are created |

Press Ctrl-C to stop scanning and close the store. Startup and scan shutdown have
bounded timeouts. A database-write failure stops the monitor; it never reports a
failed transaction as a stored sample. Keep only one monitor process per selected
adapter: scan lifecycle ownership is not coordinated across separate processes.

## Reception and recovery

The adapter subscribes to events before starting an unfiltered BLE scan. It reads
live manufacturer-data events under company ID `0x02e1`; cached peripheral
properties are not treated as fresh telemetry. Only the selected source ID enters
the decoder. Address/UUID matching is case-insensitive at the monitor boundary.

The console shows `Waiting`, `Live`, `Stale`, `Recovering`, `KeyRequired` and
`Stopped`. Exact duplicate packets do not produce extra database rows or refresh
measurements. A bounded recent-packet cache suppresses short-term retransmissions;
consecutive identical packets stay suppressed even after the freshness timeout.
A changed valid packet restores `Live`. The data counter can wrap and is not used
as a timestamp or a permanent database uniqueness key.

Power-off/stream failures trigger cleanup and retries. If the selected device
stays silent, the scanner restarts after the greater of 60 seconds or three times
the stale timeout. This can recover a silently stopped backend, but cannot make
an unavailable device transmit. Malformed packets are reported and skipped;
identical consecutive warnings are suppressed. Protocol errors do not count as
valid samples.

The key file is checked once per second. Replace its contents (preferably by
atomic file replacement preserving private permissions) to load a new key.
A key-check mismatch stops decoding and storing until the file contains a changed,
valid key. A missing or malformed replacement file is reported without replacing
the existing in-memory key. Keys are not printed or stored in SQLite.

Victron AES-CTR broadcasts are unauthenticated. A matching key-check byte cannot
prove the complete key or sender is correct. Compare real readings with
VictronConnect before relying on a device's decoded data. `Good` quality means a
present decoded measurement; it is not a cryptographic-authenticity assertion.

## Local store

The SQLite database uses WAL mode and a versioned application schema. One transaction
stores a sample and all of its observations. The library rejects unrelated or
newer-schema databases rather than modifying them.

| Table | Contents |
| --- | --- |
| `samples` | Source ID, receipt timestamp in Unix milliseconds, product ID, counter, record type, raw encrypted manufacturer bytes |
| `observations` | Sample reference, quantity, unit, nullable numeric value, ingestion quality and optional instrument timestamp |
| `monitor_events` | Receipt/status times, source, state transitions and diagnostic details |

DC/DC observations include input/output voltage, device state, charger error and
off-reason bitmask. XS adds input/output current. Missing measurements use SQL
`NULL` and `Unknown` quality. Unknown codes/bits are preserved. No instrument
measurement timestamp is invented: `observed_at_ms` is null for these broadcasts.

Historical quality is never overwritten merely because a device later disappears.
Use receipt time to calculate age; monitor events describe the running session.
If the process is forcibly killed, its last stored state may be `Live`, so consumers
must not infer current freshness from that state alone.

Read stored observations without Bluetooth access:

```sh
cargo run --locked -- observations data/observations.sqlite3 25
```

This command opens the existing database read-only and lists historical rows.
The limit applies to observations, not whole samples. You can also inspect it with
any SQLite tool:

```sql
SELECT s.id, s.source_id, s.received_at_ms, o.quantity, o.value, o.unit, o.quality
FROM samples AS s JOIN observations AS o ON o.sample_id = s.id
ORDER BY s.id DESC, o.quantity;
```

The store is local and unencrypted, with no automatic retention or upload. Protect
and back it up as application data. Stop the monitor before copying it, or use a
SQLite-aware backup tool so WAL data is included. Real device keys must not be
placed in test fixtures. Raw packet captures can support private debugging; only
synthetic or intentionally sanitized captures belong in the repository.

## Validation status and hardware procedure

Automated tests cover independent ciphertext decoding, source filtering, nullable
values, SQLite reopening/rollback, duplicate/freshness rules, startup and stream
recovery, key-file changes, malformed packets, fatal storage errors, and shutdown
while initialization is pending. They use fake advertisement sources and temporary
stores, not a physical radio.

The live preflight in this WSL environment failed with:

```text
The name org.bluez was not provided by any .service files
```

Windows PowerShell is available, but native Windows Cargo was not found on its
PATH. Therefore no Windows executable was rebuilt or Orion telemetry compared in
this environment. A valid 32-character hexadecimal advertisement key has not been supplied to this process. A six-digit value cannot be used as that key.

To finish physical validation on your Bluetooth-capable host:

1. Verify the selected ID and enable Instant Readout.
2. Run the monitor and confirm both console values and newly inserted SQLite rows.
3. Compare input/output voltages, state and errors with VictronConnect; for XS,
   compare both currents. Record model, firmware, OS and observed differences.
4. Move out of range or turn the charger off; verify stale status. Restore it and
   verify fresh samples resume without restarting the application.
5. Disable/re-enable the host's Bluetooth; verify scan recovery and no false samples.
6. Test an incorrect key and then replace the file with the correct key; verify
   the monitor requires a key and recovers. Do not change charger settings merely
   to create a fault for testing.
7. Stop with Ctrl-C, restart, and verify history remains readable and new samples
   append. Confirm no key appears in output or database tables.

The iOS/Android app bridge, OS permission UI, secure credential-store integration,
background execution and platform/device validation remain pending. This work
provides the Rust foreground service and local store for that later app integration.

## Code

- [BLE scan lifecycle](../device-interface/src/bluetooth.rs)
- [Monitor supervisor and processing](../device-interface/src/monitor.rs)
- [Normalization](../device-interface/src/normalizer.rs)
- [SQLite store](../device-interface/src/store.rs)
- [CLI](../device-interface/src/monitor_cli.rs)
- [Integration tests](../device-interface/tests/live_monitor.rs)
- [Vendor protocol guide](../device-interface/vendor/README.md)

## Development without Orion hardware

Use the [Orion simulator](../simulation/orion/README.md) to exercise decoding,
freshness and SQLite storage with synthetic advertisements. No private key is
needed. It uses a separate, newly created database and does not test Bluetooth.
