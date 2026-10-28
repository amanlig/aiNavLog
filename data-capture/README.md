# Data capture

`CaptureService` validates normalized observations and coordinates atomic saves
through `ainavlog-local-storage`. It implements the shared `ObservationSink`.
It rejects malformed/mismatched provenance, duplicate quantities, non-finite values
and missing values incorrectly marked Good before writing. Unknown/missing values
remain explicit. Receiver events use the same write boundary.

No Bluetooth, protocol decoding or UI queries belong here. Manual/media/voice
capture and maintenance workflows are not implemented by this observation service.
Run `cargo test --locked --manifest-path data-capture/Cargo.toml` from the repository root.
See [ADR-0002](../docs/decisions/0002-capture-storage-boundaries.md).
