# Corrected Android package build — October 13, 2026

**Historical October 13 rejection; resolved October 14:** the owner’s latest
Console screenshot shows this version available to internal testers. See
[October 14 status](status/status-2026-10-14.md).

Original record: the owner attempted upload and Play rejected this AAB's
signing certificate. See the [end-of-day handoff](status/status-2026-10-13.md)
for fingerprints and recovery steps. Do not retry this artifact unchanged.

Signed AAB rebuilt successfully with the owner's exact required package:
`com.kodekomplete.com.ainavlog`. The original `com.kodekomplete.ainavlog` build
is superseded; keep it only as historical evidence.

- Version: 1.0.0 / version code 1.
- Build ID: 3afa7a66-0cdb-4dab-b666-0820880c8fa6.
- Completed: 2026-10-13 03:08:57 PM America/Vancouver.
- Local file: user-interface/builds/ainavlog-corrected-package-1.0.0-1.aab.
- Size: 72,177,350 bytes.
- [Download corrected AAB](https://expo.dev/artifacts/eas/1KKm0M4YkfGByxPxLR8v6w0aXSTEzSKfJreogqNE_HI.aab).
- [EAS build page](https://expo.dev/accounts/kode-komplete-software-services-ltd/projects/ainavlog-mobile/builds/3afa7a66-0cdb-4dab-b666-0820880c8fa6).

SHA-256:

```text
229cfe8b7a1e0fc05d286e90a4613bfa3c0b70583e749c10d936cc164174c911
```

Checks passed: typecheck, lint, ZIP CRC, compiled manifest contains the corrected
package and no old package marker, and signing-block presence. The configuration
uses the same standalone Electrical demo as the original artifact; UI behavior
was not changed. No Rust/Bluetooth integration, successful Google Play upload or physical
installation is claimed. The new package's signing credentials are managed by
Expo; no private keystore was downloaded or committed.

This AAB has the required package and was subsequently accepted for internal testing on October 14. The store
package identity is permanent; the visible app name is separately editable.
See [Google's documentation](https://support.google.com/googleplay/android-developer/answer/9859152)
and [manual deployment instructions](android-manual-deployment.md).
