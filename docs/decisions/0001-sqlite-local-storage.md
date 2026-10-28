# ADR-0001 — SQLite for device-local beta storage

**Status:** Accepted by the product owner. **Decision date:** October 28, 2026.
**Scope:** aiNavLog 2027 beta on iOS and Android.

## Decision

Use **SQLite** as the local database engine for structured application records,
with **Rust `rusqlite` and bundled SQLite** as the shared database implementation
on both mobile platforms. This is the confirmed beta choice, not a temporary
evaluation database or an unresolved technology selection.

The scope includes instrument samples/observations, reception events, Historical
Usage queries, Maintenance Logs and future structured equipment, entry and route
records as those features are implemented. Selecting the engine does not mark
these schemas, APIs or UI workflows complete.

Media may remain local files with metadata/references in SQLite; media-file layout
and consistency rules still need design. Existing boat-name/appearance settings
remain in AsyncStorage; this decision does not require moving them. Authentication
secrets belong in platform-protected credential storage, not observation tables.
Cloud synchronization/general upload remain outside the confirmed beta scope.

## Rationale

The beta stores data locally and operates offline. The existing Rust observation
store already uses SQLite, allowing common schema/query/migration logic on both
platforms. Bundling SQLite avoids depending on differing OS-provided SQLite builds.
No database server or separate per-platform database implementation is required.

## Current implementation and outstanding work

Schema version 1 contains `samples`, `observations` and `monitor_events`.
The monitor commits samples/measurements atomically; WAL and foreign keys are
enabled. The native bridge exposes latest BMV readings and a disconnected range-history
query API; Rust and desktop HTTP provide the same chart-history query. History
native binary delivery and phone acceptance remain pending. The database is currently `bmv-simulator.sqlite3` in private app
storage; no production filename transition or data migration is performed by this
ADR. The current database itself is unencrypted and has no automatic retention.

Finalize stable equipment/source identities, maintenance schema and
maintenance bridge APIs, versioned migrations, retention/capacity, export/deletion,
data protection, realistic-volume checks and restart/app-update retention tests.
Database-file count/layout and media handling are still design choices. Do not
claim physical Android qualification or feature acceptance from this decision.

## Related documentation

- [SAD](../../architecture.md#546-user-device-and-local-persistence)
- [Project plan](../project-plan.md)
- [Device Interface implementation view](../device-interface-implementation-view.md)
- [User Interface implementation view](../user-interface-implementation-view.md)
- [Current SQLite implementation](../../local-storage/src/lib.rs)

Ownership is now implemented by the independent `local-storage/` package, with writes validated by `data-capture/` and queries routed by `native-integration/`; see [ADR-0002](0002-capture-storage-boundaries.md).
