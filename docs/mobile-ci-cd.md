# aiNavLog mobile CI/CD

**Storage baseline — October 28, 2026:** SQLite via bundled Rust rusqlite is confirmed for structured local iOS/Android beta records ([ADR-0001](decisions/0001-sqlite-local-storage.md)). Native simulator integration is recorded in the [October 26 handoff](status/status-2026-10-26.md); older build-specific demo descriptions below remain historical evidence. Restart/update retention and physical Android acceptance remain open.

Verified October 20, 2026: shared GitHub checks, signed iOS build, exact-build
App Store Connect upload and owner-confirmed verification on iPhone 13 have
succeeded. Android signed build, exact-build Play internal submission and owner-confirmed
emulator update to version code 3 have also succeeded. The shared
checks already cover the common React Native/TypeScript code for both platforms.

## Pipeline

```mermaid
flowchart LR
  Code[UI pull request / push] --> CI[GitHub Actions: locked install, typecheck, lint, tests]
  Manual[Manual iOS workflow: selected ref + unused build number] --> Checks[Run the same shared checks]
  Checks --> Env[ainavlog-testflight environment]
  Env --> EAS[EAS signed iOS build: wait for result]
  EAS --> Receipt[Exact build ID + source SHA receipt]
  Receipt --> Choice{Submit selected?}
  Choice -->|No| Artifact[Build-only result]
  Choice -->|Yes| Submit[EAS Submit: exact build ID]
  Submit --> Apple[Apple processing / TestFlight]
  Apple --> Phone[Owner verifies update on iPhone 13]
```

Files:
- `.github/workflows/ainavlog-checks.yml`: pull requests and pushes changing the UI
  or these workflows; also reusable by the iOS release workflow. No release secrets.
- `.github/workflows/ainavlog-ios.yml`: manual `workflow_dispatch` only; checks must
  pass before the release job. Optional submission defaults off.
- `user-interface/scripts/ci-ios.mjs`: validates app/project/store identity and
  runner-only build number; accepts only one completed iOS build for submission.
- `user-interface/scripts/eas-ios.mjs`: UI-root archive scope, demo-mode environment
  and optional pinned CLI override. CI uses EAS CLI 24.13.0 and Node 24.21.0.

These checks do not establish Bluetooth reception, native Rust integration, phone
SQLite or physical instrument compatibility. Builds still explicitly use demo mode.

## One-time setup

1. Publish the reviewed workflow and UI helper changes to this repository. The
   manual workflow must exist on the default branch (`main`) before GitHub exposes
   its Run workflow control. Check Actions permissions allow the used actions.
