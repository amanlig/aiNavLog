# ADR-0002 — Separate device communication, capture and storage

**Status:** Accepted and implemented for the electrical slice. **Date:** October 28, 2026.

## Decision

Device Interface owns instrument transport, decoding/normalization and receiver
state. Data capture validates incoming observations and coordinates durable saves.
Local data store owns SQLite and record queries. UI reads go through query APIs
directly to storage; they do not traverse capture or require Bluetooth.
Native integration assembles these dependencies and translates platform calls.

| Responsibility | Physical package |
| --- | --- |
| Shared contracts and injected write interface | `data-model/` |
| Bluetooth, protocol decoding, monitor state | `device-interface/` |
| Observation validation and write coordination | `data-capture/` |
| SQLite schema, transactions and queries | `local-storage/` |
| C ABI, sessions, mobile worker, desktop supervisor/HTTP/CLI | `native-integration/` |
| React Native client plus Swift/Kotlin/JNI adapters | `user-interface/` |

`VictronMonitor<S: ObservationSink>` emits normalized observations and
`SampleMetadata` into `CaptureService`; it never opens or queries a database.
`CaptureService` validates provenance, required fields, unique quantities, finite
values and missing-value quality before calling `ObservationStore::save`.
The store accepts shared metadata, not a Victron-specific message.
`ainavlog_latest` and `ainavlog_history` call storage directly.

Device Interface has no runtime dependency on storage/capture/rusqlite. Storage
and capture have no dependency on Device Interface, Bluetooth or vendor types.
The native composition package depends on all three and injects the capture sink.
Dependency directions are covered by a regression check. Device integration tests
may intentionally use capture/storage as development-only dependencies.

## Consequences

The old nested `device-interface/native-bridge/` is moved to top-level
`native-integration/`. Its package/library name `ainavlog-native-bridge` and C ABI
names stay stable. The desktop executable name is retained but its Cargo host is
now `native-integration` with `--features desktop`; build/staging scripts and
current runnable guides are updated. All five sibling crates are staged within
the app's existing cloud-upload boundary; no unrelated/private files are added.

Database filenames, schema version 1, application ID, rows and receipt timestamps
are unchanged. No existing database is deleted or migrated. Bluetooth simulator
selection, duplicate suppression and freshness rules remain unchanged. Failed
capture cannot mark data committed or refresh freshness/dedup state.

The observation slice is implemented. Media/manual/voice capture, maintenance
CRUD, broader schemas and storage policies remain future work. New native binaries
and physical-device regression remain required; this refactor performs no store
submission or release. See [SAD §5.2](../../architecture.md#52-building-block-responsibilities)
and [ADR-0001](0001-sqlite-local-storage.md).
