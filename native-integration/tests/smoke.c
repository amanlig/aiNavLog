#include "ainavlog.h"
#include <assert.h>
#include <math.h>
#include <stdio.h>
#include <string.h>

int main(int argc, char **argv) {
    assert(argc == 3);
    FILE *log = fopen(argv[2], "rb");
    assert(log);
    uint8_t packet[255];
    size_t length = fread(packet, 1, sizeof packet, log);
    assert(!ferror(log) && feof(log));
    fclose(log);
    assert(length);
    const char *key = "000102030405060708090a0b0c0d0e0f";
    uint64_t handle = 0;
    ainavlog_electrical_sample sample;
    assert(ainavlog_open(argv[1], "bmv-simulator", key, &handle) == 0);
    assert(handle != 0);
    assert(ainavlog_latest(handle, &sample) == 1);
    assert(sample.present == 0);
    assert(ainavlog_ingest(handle, "other-device", 0x02e1, packet, length, 123456789) == 1);
    assert(ainavlog_ingest(handle, "bmv-simulator", 0x02e1, packet, length, -1) == -1);
    assert(ainavlog_ingest(handle, "bmv-simulator", 0x02e1, packet, length, 123456789) == 0);
    assert(ainavlog_ingest(handle, "bmv-simulator", 0x02e1, packet, length, 123456790) == 1);
    assert(ainavlog_latest(handle, &sample) == 0);
    assert(sample.present == 15 && sample.received_ms == 123456789);
    assert(fabs(sample.house_voltage - 12.76) < 1e-9);
    assert(fabs(sample.house_current + 5.25) < 1e-9);
    assert(fabs(sample.house_charge - 77.5) < 1e-9);
    assert(fabs(sample.starter_voltage - 12.8) < 1e-9);
    uint64_t closed = handle;
    assert(ainavlog_close(handle) == 0);
    assert(ainavlog_close(handle) == -1);
    assert(ainavlog_latest(handle, &sample) == -1 && sample.present == 0);
    assert(ainavlog_open(argv[1], "bmv-simulator", key, &handle) == 0);
    assert(handle != closed);
    assert(ainavlog_latest(handle, &sample) == 0);
    assert(sample.received_ms == 123456789 && sample.present == 15);
    assert(ainavlog_latest(handle, NULL) == -1);
    assert(ainavlog_close(handle) == 0);
    ainavlog_history_point history[8];
    size_t count = 99;
    assert(ainavlog_history(argv[1], "bmv-simulator", 123456000, 123457000, history, 8, &count) == 0);
    assert(count == 1 && history[0].sample.received_ms == 123456789);
    assert(history[0].segment == 1 && history[0].sample.present == 15);
    assert(fabs(history[0].sample.house_charge - 77.5) < 1e-9);
    assert(ainavlog_history(argv[1], "other", 123456000, 123457000, history, 8, &count) == 0 && count == 0);
    assert(ainavlog_history(argv[1], "bmv-simulator", 0, 0, history, 8, &count) == -2 && count == 0);
    assert(ainavlog_history(argv[1], "bmv-simulator", 0, 1, NULL, 8, &count) == -1);
    assert(ainavlog_open(argv[1], "bmv-simulator", "invalid", &handle) == -2);
    assert(handle == 0);
    puts("C ABI: decode, identity filter, duplicate filter, timestamp, values, restart, invalid handles passed");
}
