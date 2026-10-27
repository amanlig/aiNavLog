# Native Rust bridge

This crate exposes the shared Device Interface monitor and SQLite store through
[a C ABI](include/ainavlog.h). Default builds omit Bluetooth and NMEA dependencies.
The `mobile` feature adds the existing Rust btleplug adapter and a foreground
Windows BMV simulator scanner; phone targets use their native btleplug backends,
without Linux D-Bus. Host tests do not establish phone radio reception.

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

It compiles a real C caller against the shared library and submits the committed
independent starter-voltage ciphertext fixture. It checks production decoding, source filtering, duplicate
filtering, original timestamps, values, reopening persisted data and invalid
handles. The test bypasses Bluetooth reception and does not exercise the UI.

## iOS library packaging

On macOS with full Xcode selected and Rust installed:

```sh
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
python3 build-ios.py
```

This builds the locked release crate separately for physical iPhones and Apple
Silicon simulators, then packages static libraries, the C header and a Swift
import module map as `builds/AiNavLogBridge.xcframework`. Intel simulators are not
included. An existing output is preserved; choose another `--output` for a new
package. Temporary build outputs are removed after success or failure.
The script requires a Mac; no Apple-target build has been validated on Linux.
Packaging does not install the framework in Expo, sign an app or validate phone
reception. The consuming native target must link the required Apple system
libraries reported by Rust, including the SQLite crate's platform requirements.

## Expo mobile integration

`user-interface/modules/ainavlog` contains Swift and Kotlin/JNI wrappers. These
connect Expo to the C ABI; all scanning is in Rust btleplug. `ainavlog_simulator_start`
opens private SQLite storage and starts a cancellable worker. It accepts only the
Windows simulator's synthetic product/record and public key. Matching simulator
packets retain a logical source across OS radio-identity changes; run one simulator
at a time. It never automatically selects a real BMV. Session diagnostics expose
packet, stored-update and identity-change counters through the native wrappers.
`stop` cancels startup/reception, stops the scan and closes the session. Silent or
failed reception triggers a retry. Receipt timestamps, not scanner status or UI
poll times, determine freshness.

The app scopes EAS uploads to `user-interface`. `native:prepare` copies shared
Rust source into that boundary; its generated copy is ignored by Git. The EAS
pre-install hook builds iOS device/simulator libraries or Android ARM64/x86_64
shared libraries, and copies the locked btleplug Java backend into Android's
source set. Build artifacts precede CocoaPods/Gradle setup. iOS/Android builds
need platform compilation and physical validation separately.

iOS artifacts live under the pod root at `modules/ainavlog/ios/builds`, so
CocoaPods discovers the vendored XCFramework and notices. The builder compiles
and links a real C caller against each target archive without running it. This
checks the mobile ABI and native dependencies. An EAS post-install check also
requires the Rust archive in CocoaPods' app linker flags before Xcode runs.

Native mode requires `EXPO_PUBLIC_NATIVE_BMV=1`; it never falls back to demo values.
The Electrical page's Connect simulator action requests permission and starts
reception. Scanning stops in the background and resumes an active session when
foregrounded. One app-wide provider shares samples across Home and Electrical.
See [the physical acceptance checklist](../../docs/bmv-phone-test.md).
