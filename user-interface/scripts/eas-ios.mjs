/** Scope cloud uploads to the UI app, excluding private sibling projects. */
import { spawn, execFileSync } from 'node:child_process';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const action = process.argv[2];
const actions = {
  build: ['build', '--platform', 'ios', '--profile', 'testflight'],
  submit: ['submit', '--platform', 'ios', '--profile', 'testflight'],
  credentials: ['credentials', '--platform', 'ios'],
  whoami: ['whoami'],
  inspect: ['build:inspect', '--platform', 'ios', '--profile', 'testflight',
    '--stage', 'archive', '--output', resolve(projectRoot, 'builds/ios-archive')],
};
if (!actions[action]) {
  console.error('Usage: node scripts/eas-ios.mjs build|submit|credentials|inspect|whoami');
  process.exit(2);
}
if (['build', 'apk', 'inspect'].includes(action)) execFileSync(process.execPath, [resolve(projectRoot, 'scripts/prepare-native.mjs')], { stdio: ['ignore', 'ignore', 'inherit'] });
const child = spawn(process.platform === 'win32' ? 'npx.cmd' : 'npx',
  ['--yes', `eas-cli@${process.env.EAS_CLI_VERSION || 'latest'}`, ...actions[action], ...process.argv.slice(3)],
  { cwd: projectRoot, env: { ...process.env, EAS_NO_VCS: '1', EAS_PROJECT_ROOT: projectRoot,
    EXPO_NO_DOTENV: '1', EXPO_PUBLIC_DEMO_MODE: '0', EXPO_PUBLIC_NATIVE_BMV: '1', EXPO_PUBLIC_DEVICE_INTERFACE_URL: '' }, stdio: 'inherit' });
child.on('error', () => { console.error('Could not start EAS CLI. Check Node/npm installation.'); process.exitCode = 1; });
child.on('exit', code => { process.exitCode = code ?? 1; });
