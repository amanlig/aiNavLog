/** EAS post-install: catch an omitted Rust archive before an expensive Xcode build. */
import { existsSync, readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

export function assertRustLinked(configuration) {
  // CocoaPods emits quoted or unquoted -l flags depending on its version.
  if (!/-l\s*"?ainavlog_native_bridge"?(?:\s|$)/m.test(configuration)) {
    throw Error('CocoaPods omitted libainavlog_native_bridge from OTHER_LDFLAGS. Check the pod-root XCFramework path.');
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url) && process.env.EXPO_PUBLIC_NATIVE_BMV === '1' && process.env.EAS_BUILD_PLATFORM === 'ios') {
  const app = resolve(dirname(fileURLToPath(import.meta.url)), '..');
  const pod = join(app, 'modules/ainavlog/ios');
  for (const file of ['builds/AiNavLogBridge.xcframework/Info.plist', 'include/ainavlog.h', 'builds/notices/NativeNotices.txt']) {
    if (!existsSync(join(pod, file))) throw Error(`Missing native pod artifact: ${file}`);
  }
  const config = join(app, 'ios/Pods/Target Support Files/Pods-aiNavLog/Pods-aiNavLog.release.xcconfig');
  assertRustLinked(readFileSync(config, 'utf8'));
  console.log('Verified CocoaPods links the Rust mobile archive.');
}
