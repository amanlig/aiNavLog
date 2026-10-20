# Android emulator for aiNavLog

Use the Windows Android Emulator with a Google Play system image. The development
workspace is WSL2; run the emulator on Windows to use the host hypervisor. The
accepted Android bundle includes x86_64, x86, arm64-v8a and armeabi-v7a libraries.

## Setup — October 14, 2026

Installed Android Studio 2026.2.1.8, platform tools, Android Emulator 37.2.12,
and the Android 16 (API 36) Google Play x86_64 image on Windows. Google's SDK
command-line archive was verified against the published repository checksum.
Hardware acceleration reports WHPX installed and usable; no reboot was needed.
Created `aiNavLog_Pixel` using the Pixel 7 profile and launched its desktop window.
ADB confirmed `sys.boot_completed=1` and the Play Store package
`com.android.vending` is installed.

## Reopen the phone

Run this in Windows PowerShell:

```powershell
& "$env:LOCALAPPDATA\Android\Sdk\emulator\emulator.exe" -avd aiNavLog_Pixel
```

The reusable `scripts/setup-android-emulator.ps1` also installs missing packages
and preserves an existing virtual device. Windows currently blocks this unsigned
script when run directly from the WSL network share; the initial setup used the
installed SDK tools directly without changing Windows execution policy.

## Finish setup

1. Sign into the emulator's Play Store with the Google account included in the
   internal tester list. Open the track's opt-in link and install aiNavLog.
2. Validate launch, navigation and the labeled Electrical demo. Record emulator
   installation separately from physical-device/BMV Bluetooth verification.

The Google Play image includes the Play Store; a Google APIs-only image is a
different choice and does not provide the same store-install workflow. The
existing AAB is a Play upload artifact; direct sideloading needs an APK. An
optional APK build profile already exists (`npm run android:apk`), but a new
cloud build was not started for emulator setup.

Sources: [Google virtual-device guide](https://developer.android.com/studio/run/managing-avds),
[Windows acceleration](https://developer.android.com/studio/run/emulator-acceleration),
[Expo APK guidance](https://docs.expo.dev/build-reference/apk/).

## Installation update — October 16, 2026

The owner reported successful installation in the emulator. The earlier
availability blocker is superseded for this installation. Launch, navigation,
demo readings, background/resume and installed version still need validation;
physical-device installation remains a separate milestone. See the
[October 16 handoff](status/status-2026-10-16.md).
