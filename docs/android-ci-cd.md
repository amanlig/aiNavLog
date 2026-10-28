# Android CI/CD — Play internal testing

**Storage baseline — October 28, 2026:** SQLite via bundled Rust rusqlite is confirmed for structured local iOS/Android beta records ([ADR-0001](decisions/0001-sqlite-local-storage.md)). Native simulator integration is recorded in the [October 26 handoff](status/status-2026-10-26.md); older build-specific demo descriptions below remain historical evidence. Restart/update retention and physical Android acceptance remain open.

Prepared October 20, 2026, following owner verification of iOS 1.0.0 build 3
on iPhone 13. Android automation was demonstrated against Play later the same day; see the verified evidence below.

The manual **aiNavLog Android Play internal** workflow runs shared checks before
using the `ainavlog-play-internal` environment. It builds an AAB with EAS CLI
24.13.0, waits for completion, validates package/project/version/code, retains a
sanitized source/build receipt for 30 days, and optionally submits that exact ID.
Submission defaults off. No production track or automatic push-triggered release.
The Android version code changes only on the runner; iOS numbering is preserved.
These builds remain the labeled demo; native Bluetooth/Rust/SQLite is separate.

## Private setup

1. Publish the reviewed workflow and helpers on main so GitHub exposes Run workflow.
2. Create the GitHub environment `ainavlog-play-internal`, configure allowed branches
   and any desired reviewers, and privately add EXPO_TOKEN with access to the
   existing Expo organization/project. The iOS environment secret is not automatically
   available to this different environment.
3. Follow [Expo's Android submission guide](https://docs.expo.dev/submit/android/)
   and its linked Google service-account setup guide. Enable Android Publisher API,
   grant the service account appropriate app-level Play testing-release permissions,
   and assign its JSON key to this app's **EAS Submit** service credentials via the
   EAS dashboard or `npm run android:credentials` (choose play-internal).
   Keep the key out of chat, Git and the UI archive. This workflow uses the key
   stored by EAS; no service-account JSON GitHub secret or source path is needed.
4. Confirm the existing EAS Android upload keystore matches Play's active upload
   certificate. Preserve it; do not generate or reset keys. A manual first Play
   release is already recorded, but confirm current app/API eligibility and status.
5. Check every Play track and EAS for used version codes. Choose a fresh integer
   greater than all existing codes, at most 2100000000. Preflight validates format
   and configuration; it cannot prove remote version-code availability.

## Acceptance and retries

Run once with Submit unchecked and inspect the exact EAS ID, source SHA and
version code in the receipt. For delivery choose Submit with another fresh code;
then verify Play processing, internal tester access and an actual Android update.
Record device/OS/version/code and demo navigation/reopen results independently.
Also demonstrate that a failing shared check skips the release job on an isolated
branch. A successful upload alone proves neither installation nor native features.

If submission fails after a successful build, repair credentials and submit that
exact receipt ID with `npm run android:submit -- --id <BUILD_ID> --non-interactive --wait`.
Inspect Play and EAS before retrying; avoid rebuilding an already accepted code.
Timeout/cancellation may leave cloud work running. Releases are serialized with
cancellation disabled; GitHub may replace pending runs. Re-running the whole
workflow can attempt a duplicate code; dispatch a fresh code for a new build.

References: [EAS submission profiles](https://docs.expo.dev/submit/eas-json/) and
[CI builds](https://docs.expo.dev/build/building-on-ci/).

## Android full delivery verified — October 20, 2026

[Run 39525703279](https://github.com/amanlig/aiNavLog/actions/runs/39525703279)
passed shared checks, signed Android build, corrected receipt validation, artifact
upload and exact-build submission to Google Play internal testing. Source commit:
`4452c54eac2445ae7e7c91c4275cc0c41bca66e2`; version 1.0.0, Android code 3.
[EAS build 545a1817-6886-4b8a-9c4d-ebfd9fcac284](https://expo.dev/accounts/kode-komplete-software-services-ltd/projects/ainavlog-mobile/builds/545a1817-6886-4b8a-9c4d-ebfd9fcac284).
[EAS submission 78f899fc-bd3d-4252-9bb8-dab34c6b4f69](https://expo.dev/accounts/kode-komplete-software-services-ltd/projects/ainavlog-mobile/submissions/78f899fc-bd3d-4252-9bb8-dab34c6b4f69).
Job logs explicitly confirmed submission to Google Play. Play processing, tester
availability and an installed Android update remain unverified. Demo builds do
not establish native Bluetooth/Rust/phone SQLite functionality.

## Android emulator updated to CI build 3 — October 20, 2026

Owner supplied ADB output from `emulator-5554` confirming the installed package
`com.kodekomplete.com.ainavlog` has `versionCode=3` and `versionName=1.0.0`,
after opening its Play Store page and updating from code 1. This establishes
owner-reported emulator installation of the version delivered by CI run
39525703279. Launch/navigation/close-and-reopen checks for code 3 remain pending;
physical Android and native Bluetooth/Rust/SQLite verification remain separate.

ADB initially showed no connected devices. Restarting the ADB server alone did
not resolve it; after emulator relaunch, the owner used a new Windows PowerShell
terminal and ADB detected the emulator. When navigating the emulator UI was
difficult, an ADB VIEW intent opened aiNavLog's Play Store page directly.

## Android code-3 emulator checks passed — October 20, 2026

Owner confirmed launch, navigation, Electrical demo and close/reopen checks pass
for installed version 1.0.0, Android version code 3 on `aiNavLog_Pixel`
(`emulator-5554`). This completes owner-reported emulator acceptance for the
first automated Android internal-testing delivery (run 39525703279).
Physical Android verification and native Bluetooth/Rust/phone SQLite validation
remain separate; this is the demo-mode UI build.
