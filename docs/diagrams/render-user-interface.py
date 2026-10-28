#!/usr/bin/env python3
"""Render the inspection UML overview as SVG/PNG using system librsvg and Cairo.

Run from any directory with Python 3; no Python packages or network required.
The companion Mermaid source is generated from the same classes and relations.
"""
import ctypes as C
from html import escape
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'docs/images/user-interface-implementation-view'
# Same six logical boundaries as SAD section 5.4.1. Each entry is a real
# class/struct unless its stereotype explicitly identifies a function or ABI.
GROUPS = [
 ('1. Application shell', 'Routes · provider composition · navigation', [
  ('Layout', 'React component · src/app/_layout.tsx', 'SafeAreaProvider → PreferencesProvider → ElectricalProvider'),
  ('Stack / Link', 'External Expo Router components', 'Home / · Electrical /elec · Engine /mech · Routes /rout'),
  ('AppHeader', 'React component', 'Boat-name dialog · appearance settings · navigation'),
 ], 'done'),
 ('2. Screens and presentation', 'Render shared state · submit user intent', [
  ('HomeScreen', 'React component · src/app/index.tsx', 'Electrical summary; links to electrical / engine / routes'),
  ('ElectricalScreen', 'React component · src/app/elec/index.tsx', 'House / starter cards; connect / disconnect; diagnostics'),
  ('Metric / BatteryGauge / Icon', 'Local React components', 'Format battery values and charge indicators'),
  ('FeatureIcon', 'Shared React component', 'Electrical / engine / route illustrations'),
  ('BatteryHistory / HistoryChart', 'React components · history screen', 'House / Starter; pinch, pan, +/-; visible-range queries'),
 ], 'done'),
 ('3. Shared electrical state', 'One provider for Home and Electrical', [
  ('ElectricalProvider / useElectrical', 'React context provider / hook', 'Expose sample · clock · errors · connection · counters'),
  ('useElectricalState', 'Internal React hook', 'Poll every second; serialize receiver operations'),
  ('AppState subscription', 'Hook lifecycle integration', 'Pause on background; resume requested native session'),
  ('Data-mode selection', 'Environment / platform configuration', 'Native BMV → configured HTTP endpoint → local demo'),
 ], 'done'),
 ('4. Data clients and native boundary', 'Receiver commands and direct storage query APIs', [
  ('native-client.ts', 'TypeScript module', 'startReceiver / stopReceiver / readReceiver; permissions'),
  ('AiNavLogModule / JNI', 'Swift / Kotlin / Android C++ integration', 'start / stop / snapshot / history → native-integration'),
  ('HTTP fetch in useElectricalState', 'Hook implementation · desktop development', 'GET /electrical; 3s abort; no demo fallback on failure'),
  ('parseDeviceSample', 'TypeScript function · device-client.ts', 'Validate source / timestamp / nullable finite readings'),
  ('history-client / parseHistory', 'TypeScript modules', 'Read native or HTTP history; validate bounded chart points'),
 ], 'radio'),
 ('5. Measurement rules and contract', 'Pure TypeScript · no radio or database access', [
  ('ElectricalSample', 'TypeScript type', 'source · receivedAt · house V/A/SOC · starter V'),
  ('isFresh / STALE_MS', 'Functions / constant · model.ts', 'Hide values older than 5s; reject future timestamps'),
  ('reading / simulate', 'Functions · model.ts', 'Format nullable numbers; explicitly labelled demo data'),
  ('starterAlert / parseThreshold', 'Functions · model.ts', 'Helpers exist; configurable alert UI is not connected'),
  ('history.ts / history-viewport.ts', 'Types / pure functions', 'Calendar bounds; pinch/step zoom, pan, extrema, selection'),
 ], 'done'),
 ('6. Preferences and unfinished views', 'Local settings persist · feature placeholders explicit', [
  ('PreferencesProvider / usePreferences', 'React provider / hook', 'Boat name · white/black palette · load/save errors'),
  ('AsyncStorage', 'External local persistence library', 'ainavlog.preferences.v1; settings only'),
  ('ElectricalRecords', 'React component', 'Maintenance empty state; recording not implemented'),
  ('EngineScreen / RoutesScreen', 'React route components', 'ComingSoon component; no engine or route capture'),
 ], 'partial'),
]
COLORS = {'done': ('#ecfdf5', '#15803d'), 'radio': ('#fffbeb', '#b45309'), 'partial': ('#fff7ed', '#c2410c')}
W, H = 1960, 1730
BW, BH = 600, 590
svg = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}">',
 '<defs><marker id="arrow" markerWidth="9" markerHeight="9" refX="8" refY="4" orient="auto"><path d="M0,0 L8,4 L0,8" fill="none" stroke="#475569"/></marker></defs>',
 '<rect width="100%" height="100%" fill="#f8fafc"/>']
