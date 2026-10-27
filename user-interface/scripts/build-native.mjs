/** EAS pre-install: build native artifacts before CocoaPods/Gradle inspect them. */
import { execFileSync } from 'node:child_process';
import { cpSync, existsSync, mkdirSync, readdirSync, rmSync } from 'node:fs';
import { homedir, platform } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { writeNativeNotices } from './native-notices.mjs';

if (process.env.EXPO_PUBLIC_NATIVE_BMV !== '1') process.exit(0);
const app = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const moduleRoot = join(app, 'modules/ainavlog');
const bridge = join(moduleRoot, 'rust/device-interface/native-bridge');
if (!existsSync(join(bridge, 'Cargo.lock'))) throw Error('Run node scripts/prepare-native.mjs before uploading this build.');
const env = { ...process.env, PATH: `${join(homedir(), '.cargo/bin')}:${process.env.PATH}` };
const run = (cmd, args, options = {}) => execFileSync(cmd, args, { cwd: bridge, env, stdio: 'inherit', ...options });
const rustVersion = '1.98.1';
try { run('rustup', ['--version']); } catch {
  const installer = join(moduleRoot, 'rustup-init.sh');
  run('curl', ['--proto', '=https', '--tlsv1.2', '-fsSL', 'https://sh.rustup.rs', '-o', installer]);
  run('sh', [installer, '-y', '--profile', 'minimal', '--default-toolchain', rustVersion]);
  rmSync(installer);
}
run('rustup', ['toolchain', 'install', rustVersion, '--profile', 'minimal']);
env.RUSTUP_TOOLCHAIN = rustVersion;
// CocoaPods only discovers vendored artifacts inside the pod's root directory.
const output = process.env.EAS_BUILD_PLATFORM === 'ios'
  ? join(moduleRoot, 'ios/builds') : join(moduleRoot, 'builds');
mkdirSync(output, { recursive: true });
const target = process.env.EAS_BUILD_PLATFORM === 'ios' ? 'aarch64-apple-ios' : 'aarch64-linux-android';
const metadata = JSON.parse(run('cargo', ['metadata', '--locked', '--features', 'mobile', '--filter-platform', target, '--format-version', '1'], { stdio: ['ignore', 'pipe', 'inherit'], encoding: 'utf8' }));
writeNativeNotices(metadata, join(output, 'notices/NativeNotices.txt'));
if (process.env.EAS_BUILD_PLATFORM === 'ios') {
  if (platform() !== 'darwin') throw Error('iOS requires a macOS worker');
  // Expose the ABI through the Expo pod's umbrella module; the XCFramework
  // supplies its target-specific static archive at link time.
  cpSync(join(bridge, 'include'), join(moduleRoot, 'ios/include'), { recursive: true });
  run('rustup', ['target', 'add', 'aarch64-apple-ios', 'aarch64-apple-ios-sim']);
  const framework = join(output, 'AiNavLogBridge.xcframework');
  rmSync(framework, { recursive: true, force: true });
  run('python3', ['build-ios.py', '--output', framework]);
} else if (process.env.EAS_BUILD_PLATFORM === 'android') {
  const sdk = env.ANDROID_HOME || env.ANDROID_SDK_ROOT;
  let ndk = env.ANDROID_NDK_HOME || env.ANDROID_NDK_ROOT;
  if (!ndk && sdk) {
    const versions = readdirSync(join(sdk, 'ndk')).sort((a, b) => a.localeCompare(b, undefined, { numeric: true }));
    ndk = join(sdk, 'ndk', versions.at(-1));
  }
  if (!ndk) throw Error('Android NDK not found');
  const tools = join(ndk, 'toolchains/llvm/prebuilt', platform() === 'darwin' ? 'darwin-x86_64' : 'linux-x86_64', 'bin');
  const targets = [['aarch64-linux-android', 'arm64-v8a'], ['x86_64-linux-android', 'x86_64']];
  run('rustup', ['target', 'add', ...targets.map(([target]) => target)]);
  for (const [target, abi] of targets) {
    const compiler = join(tools, `${target}24-clang`);
    const targetEnv = {
      ...env, [`CARGO_TARGET_${target.toUpperCase().replaceAll('-', '_')}_LINKER`]: compiler,
      [`CC_${target.replaceAll('-', '_')}`]: compiler,
      [`AR_${target.replaceAll('-', '_')}`]: join(tools, 'llvm-ar'),
      RUSTFLAGS: '-C link-arg=-Wl,-z,max-page-size=16384 -C link-arg=-Wl,-soname,libainavlog_native_bridge.so',
    };
    run('cargo', ['build', '--locked', '--release', '--lib', '--features', 'mobile', '--target', target], { env: targetEnv });
    const directory = join(output, 'jniLibs', abi);
    mkdirSync(directory, { recursive: true });
    cpSync(join(bridge, 'target', target, 'release/libainavlog_native_bridge.so'), join(directory, 'libainavlog_native_bridge.so'));
  }
  const btleplug = metadata.packages.find(pkg => pkg.name === 'btleplug' && pkg.version === '0.13.3');
  if (!btleplug) throw Error('Expected btleplug 0.13.3');
  const javaSource = join(dirname(btleplug.manifest_path), 'src/droidplug/java/src/main/java');
  cpSync(javaSource, join(output, 'btleplug-java'), { recursive: true });
  cpSync(join(bridge, 'include'), join(output, 'include'), { recursive: true });
} else throw Error('Unknown native build platform');
console.log('Rust btleplug mobile artifacts ready.');
