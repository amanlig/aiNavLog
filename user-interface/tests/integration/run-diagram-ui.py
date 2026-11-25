"""Real Rust/SQLite diagram persistence; AI response is explicitly stubbed in the browser."""
from functools import partial
from http.server import ThreadingHTTPServer, SimpleHTTPRequestHandler
from pathlib import Path
import os, subprocess, sys, tempfile, threading, time, urllib.request, json
root=Path(__file__).resolve().parents[3]
export=Path(sys.argv[1] if len(sys.argv)>1 else '/tmp/ainavlog-diagram-web')
class Handler(SimpleHTTPRequestHandler):
    def do_GET(self):
        if self.path.split('?')[0] in ['/', '/elec/maintenance', '/elec/service-history', '/mech/maintenance', '/mech/service-history']: self.path='/index.html'
        super().do_GET()
    def log_message(self,*_): pass
with tempfile.TemporaryDirectory(prefix='ainavlog-diagram-') as work:
    web=ThreadingHTTPServer(('127.0.0.1',18081),partial(Handler,directory=str(export)))
    threading.Thread(target=web.serve_forever,daemon=True).start()
    database=Path(work)/'records.sqlite3'
    service=subprocess.Popen([str(root/'native-integration/target/debug/ainavlog-device-interface'),'serve-ui',str(database),'windows-bmv-simulator','18787','http://127.0.0.1:18081','--simulated'])
    try:
        request=urllib.request.Request('http://127.0.0.1:18787/maintenance',data=b'{"op":"list"}',headers={'Origin':'http://127.0.0.1:18081','Content-Type':'application/json'})
        for attempt in range(50):
            try: json.load(urllib.request.urlopen(request));break
            except OSError:time.sleep(.1)
        subprocess.run(['node',str(Path(__file__).with_name(sys.argv[2] if len(sys.argv)>2 else 'diagram-ui.cjs'))],check=True,env={**os.environ,'DIAGRAM_TEST_DB':str(database)})
    finally:
        service.terminate();service.wait(timeout=5);web.shutdown();web.server_close()