def text(x, y, value, size=17, weight='normal', fill='#0f172a'):
    svg.append(f'<text x="{x}" y="{y}" font-family="DejaVu Sans, sans-serif" font-size="{size}" font-weight="{weight}" fill="{fill}">{escape(value)}</text>')
text(40, 45, 'aiNavLog · User Interface implementation view', 30, 'bold')
text(40, 80, '2026-10-28 · Boundaries match SAD §5.4.1 · Components, hooks, modules and types inside each logical block', 20)
text(40, 115, 'Green: implemented scope     Amber: platform acceptance incomplete; orange: mixed implemented / placeholder scope     Arrows: selected dependencies / data paths', 17)
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
# Dependencies between logical groups; details and cross-links are in Mermaid.
def arrow(points):
    svg.append(f'<path d="M{points}" fill="none" stroke="#475569" stroke-width="2" marker-end="url(#arrow)"/>')
arrow('640,420 L680,420')
arrow('1280,420 L1320,420')
arrow('1620,750 L1620,785 L660,785 L660,1120 L640,1120')
arrow('980,750 L980,820')
text(40, 1452, 'Flow: Home/Electrical share live state. History owns range queries, reads saved SQLite via native/HTTP and remains usable disconnected.', 18)
text(40, 1485, 'Physical boundary: UI/platform adapters: user-interface/; BLE/decoding: device-interface/; capture: data-capture/; SQLite: local-storage/.', 17, 'bold')
svg.append('<rect x="40" y="1510" width="1880" height="96" rx="8" fill="#eff6ff" stroke="#64748b"/>')
text(58, 1539, 'Native integration routes reads directly to Local data store. Receiver writes pass through Data capture; history needs no Bluetooth.', 18, 'bold')
text(58, 1566, 'Rust composition lives in native-integration/. Latest/history read storage directly; Connect assembles Device Interface → capture → storage.', 17)
text(58, 1591, 'Both external paths preserve packet receipt time. Starter amperage and charge are unavailable; stale readings are hidden on both screens.', 17)
text(40, 1640, 'Not implemented: account/sign-in screens; media/manual/voice entry; route capture/sharing; engine integration; maintenance recording.', 16, fill='#be123c')
text(40, 1675, 'Zoom: local source after TestFlight build 7; new delivery and phone gesture acceptance pending. Android replacement remains deferred.', 16)
text(40, 1705, 'Confirmed 2027 beta structured store: SQLite via bundled Rust rusqlite. UI preferences remain in AsyncStorage; broader storage APIs remain unfinished.', 16)
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
 ('Layout','HomeScreen','navigation'),
 ('Layout','ElectricalProvider / useElectrical','mounts'),
 ('Layout','PreferencesProvider / usePreferences','mounts'),
 ('HomeScreen','ElectricalProvider / useElectrical','consumes'),
 ('ElectricalScreen','ElectricalProvider / useElectrical','consumes / commands'),
 ('ElectricalProvider / useElectrical','useElectricalState','owns state'),
 ('useElectricalState','native-client.ts','native mode'),
 ('native-client.ts','AiNavLogModule / JNI','calls'),
 ('BatteryHistory / HistoryChart','history-client / parseHistory','range query'),
 ('BatteryHistory / HistoryChart','history.ts / history-viewport.ts','zoom / gaps / selection'),
 ('native-client.ts','parseDeviceSample','validates snapshot'),
 ('useElectricalState','HTTP fetch in useElectricalState','HTTP mode'),
 ('HTTP fetch in useElectricalState','parseDeviceSample','validates response'),
 ('HomeScreen','isFresh / STALE_MS','hides stale'),
 ('ElectricalScreen','isFresh / STALE_MS','hides stale'),
 ('PreferencesProvider / usePreferences','AsyncStorage','load / save'),
 ('AppHeader','PreferencesProvider / usePreferences','edit settings'),
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
