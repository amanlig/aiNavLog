import assert from 'node:assert/strict';
import { chmodSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { delimiter, join } from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import { prepare, buildReceipt } from '../scripts/ci-android.mjs';

const app = JSON.parse(readFileSync(new URL('../app.json', import.meta.url), 'utf8'));
const eas = JSON.parse(readFileSync(new URL('../eas.json', import.meta.url), 'utf8'));
const env = { EXPO_TOKEN: 'test-token', ANDROID_VERSION_CODE: '2', GITHUB_SHA: 'source-sha' };
const build = { id: '12345678-1234-1234-1234-123456789abc', platform: 'ANDROID', status: 'FINISHED', appBuildVersion: '2', appIdentifier: app.expo.android.package, appVersion: app.expo.version, app: { id: app.expo.extra.eas.projectId } };

test('prepare changes only the runner Android number, preserving iOS and source objects', () => {
  const result = prepare(app, eas, env);
  assert.equal(result.expo.android.versionCode, 2);
  assert.equal(app.expo.android.versionCode, 1);
  assert.deepEqual(result.expo.ios, app.expo.ios);
});

test('credentials, invalid numbers and wrong application identity block release', () => {
  assert.throws(() => prepare(app, eas, {}), /EXPO_TOKEN/);
  for (const number of ['1', '0', '-2', '2;echo unsafe', '2.0', '999999999999999999999']) {
    assert.throws(() => prepare(app, eas, { ...env, ANDROID_VERSION_CODE: number }), /unused integer/);
  }
  const wrong = structuredClone(app); wrong.expo.android.package = 'other';
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
  for (const raw of [[], [build, build], { ...build, status: 'ERRORED' }, { ...build, platform: 'IOS' }, { ...build, id: 'bad\noutput=1' }, { ...build, appBuildVersion: '1' }, { ...build, appIdentifier: 'wrong' }, { ...build, appVersion: '9' }, { ...build, app: { id: 'wrong' } }, { ...build, appBuildVersion: undefined }, { ...build, appIdentifier: undefined }]) {
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
    const child = spawnSync(process.execPath, [fileURLToPath(new URL('../scripts/eas-android.mjs', import.meta.url)), 'submit', '--id', build.id, '--non-interactive', '--wait'], {
      cwd: temp, encoding: 'utf8', env: { ...process.env, PATH: temp + delimiter + process.env.PATH, EAS_CLI_VERSION: '24.13.0', CI_CAPTURE_FILE: join(temp, 'capture.json'), EXPO_PUBLIC_DEVICE_INTERFACE_URL: 'must-not-ship' },
    });
    assert.equal(child.status, 0, child.stderr);
    const capture = JSON.parse(readFileSync(join(temp, 'capture.json'), 'utf8'));
    assert.equal(capture.cwd, fileURLToPath(new URL('../', import.meta.url)).replace(/\/$/, ''));
    assert.equal(capture.root, capture.cwd);
    assert.equal(capture.vcs, '1');
    assert.equal(capture.demo, '1');
    assert.equal(capture.endpoint, '');
    assert.deepEqual(capture.args, ['--yes', 'eas-cli@24.13.0', 'submit', '--platform', 'android', '--profile', 'play-internal', '--id', build.id, '--non-interactive', '--wait']);
  } finally { rmSync(temp, { recursive: true, force: true }); }
});

test('unsafe build or submission profiles block preparation', () => {
  for (const change of [
    config => { config.submit['play-internal'].android.track = 'production'; },
    config => { config.submit['play-internal'].android.releaseStatus = 'draft'; },
    config => { config.build['play-internal'].autoIncrement = true; },
    config => { config.build['play-internal'].android.buildType = 'apk'; },
    config => { config.cli.appVersionSource = 'remote'; },
  ]) {
    const config = structuredClone(eas); change(config);
    assert.throws(() => prepare(app, config, env));
  }
  assert.throws(() => prepare(app, eas, { ...env, ANDROID_VERSION_CODE: '2100000001' }));
});

// EAS CLI 24.13.0 BuildFragment returns app.id, not project.id.
test('receipt requires EAS app.id even when legacy project.id matches', () => {
  const prepared = prepare(app, eas, env);
  assert.equal(buildReceipt(build, prepared, env).expo_project_id, app.expo.extra.eas.projectId);
  for (const raw of [
    { ...build, app: undefined },
    { ...build, app: undefined, project: { id: app.expo.extra.eas.projectId } },
    { ...build, app: { id: 'wrong' }, project: { id: app.expo.extra.eas.projectId } },
  ]) assert.throws(() => buildReceipt(raw, prepared, env), /Expo project/);
});
