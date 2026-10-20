# First iPhone installation through TestFlight

## Prepared October 16, 2026

Owner confirmed Apple Developer membership is active and there is no existing iOS
app/bundle ID. Owner explicitly selected `com.kodekomplete.com.ainavlog` to match
Android. Expo app config sets that ID, version 1.0.0 and iOS build number 1.
The Apple-side bundle ID registration and App Store Connect app creation have not
been verified. The local identifier alone does not establish availability.

The `testflight` EAS profile builds a signed store-distribution iOS app using
remote credentials, Node 24.21.0 and forced Electrical demo mode. This initial
build has the same UI/demo scope as Android: Rust/Bluetooth reception and phone
SQLite integration are not included. The app must not claim those capabilities.

## Commands

Run from `user-interface/`:

```sh
npm run ios:archive -- --force
npm run ios:build
```

The wrapper scopes EAS_PROJECT_ROOT to this UI app and uses EAS_NO_VCS to avoid
uploading the repository's private sibling investment/model data. The corrected
archive inspection contained 28 files / 963,399 bytes. app.json in the archive
was checked for the selected bundle ID; private sibling and credential directories
were excluded. `.easignore` also excludes generated native directories and builds.

The build uploads app source to Expo's EAS cloud service. The owner explicitly
approved this upload and signed cloud build on October 16. Do not ask again for
this same approved operation. The non-interactive attempt reached remote iOS
credentials but stopped because they are not configured; no native cloud build
ID was returned. The next step is the interactive `npm run ios:build` command.

EAS also warned that `ios.infoPlist.ITSAppUsesNonExemptEncryption` is absent.
Complete the accurate encryption/export-compliance declaration before TestFlight
use; no declaration has been guessed or added.

Use existing Expo project credentials; if Apple credentials are
not yet configured, the owner should complete the guided Apple login, 2FA and
team-selection prompts in their terminal. Do not request passwords, 2FA codes,
private keys or account credentials in chat. Verify the bundle ID and Apple team
before accepting certificate/profile creation. Version/build number availability
must be checked if any earlier iOS build exists by then.

After a successful build, record the EAS build ID and artifact, then submit the
specific verified build to App Store Connect (not an unspecified latest build):

```sh
npm run ios:submit -- --id <EAS_BUILD_ID>
```

Submission may require creation/selection of the App Store Connect app record
and signing/submission credentials. The submit profile currently has no ascAppId;
configure the real ID once created. Never invent it. TestFlight processing and
internal tester/group setup remain separate from a successful EAS build.

## Installation and validation

Install Apple's TestFlight app on the iPhone. Once the build is processed and
assigned to the appropriate internal testing group, accept the invitation using
the eligible Apple account and install aiNavLog. Internal testers need appropriate
App Store Connect access; external beta testers use a different invitation/review
workflow. Record the actual account/group setup without assuming access.

Verify launch without Metro, Home navigation, visibly labeled Electrical demo,
changing House readings, Starter volts with unavailable current/charge as dashes,
Engine/Routes coming-next screens, background/resume and close/reopen.
Record iPhone model, iOS version, installed app version/build and errors.

The iOS Simulator requires macOS/Xcode; the existing Windows Android emulator
cannot run this iOS app. EAS cloud build and submission can run from Windows/WSL
for installation on the physical iPhone.

## Validation performed

Typecheck, lint and both existing UI test files passed. iOS JS/Hermes export
completed successfully; this is not a signed native build or device install.
Corrected source archive identity and exclusions were inspected.

Sources: [Expo TestFlight workflow](https://docs.expo.dev/submit/testflight/),
[EAS configuration](https://docs.expo.dev/eas/json/),
[SDK 57 app configuration](https://docs.expo.dev/versions/v57.0.0/config/app/).

## Installation update — October 16

Owner confirmed installation on a physical iPhone 13. Launch/demo/navigation and
lifecycle validation, iOS version and installed build number remain to record.
Native Rust/Bluetooth and phone-local SQLite are not part of the released demo.
