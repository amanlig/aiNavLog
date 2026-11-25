"""Package app files using the fixed NSIS Modern UI 2 template; do not restyle it.

Release contract: docs/windows-delivery.md. Routine builds change payload/version
and generated receipts, retaining the accepted installer appearance and behavior.
"""
from pathlib import Path
import argparse, hashlib, json, os, re, shutil, subprocess
app=Path(__file__).resolve().parents[1]
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--makensis',default=shutil.which('makensis'))
parser.add_argument('--nsis-root',type=Path)
args=parser.parse_args()
if not args.makensis:parser.error('NSIS makensis is required; see docs/windows-delivery.md. Do not substitute a ZIP or script installer.')
payload=app/'builds/windows-x64'
receipt=json.loads((payload/'build-receipt.json').read_text())
version=receipt['version']
assert re.fullmatch(r'\d+\.\d+\.\d+',version),'Expected a three-part app version'
assert hashlib.sha256((payload/'aiNavLog.exe').read_bytes()).hexdigest()==receipt['exeSha256'],'App checksum mismatch'
for name in ['NativeNotices.txt','README.txt']:assert (payload/name).is_file(),f'Missing {name}'
output=app/f'builds/aiNavLog-Setup-{version}.exe'
env=dict(os.environ)
if args.nsis_root:env['NSISDIR']=str(args.nsis_root.resolve())
subprocess.run([args.makensis,'-V3',f'-DAPP_VERSION={version}',f'-DPAYLOAD_DIR={payload}',f'-DOUTPUT_FILE={output}',str(app/'installer/ainavlog.nsi')],cwd=app/'installer',env=env,check=True)
installer_receipt={**receipt,'format':'NSIS setup EXE','installer':output.name,'installerSha256':hashlib.sha256(output.read_bytes()).hexdigest(),'installerBytes':output.stat().st_size,'automaticallyInstalled':False}
output.with_suffix('.json').write_text(json.dumps(installer_receipt,indent=2)+'\n')
print(output)
