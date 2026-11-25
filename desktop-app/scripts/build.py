"""Build the Windows app and standard NSIS setup EXE.
Example: python3 desktop-app/scripts/build.py --llvm-mingw /path/to/llvm-mingw --makensis /path/to/makensis
Downloads/installs nothing. Requires Rust Windows gnullvm target, UI dependencies and NSIS.
"""
from pathlib import Path
import argparse, hashlib, json, os, shutil, subprocess, tomllib, sys
app=Path(__file__).resolve().parents[1]
root=app.parent
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--llvm-mingw',type=Path,required=True)
parser.add_argument('--makensis',default=shutil.which('makensis'))
parser.add_argument('--nsis-root',type=Path)
parser.add_argument('--app-only',action='store_true',help='Build app files only; not a user-delivery artifact')
args=parser.parse_args()
if not args.app_only and not args.makensis:parser.error('NSIS makensis is required for the standard setup EXE. See docs/windows-delivery.md; do not substitute a ZIP.')
tools=args.llvm_mingw.resolve()
assert (tools/'bin/x86_64-w64-mingw32-clang').is_file(), 'Expected LLVM-MinGW bin directory'
env={**os.environ,'EXPO_NO_DOTENV':'1','EXPO_PUBLIC_NATIVE_BMV':'0','EXPO_PUBLIC_DEMO_MODE':'0','EXPO_PUBLIC_DEVICE_INTERFACE_URL':'','EXPO_PUBLIC_DIAGRAM_SCAN_URL':'https://ainavlog.kodekomplete.com'}
subprocess.run(['node','node_modules/expo/bin/cli','export','--platform','web','--clear','--output-dir',str(app/'web')],cwd=root/'user-interface',env=env,check=True)
output=app/'builds/windows-x64';output.mkdir(parents=True,exist_ok=True)
metadata=subprocess.check_output(['cargo','metadata','--manifest-path',str(app/'Cargo.toml'),'--locked','--offline','--filter-platform','x86_64-pc-windows-gnullvm','--format-version','1'],cwd=root)
metadata_file=app/'builds/metadata-windows.json';metadata_file.write_bytes(metadata)
script="""import {readFileSync} from 'node:fs';import {writeNativeNotices} from './user-interface/scripts/native-notices.mjs';writeNativeNotices(JSON.parse(readFileSync(process.argv[1],'utf8')),process.argv[2],{includeAndroidHelper:false});"""
subprocess.run(['node','--input-type=module','-e',script,str(metadata_file),str(app/'web/native-notices.txt')],cwd=root,check=True)
# Static LLVM unwinder avoids shipping an extra libunwind.dll.
llvm_license=tools/'LICENSE.txt'
if not llvm_license.exists():llvm_license=tools/'LICENSE.TXT'
if llvm_license.exists():
    with (app/'web/native-notices.txt').open('a') as f:f.write('\nLLVM-MinGW compiler/runtime notices:\n'+llvm_license.read_text())
env.update(PATH=str(tools/'bin')+os.pathsep+env['PATH'],CARGO_TARGET_X86_64_PC_WINDOWS_GNULLVM_LINKER='x86_64-w64-mingw32-clang',CARGO_TARGET_X86_64_PC_WINDOWS_GNULLVM_RUSTFLAGS='-C target-feature=+crt-static -C link-arg=-static',CC_x86_64_pc_windows_gnullvm='x86_64-w64-mingw32-clang',AR_x86_64_pc_windows_gnullvm='llvm-ar')
subprocess.run(['cargo','build','--manifest-path',str(app/'Cargo.toml'),'--release','--locked','--offline','--target','x86_64-pc-windows-gnullvm'],cwd=root,env=env,check=True)
executable=app/'target/x86_64-pc-windows-gnullvm/release/aiNavLog.exe'
shutil.copy2(executable,output/'aiNavLog.exe')
shutil.copy2(app/'web/native-notices.txt',output/'NativeNotices.txt')
shutil.copy2(app/'README.md',output/'README.txt')
version=tomllib.loads((app/'Cargo.toml').read_text())['package']['version']
receipt={'version':version,'target':'x86_64-pc-windows-gnullvm','exeSha256':hashlib.sha256(executable.read_bytes()).hexdigest(),'exeBytes':executable.stat().st_size,'purpose':'Local shared UI/Rust/SQLite maintenance testing; no BLE or cloud sync','signed':False}
(output/'build-receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(f'App executable: {output / "aiNavLog.exe"}')
if not args.app_only:
    installer_command=[sys.executable,str(app/'scripts/build-installer.py'),'--makensis',args.makensis]
    if args.nsis_root:installer_command.extend(['--nsis-root',str(args.nsis_root)])
    subprocess.run(installer_command,cwd=root,check=True)
