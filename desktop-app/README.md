# aiNavLog Windows local-test app

Windows delivery is a single **`aiNavLog-Setup-<version>.exe` NSIS installer**.
Double-click setup, follow the installer, then open the aiNavLog desktop/Start menu
shortcut. No ZIP extraction, script execution or administrator access is required.
The setup verifies the bundled app checksum, checks that aiNavLog is closed,
backs up existing database files and retains existing data. It does not launch the
app or change antivirus settings. See the permanent
[Windows delivery procedure](../docs/windows-delivery.md).

The current NSIS Modern UI 2 wizard is the fixed release template. Future app
updates change the payload/version, preserving its appearance, page flow,
shortcuts, installation location and upgrade/data behavior. Use the existing
builder; do not recreate or restyle the installer for each release.

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
receipt upload/export/reminder feature in this package. Electrical maintenance
now includes the example/uploaded diagram, equipment regions and service-history
dialogs. Saved diagrams and records work offline; optional AI scanning uses
`https://ainavlog.kodekomplete.com` and requires the private gateway access token.
The scan URL is preconfigured; the token is entered for the current screen session. Battery
history is empty until compatible observations are present in this Windows store;
no synthetic readings are silently generated. It does not replace phone-specific
permission/lifecycle/native-wrapper testing.

## Architecture and local interface

`desktop-app` is a thin launcher/local asset host. It calls the same
`native-integration::maintenance::execute` and Local Storage as iOS/Android.
All UI assets are embedded in the executable; arbitrary filesystem paths are not
served. The service listens only on `127.0.0.1:49378`, checks the Host and Origin,
and requires a per-process random token for maintenance commands. It has no remote
listener. Its web security policy allows outbound scans only to the configured
production gateway in addition to the local service; the gateway explicitly
allows this app's `http://127.0.0.1:49378` Origin. Third-party notices are included beside the executable
and at `http://127.0.0.1:49378/native-notices.txt` while the app runs.

The window uses Edge app mode, not a Tauri/WebView2 binding or React Native Windows.
This keeps the local testing package small and shares the existing UI. Initial
startup may use Edge's normal profile-initialization behavior. Closing every
window for this app's profile ends the host process.

## Rebuild from the repository

On Linux/WSL with UI dependencies, Rust, the Windows gnullvm Rust target and an
existing LLVM-MinGW toolchain:

```
python3 desktop-app/scripts/build.py --llvm-mingw /path/to/llvm-mingw --makensis /path/to/makensis
```

The script exports the UI without local endpoint/demo overrides, generates native
notices, builds the app and its NSIS setup EXE, and writes receipts/checksums. It does not
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

## Diagram test build — November 4, 2026

The rebuilt 0.1.2 executable includes SQLite schema 3 and the diagram workflow.
A copy of existing Windows database files was made under the app data directory's
`Backups/before-diagram-*` before launch. Older schema-2 binaries cannot read the
upgraded database. The new EXE passed isolated Windows save/retry, session-token
and restart-retention smoke checks; UI typecheck/lint also passed.

For testing: Electrical → Maintenance → Use example diagram (or Add diagram),
then Scan with AI. Enter the gateway token from the ignored local
`.secrets/diagram_scan_token`; do not enter an OpenAI key. This package's CSP
permits the deployed scan hostname, and its exact loopback origin is allowed by
the gateway.


## Engine views test build — November 24, 2026

Version 0.1.4 reuses the Electrical maintenance flow for multiple 2D views of the
same engine: upper, port side, starboard side and bottom. Add each JPEG/PNG with
**Add diagram / Add/update diagram**, mark components, and link existing equipment across views.
Swipe left/right on a touchscreen to move between engine diagrams. Electrical
is the layout/interaction baseline; Engine has no separate view menu or arrows.
Tap for service, touch-and-hold to edit, and hover for the Electrical-style summary.
Completed work uses the shared maintenance service and saves in the existing local
database. Electrical diagrams and records are retained.

The NSIS setup copies app files to `%LOCALAPPDATA%\Programs\aiNavLog`; data stays
in `%LOCALAPPDATA%\aiNavLog\Desktop`. It creates a dated backup under
`Desktop\Backups\before-update-*` when existing records are present.
Storage upgrades to schema 5 on first use. Older app binaries cannot open that
upgraded database; continue using this version. Running the app executable directly skips setup's backup step. No device connectivity or mobile store deployment is
included in this package.


The ZIP/Install.cmd delivery attempted during November 24 was superseded by the
NSIS setup EXE. Future releases must use the setup format unless the owner explicitly
requests another format. `aiNavLog.exe` is the application, not the installer.
