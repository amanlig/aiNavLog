"""Render the proposed battery history screens. Illustrative data only."""
import ctypes as C
from html import escape
from pathlib import Path
ROOT=Path(__file__).resolve().parent
OUT=ROOT/'battery-history'
W,H=1240,1450
INK='#092443'; MUTED='#536780'; TEAL='#16887f'; LINE='#dfe9f1'
s=[f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}"><rect width="100%" height="100%" fill="#edf2f6"/>']
def rect(x,y,w,h,fill='white',stroke='none',r=0):s.append(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{r}" fill="{fill}" stroke="{stroke}"/>')
def text(x,y,t,size=16,color=INK,weight='normal'):
 s.append(f'<text x="{x}" y="{y}" font-family="DejaVu Sans,sans-serif" font-size="{size}" fill="{color}" font-weight="{weight}">{escape(t)}</text>')
def path(d,color=TEAL,width=3,dash=''):
 s.append(f'<path d="{d}" fill="none" stroke="{color}" stroke-width="{width}" stroke-linecap="round" stroke-linejoin="round" stroke-dasharray="{dash}"/>')
text(55,55,'Battery history',34,weight='bold')
text(55,88,'Proposed mobile experience · House and Starter · illustrative data',18,MUTED)
for x,house in [(55,True),(660,False)]:
 rect(x-5,124,535,1255,'#cfdbe4',r=34)
 rect(x,118,525,1255,'white',r=30)
 # Device status and application brand
 text(x+25,152,'9:41',14,weight='bold');text(x+430,152,'▰ 100%',12)
 rect(x,173,525,66,'#153b5e')
 text(x+23,215,'✦ aiNavLog',23,'white','bold');text(x+360,213,'My boat  ▾',16,'white')
 text(x+24,274,'‹  Electrical',16,TEAL,'bold')
 text(x+24,318,'Historical Usage',30,weight='bold')
 text(x+24,347,'Review the readings recorded on this device.',15,MUTED)
 # Segmented battery selector
 rect(x+24,370,477,52,'#eef3f7',r=13)
 rect(x+(28 if house else 266),374,231,44,'white',LINE,10)
 text(x+99,403,'House',17,TEAL if house else MUTED,'bold' if house else 'normal')
 text(x+337,403,'Starter',17,MUTED if house else TEAL,'normal' if house else 'bold')
 text(x+24,456,'‹',24,TEAL);text(x+107,454,'Wed, Oct 28, 2026',19,weight='bold');text(x+480,456,'›',24,TEAL)
 # Time range
 for i,label in enumerate(['24 hours','7 days','30 days']):
  xx=x+24+i*161
  rect(xx,475,153,40,TEAL if i==0 else 'white',TEAL if i==0 else LINE,10)
  text(xx+36,501,label,14,'white' if i==0 else MUTED,'bold' if i==0 else 'normal')
 rect(x+24,535,477,34,'#e4f4f1',r=8)
 text(x+37,557,'SIMULATED DATA  ·  Windows BMV simulator',12,'#146f67','bold')
 # Chart card
 rect(x+24,587,477,376,'white',LINE,16)
 text(x+43,621,'Charge level' if house else 'Voltage',21,weight='bold')
 text(x+43,648,'Recorded samples · tap chart to inspect',13,MUTED)
 if house:
  text(x+43,696,'77.2',38,weight='bold');text(x+145,695,'%',19,MUTED)
 else:
  text(x+43,696,'12.8',38,weight='bold');text(x+145,695,'V',19,MUTED)
 text(x+43,720,'Selected reading · 12:00',13,MUTED)
 if house:
  # Metric choices are available only for the house battery.
  for i,label in enumerate(['Charge','Voltage','Current']):
   xx=x+200+i*84
   rect(xx,674,78,29,'#e4f4f1' if i==0 else '#f5f8fb',r=7)
   text(xx+10,693,label,11,TEAL if i==0 else MUTED,'bold' if i==0 else 'normal')
 else:
  text(x+268,688,'Starter voltage only',13,MUTED)
 plotx=x+83;ploty=751;pw=386;ph=135
 for j,lab in enumerate(['90%','80%','70%'] if house else ['14.5','13.5','12.5']):
  yy=ploty+j*ph/2
  path(f'M{plotx},{yy} H{plotx+pw}',LINE,1)
  text(x+40,yy+5,lab,12,MUTED)
 # No line bridges the shaded recording gap.
 gapx=plotx+pw*.29;gapw=pw*.11
 rect(gapx,ploty,gapw,ph,'#f1f4f7')
 values=([(0,.27),(.08,.32),(.16,.39),(.25,.46)],[(.42,.54),(.5,.64),(.6,.7),(.68,.48),(.74,.24),(.84,.18),(.94,.28),(1,.36)]) if house else ([(0,.77),(.08,.79),(.17,.81),(.25,.78)],[(.42,1),(.5,.85),(.6,.79),(.64,.68),(.68,.17),(.76,.15),(.83,.3),(.87,.68),(.94,.75),(1,.77)])
 for segment in values:
  path(' '.join(('M' if i==0 else 'L')+f'{plotx+a*pw},{ploty+b*ph}' for i,(a,b) in enumerate(segment)),TEAL,3)
 # selected point
 sx=plotx+pw*.5;sy=ploty+ph*(.64 if house else .85)
 path(f'M{sx},{ploty} V{ploty+ph}', '#a1b8c4',1,'3 4')
 s.append(f'<circle cx="{sx}" cy="{sy}" r="5" fill="{TEAL}" stroke="white" stroke-width="2"/>')
 for i,lab in enumerate(['00:00','06:00','12:00','18:00','24:00']):text(plotx+i*pw/4-17,912,lab,11,MUTED)
 rect(x+43,932,11,11,'#f1f4f7',LINE,2);text(x+62,942,'Gap = no readings recorded',12,MUTED)
 # Recorded range summary
 text(x+24,1000,'RECORDED RANGE',12,MUTED,'bold')
 for i,(label,value) in enumerate([('Lowest','76.0%' if house else '12.5 V'),('Highest','86.4%' if house else '14.2 V')]):
  xx=x+24+i*244;rect(xx,1016,233,93,'#f7fafc',LINE,12)
  text(xx+17,1044,label,14,MUTED);text(xx+17,1081,value,27,weight='bold')
 if house:
  text(x+24,1147,'At selected time',17,weight='bold')
  text(x+24,1181,'Voltage  12.8 V',16);text(x+269,1181,'Current  −5.3 A',16)
  text(x+24,1211,'− Discharging   /   + Charging',13,MUTED)
 else:
  rect(x+24,1133,477,91,'#f5f8fb',r=12)
  text(x+40,1163,'Why voltage only?',16,weight='bold')
  text(x+40,1188,'The auxiliary input measures starter voltage.',13,MUTED)
  text(x+40,1209,'Starter charge and current are unavailable.',13,MUTED)
 path(f'M{x+24},1246 H{x+501}',LINE,1)
 text(x+24,1274,'Recorded history · not a live battery reading',13,MUTED)
 text(x+24,1298,'Only periods when the receiver was active are shown.',12,MUTED)
 rect(x+196,1345,133,5,INK,r=3)
text(55,1417,'Design mockup only · history queries and chart interactions are not implemented yet.',17,MUTED)
s.append('</svg>');OUT.with_suffix('.svg').write_text('\n'.join(s))

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
