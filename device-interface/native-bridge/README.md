# Native Rust bridge

This crate exposes the shared Device Interface monitor and SQLite store through
[a C ABI](include/ainavlog.h). It builds static and shared libraries without the
desktop Bluetooth/D-Bus or NMEA dependencies. It is a host-tested boundary, not
an installed Expo module or a verified iOS/Android build.

The platform adapter opens a session with its selected source identity, private
database path and advertisement key. It submits manufacturer data with the
company ID supplied separately and the original receipt timestamp. Rust filters,
decodes, normalizes and atomically persists observations. Reading the latest BMV
sample returns four measurements, a presence mask and the original timestamp.
Absent measurements must become JavaScript `null`; polling must never refresh
the receipt time. The caller retains source identity and live/simulated metadata.

Calls are synchronous and globally serialized. Run them on a background queue.
Pointers must obey the header's lifetime/storage contract. Rust owns session
memory and returns numeric handles; closed handles cannot be reused. Panics are
contained when Rust uses unwind panic behavior; invalid foreign pointers remain
the caller's responsibility. Errors expose only status codes, never key text.
The native adapter must clear its key input buffer after opening. To change a
key, close the session and reopen it with the replacement key.

Run the Linux C integration check from this directory:

```sh
python3 tests/run-smoke.py
```

It compiles a real C caller against the shared library and submits captured
simulator ciphertext. It checks production decoding, source filtering, duplicate
filtering, original timestamps, values, reopening persisted data and invalid
handles. The test bypasses Bluetooth reception and does not exercise the UI.

Next: package target libraries, add Swift/Kotlin/JNI Expo adapters and platform
scanning/permission handling, then connect the UI data provider. Phone reception,
background behavior and signed installs still require device validation.
