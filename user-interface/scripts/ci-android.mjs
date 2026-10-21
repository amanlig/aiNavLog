/** Validate runner-only release settings and record the exact completed EAS build. */
import { appendFileSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const bundleId = 'com.kodekomplete.com.ainavlog';
const projectId = '6b2c9577-6a5b-4380-a200-d5221085458c';
const readJson = path => JSON.parse(readFileSync(path, 'utf8'));

export function prepare(app, eas, env) {
  if (!env.EXPO_TOKEN?.trim()) throw Error('Set EXPO_TOKEN in the ainavlog-play-internal GitHub environment secrets.');
  const number = env.ANDROID_VERSION_CODE;
  if (!/^[1-9]\d*$/.test(number || '') || !Number.isSafeInteger(Number(number)) || Number(number) <= 1 || Number(number) > 2100000000) {
    throw Error('Choose an unused integer Android version code above 1, at most 2100000000.');
  }
  if (app.expo?.android?.package !== bundleId || app.expo?.extra?.eas?.projectId !== projectId) throw Error('Unexpected Android package or Expo project identity.');
  if (eas.submit?.['play-internal']?.android?.track !== 'internal' || eas.submit?.['play-internal']?.android?.releaseStatus !== 'completed') throw Error('Expected completed release on the Play internal track.');
  if (eas.cli?.appVersionSource !== 'local' || eas.build?.['play-internal']?.autoIncrement !== false || eas.build?.['play-internal']?.distribution !== 'store' || eas.build?.['play-internal']?.credentialsSource !== 'remote') {
    throw Error('Review the Play internal versioning, distribution and remote-signing configuration.');
  }
  if (eas.build?.['play-internal']?.android?.buildType !== 'app-bundle') throw Error('Play requires an app bundle.');
  return { ...app, expo: { ...app.expo, android: { ...app.expo.android, versionCode: Number(number) } } };
}

export function buildReceipt(raw, app, env) {
  const builds = Array.isArray(raw) ? raw : [raw];
  if (builds.length !== 1) throw Error('Expected exactly one EAS Android build result.');
  const build = builds[0];
  if (!/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(build?.id || '') || build.platform !== 'ANDROID' || build.status !== 'FINISHED') {
    throw Error('EAS did not return one successfully finished Android build; submission blocked.');
  }
  if (build.appIdentifier !== bundleId) throw Error('Built bundle identity does not match aiNavLog.');
  if (String(build.appBuildVersion) !== String(app.expo.android.versionCode)) throw Error('Built Android number differs from the requested number.');
  if (build.appVersion !== app.expo.version) throw Error('Built app version differs from requested version.');
  if (build.app?.id !== projectId) throw Error('Built Expo project differs from expected project.');
  return {
    build_id: build.id, platform: build.platform, status: build.status,
    build_url: `https://expo.dev/accounts/${app.expo.owner}/projects/${app.expo.slug}/builds/${build.id}`,
    package_name: bundleId, expo_project_id: projectId,
    app_version: app.expo.version, android_version_code: app.expo.android.versionCode,
    source_commit: env.GITHUB_SHA, source_ref: env.GITHUB_REF,
    github_run: `${env.GITHUB_SERVER_URL}/${env.GITHUB_REPOSITORY}/actions/runs/${env.GITHUB_RUN_ID}`,
    run_attempt: env.GITHUB_RUN_ATTEMPT, eas_cli_version: env.EAS_CLI_VERSION,
  };
}

function main() {
  const appPath = resolve(root, 'app.json');
  if (process.argv[2] === 'prepare') {
    const app = prepare(readJson(appPath), readJson(resolve(root, 'eas.json')), process.env);
    writeFileSync(appPath, JSON.stringify(app, null, 2) + '\n');
    console.log(`Prepared ${bundleId}, version ${app.expo.version}, Android build ${app.expo.android.versionCode}.`);
  } else if (process.argv[2] === 'record') {
    if (!process.env.BUILD_RESULT_PATH) throw Error('Missing EAS build result path.');
    const receipt = buildReceipt(readJson(process.env.BUILD_RESULT_PATH), readJson(appPath), process.env);
    mkdirSync(resolve(root, 'builds'), { recursive: true });
    writeFileSync(resolve(root, 'builds/ci-receipt.json'), JSON.stringify(receipt, null, 2) + '\n');
    if (process.env.GITHUB_OUTPUT) appendFileSync(process.env.GITHUB_OUTPUT, `build_id=${receipt.build_id}\n`);
    if (process.env.GITHUB_STEP_SUMMARY) appendFileSync(process.env.GITHUB_STEP_SUMMARY,
      `## aiNavLog Android build\n\n- Commit: ${receipt.source_commit}\n- Version: ${receipt.app_version} (${receipt.android_version_code})\n- EAS: [${receipt.build_id}](${receipt.build_url})\n\nBuild receipt retained as a workflow artifact.\n`);
    console.log(`Verified completed Android build ${receipt.build_id}.`);
  } else throw Error('Usage: node scripts/ci-android.mjs prepare|record');
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try { main(); } catch (error) { console.error(error.message); process.exitCode = 1; }
}
