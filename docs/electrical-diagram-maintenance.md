# Electrical maintenance diagram

Implemented in source November 4, 2026. Not included in installed mobile build 10.
A new signed native build is required for the image-picker dependency and Rust
commands/schema. The gateway is deployed at `https://ainavlog.kodekomplete.com`; a real example
scan returned 10 component proposals. Physical-device acceptance of this new UI
and broader model accuracy remain pending.

## User workflow

Electrical maintenance shows the owner's example diagram, reproduced from
[Custom Marine Products' wiring diagrams, page 8](https://www.custommarineproducts.com/uploads/1/4/3/7/1437708/wiring_diagrams_5-2-24.pdf).
Its source branding and labels are retained. The bundled example (`example.json`)
is illustrative and does not create equipment or service records automatically.

Choose Add diagram / Add/update diagram for a JPEG or PNG up to 6 MB, or Use
example diagram. Scan with AI explicitly sends the diagram image to the configured
gateway/OpenAI. Review proposed regions and equipment names; link existing records
or create new equipment. Type/model/serial values can be corrected before saving.
Unknowns have searchable common types including LiFePO₄, AGM, flooded/gel batteries,
solar panels, MPPT/PWM controllers, alternators, chargers, inverters, monitors,
shunts, distribution panels, fuses, breakers, isolation switches, BMS and busbars.
Custom types are accepted. Chemistry and serial numbers must not be invented.

Tap the diagram or Add component manually for missed equipment. Adjust region
coordinates as percentages of image width/height; ignore illustrations that are
not maintained equipment. Rescanning explicitly replaces the unsaved review.
Cancel leaves the current diagram/equipment unchanged. Saving an image without
regions is supported so linking can be completed later.

Hover with a mouse or tap equipment to open its modal. It shows name, model,
serial, latest Electrical service date/description, a × close button and a link
to equipment-filtered service history. No service recorded and Not recorded are
explicit empty values. Latest service sorts by service date, creation time and ID.
Legacy records on shared equipment remain unassigned and explicitly disclosed in
history; they are not silently treated as Electrical work.

Replacements require review again. Local matching suggests unique exact serial
or name matches, rejects conflicting serials and never matches by repeated model
alone. Stable equipment IDs preserve service history; missing diagram components
are not deleted from the inventory. Existing name/system identity is preserved.

## Server and credentials

The [SAD §5.6](../architecture.md#56-electrical-maintenance-diagram-and-ai-scan-gateway)
shows the device, Node gateway, OpenAI and local storage boundaries.

The gateway needs Node 24 and no npm dependencies:

```sh
cd diagram-ai-server
cp .env.example .env
# In .env, set OPENAI_API_KEY and DIAGRAM_SCAN_TOKEN (at least 24 characters).
# Generate the gateway token locally, without putting it in repository files:
node -e "console.log(require('crypto').randomBytes(32).toString('hex'))"
npm start
```

Enter the gateway URL and gateway access token in the diagram review screen.
The token is session-only; do not enter the OpenAI key in the app. The endpoint can
also be preconfigured using `EXPO_PUBLIC_DIAGRAM_SCAN_URL`; never use an
`EXPO_PUBLIC_` variable for a secret. The default model is `gpt-4.1`; set
`OPENAI_VISION_MODEL` on the server to evaluate another compatible image/structured
output model. Default rate limit is six requests/minute with two in flight.

Local web development can use `http://127.0.0.1:8788`. On a phone, loopback means
the phone itself. Host the gateway behind HTTPS on a trusted server and configure
`DIAGRAM_ALLOWED_ORIGINS` for browser/packaged-desktop origins. The single-owner
access token is a first-version setup, not a multiuser account system. Keep the
provider key in server environment/secrets. The existing EC2/Caddy deployment is documented in
[deployment operations](../diagram-ai-server/deploy/README.md).

The request contains the image only; equipment inventory/history stays local.
The server validates MIME/signature/size, authenticates the token before invoking
AI, uses a strict component JSON schema, bounds-checks returned regions, and sets
`store: false` on Responses requests. It does not log or persist image bodies.
AI error/refusal/incomplete output does not modify local data. Manual region
marking remains available without the server.

## Storage and compatibility

SQLite schema 3 adds `maintenance_equipment_details`,
`electrical_diagram_assets` and `electrical_diagram`. Images are base64 chunks of
at most 16,000 ASCII characters so every command fits the existing 32 KiB native
and HTTP request limit. The current diagram and reviewed equipment metadata save
in one transaction. Interrupted uploads do not publish a partial diagram.
Previous published image chunks are removed on replacement; equipment and service
records are retained. A base-revision check rejects stale concurrent saves.

Schema-1/2 data migrates without rewriting existing records. Schema-2 binaries
reject schema 3, so downgrading is not supported. Only one current Electrical
diagram per local boat dataset is implemented. No cloud sync, PDF import, wiring
reconstruction, built-in schematic editor or offline vision model is included.

## Verification

```sh
cd user-interface
npm run typecheck
npm run lint
npm test
# The explicit endpoint must be in the export; clear Metro's cached env transform.
EXPO_NO_DOTENV=1 EXPO_PUBLIC_DEVICE_INTERFACE_URL=http://127.0.0.1:18787/electrical npx expo export --clear --platform web --output-dir /tmp/ainavlog-diagram-web
```

From repository root:

```sh
cargo test --locked --manifest-path local-storage/Cargo.toml
cargo test --locked --manifest-path native-integration/Cargo.toml --test maintenance
node --test diagram-ai-server/server.test.mjs
cargo build --locked --manifest-path native-integration/Cargo.toml --features desktop
NODE_PATH=/path/to/playwright/node_modules python3 user-interface/tests/integration/run-diagram-ui.py /tmp/ainavlog-diagram-web
```

Browser integration uses temporary real Rust/SQLite storage and explicitly stubbed
AI responses. Gateway tests stub the OpenAI transport. These checks do not
establish live model accuracy, configured production hosting or physical-device
acceptance. Native rebuild and phone/tablet regression remain necessary.

## Live server verification — November 4

Release `ainavlog-diagram-ai:2026-11-04-3` runs on the existing EC2 host behind
Caddy at `https://ainavlog.kodekomplete.com`. HTTPS health returned 200, an
unauthorized scan 401, authenticated invalid input 400, allowed-origin preflight
204 and a disallowed origin 403. A real OpenAI scan returned 10 proposals,
including unknown/custom battery types for user review. This is smoke evidence,
not a guarantee that every diagram/component or model/serial label is recognized.

Live checks exposed ambiguous width/height output from the initial AI contract.
The final schema uses bounded corner coordinates and converts them to UI regions
after validation. A regression test covers corner conversion and invalid boxes.
The gateway logs static failure codes without image data, credentials or provider
response bodies. Client build profiles include the public endpoint; new mobile
builds and device acceptance remain required.
