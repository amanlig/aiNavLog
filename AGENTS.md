# aiNavLog repository instructions

## Windows delivery — established owner preference

- A Windows build/deploy/installer request means a **single NSIS setup `.exe`**,
  named `aiNavLog-Setup-<version>.exe`, copied to the owner's Windows Downloads.
- Follow [docs/windows-delivery.md](docs/windows-delivery.md) and use the reusable
  builder. **The accepted template is the current NSIS Modern UI 2 wizard in
  `desktop-app/installer/ainavlog.nsi`, delivered as Setup 0.1.4.** The earlier
  windows-touch setup is historical evidence, not a reason to change styles again.
- Reuse that exact template, appearance, page flow, branding/icons, installation
  location, shortcuts and data/upgrade behavior. Routine releases change app
  contents, version and generated receipts only. Do not redesign/recreate the
  installer, introduce a one-off packaging command or reopen packaging choices
  unless the owner requests a change. Building an update uses this convention
  automatically; do not ask the owner to choose an installer style again.
- Do not substitute ZIP, extracted app folders, Install.cmd, PowerShell scripts or
  MSI unless the owner explicitly requests that format. `aiNavLog.exe` is the app,
  not a setup installer; never describe the app executable as an installer.
- Keep the established per-user app location and existing data. Back up existing
  records before updating. Do not install/run the Windows package automatically
  or change antivirus settings; the owner handles installation and testing.
- If installer tooling is unavailable, obtain the documented build dependency or
  report the actual blocker. Do not silently change delivery format.
- Deliver one setup EXE link for the new release, not a collection of alternatives.
  Preserve historical user files unless cleanup is explicitly requested.
- Check whether the requested changes are already in a verified, delivered build.
  If inputs are unchanged, point to that installer instead of rebuilding or
  incrementing versions solely because the owner repeats "build". A source change
  or explicit fresh-rebuild request justifies a new build.

Read the latest dated document under `docs/status/` before continuing prior work.
Verify earlier artifacts/documented conventions before claiming what was delivered.

## Shared maintenance regression baseline

Electrical POC intentionally omits the linking explanation, Create separate
equipment and Link equipment buttons. Do not restore them when sharing code with
Engine. Existing equipment identities and service records must survive edits.
For shared maintenance changes, run the embedded-host regression before delivery:
`python3 desktop-app/tests/run-engine-views.py desktop-app/tests/electrical-poc.cjs --reopen`
(with the browser test environment configured). Report the checks actually run;
typecheck/lint alone do not establish preserved UI behavior.
Hover must show the latest service again after an equipment editor is closed,
including unsaved review mode. Popups must remain visible for lower components of
tall diagrams. The editor's Save changes action belongs in its fixed top bar,
must stay visible while scrolling/resizing/zooming, and must persist changes
through reload and host restart. Check smaller windows as well as the default.
