# First iOS build evidence — October 16, 2026

The owner completed interactive Apple signing setup and supplied this EAS IPA:
https://expo.dev/artifacts/eas/PIqkDkaVB1yzi8daNt0BQAwl-fnQGIEqAK58kcFBsZo.ipa

Local artifact: `user-interface/builds/ainavlog-ios-1.0.0-1.ipa` (ignored).
Size: 17,354,159 bytes.
SHA-256: `ae829f19570e185372f42f5e4a4457bd796462e74e293138d508e910e5e50abf`.

## Checks performed

- ZIP CRC check passed.
- Compiled bundle ID: `com.kodekomplete.com.ainavlog`.
- Version 1.0.0, build 1; iPhoneOS; iPhone/iPad device families.
- Minimum iOS version 16.4.
- CodeResources signature metadata and embedded provisioning profile present.
- Embedded profile CMS signature verified with OpenSSL (`-noverify` skips trust
  chain validation; this is not Apple's App Store validation).
- Profile application identifier: `QHM3ZUQBTP.com.kodekomplete.com.ainavlog`.
- `get-task-allow=false`; no registered-device list and no all-devices flag,
  consistent with App Store distribution rather than development/ad-hoc/enterprise.
- Profile expiration: October 16, 2027, 22:01:15 UTC.
- Compiled `ITSAppUsesNonExemptEncryption=false`; local app.json now also contains
  that declaration from the interactive build setup. The assistant did not make
  this legal declaration on the owner's behalf.

The artifact and metadata establish a built IPA, not App Store Connect acceptance,
TestFlight processing, successful installation, or full executable-signature
validation. EAS build ID was not supplied; the artifact URL was used directly.

## Pending TestFlight submission

The exact verified artifact can be submitted from `user-interface/` with:

```sh
npm run ios:submit -- --path builds/ainavlog-ios-1.0.0-1.ipa
```

Automatic approval review rejected the assistant's initial non-interactive
submission attempt because separate explicit owner approval is required for the
App Store Connect artifact upload and TestFlight account-state change. No
submission command ran. Do not bypass this rejection; ask for explicit TestFlight
submission approval, then reuse this artifact rather than unnecessarily rebuilding.

App Store Connect app selection/creation, submission authentication, processing,
tester setup and physical iPhone installation remain to complete. Public App Store
release is not part of this internal beta installation task.

### Submission update

Owner subsequently approved TestFlight submission explicitly. The non-interactive
attempt uploaded the artifact to EAS Submit but stopped for missing `ascAppId`.
It did not establish successful delivery to Apple. Complete interactive submission
or configure the actual numeric App Store Connect Apple ID. No renewed submission
approval is needed for this same verified artifact.

### Apple receipt evidence

Owner's TestFlight screenshot confirms version 1.0.0 (1) in Build Uploads with
status Processing, displayed October 16, 2026, 3:22 PM. The `Team (Expo)` internal
group is visible. Do not upload build 1 again. Processing completion, tester
availability and physical iPhone installation remain pending.
