# Mobile development workflow — Electrical slice

**Confirmed storage decision — October 28, 2026:** SQLite, accessed through Rust `rusqlite` with bundled SQLite, is the local structured database for the 2027 iOS/Android beta. See [ADR-0001](decisions/0001-sqlite-local-storage.md) for scope, current implementation and remaining work. Existing AsyncStorage preferences and protected credential storage remain separate.

**October 16 update:** Apple membership is active. Android emulator installation
and Home launch are confirmed. Signed iOS 1.0.0 (1) was received by Apple and is
uploaded to TestFlight; owner confirms physical iPhone 13 installation. See the
[build/deployment physical architecture](mobile-build-deployment.md) and
[iPhone workflow](ios-testflight.md). Older enrollment entries below are historical.

**October 13 update:** Google Play verification is complete; Apple remains pending.
Expo is authenticated and the app is linked to the company's organization project.
The first standalone Android UI/demo AAB build succeeded for manual Google Play
internal testing. Its package is `com.kodekomplete.com.ainavlog`; native Rust/Bluetooth
integration remains a subsequent build. See [build and manual upload instructions](android-manual-deployment.md).
The older entries below describe the earlier environment and prerequisites.

**October 6, 2026:** current environment is Windows/WSL, without a Mac or an
enrolled Apple Developer account (confirmed by the user). Repository currently
contains HTML UI references, Rust components and a new Expo/React Native mobile
app shell under `user-interface/`.
The WSL development runtime is Node 24 LTS (24.21.0), installed in the user
account. The mobile app pins this runtime in `.nvmrc`; Node 18 is too old for
Expo SDK 57. Open a new terminal after upgrading the runtime.

## Proposed build path

Use React Native with Expo development builds and EAS cloud builds for iOS from
Windows/WSL. A development build supports custom native libraries; Expo Go cannot
load the planned Rust bridge or arbitrary BLE libraries. A UI-only demo can be
previewed separately while enrollment is pending, but it does not validate BLE.

The custom native build must package the Rust decoder with an iOS/Android native
module. Plan the Expo module/config plugin and Rust cross-compilation hooks before
claiming EAS supports the repository's Rust crate without additional work. Desktop
btleplug reception is not automatically a mobile BLE integration; use platform
scanning APIs and pass manufacturer payloads into the shared decoder.

References: [custom native code](https://docs.expo.dev/workflow/customizing/),
[development builds](https://docs.expo.dev/develop/development-builds/introduction/),
[EAS setup](https://docs.expo.dev/build/setup/),
[internal distribution](https://docs.expo.dev/build/internal-distribution/).

## External prerequisites

The owner needs to enroll in the Apple Developer Program or obtain access to an
enrolled organization for the planned signed physical-iPhone cloud build and
subsequent TestFlight distribution. Enrollment/payment is an owner action; no
account was created or payment made. An Expo account/project and authorized signing
setup are needed before cloud builds. Builds upload project source to a third-party
service; no cloud build or upload has been performed in this session.

For development distribution, register the iPhone in the provisioning workflow.
For release, use App Store Connect/TestFlight after signing and submission setup.
Android can use a signed development APK on a physical phone before Play Console
publication; emulator UI tests do not prove reception from the Windows BLE radio.

## Electrical acceptance path

1. Establish local boat and battery identifiers and an observation contract.
   Keep house and starter quantities distinct; future account access rules must
   not require treating one device's entire database as one battery.
2. Build the React Native electrical screen with House / Starter views. Label
   simulated readings; show unavailable starter current/SOC explicitly. Do not
   carry forward the starter percentage gauge from the existing HTML reference.
3. Create the native BLE/Rust module and prove a custom signed install. Keep keys
   in platform-protected storage; simulator uses only its public synthetic key.
4. Receive Windows BMV packets, decode house and auxiliary voltage, persist locally,
   and update UI with timestamps, stale/missing states and recovery.
5. Validate physical iPhone and Android separately. Record foreground/background
   limits and remaining hardware accuracy checks rather than assuming support.

## Current progress

The Windows BMV simulator now emits starter voltage through auxiliary mode 0 in
addition to house voltage/current/SOC. Its encoder matches the independent starter
fixture already consumed by the Rust decoder regression tests. The electrical UI
now renders readings fed by the desktop Rust service. Browser verification passed
for encrypted simulator packet replay through production decoding, SQLite and HTTP
to `/elec`, including stale hiding and recovery. This bypasses radio reception.
The native module, mobile storage integration and signed iPhone installation remain
unimplemented. See the [October 6 handoff](status/status-2026-10-06.md) for tomorrow's work.

The account/enrollment dependency is open and affects the October 16 iPhone
end-to-end milestone. Simulator and local UI work can continue while it is resolved.

## October 8 continuation — shared Rust core

The Device Interface now has independent Cargo features: default builds enable
`desktop` and `nmea`, while `--no-default-features` builds the transport-independent
Victron decoder/monitor, normalization and SQLite store. The platform scanner can
feed manufacturer payloads into this existing monitor without importing desktop
btleplug or Linux D-Bus. See [core integration instructions](../device-interface/README.md#shared-rust-core-for-native-integration).

The next adapter will use a local Expo module to connect platform Bluetooth
reception to this Rust core and deliver stored observations to the electrical UI.
Expo's [Modules API](https://docs.expo.dev/modules/overview/) supports native Swift
and Kotlin modules. Rust FFI, target builds, module packaging, platform permissions
and secure key storage still need implementation and platform verification.
No phone build is implied by successful Linux core tests. Only Linux and Windows
Rust targets are installed in the current environment.
