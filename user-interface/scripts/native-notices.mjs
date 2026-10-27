/** Preserve shipped native dependency notices in the phone bundle. */
import { readFileSync, readdirSync, existsSync, mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';

export function writeNativeNotices(metadata, destination) {
  const reachable = new Set(metadata.resolve.nodes.map(node => node.id));
  const blocks = ['aiNavLog native third-party notices. Versions follow the native bridge Cargo.lock.'];
  for (const pkg of metadata.packages.filter(pkg => reachable.has(pkg.id) && pkg.source).sort((a, b) => a.name.localeCompare(b.name))) {
    const root = dirname(pkg.manifest_path);
    const files = [];
    const collect = (directory, depth) => {
      if (!existsSync(directory)) return;
      for (const item of readdirSync(directory, { withFileTypes: true })) {
        const path = join(directory, item.name);
        if (item.isFile() && /^(licen[sc]e|copying|copyright|notice|unlicense)([._-]|$)/i.test(item.name)) files.push(path);
        if (item.isDirectory() && depth > 0 && /^(licenses?|notices?|LICENSES)$/.test(item.name)) collect(path, depth - 1);
      }
    };
    collect(root, 2);
    if (pkg.license_file) {
      const path = join(root, pkg.license_file);
      if (existsSync(path) && !files.includes(path)) files.push(path);
    }
    if (pkg.name === 'btleplug') collect(join(root, 'src/droidplug/java'), 1);
    blocks.push(`\n=== ${pkg.name} ${pkg.version} ===\nDeclared license: ${pkg.license || 'see notice files'}\n${pkg.repository || ''}`);
    for (const file of files) blocks.push(readFileSync(file, 'utf8'));
    if (pkg.name === 'libsqlite3-sys') blocks.push('SQLite is in the public domain. https://www.sqlite.org/copyright.html');
  }
  blocks.push('Android Java helper code vendored by btleplug 0.13.3: jni-utils-rs.\nUpstream notice: https://github.com/deviceplug/jni-utils-rs/blob/master/LICENSE');
  blocks.push(readFileSync(new URL('../modules/ainavlog/third-party/JNI-UTILS-LICENSE.txt', import.meta.url), 'utf8'));
  mkdirSync(dirname(destination), { recursive: true });
  writeFileSync(destination, blocks.join('\n\n') + '\n');
}
