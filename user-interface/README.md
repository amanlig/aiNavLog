# aiNavLog user interface — Electrical UI

React Native / Expo SDK 57 with Expo Router. Use Node **24 LTS** (tested with 24.21.0) (see
`.nvmrc`). Without a configured device endpoint, this screen generates local demo data matching the Windows BMV scenario.
The home screen follows [HTML reference](../index.html), with Electrical, Engine and Routes
feature cards using `/elec`, `/mech`, and `/rout`. Their screens live in
`src/app/elec/`, `src/app/mech/`, and `src/app/rout/`. Electrical opens the working demo; Engine and Routes open explicit
coming-next screens. All detail screens provide a Home link.

Live mode reads BMV observations from the local Rust service. The Rust monitor receives Bluetooth packets, decodes them and persists them in SQLite; the UI reads the latest complete packet for the configured device.

```sh
cd user-interface
# If using nvm: nvm install && nvm use
node --version # expected v24.x
npm ci
npm start
# or a browser preview of the React Native screen:
npm run web
```

A compatible Expo Go app can preview this UI-only version on iPhone/Android.
Network access from the phone to the development server is required; WSL networking
may need configuration. A custom development build is required when native BLE/Rust
is added; initial signing/cloud builds and iPhone installation are established; native BLE/Rust integration remains pending.
The first Android internal-test build is configured with package
`com.kodekomplete.com.ainavlog`, linked to the company's Expo organization, and uses
standalone demo readings. See [manual Google Play deployment](../docs/android-manual-deployment.md).
Phone-local Rust/Bluetooth reception is still pending.

## Manual Android build

```sh
npx eas-cli@latest login
npm run android:configure
npm run android:archive
npm run android:build
```

The `play-internal` profile produces a signed AAB for manual upload to Google Play
internal testing. `npm run android:apk` optionally produces a standalone APK for
direct installation. Profiles explicitly force demo mode and exclude local
dotenv files; normal development HTTP/live mode remains unchanged when
`EXPO_PUBLIC_DEMO_MODE` is unset. The wrapper limits uploads to this UI project.
No automatic Play submission or OTA update is configured. Shared GitHub checks and a manual iOS build/optional TestFlight workflow are prepared; credentials and the first hosted run remain pending. See [CI/CD setup](../docs/mobile-ci-cd.md).

Android version codes are managed manually. Increase `expo.android.versionCode`
before uploading a later build. Local artifacts under `builds/` and private signing
files are excluded from Git and the EAS upload archive.

The electrical screen follows the original [battery mockup](../elec/index.html):
House and Starter cards appear together, side by side on wide screens and stacked
on phones, with battery-shaped gauges. House displays volts, signed amps, charge
percentage, direction and sample age.
Starter displays auxiliary voltage; its charge gauge is empty and amps/SOC are
explicitly unavailable. The screen retains the mockup's two battery cards and
footer, without trend charts, alert settings or prototype controls. The screen uses the landing page's shared navy header, pale blue background
(`#f0f6fb`), typography and white cards. Demo readings update every second and generation stops while backgrounded.
Stale readings are hidden; missing values show dashes.

## Verification

```sh
npm run typecheck
npm run lint
npm test
npx expo export --platform all
```

Exports check JavaScript bundles, not signed native builds or physical-device
behavior. The model tests check missing/stale alert rules, formatting and the
charge/discharge demo contract. The native data seam is `src/electrical/model.ts`;
future normalized BMV observations must populate that sample contract with a real
receipt timestamp and accurate source label.

Dependency audit at creation reports 28 findings (10 moderate, 18 high) in the
Expo/React Native dependency tree. Suggested automatic fixes include incompatible
major upgrades/downgrades; no forced fix was applied. Review and resolve applicable
findings before release. SDK-compatible reanimated/worklets versions are pinned.

## WSL React Native DevTools dependencies

If Expo reports an error installing React Native DevTools with missing
`libnspr4.so`, install the missing Ubuntu system libraries in a WSL terminal:

```sh
sudo apt-get update
sudo apt-get install libnspr4 libnss3
```

Restart Expo after installation. The DevTools binary is already downloaded; these
packages provide its missing NSPR/NSS shared libraries. This is separate from
the mobile app bundle and Node runtime.

## Connect the Rust device interface locally

Run these in separate terminals, starting from `device-interface`:

```sh
# Receive and store BMV advertisements (requires Bluetooth and your private key file):
cargo run --locked -- monitor-victron '<bmv-device-id>' --key-file /path/to/bmv.key --database data/observations.sqlite3
# Expose that same database and device to the UI:
cargo run --locked -- serve-ui data/observations.sqlite3 '<bmv-device-id>'
```

Then from `user-interface`:

```sh
cp .env.example .env.local
npm run web
```

The bridge listens only on `127.0.0.1:8787`; its default allowed browser origin is
`http://localhost:8081`. If Expo uses a different origin or port, specify
`serve-ui <database> <device-id> <port> <ui-origin>` and adjust the environment URL.
Restart Expo after changing the environment. The URL contains no advertisement key.
Keys stay in the monitor's private file. This is a desktop development bridge;
a phone cannot reach the desktop through its own loopback address. Native mobile
Bluetooth/Rust integration remains pending. Windows-hosted monitoring and WSL-hosted
Expo may require localhost forwarding; run the bridge alongside the monitor and
verify the endpoint is reachable from the browser.

Live mode never falls back to demo values. An empty database shows waiting, an
unavailable service shows a retry message, and receipt timestamps older than five
seconds hide readings. The bridge reads existing SQLite data without creating the
database. Starter amps/SOC remain unavailable. Demo mode is selected only by leaving
`EXPO_PUBLIC_DEVICE_INTERFACE_URL` unset.

### Home overview and preferences

Home summarizes fresh Electrical readings using the same data client and freshness
rules as `/elec`; Engine and Routes remain placeholders. Tap the boat name in the
header to open the boat profile and rename it (1–40 characters). Open Settings
inside the profile to choose White or Black.
Save persists both preferences locally on this device/browser using AsyncStorage;
Cancel discards edits. Preferences apply across the app and do not sync to an account.
The added native storage dependency requires a new native build for existing store
installs. Web validation does not establish native-device acceptance.

The Electrical home card links to Historical Usage (`/elec/history`) and
Maintenance Logs (`/elec/maintenance`). These screens currently show explicit empty
states; historical storage/charts and maintenance recording remain unimplemented.
The Electrical heading opens the existing battery overview.

In-app version checks and updates are deferred; see [backlog](../docs/backlog.md#in-app-version-checks-and-updates).
