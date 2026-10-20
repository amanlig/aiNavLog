#ifndef AINAVLOG_H
#define AINAVLOG_H
#include <stdint.h>
#include <stddef.h>
#ifdef __cplusplus
extern "C" {
#endif
/* Status: 0 success, 1 ignored/no sample, 2 replacement key required,
 * -1 invalid argument/handle, -2 device/storage failure, -3 internal failure.
 * Strings: caller-owned NUL-terminated UTF-8. Buffers must remain valid for calls.
 * Calls are synchronous and serialized. Dispatch off the platform UI thread.
 * No Rust allocations cross this ABI. Handles are never reused. */
typedef struct {
    int64_t received_ms;
    uint32_t present; /* bits 0..3: voltage, current, SOC, starter voltage */
    double house_voltage, house_current, house_charge, starter_voltage;
} ainavlog_electrical_sample;
int32_t ainavlog_open(const char *path, const char *source, const char *key, uint64_t *handle);
int32_t ainavlog_close(uint64_t handle);
/* Company ID supplied separately; payload excludes that prefix. Receipt time is
 * original Unix milliseconds. Source must match the session's selected identity. */
int32_t ainavlog_ingest(uint64_t handle, const char *source, uint16_t company,
    const uint8_t *payload, size_t length, int64_t received_ms);
/* Output is zeroed on failure/no sample. Values with absent bits are unavailable.
 * Native wrapper must map absent values to JS null and retain received_ms for
 * UI freshness checks. Source/live-vs-simulated metadata belongs to the adapter. */
int32_t ainavlog_latest(uint64_t handle, ainavlog_electrical_sample *output);
#ifdef __cplusplus
}
#endif
#endif
