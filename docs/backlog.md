# Exploration backlog

Deferred ideas and investigations to revisit as aiNavLog develops. These entries are not committed delivery plans. Explicit scope decisions are noted per entry.

## In-app version checks and updates

- [ ] **Add Profile → Check for updates.**
- **Status:** Shelved at the owner's request, October 23, 2026; no delivery date.
- **Desired flow:** Check → show current / available version → confirm update →
  download → restart, with useful offline, failure and cancellation states.
- **Investigate:** Expo EAS Update for compatible JavaScript/assets on both platforms;
  native/runtime changes still require new signed store builds. Enabling OTA requires
  an initial native build, runtime compatibility policy and tested release channels.
  Android full-binary updates may use Google Play's in-app update flow; iPhone
  native beta updates continue through TestFlight. Do not promise silent native installs.
- **Done when:** Agreed scope works on physical iPhone/Android, handles no-update,
  offline/error and incompatible-runtime cases, and preserves local preferences/data.

The provisional store-opening Profile item was removed when this work was shelved.
No OTA configuration, publishing or new release build was initiated for this feature.

## CI/CD maintenance todo

Capture actionable workflow notices here as they appear, with a source, next action
and completion evidence. Track informational notices without treating them as failures.

- [ ] **Verify Ubuntu 26.04 runner compatibility.** Added October 20, 2026,
  from the annotation on [Android run 39523673243](https://github.com/amanlig/aiNavLog/actions/runs/39523673243).
  Owner also observed the same notice in the iOS build; this applies to both
  mobile delivery paths and their shared checks.
  GitHub is migrating `ubuntu-latest` from Ubuntu 24.04 to 26.04 between
  October 19 and November 19, 2026; see [runner-images announcement #14748](https://github.com/actions/runner-images/issues/14748).
  Review `.github/workflows/ainavlog-checks.yml`, `ainavlog-android.yml` and
  `ainavlog-ios.yml`. Verify shared checks and EAS CLI invocation on an explicitly
  selected `ubuntu-26.04` runner in a review branch; reuse release evidence when
  available rather than triggering unnecessary signed builds or store submissions.
  Record the actual runner image and passing run links, then decide whether to keep
  `ubuntu-latest` or pin a validated image. If compatibility fails, consider
  `ubuntu-24.04` temporarily and retain a migration follow-up.
  **Done when:** compatibility evidence and the runner-version decision are recorded.

## Maritime positioning and marketing

- **Status:** Planned later; direction agreed October 12, 2026.
- **Revisit when:** Core workflows can be demonstrated and representative operator
  trials can provide evidence; customer discovery may begin earlier.

Align aiNavLog with maritime operational needs, beginning with recognition and
follow-up of developing equipment conditions. Validate the first customer segment,
interview operators, collect physical trial evidence and prepare substantiated
messaging and demonstrations before planning a campaign. Preserve the distinction
between simulator demonstrations, real-device capability and measured user outcomes.

See [maritime alignment and future marketing](research/maritime-risk-alignment.md)
for the ShipIn/NorthStandard relevance assessment and proposed requirements.

**Expected outcome:** An agreed audience, evidence-backed positioning and a separate
marketing project plan. No campaign, external contact or publication is initiated.

## Explore iPhone development and testing workflow

- **Status:** Deferred
- **Added:** 2026-09-30
- **Revisit when:** Preparing the first React Native iPhone prototype.

Evaluate VS Code with Expo EAS cloud builds from the current Windows/WSL environment versus local iOS builds and debugging on a Mac with Xcode. VS Code can remain the primary editor in either workflow.

Compare:

- Physical iPhone installation, signing, and daily testing workflow.
- Native Rust integration and rebuild requirements.
- NMEA 0183 and NMEA 2000 gateway connectivity, starting with simulated instrument observations and progressing to real onboard devices.
- Debugging, offline operation, interrupted connections, and app background behavior.
- Hardware, Apple Developer membership, and cloud-build costs.

**Expected outcome:** Select and document a development setup and a repeatable iPhone testing workflow before implementing the prototype's native integrations.

## Wi-Fi instrument connectivity

- **Status:** Shelved; excluded from beta (confirmed scope decision).
- **Added:** 2026-10-01
- **Revisit when:** After beta, if supported instrument/gateway requirements justify
  an additional transport.

Beta device connectivity is Bluetooth-only. NMEA 0183 and NMEA 2000 encoding and
decoding remain in scope independently of transport. The unimplemented
`WifiAdapter` and `TransportKind::Wifi` were removed from the active Rust API,
tests and class diagrams; no working Wi-Fi integration was removed.

If revisited, evaluate compatible gateways, TCP/UDP transport, discovery,
permissions, reconnect behavior, and iOS/Android support. Define transport framing
and buffering before adding a concrete adapter to the shared transport interfaces.

**Expected outcome:** A separately scoped post-beta proposal backed by gateway
requirements and platform validation. No beta Wi-Fi implementation is planned.

## Orion key setup and physical validation

- **Status:** Shelved at the user's request on 2026-10-02; the Orion is not nearby.
- **Revisit when:** The user has the powered Orion within Bluetooth range.

Defer obtaining the advertisement key, creating `orion.key`, and running physical
reception/accuracy/recovery checks. Do not request the key again until this work is
resumed. Preserve device ID `D4:B3:CB:26:5E:75` and the implemented decoder,
foreground monitor and SQLite store. Software tests pass; physical validation is
still pending. No substitute key or pairing PIN should be used for validation.

See the [October 2 handoff](status/status-2026-10-02.md) for the current continuation point.
