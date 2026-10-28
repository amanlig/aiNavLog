# Mobile build and deployment — physical architecture

**Storage baseline — October 28, 2026:** SQLite via bundled Rust rusqlite is confirmed for structured local iOS/Android beta records ([ADR-0001](decisions/0001-sqlite-local-storage.md)). Native simulator integration is recorded in the [October 26 handoff](status/status-2026-10-26.md); older build-specific demo descriptions below remain historical evidence. Restart/update retention and physical Android acceptance remain open.

This view shows the actual tools and services used to build and distribute the
current Android and iOS UI/demo. It complements the [boat/instrument physical
architecture](orion-physical-architecture.md); build infrastructure is separate
from runtime instrument communication and local boating data.

![Android and iOS build and deployment tools](images/mobile-build-deployment.svg)

```mermaid
flowchart TB
    subgraph Local["Developer computer — Windows + WSL2"]
        EDIT["VS Code · shared Expo SDK 57 / React Native / TypeScript app"]
        CHECK["Node 24 + npm · tsc · ESLint · Node tests · Expo/Metro export"]
        SCOPE["EAS CLI wrappers + eas.json + .easignore: UI-only source archive"]
        EDIT --> CHECK --> SCOPE
    end
    subgraph Cloud["Expo EAS cloud — source upload and remote signing credentials"]
        ANDROID["EAS Build: Android Linux worker · Gradle / Android SDK"]
        IOS["EAS Build: macOS worker · Xcode / iOS SDK"]
        AAB["Signed Android App Bundle .aab"]
        IPA["Signed iOS archive .ipa"]
        ANDROID --> AAB
        IOS --> IPA
    end
    SCOPE -->|"npm run android:build · play-internal profile"| ANDROID
    SCOPE -->|"npm run ios:build · testflight profile"| IOS
    UPLOADKEY["EAS-managed Android upload key"] --> ANDROID
    APPLECERT["Apple distribution certificate + provisioning profile; EAS-managed"] --> IOS
    AAB -->|"Owner manually uploads in browser"| PLAY["Google Play Console · Play App Signing · Internal testing"]
    IPA -->|"npm run ios:submit · EAS Submit"| ASC["App Store Connect · Apple processing · TestFlight"]
    PLAY -->|"Tester list + opt-in link"| STORE["Google Play Store"]
    STORE --> EMU["Windows Android Studio emulator · aiNavLog_Pixel · Google Play image"]
    STORE --> PHONE["Physical Android phone · not yet verified"]
    ASC -->|"Internal tester group: Team (Expo)"| TF["TestFlight distribution → physical iPhone 13 · install reported"]
```

## Tool responsibilities

| Stage | Shared tools | Android path | iOS path |
| --- | --- | --- | --- |
| Edit and configure | VS Code, Windows/WSL2, Expo SDK 57, React Native, TypeScript, Expo Router | `expo.android.package` | `expo.ios.bundleIdentifier` |
| Local checks | Node 24/npm, TypeScript, ESLint, Node tests, Expo CLI/Metro export | Android JS export | iOS JS/Hermes export |
| Scope cloud upload | `.easignore`, UI-root EAS wrapper, `EAS_PROJECT_ROOT`, `EAS_NO_VCS` | `scripts/eas-android.mjs` | `scripts/eas-ios.mjs` |
| Native cloud build | EAS CLI → Expo EAS Build | Android/Linux worker, Gradle and Android SDK | macOS worker, Xcode and iOS SDK; no local Mac required |
| Signing | Credentials managed remotely by EAS | Upload keystore signs AAB; Google Play App Signing signs distributed APKs | Apple distribution certificate and provisioning profile |
| Artifact | Local download and identity/checksum verification | `.aab` for Play; optional `.apk` profile for direct install | `.ipa` for App Store Connect/TestFlight |
| Upload | Explicit operations; no scheduled builds or uploads | Manual AAB upload in Google Play Console | EAS Submit uploads the verified IPA to App Store Connect |
| Beta distribution | Account/tester setup is distinct from build success | Internal testing tester list and opt-in link | Apple processing, internal group and eligible testers, TestFlight app |
| Installation validation | Launch, navigation, demo and lifecycle checks | Android Studio emulator on Windows; physical Android separately | Physical iPhone 13 installation reported; local iOS Simulator requires macOS/Xcode |

Both platforms use `com.kodekomplete.com.ainavlog`. Their store records, signing
credentials and version/build counters are separate. iOS App Store Connect ID:
`6823786033`; Expo project ID: `6b2c9577-6a5b-4380-a200-d5221085458c`.

The Android emulator is installed on Windows and uses WHPX acceleration and an
Android 16/API 36 Google Play image. WSL2 hosts the source/build commands, not the
Android emulator's hardware acceleration. It cannot emulate iOS.

## Current evidence — October 16, 2026

- Android: corrected AAB 1.0.0 / version code 1 accepted by Google Play internal
  testing. Owner confirmed emulator installation and supplied Home launch evidence.
  Electrical/navigation/lifecycle checks and physical Android install remain open.
- iOS: signed IPA 1.0.0 / build 1 downloaded and inspected. Owner's App Store
  Connect screenshot showed **Processing**. The owner subsequently confirmed installation on a
  physical **iPhone 13**. Owner then confirmed Electrical, navigation and close/reopen checks passed.
  Installed version and iOS version remain unreported; installation source was
  not independently inspected.
- Apple membership is active, per owner confirmation. The local iOS configuration
  and compiled app declare `ITSAppUsesNonExemptEncryption=false`.

The current binaries contain the UI shell and visibly labeled Electrical demo.
Rust decoding, mobile Bluetooth scanning/bridge and phone-local SQLite are not
packaged in these releases. The Linux-tested Rust/C bridge and desktop monitor
remain a separate development path. Cloud build/store services do not implement
or host aiNavLog boating-data storage. An accepted build is not proof of physical
instrument reception or persistence.

## Repeatable commands

From `user-interface/`:

```sh
npm run typecheck
npm run lint
npm test

# Android: inspect → build → manually upload AAB in Play Console.
npm run android:archive -- --force
npm run android:build

# iOS: inspect → build → submit the verified downloaded artifact.
npm run ios:archive -- --force
npm run ios:build
npm run ios:submit -- --path builds/ainavlog-ios-1.0.0-1.ipa
```

Do not resubmit an artifact already processing. Before future builds, verify
package identity, intended signing team/key and available version code/build
number, then increment the relevant counter. TestFlight submission is separate
from public App Store release. Android's current path is manual Play upload;
iOS uses EAS Submit, not manual Transporter upload.

See [Android deployment](android-manual-deployment.md),
[emulator setup](android-emulator.md), [iOS TestFlight](ios-testflight.md),
[iOS build evidence](ios-first-build-2026-10-16.md), and
[latest status](status/status-2026-10-16.md).

Sources: [EAS Build](https://docs.expo.dev/build/setup/),
[EAS configuration](https://docs.expo.dev/eas/json/),
[TestFlight through EAS](https://docs.expo.dev/submit/testflight/),
[Google internal testing](https://support.google.com/googleplay/android-developer/answer/9845334).

## CI/CD follow-up — October 20

Shared GitHub Actions checks and a manual iOS build/optional TestFlight pipeline are prepared. Remote activation and credentials remain pending; see [CI/CD setup and acceptance](mobile-ci-cd.md). The existing deployment SVG is preserved as requested; this addition does not replace its manual build/distribution view.
