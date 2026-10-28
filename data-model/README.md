# Shared application data contracts

Dependency-free Rust records and errors shared by Device Interface, Data capture
and Local data store. `SampleMetadata` carries source/receipt/product/record
provenance; `Observation` carries nullable values, units and quality.
`ObservationSink` is the injected write boundary, not a database implementation.

See [SAD §5.2](../architecture.md#52-building-block-responsibilities) and
[ADR-0002](../docs/decisions/0002-capture-storage-boundaries.md).