2. In [repository environments](https://github.com/amanlig/aiNavLog/settings/environments),
   create `ainavlog-testflight`. Restrict deployment refs to approved branches and
   configure an appropriate reviewer if desired. Workflow dispatch is the explicit
   release trigger; environment protection is only effective once actually configured.
3. Create an Expo access token for an account with access to organization
   `kode-komplete-software-services-ltd` and project `ainavlog-mobile` at
   [Expo access tokens](https://expo.dev/settings/access-tokens). Store it as
   **EXPO_TOKEN** in the GitHub environment's secrets. Never put it in source or chat.
4. For Apple submission, assign an App Store Connect API key to this app in EAS.
   From `user-interface/`, run:

   ```sh
   npm run ios:credentials
   ```

   Select the `testflight` profile and the intended Apple team. Under App Store
   Connect API-key management, configure the project to use a key for EAS Submit.
   Existing signing certificates/profiles are managed by EAS; do not rotate them
   unnecessarily. Apple login/2FA and private key handling stay in your terminal
   or the provider portal. The workflow retrieves the assigned submission key
   through EAS; no Apple password or `.p8` GitHub secret is required by this design.
5. Confirm the existing signing identity and profile remain valid, and that the
   API key has submission permission. Configuration is not proof until a real
   non-interactive submission succeeds.

See [Expo CI authentication](https://docs.expo.dev/build/building-on-ci/) and
[non-interactive iOS submission](https://docs.expo.dev/submit/ios/). Keep the manual
[iOS](ios-testflight.md) and [Android](android-manual-deployment.md) paths as fallback.

## First run and acceptance

1. Open Actions → **aiNavLog checks** and confirm the intended commit passes.
2. Open Actions → **aiNavLog iOS TestFlight** → Run workflow. Select the reviewed
   ref and an **unused integer build number above 1**. Check App Store Connect
   **and EAS** first; `2` is an example, not a verified available number. Workflow
   preflight checks format/identity, not remote build-number availability.
3. For the first build-only check, leave Submit unchecked. To demonstrate full
   delivery after credentials are ready, select Submit. A new invocation creates
   a new build: use a fresh number, not the previous successful number.
4. The job installs locked dependencies, authenticates, invokes the scoped EAS
   wrapper and **waits** for build completion. It does not use `--no-wait` or
   `--latest`. Submission receives exactly the validated returned build ID.
5. Inspect the run summary and downloadable JSON receipt: source SHA/ref, version,
   build number, EAS build URL/ID, CLI version and GitHub run/attempt. The receipt
   excludes tokens and signed artifact-download URLs. The runner changes only its
   app.json iOS build number; it does not commit back or change Android's counter.
6. If submission was selected, verify Apple processing and tester availability
   separately. Install the update on iPhone 13, record the actual version/build,
   and repeat demo/navigation/reopen checks. Upload success alone is not an install.
7. On an isolated test branch, temporarily cause an existing check to fail and
   confirm the release job is skipped. Restore it; never deliberately break main.

## Failure and retry behavior

Failed checks block signing/upload. Missing EXPO_TOKEN fails preflight. Build
failure, unexpected identity/version or ambiguous build output blocks submission.
The raw EAS build result stays in runner temp; only the sanitized receipt is
uploaded. Submission failure still leaves the receipt available. Retry submission
of that exact successful build manually with `npm run ios:submit -- --id <BUILD_ID>`
after repairing credentials; do not rebuild or resubmit an artifact Apple is
already processing. Inspect EAS and Apple state before any retry.

A workflow timeout/cancellation does not guarantee cancellation of a running EAS
cloud build/submission. Release jobs are serialized with cancellation disabled;
GitHub may replace pending runs in a concurrency group. Re-running all jobs may
attempt a duplicate build number; choose a new unused number for a new build.
There is no automatic production publication or OTA rollout. Android submission
is available through its separate manual workflow.

## Android delivery implementation

The published Android workflow reuses shared checks, the UI-scoped wrapper,
exact-build receipts and a manual trigger with optional internal-track submission.
See [Android CI/CD setup and acceptance](android-ci-cd.md) for the GitHub environment,
EAS-managed Google service account and first-run checklist. The initial manual Play
release is already recorded. [Run 39525703279](https://github.com/amanlig/aiNavLog/actions/runs/39525703279)
passed signed build, receipt and exact-build Google Play submission for 1.0.0,
version code 3. Owner supplied ADB confirmation of code 3 installed on the emulator
and confirmed launch, navigation, Electrical demo and close/reopen checks.
Physical Android validation remains pending.

## Verified evidence — October 20, 2026

- Public standalone repository: [amanlig/aiNavLog](https://github.com/amanlig/aiNavLog).
- [Shared check run](https://github.com/amanlig/aiNavLog/actions/runs/39512899429): passed.
- [Build-only run](https://github.com/amanlig/aiNavLog/actions/runs/39513745444):
  passed; version 1.0.0 (2), EAS build `e9e227cf-3dcd-48c5-bf12-30b8afb60673`.
- Owner configured EXPO_TOKEN in GitHub and confirmed EAS reports an existing
  App Store Connect API key assigned for submission. No secret was shared in chat.
- [Full delivery run](https://github.com/amanlig/aiNavLog/actions/runs/39515378354):
  passed checks, signed build, receipt upload and exact-build Apple submission.
  Version 1.0.0 (3), EAS build `3b990ab9-2a65-4c33-a40c-716e83f71553`.
- Owner confirmed verification on physical iPhone 13 after the build-3 TestFlight
  handoff. This is owner-reported device evidence, not independent device inspection;
  the iOS version and detailed checklist results were not separately supplied.

## Evidence still required

- Intentionally failed-check blocking demonstration.
- Physical Android install/update verification.
- Ubuntu 26.04 runner compatibility follow-up in [backlog](backlog.md).
- Native Bluetooth/Rust/phone SQLite and eventual stored-data update retention;
  the verified binaries remain the labeled UI/demo.
