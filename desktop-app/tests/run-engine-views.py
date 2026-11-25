"""Exercise the embedded desktop UI and real SQLite via its session-token API."""
from pathlib import Path
import os, subprocess, sys, tempfile, time, urllib.request
root=Path(__file__).resolve().parents[2]
with tempfile.TemporaryDirectory(prefix='ainavlog-engine-desktop-') as work:
    phases=['initial','reopen'] if '--reopen' in sys.argv[2:] else ['initial']
    for phase in phases:
        app=subprocess.Popen([str(root/'desktop-app/target/debug/aiNavLog'),'--serve-only','--port','49380','--data-dir',work])
        try:
            for i in range(50):
                try:
                    urllib.request.urlopen('http://127.0.0.1:49380/health',timeout=1)
                    break
                except OSError:
                    time.sleep(.1)
            subprocess.run(['node',str(root/(sys.argv[1] if len(sys.argv)>1 else 'user-interface/tests/integration/engine-diagram-ui.cjs'))],check=True,
                env={**os.environ,'ENGINE_TEST_ORIGIN':'http://127.0.0.1:49380','ENGINE_TEST_API':'http://127.0.0.1:49380','POC_VERIFY_REOPEN':'1' if phase=='reopen' else '0'})
        finally:
            app.terminate()
            app.wait(timeout=5)
