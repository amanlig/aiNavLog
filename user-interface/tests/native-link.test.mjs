import assert from 'node:assert/strict';
import test from 'node:test';
import { assertRustLinked } from '../scripts/check-native-link.mjs';

test('native preflight rejects the failed build configuration with only the Expo wrapper', () => {
  assert.throws(() => assertRustLinked('OTHER_LDFLAGS = $(inherited) -ObjC -l"AiNavLog" -framework "CoreBluetooth"'), /omitted/);
});
test('native preflight requires the exact Rust archive link flag', () => {
  for (const flag of ['-l"ainavlog_native_bridge"', '-lainavlog_native_bridge']) {
    assert.doesNotThrow(() => assertRustLinked(`OTHER_LDFLAGS = $(inherited) -l"AiNavLog" ${flag} -framework "CoreBluetooth"`));
  }
  assert.throws(() => assertRustLinked('LIBRARY_SEARCH_PATHS = /some/ainavlog_native_bridge'), /omitted/);
  assert.throws(() => assertRustLinked('OTHER_LDFLAGS = -lainavlog_native_bridge_wrong'), /omitted/);
});
