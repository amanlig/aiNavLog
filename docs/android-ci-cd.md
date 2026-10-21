# Android CI/CD — Play internal testing

Prepared October 20, 2026, following owner verification of iOS 1.0.0 build 3
on iPhone 13. Android automation has not yet been demonstrated against Play.

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
