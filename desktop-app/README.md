# aiNavLog Windows local-test app

Double-click `aiNavLog.exe` on Windows 10/11 x64 with Microsoft Edge installed.
The executable includes the exported React Native web UI and the shared Rust
application/storage code. It opens an Edge app window and runs without Expo,
Node, WSL, an internet connection or a separately started data service.
This is an unsigned local testing build, not a Microsoft Store release.

Maintenance records are saved in:
`%LOCALAPPDATA%\aiNavLog\Desktop\ainavlog.sqlite3`

The executable uses its own database, separate from mobile storage and the
localhost:8081 development database. It starts with no completed service records.
Close the app's window to stop its local service. Preferences use the app's
separate Edge profile, retained in the same application-data folder.

## What to test

Open Maintenance Logs from Engine or Electrical. Record completed work, add
equipment, verify new records stay in the selected Engine/Electrical log even for shared equipment, then close/reopen
and check persistence. Home includes Fire Safety's sample layout and hover zoom.
There is no Windows Bluetooth receiver, engine telemetry, cloud account/sync or
new maintenance attachment/export/reminder feature in this package. Battery
history is empty until compatible observations are present in this Windows store;
no synthetic readings are silently generated. It does not replace phone-specific
permission/lifecycle/native-wrapper testing.

## Architecture and local interface

`desktop-app` is a thin launcher/local asset host. It calls the same
`native-integration::maintenance::execute` and Local Storage as iOS/Android.
All UI assets are embedded in the executable; arbitrary filesystem paths are not
served. The service listens only on `127.0.0.1:49378`, checks the Host and Origin,
and requires a per-process random token for maintenance commands. It has no remote
listener or external API. Third-party notices are included beside the executable
and at `http://127.0.0.1:49378/native-notices.txt` while the app runs.

The window uses Edge app mode, not a Tauri/WebView2 binding or React Native Windows.
This keeps the local testing package small and shares the existing UI. Initial
startup may use Edge's normal profile-initialization behavior. Closing every
window for this app's profile ends the host process.

## Rebuild from the repository

On Linux/WSL with UI dependencies, Rust, the Windows gnullvm Rust target and an
existing LLVM-MinGW toolchain:

```
python3 desktop-app/scripts/build.py --llvm-mingw /path/to/llvm-mingw
```

The script exports the UI without local endpoint/demo overrides, generates native
notices, builds and packages the EXE, and writes a receipt/checksum. It does not
install tooling or upload source. Outputs are ignored under `desktop-app/builds/`.
The one-time cross compiler is available from the upstream LLVM-MinGW releases;
install the Rust target with `rustup target add x86_64-pc-windows-gnullvm`.

For automation only: `aiNavLog.exe --serve-only --port 49379 --data-dir <test-path>`.
This mode has no app window; stop the process explicitly when done. Never point
an automated test at the production application-data directory. The Windows smoke
script saves/retries/reopens a test record using a caller-supplied isolated folder.

Version 0.1.1 separates maintenance logs. Older unassigned records on shared
equipment remain under an explicit Earlier shared records disclosure. No records
are deleted or silently assigned to a log.
