import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

test('device runtime depends only on shared contracts, not capture or SQLite storage', () => {
  const manifest=readFileSync(new URL('../../device-interface/Cargo.toml',import.meta.url),'utf8');
  const dependencies=manifest.split('[dependencies]')[1].split('[target.')[0];
  assert(!/rusqlite|ainavlog-local-storage|ainavlog-data-capture/.test(dependencies));
  const store=readFileSync(new URL('../../local-storage/Cargo.toml',import.meta.url),'utf8');
  assert(!/ainavlog-device-interface|btleplug/.test(store));
  const capture=readFileSync(new URL('../../data-capture/Cargo.toml',import.meta.url),'utf8');
  assert(!/ainavlog-device-interface|btleplug/.test(capture));
});
// Staging is checked separately using the actual source graph and Cargo metadata;
// avoid mutating the application's generated upload directory from npm tests.
