import assert from 'node:assert/strict';
import { chmodSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { delimiter, join } from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import { prepare, buildReceipt } from '../scripts/ci-ios.mjs';

const app = JSON.parse(readFileSync(new URL('../app.json', import.meta.url), 'utf8'));
const eas = JSON.parse(readFileSync(new URL('../eas.json', import.meta.url), 'utf8'));
const env = { EXPO_TOKEN: 'test-token', IOS_BUILD_NUMBER: '2', GITHUB_SHA: 'source-sha' };
const build = { id: '12345678-1234-1234-1234-123456789abc', platform: 'IOS', status: 'FINISHED', appBuildVersion: '2' };

test('prepare changes only the runner iOS number, preserving Android and source objects', () => {
  const result = prepare(app, eas, env);
  assert.equal(result.expo.ios.buildNumber, '2');
  assert.equal(app.expo.ios.buildNumber, '1');
  assert.deepEqual(result.expo.android, app.expo.android);
});

test('credentials, invalid numbers and wrong application identity block release', () => {
  assert.throws(() => prepare(app, eas, {}), /EXPO_TOKEN/);
  for (const number of ['1', '0', '-2', '2;echo unsafe', '2.0', '999999999999999999999']) {
    assert.throws(() => prepare(app, eas, { ...env, IOS_BUILD_NUMBER: number }), /unused integer/);
  }
  const wrong = structuredClone(app); wrong.expo.ios.bundleIdentifier = 'other';
  assert.throws(() => prepare(wrong, eas, env), /identity/);
});

test('receipt retains exact successful build and source without tokens', () => {
  const result = buildReceipt([build], prepare(app, eas, env), env);
  assert.equal(result.build_id, build.id);
  assert.equal(result.source_commit, env.GITHUB_SHA);
  assert.ok(!JSON.stringify(result).includes(env.EXPO_TOKEN));
});

test('failed, ambiguous, wrong-platform or wrong-version builds cannot be submitted', () => {
  const prepared = prepare(app, eas, env);
  for (const raw of [[], [build, build], { ...build, status: 'ERRORED' }, { ...build, platform: 'ANDROID' }, { ...build, id: 'bad\noutput=1' }, { ...build, appBuildVersion: '1' }, { ...build, appIdentifier: 'wrong' }]) {
    assert.throws(() => buildReceipt(raw, prepared, env));
  }
});

test('EAS wrapper keeps release source app-scoped and forwards the exact build ID', { skip: process.platform === 'win32' }, () => {
  const temp = mkdtempSync(join(tmpdir(), 'ainavlog-ci-'));
  try {
    const mock = join(temp, 'npx');
    writeFileSync(mock, `#!/usr/bin/env node
require('node:fs').writeFileSync(process.env.CI_CAPTURE_FILE, JSON.stringify({ args: process.argv.slice(2), cwd: process.cwd(), root: process.env.EAS_PROJECT_ROOT, vcs: process.env.EAS_NO_VCS, demo: process.env.EXPO_PUBLIC_DEMO_MODE, endpoint: process.env.EXPO_PUBLIC_DEVICE_INTERFACE_URL }));
`);
    chmodSync(mock, 0o755);
    const child = spawnSync(process.execPath, [fileURLToPath(new URL('../scripts/eas-ios.mjs', import.meta.url)), 'submit', '--id', build.id, '--non-interactive', '--wait'], {
      cwd: temp, encoding: 'utf8', env: { ...process.env, PATH: temp + delimiter + process.env.PATH, EAS_CLI_VERSION: '24.13.0', CI_CAPTURE_FILE: join(temp, 'capture.json'), EXPO_PUBLIC_DEVICE_INTERFACE_URL: 'must-not-ship' },
    });
    assert.equal(child.status, 0, child.stderr);
    const capture = JSON.parse(readFileSync(join(temp, 'capture.json'), 'utf8'));
    assert.equal(capture.cwd, fileURLToPath(new URL('../', import.meta.url)).replace(/\/$/, ''));
    assert.equal(capture.root, capture.cwd);
    assert.equal(capture.vcs, '1');
    assert.equal(capture.demo, '1');
    assert.equal(capture.endpoint, '');
    assert.deepEqual(capture.args, ['--yes', 'eas-cli@24.13.0', 'submit', '--platform', 'ios', '--profile', 'testflight', '--id', build.id, '--non-interactive', '--wait']);
  } finally { rmSync(temp, { recursive: true, force: true }); }
});
