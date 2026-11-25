# Windows delivery contract and reusable procedure

Owner-confirmed November 24, 2026. This is the authoritative Windows delivery
procedure; it supersedes the ZIP/Install.cmd attempt recorded earlier that day.

## Accepted fixed template — do not redesign on each release

The owner accepted the **current NSIS Modern UI 2 template**, first delivered as
`aiNavLog-Setup-0.1.4.exe`. Its authoritative source is
`desktop-app/installer/ainavlog.nsi`; reuse it through the checked-in builder.
The older `aiNavLog-Setup-windows-touch.exe` establishes the historical setup-EXE
convention, but its earlier appearance is not the chosen template going forward.

Keep these consistent across ordinary app releases:

- **Appearance:** current Modern UI 2 wizard, icons, branding and English labels.
- **Page flow:** Welcome → installation progress → Finish; no automatic app launch.
- **Name:** `aiNavLog-Setup-<version>.exe`; no feature-named or date-named variants.
- **Install/upgrade:** current-user location, desktop/Start menu shortcuts,
  preflight checks, existing-record backup and data preservation described below.
- **Uninstall:** current confirmation/progress flow and retention of boat data.
- **Delivery:** one setup EXE in Windows Downloads, with one clear link to it.

Routine builds change app contents, version and generated metadata/checksums only.
Do not copy/reinvent the installer, select a new wizard style, change icons or
present packaging alternatives as part of an app update. A normal build request
uses this procedure automatically; the owner should not be asked to choose again.
Installer defect fixes and necessary tooling maintenance must preserve this
contract and be documented. Visual/workflow redesign requires an owner request.
Existing historical files are retained unless the owner requests cleanup; “one
delivery” means one new setup artifact, not deleting earlier files automatically.

## User-facing artifact

Deliver **one NSIS setup executable**:
`C:\Users\amanl\Downloads\aiNavLog-Setup-<version>.exe`.
The owner double-clicks it and follows the installer. Setup creates desktop and
Start menu shortcuts. Do not automatically run setup or launch the app.

Do not substitute a ZIP, an extracted directory, Install.cmd/PowerShell script or
MSI without an explicit owner request. `aiNavLog.exe` is the running application;
`aiNavLog-Setup-<version>.exe` is its installer. Keep those names and descriptions
accurate. The owner should not need to repeat this preference next session.

Verified historical evidence: `aiNavLog-Setup-windows-touch.exe` in Windows
Downloads is an NSIS 3 Unicode installer, as are the earlier component-entry,
electrical-diagram and simple-form setup files. The November 20/21 handoff records
the windows-touch setup. The existing Windows uninstall registry entry points to
`%LOCALAPPDATA%\Programs\aiNavLog\Uninstall.exe`.

## Persistent implementation

- `desktop-app/installer/ainavlog.nsi`: NSIS wizard, user-level install, file copy,
  shortcuts, Windows uninstall registration and data-preserving uninstallation.
- `desktop-app/installer/preflight.ps1`: internal installer helper; checks the app
  is closed, verifies payload SHA-256 and backs up existing SQLite/WAL/SHM files.
  It is embedded and run by setup, not a separate user step.
- `desktop-app/scripts/build.py`: normal app build followed by setup packaging.
- `desktop-app/scripts/build-installer.py`: repackage existing verified app files
  into setup without rebuilding the UI or Rust app.
- Root `AGENTS.md`: carries this contract into future agent sessions.

## Build commands

Requires existing UI dependencies, Rust Windows gnullvm target, LLVM-MinGW and
NSIS (`makensis`). Normal build:

```sh
python3 desktop-app/scripts/build.py \
  --llvm-mingw /path/to/llvm-mingw \
  --makensis /path/to/makensis
```

For an extracted Linux NSIS toolchain, also pass
`--nsis-root /path/to/usr/share/nsis`. The builder fails if setup tooling is missing;
it does not silently deliver a different format. `--app-only` explicitly builds
internal app files only and is not a user-facing delivery workflow.

To package an already-built app with a matching receipt:

```sh
python3 desktop-app/scripts/build-installer.py \
  --makensis /path/to/makensis \
  --nsis-root /path/to/usr/share/nsis
```

Inputs are `desktop-app/builds/windows-x64/{aiNavLog.exe,NativeNotices.txt,README.txt,build-receipt.json}`.
The builder validates the app receipt checksum and reads the version from it.
Output is `desktop-app/builds/aiNavLog-Setup-<version>.exe` plus a JSON receipt
recording setup hash/size and app hash. Build artifacts remain ignored by Git.

Temporary Linux tooling can be obtained from Ubuntu's `nsis` and `nsis-common`
packages using `apt-get download` plus `dpkg-deb -x` into `/tmp`, without installing
system packages. An extraction tool such as 7-Zip supports payload inspection.
Reference: [NSIS command line](https://nsis.sourceforge.io/Docs/Chapter3.html)
and [NSIS scripting](https://nsis.sourceforge.io/Docs/Chapter4.html).

## Installation and data rules

- Install app files to `%LOCALAPPDATA%\Programs\aiNavLog` for the current user.
  Reuse that established location and uninstall key on updates.
- Retain `%LOCALAPPDATA%\aiNavLog\Desktop` (SQLite database, Edge profile, backups).
- Require the app closed and back up existing database/WAL/SHM under
  `Desktop\Backups\before-update-*` before copying new app files.
- Uninstall removes only the named app files, shortcuts and uninstall registration;
  it retains boat records, diagrams, browser preferences and backups.
- Do not alter McAfee exclusions or other security settings. These test builds are
  unsigned; never claim that an installer is signed or antivirus-approved.

## Verify and hand off

1. Run checks appropriate to changed source; packaging alone does not require
   rebuilding/retesting unchanged application behavior.
2. Compile the NSIS source; inspect setup as an NSIS Windows PE archive.
3. Extract to a temporary directory and compare the embedded app's SHA-256 to the
   existing app receipt. Confirm notices/README/receipt and uninstaller are present.
4. Check the helper's PowerShell syntax without running installation. Record what
   was and was not executed; do not claim Windows install acceptance from inspection.
5. Copy the setup EXE to Windows Downloads and verify the destination checksum.
   Preserve earlier artifacts; use a fresh version/name for different delivered builds.
6. Update the dated handoff with the exact setup path/hash and relevant validation.
   Give the owner the setup EXE link, not an app EXE or ZIP link.
