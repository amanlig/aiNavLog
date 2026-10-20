"""Build and test the actual exported C ABI on Linux; no Bluetooth radio used."""
from pathlib import Path
import os
import subprocess
import tempfile

root = Path(__file__).resolve().parents[1]
subprocess.run(["cargo", "build", "--locked", "--offline"], cwd=root, check=True)
subprocess.run(["cargo", "clippy", "--locked", "--offline", "--", "-D", "warnings"], cwd=root, check=True)
with tempfile.TemporaryDirectory(prefix="ainavlog-native-") as work:
    executable = Path(work) / "smoke"
    library = root / "target/debug"
    subprocess.run([os.environ.get("CC", "cc"), "-std=c11", "-Wall", "-Wextra", "-Werror",
                    "-I", str(root / "include"), str(root / "tests/smoke.c"),
                    "-L", str(library), "-lainavlog_native_bridge", "-lm",
                    f"-Wl,-rpath,{library}", "-o", str(executable)], check=True)
    subprocess.run([str(executable), str(Path(work) / "samples.sqlite3"),
                    str(root.parents[1] / "simulation/bmv712/integration-packets.log")], check=True)
