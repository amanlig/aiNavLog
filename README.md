# aiNavLog

The electrical UI can read BMV battery observations through this local development path:

**Bluetooth → Rust device monitor → SQLite → local HTTP service → Expo/web UI**

## Run the device interface and UI together

Run the following in three separate terminals, starting from this `aiNavLog` directory.
Replace `<bmv-device-id>` with your BMV device ID and `/path/to/bmv.key` with your
private advertisement-key file. Use the same device ID and database in both Rust commands.

### 1. Receive and store BMV readings

```sh
cd device-interface
cargo run --locked -- monitor-victron '<bmv-device-id>' --key-file /path/to/bmv.key --database data/observations.sqlite3
```

Bluetooth reception requires a supported Bluetooth adapter and a powered device
within range. The advertisement key is separate from the Bluetooth pairing PIN.
On Unix, protect the key file with `chmod 600 /path/to/bmv.key`.

### 2. Start the device-interface service

```sh
cd device-interface
cargo run --locked -- serve-ui data/observations.sqlite3 '<bmv-device-id>'
```

The service exposes `http://127.0.0.1:8787/electrical` and reads the latest complete
BMV packet for the selected device. It does not create the database; the monitor
creates and populates it. Advertisement keys and raw packets are not returned.

### 3. Start the web UI

Use Node 24. Install dependencies with `npm ci` on first setup.

```sh
cd user-interface
npm ci
cp .env.example .env.local
npm run web
```

If `.env.local` already exists, edit it rather than overwriting it. Live mode needs:

```dotenv
EXPO_PUBLIC_DEVICE_INTERFACE_URL=http://127.0.0.1:8787/electrical
```

Restart Expo after changing the environment file. Open the web address printed by Expo
and choose **Electrical**.

## Connection behavior

- The UI polls the service and preserves each packet's original receipt timestamp.
- Readings older than five seconds are hidden. Missing values display dashes.
- Service failures show a retry message; live mode never falls back to demo readings.
- Starter voltage comes from the BMV auxiliary input. Starter current and charge
  percentage remain unavailable.
- To use demo mode, unset `EXPO_PUBLIC_DEVICE_INTERFACE_URL` and restart Expo.

The default allowed browser origin is `http://localhost:8081`. If Expo uses another
origin or the service needs another port, run:

```sh
cargo run --locked -- serve-ui data/observations.sqlite3 '<bmv-device-id>' 8787 'http://localhost:8081'
```

Replace the port and origin as needed, and update the environment URL if the service
port changes.

This service listens only on desktop loopback. A phone's `127.0.0.1` refers to the
phone itself; native mobile Bluetooth/Rust integration remains pending. When the
monitor runs on Windows and Expo runs in WSL, run the service alongside the monitor
and check that the browser can reach its endpoint through localhost forwarding.
Physical Bluetooth validation remains pending.

## Further documentation

- [October 14 internal release active](docs/status/status-2026-10-14.md)
- [October 13 Google account verification update](docs/status/status-2026-10-13.md)
- [First Android build and manual internal testing](docs/android-manual-deployment.md)
- [Android phone emulator](docs/android-emulator.md)

- [October 8 accomplishments and next steps](docs/status/status-2026-10-08.md)

- [October 6 success and October 7 next steps](docs/status/status-2026-10-06.md)

- [Device interface](device-interface/README.md)
- [User interface](user-interface/README.md)
- [Live monitoring guide](docs/victron-live-monitoring.md)
- [Architecture](architecture.md)
- [Android/iOS physical build and deployment](docs/mobile-build-deployment.md)

## Demonstrate simulator data reaching the UI

Verified on October 6, 2026: the browser integration test passed for simulator
packets → Rust monitor/decoder → SQLite → HTTP → rendered electrical UI, including
changing SOC, stale hiding and recovery. Bluetooth reception is not covered.

The existing Windows BMV broadcaster's encrypted output is captured in
[`simulation/bmv712/integration-packets.log`](simulation/bmv712/integration-packets.log).
The replay example feeds those packets through the production Rust monitor,
decoder, normalizer and SQLite store with new receipt timestamps. It bypasses
Bluetooth reception and uses the simulator's public development key.

Run in separate terminals from `aiNavLog`:

```sh
# Terminal 1: local service, explicitly labelled simulated
cd device-interface
cargo run --locked -- serve-ui data/bmv-simulator.sqlite3 bmv-simulator 8787 http://localhost:8081 --simulated
```

```sh
# Terminal 2: replay the captured packets once (about 25 seconds)
cd device-interface
cargo run --locked --example replay_bmv -- data/bmv-simulator.sqlite3 ../simulation/bmv712/integration-packets.log
```

```sh
# Terminal 3: use the local endpoint, without changing the environment file
cd user-interface
EXPO_PUBLIC_DEVICE_INTERFACE_URL=http://127.0.0.1:8787/electrical npm run web
```

Open **Electrical** while replay runs. It shows **Simulated BMV readings · device
interface**, house voltage/current/SOC and starter voltage. When replay ends,
readings become stale and are hidden after five seconds. Run replay again to
restore updates. This verifies service-fed readings rather than the UI's built-in
sample generator.

### Automated browser verification (Linux/WSL)

The browser test checks the displayed readings, changing SOC, stale hiding and
recovery with the actual Rust service. It saves a screenshot to
[`docs/images/bmv-device-ui-integration.png`](docs/images/bmv-device-ui-integration.png).
Python 3 and Playwright's Chromium system libraries are required.

```sh
# From aiNavLog:
cargo build --locked --manifest-path device-interface/Cargo.toml
cargo build --locked --examples --manifest-path device-interface/Cargo.toml
npm install --prefix /tmp/ainavlog-browser-check playwright
/tmp/ainavlog-browser-check/node_modules/.bin/playwright install chromium
cd user-interface
EXPO_PUBLIC_DEVICE_INTERFACE_URL=http://127.0.0.1:18787/electrical npx expo export --platform web --clear --output-dir /tmp/ainavlog-sim-ui
NODE_PATH=/tmp/ainavlog-browser-check/node_modules python3 tests/integration/run-bmv-ui.py /tmp/ainavlog-sim-ui
```

Ports 18787 and 18081 must be free. The test starts and stops its own services and
uses a temporary database. Browser tooling is installed outside the application.
This test verifies the packet-to-screen path; it does not verify Bluetooth radio
reception or native iPhone/Android integration.

## Standalone repository

This project is maintained independently in the public `amanlig/aiNavLog` repository. Run commands from this repository root; the Expo app is in `user-interface/` and GitHub workflows are in `.github/workflows/`. Local runtime databases, credentials and build outputs are excluded. The separate investment application and model projects are not part of this repository. See [CI/CD setup](docs/mobile-ci-cd.md).
