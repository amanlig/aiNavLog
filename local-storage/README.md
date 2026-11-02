# Local data store

Confirmed SQLite/rusqlite local structured store for the 2027 iOS/Android beta.
`ObservationStore` owns connection setup, schema version/application checks,
transactions, WAL, indexes and read APIs. It depends on shared data contracts,
not Device Interface or Victron types. `save` accepts `SampleMetadata` and observations.

Telemetry table contents and the existing mobile file path are preserved:
`samples`, `observations`, `monitor_events`. `read_latest_bmv` and
`read_bmv_history` serve read-only UI queries independently of the capture pipeline
or receiver state. History preserves actual timestamps, extrema and discontinuities.
See [storage decision](../docs/decisions/0001-sqlite-local-storage.md),
[responsibility decision](../docs/decisions/0002-capture-storage-boundaries.md) and
[monitoring/storage guide](../docs/victron-live-monitoring.md).

Schema 2 adds `maintenance_equipment` and `maintenance_entries`, migrating schema 1
transactionally without rewriting telemetry. Stable equipment IDs support engine,
electrical or both; completed records retain local dates, manual optional hours,
integer minor-unit cost, currency, provider/parts/reference/notes and next thresholds.
A unique entry ID makes identical save retries idempotent and rejects conflicting
reuse. Foreign keys reject orphan service entries. Server creation timestamps are
separate from service dates. There are no fabricated completed records.

`maintenance_snapshot`, `save_equipment` and `save_service` are independent of the
receiver. Records currently belong to the app's single vessel; multi-vessel
ownership, attachments, corrections/deletion, scheduled reminders, general
retention/export/backup and protection policy remain unfinished. Schema-1 binaries
reject schema 2; rolling back the app requires a separately designed migration. Run `cargo test --locked --manifest-path
local-storage/Cargo.toml` from the repository root.
