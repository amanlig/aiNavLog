#!/usr/bin/env python3
"""Render the inspection UML overview as SVG/PNG using system librsvg and Cairo.

Run from any directory with Python 3; no Python packages or network required.
The companion Mermaid source is generated from the same classes and relations.
"""
import ctypes as C
from html import escape
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'docs/images/device-interface-implementation-view'
# Same six logical boundaries as SAD section 5.4.3. Each entry is a real
# class/struct unless its stereotype explicitly identifies a function or ABI.
GROUPS = [
 ('1. Native integration', 'native-integration/ + app platform adapters', [
  ('AiNavLogModule / JNI', 'Swift / Kotlin / C++', 'App lifecycle, private paths, start / stop / snapshot / history'),
  ('Registry / Session', 'Rust structs', 'Opaque handles; Session owns monitor with injected capture'),
  ('Scanner / ScanStats', 'Rust application worker / diagnostics', 'Cancellation, retry and simulator selection policy'),
  ('Latest / history C ABI', 'Rust query functions', 'Read Local data store directly; no capture or receiver needed'),
 ], 'radio'),
 ('2. Device Interface', 'device-interface/ · no runtime SQLite dependency', [
  ('BluetoothAdapter / AdvertisementScan', 'Rust structs', 'btleplug platform access; event stream and scan cleanup'),
  ('scan_victron()', 'Rust async receiver', 'Own BLE events, startup, silence timeout and radio errors'),
  ('VictronMonitor<S: ObservationSink>', 'Rust struct', 'Decode, deduplicate, freshness / key state; emit to sink'),
  ('Decoder / ObservationNormalizer', 'Rust structs', 'Vendor protocol → common observations and metadata'),
  ('NMEA codecs / interface stubs', 'Rust structs / traits', 'Codecs exist; generic gateway integration remains unfinished'),
 ], 'radio'),
 ('3. Data capture', 'data-capture/ · validated writes only', [
  ('CaptureService', 'Rust struct · implements ObservationSink', 'Owns repository; validates before coordinating commit'),
  ('capture()', 'ObservationSink method', 'Source/time, fields, unique quantities, finite values, quality'),
  ('record_event()', 'ObservationSink method', 'Validate and save receiver state / diagnostic events'),
 ], 'done'),
 ('4. Local data store', 'local-storage/ · SQLite / bundled rusqlite', [
  ('ObservationStore', 'Rust struct', 'Own connection, schema/version, WAL, indexes, transactions'),
  ('Latest / history / recent queries', 'Associated Rust functions', 'Read-only; source/range selection; no Bluetooth required'),
  ('StoredObservation / BmvHistoryPoint', 'Rust data structs', 'Nullable measurements; receipt time; gap segment IDs'),
  ('samples / observations / monitor_events', 'Existing SQLite tables', 'Schema 1 and existing database files preserved'),
 ], 'done'),
 ('5. Shared data contracts', 'data-model/ · no platform / vendor / SQL dependencies', [
  ('SampleMetadata', 'Rust struct', 'source · receipt · product · counter · record type'),
  ('Observation / DecodedReading', 'Rust structs', 'quantity · unit · optional value/time · quality'),
  ('ObservationSink', 'Rust trait · injected write interface', 'capture() / record_event(); no concrete storage dependency'),
  ('AppError / AppResult / QualityStatus', 'Shared application contracts', 'Device API retains compatible DeviceError/Result aliases'),
 ], 'done'),
 ('6. Desktop application composition', 'native-integration/ · desktop feature', [
  ('run_victron_monitor / MonitorConfig', 'Rust async supervisor / configuration', 'Assemble transport → capture → storage; retry and key reload'),
  ('serve-ui', 'Rust read-only HTTP module', 'Latest and history query APIs → Local data store'),
  ('CLI / packet replay', 'Executable / example', 'Application entry points; executable name unchanged'),
 ], 'done'),
]
COLORS = {'done': ('#ecfdf5', '#15803d'), 'radio': ('#fffbeb', '#b45309')}
W, H = 1960, 1730
BW, BH = 600, 590
svg = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}">',
 '<defs><marker id="arrow" markerWidth="9" markerHeight="9" refX="8" refY="4" orient="auto"><path d="M0,0 L8,4 L0,8" fill="none" stroke="#475569"/></marker></defs>',
 '<rect width="100%" height="100%" fill="#f8fafc"/>']
