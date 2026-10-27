/** Stage only Rust source/locks/licenses under the app's existing upload boundary. */
import { cpSync, existsSync, mkdirSync, rmSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const app = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const source = resolve(app, '../device-interface');
const destination = resolve(app, 'modules/ainavlog/rust/device-interface');
if (!existsSync(resolve(source, 'native-bridge/src/mobile.rs'))) throw Error('Missing native Rust source');
rmSync(destination, { recursive: true, force: true });
mkdirSync(destination, { recursive: true });
for (const entry of ['Cargo.toml', 'Cargo.lock', 'src', 'vendor', 'tests']) cpSync(resolve(source, entry), resolve(destination, entry), { recursive: true });
mkdirSync(resolve(destination, 'native-bridge'));
for (const entry of ['Cargo.toml', 'Cargo.lock', 'src', 'include', 'build-ios.py']) {
  cpSync(resolve(source, 'native-bridge', entry), resolve(destination, 'native-bridge', entry), { recursive: true });
}
console.log('Staged shared Rust sources for app-scoped native builds.');
