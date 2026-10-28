# Windows BMV simulator → aiNavLog phone test

**Storage baseline — October 28, 2026:** SQLite via bundled Rust rusqlite is confirmed for structured local iOS/Android beta records ([ADR-0001](decisions/0001-sqlite-local-storage.md)). Native simulator integration is recorded in the [October 26 handoff](status/status-2026-10-26.md); older build-specific demo descriptions below remain historical evidence. Restart/update retention and physical Android acceptance remain open.

Use aiNavLog 1.0.0 iPhone build 6 or Android version code 7 (or later), delivered
through TestFlight / Google Play internal testing. Android builds 5/6 have a
native-library linking defect fixed in code 7. Build 4 uses app-generated demo
values and cannot run this test. Native build completion is not phone acceptance.

October 26 status: owner repeatedly confirmed iPhone build-6 reception, including
"works beautifully". Android code 7 no longer shows the previous native error
and reaches the connected screen in the emulator; zero received radio packets
there is expected for this Windows Bluetooth test. Physical Android testing awaits
the owner's planned tablet. See the [session handoff](status/status-2026-10-26.md).

## Start and compare

1. In Windows PowerShell, open `simulation/bmv712` and run:

   ```powershell
   powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\broadcast-windows.ps1 -CheckFixture
   powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\broadcast-windows.ps1 -DurationSeconds 300
   ```

2. Keep Windows Bluetooth on and the phone nearby. On the phone, open
   **Electrical → Windows BMV simulator → Connect simulator** and grant the
   requested permission. Android 11 and earlier also require location permission
   and the device's Location switch for BLE scanning.
3. Keep aiNavLog in the foreground. Compare House voltage, signed current and
   charge, plus Starter voltage, with the Windows console at the same step.
   Initial values are House **12.76 V / −5.25 A / 77.5%**, Starter **12.80 V**;
   later steps change. The charging phase has positive current. The Electrical
   display rounds voltage/current to one decimal place.
4. Confirm the readings are labelled **Simulated BMV readings**. Starter current
   and charge must remain unavailable. Home and Electrical must agree.

## Failure and persistence checks

- Stop broadcasting. After five seconds without a fresh sample, numeric readings
  must be hidden. Restart broadcasting; readings should recover (a silent scanner
  may take roughly 30 seconds plus retry time to restart).
- Disconnect in the app. Readings should clear. Connect again to resume reception.
- Send the app to the background: scanning stops. Return to the app: an active
  simulator session resumes, and old readings must not gain a new timestamp.
- Force-close/reopen the app with broadcasting stopped, then connect. Previously
  stored samples may be loaded from phone SQLite, but stale values must stay
  hidden. Start broadcasting again and confirm recovery. Phone SQLite uses the
  app's private persistent files/application-support directory.
- Deny Bluetooth permission. Verify a useful error and no invented readings.
  Re-enable permission in phone Settings, then connect again.
- Turn phone Bluetooth off/on while receiving; verify stale hiding and recovery.

Record platform, phone model, OS version, installed build, simulator console
values, observed values and each check's result. An Android emulator cannot
establish radio reception; use a physical Android device for that acceptance.

## Implementation scope

Rust btleplug receives advertisements on both platforms; the existing Rust monitor
filters, decrypts, normalizes and persists samples. Swift/Kotlin modules connect
Expo to that Rust implementation. No separate Swift/Kotlin scanner is used.
The receiver accepts only the simulator's synthetic product `0x1234`, Victron
company `0x02E1`, Battery Monitor record `0x02`, with the documented public test key.
Build 6 accepts matching simulator packets across changing OS radio identities;
run one simulator at a time. Its Receiver details line counts matching packets,
stored updates and source-identity changes to help diagnose interrupted reception.
This build does not configure or claim reception from a real BMV or Orion.
No background recording or live equipment control is included.