def text(x, y, value, size=17, weight='normal', fill='#0f172a'):
    svg.append(f'<text x="{x}" y="{y}" font-family="DejaVu Sans, sans-serif" font-size="{size}" font-weight="{weight}" fill="{fill}">{escape(value)}</text>')
text(40, 45, 'aiNavLog · Device Interface implementation view', 30, 'bold')
text(40, 80, '2026-10-28 · Responsibilities match SAD §5.2 · Classes and functions shown inside each logical block', 20)
text(40, 115, 'Green: implemented scope     Amber: radio/platform acceptance incomplete     Arrows: selected dependencies / data paths', 17)
for index, (title, subtitle, classes, status) in enumerate(GROUPS):
    x, y = 40 + (index % 3)*640, 160 + (index//3)*660
    fill, stroke = COLORS[status]
    svg.append(f'<rect x="{x}" y="{y}" width="{BW}" height="{BH}" rx="10" fill="{fill}" stroke="{stroke}" stroke-width="2"/>')
    text(x+18, y+34, title, 23, 'bold')
    text(x+18, y+64, subtitle, 17, fill=stroke)
    for row, (name, kind, detail) in enumerate(classes):
        cy = y + 88 + row*96
        svg.append(f'<rect x="{x+15}" y="{cy}" width="570" height="84" rx="5" fill="#ffffff" stroke="{stroke}"/>')
        text(x+27, cy+21, kind, 13, fill=stroke)
        text(x+27, cy+45, name, 19, 'bold')
        text(x+27, cy+69, detail, 14)
# Communication and write flow on the top row; direct query arrow goes down.
def arrow(points, label, x, y):
    svg.append(f'<path d="M{points}" fill="none" stroke="#475569" stroke-width="2" marker-end="url(#arrow)"/>')
    text(x, y, label, 14, fill='#475569')
arrow('640,420 L680,420','',0,0)
arrow('1280,420 L1320,420','',0,0)
arrow('340,750 L340,820','direct query APIs',355,810)
arrow('1620,750 L1620,785 L20,785 L20,1120 L40,1120','capture commits to storage',690,778)
text(40, 1452, 'Writes: Device Interface → injected ObservationSink → Data capture → Local data store. Reads: UI → native query APIs → Local data store.', 17)
text(40, 1485, 'Device Interface never opens or queries SQLite. Capture and storage have no dependency on Bluetooth or vendor-specific Rust types.', 18, 'bold')
svg.append('<rect x="40" y="1510" width="1880" height="96" rx="8" fill="#eff6ff" stroke="#64748b"/>')
text(58, 1539, 'Logical ownership and physical packages are explicit; all mobile components are bundled within the same app.', 19, 'bold')
text(58, 1566, 'Native integration assembles the pipeline. Shared contracts define the sink. Storage queries remain available after disconnect.', 17)
text(58, 1591, 'The refactor preserves database filenames, schema version, rows, receipt timestamps and C ABI names; no user data is deleted or migrated.', 17)
text(40, 1640, 'Future work: maintenance CRUD, broader capture workflows, retention/export/protection and real-device qualification. See ADR-0001 / ADR-0002.', 17)
text(40, 1675, 'iPhone 1.0.0 (7): ready in TestFlight. Physical-phone history/reception regression remains pending. Android replacement is deferred.', 16)
text(40, 1705, 'Classes, functions and contracts are grouped by responsibility. Source references: docs/device-interface-implementation-view.md', 17)
svg.append('</svg>')
OUT.with_suffix('.svg').write_text('\n'.join(svg)+'\n')
# Generate a grouped Mermaid class view from the same definitions.
mermaid = ['classDiagram', '    direction TB']
ids = {}
for index, (title, subtitle, classes, status) in enumerate(GROUPS, 1):
    mermaid.append(f'    namespace Block{index} {{')
    for name, kind, detail in classes:
        symbol = ''.join(c for c in name if c.isalnum())
        ids[name] = symbol
        mermaid.append(f'        class {symbol}')
    mermaid.append('    }')
    for name, kind, detail in classes:
        symbol = ids[name]
        mermaid.append(f'    note for {symbol} "{title}: {kind}; {detail}"')
        fill, stroke = COLORS[status]
        mermaid.append(f'    style {symbol} fill:{fill},stroke:{stroke},color:#0f172a')
for a,b,label in [
 ('AiNavLogModule / JNI','Registry / Session','C ABI'),
 ('Scanner / ScanStats','scan_victron()','radio reception'),
 ('scan_victron()','BluetoothAdapter / AdvertisementScan','owns event handling'),
 ('Registry / Session','VictronMonitor<S: ObservationSink>','owns'),
 ('VictronMonitor<S: ObservationSink>','Decoder / ObservationNormalizer','decodes'),
 ('VictronMonitor<S: ObservationSink>','ObservationSink','emits normalized observations'),
 ('ObservationSink','CaptureService','implemented by'),
 ('CaptureService','ObservationStore','validated atomic save'),
 ('Latest / history C ABI','Latest / history / recent queries','direct read'),
 ('ObservationStore','samples / observations / monitor_events','schema / transactions'),
 ('serve-ui','Latest / history / recent queries','direct read'),
 ('run_victron_monitor / MonitorConfig','VictronMonitor<S: ObservationSink>','supervises'),
 ('run_victron_monitor / MonitorConfig','CaptureService','injects'),
]:
    mermaid.append(f'    {ids[a]} --> {ids[b]} : {label}')
OUT.with_suffix('.mmd').write_text('\n'.join(mermaid)+'\n')

# Render SVG with the system libraries. Keep the SVG usable without librsvg.
rsvg = C.CDLL('librsvg-2.so.2'); cairo = C.CDLL('libcairo.so.2')
rsvg.rsvg_handle_new_from_file.argtypes = [C.c_char_p, C.c_void_p]
rsvg.rsvg_handle_new_from_file.restype = C.c_void_p
rsvg.rsvg_handle_render_cairo.argtypes = [C.c_void_p, C.c_void_p]
rsvg.rsvg_handle_render_cairo.restype = C.c_int
cairo.cairo_image_surface_create.argtypes = [C.c_int, C.c_int, C.c_int]
cairo.cairo_image_surface_create.restype = C.c_void_p
cairo.cairo_create.argtypes = [C.c_void_p]; cairo.cairo_create.restype = C.c_void_p
cairo.cairo_surface_write_to_png.argtypes = [C.c_void_p, C.c_char_p]
cairo.cairo_destroy.argtypes = [C.c_void_p]
cairo.cairo_surface_destroy.argtypes = [C.c_void_p]
handle = rsvg.rsvg_handle_new_from_file(str(OUT.with_suffix('.svg')).encode(), None)
assert handle, 'Could not load generated SVG'
surface = cairo.cairo_image_surface_create(0, W, H)
context = cairo.cairo_create(surface)
assert rsvg.rsvg_handle_render_cairo(handle, context), 'SVG rendering failed'
assert cairo.cairo_surface_write_to_png(surface, str(OUT.with_suffix('.png')).encode()) == 0
cairo.cairo_destroy(context); cairo.cairo_surface_destroy(surface)
print(OUT.with_suffix('.png'))
