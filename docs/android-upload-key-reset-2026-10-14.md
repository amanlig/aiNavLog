# Android upload-key recovery — October 14, 2026

**Superseded later today:** Play now shows the release available to internal
testers. No reset request was submitted by the assistant. See
[October 14 status](status/status-2026-10-14.md). Retain the certificate as evidence.

The owner confirmed they do not have the original upload keystore. The next
route is an upload-key reset through Google Play, after confirming Play App
Signing and the reset control in the app’s upload-certificate section.

## Prepared public certificate

`user-interface/builds/upload_certificate.pem` contains only the public X.509
certificate extracted from the corrected October 13 AAB. No private keystore was
retrieved or replaced. The file remains ignored by Git and EAS build artifacts.

The source AAB’s SHA-256 was rechecked against the existing build evidence:
`229cfe8b7a1e0fc05d286e90a4613bfa3c0b70583e749c10d936cc164174c911`.
Its signing block contained exactly one certificate. OpenSSL decoded it and
verified this SHA-1 fingerprint:

`A8:6B:0C:9D:8D:7D:53:34:A8:BB:5D:5B:A3:47:AF:35:84:A8:7F:29`

The previous handoff had two transcription errors (AE/7E instead of AF/7F);
those were corrected using the actual certificate. Play’s expected original
fingerprint remains as reported by the owner’s screenshot and needs confirmation
on the current upload-certificate page.

## Manual Play Console step

1. Open the existing app’s Play app-signing page.
2. Under Upload key certificate, choose Request upload key reset.
3. Choose the reason indicating the original upload key is lost/unavailable.
4. Upload `upload_certificate.pem` and submit the request.
5. Wait for Google’s confirmation and the replacement key’s activation time.
6. Verify that EAS’s selected upload key still matches this replacement
   fingerprint before building or uploading. Keep the existing EAS keystore;
   generating another key would require a different public certificate.
7. Confirm the final AAB signer against the now-active Play upload certificate,
   and check version-code availability before a new build/upload.

No reset request, key replacement, new build or Play upload has been performed.
No passwords or private signing material should be put in chat or Git. An upload
key reset is distinct from an app-signing-key upgrade.

Sources: [Google Play upload-key reset](https://support.google.com/googleplay/android-developer/answer/9842756)
and [Expo existing credentials](https://docs.expo.dev/app-signing/existing-credentials/).
