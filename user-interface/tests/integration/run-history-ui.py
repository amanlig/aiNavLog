"""Exercise real SQLite -> Rust HTTP -> mobile-width history UI; no radio needed.
Requires an export with EXPO_PUBLIC_DEVICE_INTERFACE_URL=http://127.0.0.1:18787/electrical.
Run with NODE_PATH pointing to an external Playwright installation.
"""
from functools import partial
from http.server import HTTPServer, SimpleHTTPRequestHandler
from pathlib import Path
from datetime import datetime
from zoneinfo import ZoneInfo
import ctypes
import math
import os
import sqlite3
import subprocess
import sys
import tempfile
import threading

root = Path(__file__).resolve().parents[3]
export = Path(sys.argv[1] if len(sys.argv)>1 else '/tmp/ainavlog-history-web')
class Handler(SimpleHTTPRequestHandler):
    def do_GET(self):
        if self.path.split('?')[0] in ['/', '/elec', '/elec/history']: self.path='/index.html'
        super().do_GET()
    def log_message(self, *_): pass

with tempfile.TemporaryDirectory(prefix='ainavlog-history-test-') as work:
    database=Path(work)/'history.sqlite3'
    lib=ctypes.CDLL(str(root/'native-integration/target/debug/libainavlog_native_bridge.so'))
    lib.ainavlog_open.argtypes=[ctypes.c_char_p,ctypes.c_char_p,ctypes.c_char_p,ctypes.POINTER(ctypes.c_uint64)]
    lib.ainavlog_close.argtypes=[ctypes.c_uint64]
    handle=ctypes.c_uint64()
    assert lib.ainavlog_open(str(database).encode(),b'windows-bmv-simulator',b'000102030405060708090a0b0c0d0e0f',ctypes.byref(handle))==0
    assert lib.ainavlog_close(handle)==0
    start=int(datetime(2026,10,28,tzinfo=ZoneInfo('America/Vancouver')).timestamp()*1000)
    with sqlite3.connect(database) as db:
        # Five-second packet cadence, with a one-hour recording gap.
        for i in range(10800):
            if 8*720 <= i < 9*720: continue
            time=start+i*5000
            sample=db.execute("INSERT INTO samples(source_id,received_at_ms,product_id,data_counter,record_type,manufacturer_data) VALUES ('windows-bmv-simulator',?,4660,?,2,x'00')",(time,i%65536)).lastrowid
            charge=80+5*math.sin(i/1800)
            voltage=12.8+.2*math.sin(i/1800)
            db.executemany('INSERT INTO observations(sample_id,quantity,unit,value,quality) VALUES (?,?,?,?,?)',[(sample,'battery_voltage','V',voltage,'Good'),(sample,'battery_current','A',-5.25 if i<7000 else 12,'Good'),(sample,'state_of_charge','%',charge,'Good'),(sample,'auxiliary_voltage','V',12.8 if i<7000 else 14.2,'Good')])
    web=HTTPServer(('127.0.0.1',18081),partial(Handler,directory=str(export)))
    threading.Thread(target=web.serve_forever,daemon=True).start()
    service=subprocess.Popen([str(root/'native-integration/target/debug/ainavlog-device-interface'),'serve-ui',str(database),'windows-bmv-simulator','18787','http://127.0.0.1:18081','--simulated'])
    try:
        subprocess.run(['node',str(Path(__file__).with_name('history-ui.cjs'))],check=True,env=os.environ)
    finally:
        service.terminate();service.wait(timeout=5);web.shutdown();web.server_close()
