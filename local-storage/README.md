# Local data store

Confirmed SQLite/rusqlite local structured store for the 2027 iOS/Android beta.
`ObservationStore` owns connection setup, schema version/application checks,
transactions, WAL, indexes and read APIs. It depends on shared data contracts,
not Device Interface or Victron types. `save` accepts `SampleMetadata` and observations.

The existing schema, file path and persisted contents are unchanged:
`samples`, `observations`, `monitor_events`. `read_latest_bmv` and
`read_bmv_history` serve read-only UI queries independently of the capture pipeline
or receiver state. History preserves actual timestamps, extrema and discontinuities.
See [storage decision](../docs/decisions/0001-sqlite-local-storage.md),
[responsibility decision](../docs/decisions/0002-capture-storage-boundaries.md) and
[monitoring/storage guide](../docs/victron-live-monitoring.md).

Maintenance schema, automatic retention, general migrations, export/deletion and
protection policy remain unfinished. Run `cargo test --locked --manifest-path
local-storage/Cargo.toml` from the repository root.
