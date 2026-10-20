/** Validate runner-only release settings and record the exact completed EAS build. */
import { appendFileSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const bundleId = 'com.kodekomplete.com.ainavlog';
const projectId = '6b2c9577-6a5b-4380-a200-d5221085458c';
const appleAppId = '6823786033';
const readJson = path => JSON.parse(readFileSync(path, 'utf8'));

export function prepare(app, eas, env) {
  if (!env.EXPO_TOKEN?.trim()) throw Error('Set EXPO_TOKEN in the ainavlog-testflight GitHub environment secrets.');
  const number = env.IOS_BUILD_NUMBER;
  if (!/^[1-9]\d*$/.test(number || '') || !Number.isSafeInteger(Number(number)) || Number(number) <= 1 || Number(number) > 2147483647) {
    throw Error('Choose an unused integer iOS build number above 1, at most 2147483647.');
  }
  if (app.expo?.ios?.bundleIdentifier !== bundleId || app.expo?.extra?.eas?.projectId !== projectId) throw Error('Unexpected iOS bundle or Expo project identity.');
  if (String(eas.submit?.testflight?.ios?.ascAppId) !== appleAppId) throw Error('Unexpected App Store Connect app ID.');
  if (eas.cli?.appVersionSource !== 'local' || eas.build?.testflight?.autoIncrement !== false || eas.build?.testflight?.distribution !== 'store' || eas.build?.testflight?.credentialsSource !== 'remote') {
    throw Error('Review the TestFlight versioning, distribution and remote-signing configuration.');
  }
  return { ...app, expo: { ...app.expo, ios: { ...app.expo.ios, buildNumber: number } } };
}

export function buildReceipt(raw, app, env) {
  const builds = Array.isArray(raw) ? raw : [raw];
  if (builds.length !== 1) throw Error('Expected exactly one EAS iOS build result.');
  const build = builds[0];
  if (!/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(build?.id || '') || build.platform !== 'IOS' || build.status !== 'FINISHED') {
    throw Error('EAS did not return one successfully finished iOS build; submission blocked.');
  }
  if (build.appIdentifier && build.appIdentifier !== bundleId) throw Error('Built bundle identity does not match aiNavLog.');
  if (build.appBuildVersion && String(build.appBuildVersion) !== app.expo.ios.buildNumber) throw Error('Built iOS number differs from the requested number.');
  return {
    build_id: build.id, platform: build.platform, status: build.status,
    build_url: `https://expo.dev/accounts/${app.expo.owner}/projects/${app.expo.slug}/builds/${build.id}`,
    bundle_identifier: bundleId, expo_project_id: projectId, apple_app_id: appleAppId,
    app_version: app.expo.version, ios_build_number: app.expo.ios.buildNumber,
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
    console.log(`Prepared ${bundleId}, version ${app.expo.version}, iOS build ${app.expo.ios.buildNumber}.`);
  } else if (process.argv[2] === 'record') {
    if (!process.env.BUILD_RESULT_PATH) throw Error('Missing EAS build result path.');
    const receipt = buildReceipt(readJson(process.env.BUILD_RESULT_PATH), readJson(appPath), process.env);
    mkdirSync(resolve(root, 'builds'), { recursive: true });
    writeFileSync(resolve(root, 'builds/ci-receipt.json'), JSON.stringify(receipt, null, 2) + '\n');
    if (process.env.GITHUB_OUTPUT) appendFileSync(process.env.GITHUB_OUTPUT, `build_id=${receipt.build_id}\n`);
    if (process.env.GITHUB_STEP_SUMMARY) appendFileSync(process.env.GITHUB_STEP_SUMMARY,
      `## aiNavLog iOS build\n\n- Commit: ${receipt.source_commit}\n- Version: ${receipt.app_version} (${receipt.ios_build_number})\n- EAS: [${receipt.build_id}](${receipt.build_url})\n\nBuild receipt retained as a workflow artifact.\n`);
    console.log(`Verified completed iOS build ${receipt.build_id}.`);
  } else throw Error('Usage: node scripts/ci-ios.mjs prepare|record');
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try { main(); } catch (error) { console.error(error.message); process.exitCode = 1; }
}
