"""Test the packaged host code and embedded UI, without Expo/Metro or phone modules."""
from pathlib import Path
import json,os,subprocess,tempfile,time,urllib.request
root=Path(__file__).resolve().parents[2]
with tempfile.TemporaryDirectory(prefix='ainavlog-desktop-test-') as work:
    for phase in ['record','reopen']:
        app=subprocess.Popen([str(root/'desktop-app/target/debug/aiNavLog'),'--serve-only','--port','49380','--data-dir',work])
        try:
            for i in range(50):
                try:
                    urllib.request.urlopen('http://127.0.0.1:49380/health',timeout=1);break
                except OSError:time.sleep(.1)
            subprocess.run(['node',str(Path(__file__).with_name('ui.cjs')),phase],env=os.environ,check=True)
        finally:app.terminate();app.wait(timeout=5)
