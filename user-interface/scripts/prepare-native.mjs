/** Stage the complete sibling-crate graph inside the app upload boundary. */
import { cpSync, existsSync, mkdirSync, rmSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

export function stageNativeSources(app) {
  const root = resolve(app, '..');
  const destination = resolve(app, 'modules/ainavlog/rust');
  const crates = ['data-model', 'local-storage', 'data-capture', 'device-interface', 'native-integration'];
  for (const crate of crates) {
    for (const entry of ['Cargo.toml', 'src']) {
      if (!existsSync(resolve(root, crate, entry))) throw Error(`Missing native source: ${crate}/${entry}`);
    }
  }
  rmSync(destination, { recursive: true, force: true });
  for (const crate of crates) {
    mkdirSync(resolve(destination, crate), { recursive: true });
    for (const entry of ['Cargo.toml', 'Cargo.lock', 'src', 'vendor', 'tests', 'include', 'build-ios.py']) {
      const source = resolve(root, crate, entry);
      if (existsSync(source)) cpSync(source, resolve(destination, crate, entry), { recursive: true });
    }
  }
  return destination;
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  stageNativeSources(resolve(dirname(fileURLToPath(import.meta.url)), '..'));
  console.log('Staged shared contracts, storage, capture, device and native integration sources.');
}
