import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { mkdtempSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';

const cmake = process.env.CMAKE || 'cmake';
const available = [cmake, 'cc', 'readelf'].every(tool => spawnSync(tool, ['--version']).status === 0);
test('Android JNI guard rejects absolute worker dependencies and accepts a portable SONAME', { skip: !available }, () => {
  const dir = mkdtempSync(join(tmpdir(), 'ainavlog-needed-'));
  try {
    const rust = join(dir, 'libainavlog_native_bridge.so');
    const jni = join(dir, 'libainavlog_jni.so');
    writeFileSync(join(dir, 'rust.c'), 'int ainavlog_simulator_status(void) { return 0; }');
    writeFileSync(join(dir, 'jni.c'), 'extern int ainavlog_simulator_status(void); int snapshot(void) { return ainavlog_simulator_status(); }');
    const check = () => spawnSync(cmake, ['-DREADELF=readelf', `-DLIBRARY=${jni}`, '-P',
      fileURLToPath(new URL('../modules/ainavlog/android/src/main/cpp/check-needed.cmake', import.meta.url))], { encoding: 'utf8' });
    const link = () => execFileSync('cc', ['-shared', '-fPIC', join(dir, 'jni.c'), rust, '-o', jni]);
    execFileSync('cc', ['-shared', '-fPIC', join(dir, 'rust.c'), '-o', rust]);
    link();
    const broken = check();
    assert.notEqual(broken.status, 0);
    assert.match(broken.stderr, /must depend on/);
    execFileSync('cc', ['-shared', '-fPIC', join(dir, 'rust.c'), '-Wl,-soname,libainavlog_native_bridge.so', '-o', rust]);
    link();
    const fixed = check();
    assert.equal(fixed.status, 0, fixed.stderr);
  } finally { rmSync(dir, { recursive: true, force: true }); }
});
