"""Package the C bridge for iPhone and Apple Silicon simulators on a Mac.

No signing, app build, upload, or Rust target installation is performed.
"""
import argparse
import os
from pathlib import Path
import platform
import shutil
import subprocess
import tempfile


def check_link(library, headers, target, work):
    """Link, but never run, a real C caller using the app's native dependencies."""
    simulator = target.endswith("-sim")
    sdk = "iphonesimulator" if simulator else "iphoneos"
    triple = "arm64-apple-ios16.4" + ("-simulator" if simulator else "")
    sysroot = subprocess.check_output(
        ["xcrun", "--sdk", sdk, "--show-sdk-path"], text=True).strip()
    source = work / "check-abi.c"
    source.write_text('''#include "ainavlog.h"
int main(int argc, char **argv) {
    if (argc < 2) return 1;
    uint64_t handle = 0;
    ainavlog_electrical_sample sample = {0};
    int32_t status = ainavlog_simulator_start(argv[1], &handle);
    status += ainavlog_latest(handle, &sample);
    status += ainavlog_simulator_status();
    ainavlog_simulator_counters counters = {0};
    status += ainavlog_simulator_diagnostics(&counters);
    status += ainavlog_simulator_stop();
    return status;
}
''')
    subprocess.run([
        "xcrun", "--sdk", sdk, "clang", "-target", triple, "-isysroot", sysroot,
        "-I", str(headers), str(source), str(library),
        "-framework", "Foundation", "-framework", "CoreBluetooth",
        "-framework", "CoreFoundation", "-framework", "Security",
        "-liconv", "-lresolv", "-lc++", "-lobjc",
        "-o", str(work / f"check-abi-{target}"),
    ], check=True)
    print(f"Verified Rust mobile C ABI links for {target}")


def main():
    root = Path(__file__).resolve().parent
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path,
                        default=root / "builds/AiNavLogBridge.xcframework")
    args = parser.parse_args()
    output = args.output.resolve()
    if platform.system() != "Darwin":
        parser.error("iOS packaging requires macOS with full Xcode selected")
    for command in ("cargo", "rustup", "xcrun", "xcodebuild"):
        if not shutil.which(command):
            parser.error(f"missing required command: {command}")
    if output.exists():
        parser.error(f"output already exists; choose a new --output: {output}")
    targets = ("aarch64-apple-ios", "aarch64-apple-ios-sim")
    installed = subprocess.check_output(
        ["rustup", "target", "list", "--installed"], text=True).splitlines()
    missing = [target for target in targets if target not in installed]
    if missing:
        parser.error("install required targets first: rustup target add " + " ".join(missing))
    for sdk in ("iphoneos", "iphonesimulator"):
        subprocess.run(["xcrun", "--sdk", sdk, "--show-sdk-path"], check=True)
    output.parent.mkdir(parents=True, exist_ok=True)
    # Isolate target output from host builds and discard partial packages on error.
    with tempfile.TemporaryDirectory(prefix="ainavlog-ios-", dir=output.parent) as work:
        work = Path(work)
        env = dict(os.environ, CARGO_TARGET_DIR=str(work / "target"))
        headers = work / "include"
        headers.mkdir()
        shutil.copy2(root / "include/ainavlog.h", headers / "ainavlog.h")
        (headers / "module.modulemap").write_text(
            'module AiNavLogBridge {\n  header "ainavlog.h"\n  export *\n}\n')
        command = ["xcodebuild", "-create-xcframework"]
        for target in targets:
            subprocess.run(["cargo", "build", "--locked", "--release", "--lib", "--features", "mobile",
                            "--target", target], cwd=root, env=env, check=True)
            library = work / "target" / target / "release/libainavlog_native_bridge.a"
            if not library.is_file():
                raise RuntimeError(f"Rust did not produce {library}")
            check_link(library, headers, target, work)
            command.extend(["-library", str(library), "-headers", str(headers)])
        package = work / "AiNavLogBridge.xcframework"
        subprocess.run(command + ["-output", str(package)], check=True)
        package.rename(output)
    print(f"Created {output}")


if __name__ == "__main__":
    main()
