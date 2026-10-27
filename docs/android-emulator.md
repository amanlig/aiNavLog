# Android emulator for aiNavLog

Use the Windows Android Emulator with a Google Play system image. The development
workspace is WSL2; run the emulator on Windows to use the host hypervisor. The
current native Android module builds arm64-v8a and x86_64 libraries.

## Current recovery device — October 26, 2026

Use `aiNavLog_Recovery` for the next session. The original `aiNavLog_Pixel`
repeatedly rebooted during user-data initialization (`init_user0_failed`), with
read-only filesystem errors. Cold boot, ADB restart and a separate data image did
not fix it. Original device files were retained; a separate fresh recovery AVD
successfully booted (`sys.boot_completed=1`, boot animation stopped).

```powershell
& "$env:LOCALAPPDATA\Android\Sdk\emulator\emulator.exe" -avd aiNavLog_Recovery -no-snapshot -gpu swiftshader -cores 2 -feature -Vulkan
```

The recovery AVD uses the existing Pixel 7 profile and Android 16/API 36 Google
Play x86_64 image. First boot took roughly 2–3 minutes. Windows ADB sees it as
`emulator-5554`. Its Google account/app storage is separate from the original AVD.

Owner's latest Android screenshot shows the receiver connected, without the
previous native linking error, and counters at zero. This verifies the reported
UI behavior, not physical BLE reception or the installed version through ADB.
Use a physical Android device for the Windows BMV broadcast test; this emulator
setup does not receive those host Bluetooth advertisements. Code 7's successful
Play internal delivery is confirmed in the [latest handoff](status/status-2026-10-26.md).

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

## CI update verification — October 20, 2026

Owner confirmed update from version code 1 to code 3 (versionName 1.0.0)
using ADB on `emulator-5554`. Owner subsequently confirmed code-3 launch/navigation/demo/reopen checks passed; see below.
If ADB lists no devices, relaunch the emulator and retry from a new Windows
PowerShell terminal; this recovered detection in this session.

Open the app's Play Store page directly when needed:

```powershell
& "$env:LOCALAPPDATA\Android\Sdk\platform-tools\adb.exe" -s emulator-5554 shell am start -a android.intent.action.VIEW -d "market://details?id=com.kodekomplete.com.ainavlog" -p com.android.vending
```

Check the installed version:

```powershell
& "$env:LOCALAPPDATA\Android\Sdk\platform-tools\adb.exe" -s emulator-5554 shell dumpsys package com.kodekomplete.com.ainavlog | Select-String "versionCode|versionName"
```

## Android code-3 emulator checks passed — October 20, 2026

Owner confirmed launch, navigation, Electrical demo and close/reopen checks pass
for installed version 1.0.0, Android version code 3 on `aiNavLog_Pixel`
(`emulator-5554`). This completes owner-reported emulator acceptance for the
first automated Android internal-testing delivery (run 39525703279).
Physical Android verification and native Bluetooth/Rust/phone SQLite validation
remain separate; this is the demo-mode UI build.
