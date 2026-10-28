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
typedef struct {
    int64_t segment;
    ainavlog_electrical_sample sample;
} ainavlog_history_point;
/* Read-only [start,end) Unix milliseconds, max 32 days. Works disconnected.
 * Up to 8192 actual samples: bucket endpoints/extrema, with gap segment IDs.
 * Returns error rather than truncating; count is zero on failure.
 * Caller allocates output; insufficient capacity returns -1. Missing file = empty. */
int32_t ainavlog_history(const char *path, const char *source,
    int64_t start_ms, int64_t end_ms, ainavlog_history_point *output,
    size_t capacity, size_t *count);
/* Mobile feature: foreground public Windows BMV simulator only (product 1234).
 * start opens/reopens its private SQLite session and launches btleplug scanning.
 * Stop before closing/replacing its handle. All functions run off the UI thread.
 * status: 0 stopped, 10 starting/scanning, 11 receiving, -2 retrying after error.
 * Read freshness from received_ms; status 11 alone does not establish freshness.
 * stop joins the scanner and closes its session. Only one scanner is supported. */
int32_t ainavlog_simulator_start(const char *path, uint64_t *handle);
int32_t ainavlog_simulator_stop(void);
int32_t ainavlog_simulator_status(void);
typedef struct {
    uint64_t packets, samples, source_changes;
} ainavlog_simulator_counters;
/* Current session counters; stopped sessions return zero counters. */
int32_t ainavlog_simulator_diagnostics(ainavlog_simulator_counters *output);
#ifdef __ANDROID__
/* Call on a Java-entered thread after permissions, before start; env is JNIEnv*. */
int32_t ainavlog_android_init(void *env);
#endif
#ifdef __cplusplus
}
#endif
#endif
