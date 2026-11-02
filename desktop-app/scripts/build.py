"""Build the Windows maintenance-test executable with an existing LLVM-MinGW toolchain.
Example: python3 desktop-app/scripts/build.py --llvm-mingw /path/to/llvm-mingw
Downloads/installs nothing. Requires Rust Windows gnullvm target and UI dependencies.
"""
from pathlib import Path
import argparse, hashlib, json, os, shutil, subprocess, zipfile
app=Path(__file__).resolve().parents[1]
root=app.parent
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--llvm-mingw',type=Path,required=True)
args=parser.parse_args()
tools=args.llvm_mingw.resolve()
assert (tools/'bin/x86_64-w64-mingw32-clang').is_file(), 'Expected LLVM-MinGW bin directory'
env={**os.environ,'EXPO_NO_DOTENV':'1','EXPO_PUBLIC_NATIVE_BMV':'0','EXPO_PUBLIC_DEMO_MODE':'0','EXPO_PUBLIC_DEVICE_INTERFACE_URL':''}
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
receipt={'version':'0.1.2','target':'x86_64-pc-windows-gnullvm','exeSha256':hashlib.sha256(executable.read_bytes()).hexdigest(),'exeBytes':executable.stat().st_size,'purpose':'Local shared UI/Rust/SQLite maintenance testing; no BLE or cloud sync','signed':False}
(output/'build-receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
archive=app/'builds/aiNavLog-Windows-x64.zip'
with zipfile.ZipFile(archive,'w',zipfile.ZIP_DEFLATED) as z:
    for name in ['aiNavLog.exe','NativeNotices.txt','README.txt','build-receipt.json']:z.write(output/name,'aiNavLog/'+name)
print(f'Executable: {output / "aiNavLog.exe"}\nPackage: {archive}')
