# BMV-712 Smart telemetry simulator

Native Windows PowerShell 5.1 BLE broadcaster for prototyping battery voltage,
current and state-of-charge UI. No physical BMV, real device key or charger is
required. Shares the Windows publisher with the Orion simulator; the Rust Orion
CLI remains Orion-only. The production decoder and monitor now support battery
records and SQLite ingestion; the Windows broadcaster does not itself write SQLite.
An iPhone app and GATT pairing/control are not implemented.

In Windows PowerShell, open this directory and run:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\broadcast-windows.ps1 -CheckFixture
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\broadcast-windows.ps1 -DurationSeconds 300
```

The project may be accessed at
`\\wsl.localhost\Ubuntu-24.04\home\amanlig\Projects\aiNavLog\kodekomplete\aiNavLog\simulation\bmv712`.
Windows must own the Bluetooth adapter and support BLE advertising. Ctrl-C or
expiry stops advertising. Startup failure is reported rather than treated as success.

## Readings

First packet: **house 12.76 V, -5.25 A, 77.5%; starter 12.80 V**. Negative current means discharge.
A 120-step cycle uses 60 discharge steps followed by 60 charging steps (+12 A,
about 14 V). SOC changes in 0.1% increments. Consumed Ah corresponds to an
illustrative 200 Ah bank; time-to-go is a fixed illustrative 120 minutes. No alarms;
auxiliary input is configured for starter voltage (mode 0). Starter voltage varies
from about 12.8 V in the discharge phase to about 14.2 V in the charging phase.
Starter current and SOC are unavailable. This is a scripted demonstration, not a physical
battery model, and the repeating SOC transition is synthetic.

## Wireless format

Uses Victron's published Battery Monitor record `0x02`, AES-128 CTR, public key
`000102030405060708090a0b0c0d0e0f`, manufacturer ID `0x02E1` and deliberately
synthetic product ID `0x1234`. It does not claim the identity of a real BMV-712.
The record includes packed signed 22-bit milliamps and 10-bit SOC (0.1%).
Reserved bits are set as required by the specification.

In nRF Connect/LightBlue, scan in the foreground including unnamed devices.
Manufacturer data starts `10 02 34 12 02`, optionally prefixed by company bytes
`E1 02`. The following two bytes are the changing little-endian counter, followed
by key-check byte `00` and 16 encrypted bytes. The scanner displays hex, not
automatically decoded volts/amps/percentage. Windows prints those values alongside
each packet for comparison. Stop/start cycles between updates produce radio gaps.

Protocol source: [Victron Extra Manufacturer Data, Battery Monitor table](https://communityarchive.victronenergy.com/storage/attachments/extra-manufacturer-data-2022-12-14.pdf).

## Independent fixtures

`-CheckFixture` verifies discharge, charging and starter-voltage ciphertexts generated with
OpenSSL, not an encoder/decoder round trip. Both use counter `0x1234`; encryption:

```sh
openssl enc -aes-128-ctr -K 000102030405060708090a0b0c0d0e0f \
  -iv 34120000000000000000000000000000 -nopad
```

Plaintexts (bytes before encryption):

| Fixture | Plaintext hex | Voltage / current / SOC |
|---|---|---|
| fixture.hex | `7800fc040000fffffbadffc20170f0ff` | 12.76 V / -5.25 A / 77.5% |
| charging-fixture.hex | `780078050000ffff83bb00da01b0efff` | 14.00 V / +12 A / 76.3% |

Files contain the header `10 02 34 12 02 34 12 00` followed by ciphertext.
Nonce/key reuse is public development data, not secure deployment behavior.

Validation: October 5 iPhone reception of house-only BMV packets was demonstrated.
October 6 independent fixture checks verify starter-voltage encoding, and a short
Windows radio run reports Started with both bank readings. iPhone reception of the
new auxiliary field remains to be tested. October 6 browser verification passed
for captured simulator packets through Rust decoding/storage and the local service
to the rendered aiNavLog electrical UI; Bluetooth reception was bypassed.

Starter-voltage fixture: `device-interface/tests/fixtures/victron/bmv_starter.bin`,
with 12.76 V house, -5.25 A, 77.5% SOC and 12.80 V auxiliary starter voltage.
Legacy charge/discharge fixtures retain auxiliary Disabled for regression coverage.

## Device-interface → UI replay

`integration-packets.log` records 25 encrypted packets printed by this broadcaster
on October 6, 2026. It contains only the public development simulation.
The `device-interface/examples/replay_bmv.rs` example ingests those exact bytes
through the production monitor/decoder and SQLite with current receipt timestamps.
See [the root startup and browser-test instructions](../../README.md#demonstrate-simulator-data-reaching-the-ui).
This replay bypasses radio reception. The service must use `--simulated` so the UI
labels the readings as simulated.
