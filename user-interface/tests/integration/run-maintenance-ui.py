"""Real SQLite + Rust command service + shared maintenance UI. No BLE required."""
from functools import partial
from http.server import HTTPServer, SimpleHTTPRequestHandler
from pathlib import Path
import os, subprocess, sys, tempfile, threading, json, urllib.request, time
root=Path(__file__).resolve().parents[3]
export=Path(sys.argv[1] if len(sys.argv)>1 else '/tmp/ainavlog-maintenance-web')
class Handler(SimpleHTTPRequestHandler):
    def do_GET(self):
        if self.path.split('?')[0] in ['/', '/elec','/mech','/mech/maintenance','/elec/maintenance','/prefs']: self.path='/index.html'
        super().do_GET()
    def log_message(self,*_): pass
with tempfile.TemporaryDirectory(prefix='ainavlog-maintenance-') as work:
    web=HTTPServer(('127.0.0.1',18081),partial(Handler,directory=str(export)))
    threading.Thread(target=web.serve_forever,daemon=True).start()
    database=Path(work)/'records.sqlite3'
    service=subprocess.Popen([str(root/'native-integration/target/debug/ainavlog-device-interface'),'serve-ui',str(database),'windows-bmv-simulator','18787','http://127.0.0.1:18081','--simulated'])
    try:
        legacy={'id':'legacy-before-separation','equipmentId':'alternator','date':'2026-10-29','kind':'Inspection','description':'Earlier shared inspection','provider':'Owner','hours':None,'parts':'','costMinor':None,'currency':'','reference':'','notes':'','nextDate':None,'nextHours':None}
        request=urllib.request.Request('http://127.0.0.1:18787/maintenance',data=json.dumps({'op':'save','entry':legacy}).encode(),headers={'Origin':'http://127.0.0.1:18081','Content-Type':'application/json'})
        for attempt in range(30):
            try:
                assert json.load(urllib.request.urlopen(request))['ok'];break
            except OSError:time.sleep(.1)
        subprocess.run(['node',str(Path(__file__).with_name('maintenance-ui.cjs'))],check=True,env={**os.environ,'MAINTENANCE_TEST_DB':str(database)})
        import sqlite3
        with sqlite3.connect(database) as db:
            assert db.execute('select count(*) from maintenance_entries').fetchone()[0]==3
            assert db.execute('pragma user_version').fetchone()[0]==2
    finally:
        service.terminate();service.wait(timeout=5);web.shutdown();web.server_close()
