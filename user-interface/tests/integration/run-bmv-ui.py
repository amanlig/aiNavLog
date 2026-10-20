"""Run the local BMV browser test against an exported web UI (Linux/WSL)."""
from functools import partial
from http.server import SimpleHTTPRequestHandler, HTTPServer
from pathlib import Path
import os
import subprocess
import sys
import tempfile
import threading

root = Path(__file__).resolve().parents[3]
export = Path(sys.argv[1] if len(sys.argv) > 1 else '/tmp/ainavlog-sim-ui').resolve()
if not (export / 'index.html').is_file():
    raise SystemExit('Export the UI first; pass its output directory as the first argument.')

class Handler(SimpleHTTPRequestHandler):
    def do_GET(self):
        if self.path.split('?')[0] in ['/', '/elec', '/elec/']:
            self.path = '/index.html'
        super().do_GET()

    def log_message(self, *_args):
        pass

with tempfile.TemporaryDirectory(prefix='ainavlog-bmv-ui-') as directory:
    database = str(Path(directory) / 'observations.sqlite3')
    web = HTTPServer(('127.0.0.1', 18081), partial(Handler, directory=str(export)))
    threading.Thread(target=web.serve_forever, daemon=True).start()
    service = subprocess.Popen([
        str(root / 'device-interface/target/debug/ainavlog-device-interface'),
        'serve-ui', database, 'bmv-simulator', '18787',
        'http://127.0.0.1:18081', '--simulated',
    ])
    try:
        env = dict(os.environ, BMV_TEST_DATABASE=database)
        result = subprocess.run(['node', str(Path(__file__).with_name('bmv-ui.cjs'))], env=env)
        raise SystemExit(result.returncode)
    finally:
        service.terminate()
        service.wait(timeout=5)
        web.shutdown()
        web.server_close()
