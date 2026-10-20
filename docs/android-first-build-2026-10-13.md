# First Android build evidence — October 13, 2026

**Superseded package:** this first artifact uses `com.kodekomplete.ainavlog`.
The owner subsequently specified `com.kodekomplete.com.ainavlog` as required by
their Play entry. The [corrected artifact is ready](android-corrected-build-2026-10-13.md); do not upload this original
AAB to the entry requiring the corrected package. The evidence below is historical.

**Result: signed Android App Bundle built successfully on EAS.** Manual Google
Play internal-test upload and Android device installation remain pending.

| Item | Value |
| --- | --- |
| App / package | aiNavLog / com.kodekomplete.ainavlog |
| Version / version code | 1.0.0 / 1 |
| Expo organization | kode-komplete-software-services-ltd |
| Build profile | play-internal |
| Build ID | bac37f5e-bbe9-447d-9824-02db32b82b08 |
| EAS result | FINISHED |
| Completion | October 13, 2026, 2:40:11 PM America/Vancouver (21:40:11 UTC) |
| Local artifact | user-interface/builds/ainavlog-1.0.0-1.aab |
| Artifact size | 72,177,278 bytes (about 68.8 MiB) |

[Build page and download](https://expo.dev/accounts/kode-komplete-software-services-ltd/projects/ainavlog-mobile/builds/bac37f5e-bbe9-447d-9824-02db32b82b08)

SHA-256:

```text
19808636fd9416c9e9feeeccec0ea727a0a8df83f4c5a780c982186e61a6db1c
```

## Artifact scope and evidence

Standalone React Native app shell, Home navigation, Electrical demo battery
readings, and Engine/Routes coming-next screens. Demo mode is forced and requires
neither Metro nor a desktop endpoint. Device-interface Rust, Bluetooth scanning
and instrument SQLite integration are not packaged.

The Android upload keystore was generated through EAS remote credentials and
retained there. No private keystore was downloaded or committed. Google Play
App Signing setup is part of the owner's manual upload; no rollout has occurred.

- UI typecheck, lint and both existing model/client test files passed.
- Expo Doctor passed all 21 checks after SDK-compatible patch updates.
- Expo Android prebuild and Hermes Android bundle export succeeded.
- EAS native Gradle release compilation and artifact upload succeeded.
- Reviewed upload archive: 27 files, 961,343 bytes, rooted only at user-interface.
  Investment files, local env files, signing data, generated native folders and
  prior artifacts were absent.
- Phone-width Chromium preview confirmed forced demo ignores an invalid desktop
  endpoint, sample labels appear, navigation works and no JS errors occur.
- Downloaded AAB passed ZIP CRC and bundle-structure checks. The compiled manifest
  contains expected package/version-name markers. A signing block and arm64
  libraries are present. Blocked microphone, overlay and external-storage
  permissions are absent. This is not a full cryptographic signature or
  bundletool/device-validation pass.

Ignored local evidence under user-interface/builds/ also includes the archive's
file hashes, release metadata, completed build metadata, checksum file and
android-demo-preview.png. Source changes remain uncommitted. Existing dependency
audit findings are documented in the UI README/manual guide. This internal demo
is not public-release qualification.

## Next action

Upload the AAB to **Google Play Console → aiNavLog → Test and release → Testing →
Internal testing**, select testers, review and roll out to that track. Then opt
in and install using an eligible Android tester account. Follow
[the manual deployment guide](android-manual-deployment.md). Phone install/update
behavior and native instrument functionality are not inferred from build success.
No Git push, automated store submission or OTA deployment was performed.
