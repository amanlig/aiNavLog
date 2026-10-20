"""Render the October 16 planning baseline as a standalone SVG Gantt chart."""
from datetime import date,timedelta
from html import escape
from pathlib import Path

ROOT=Path(__file__).resolve().parent
START=date(2026,10,5)
END=date(2027,2,1)
LEFT=375
SCALE=9.5
WIDTH=LEFT+(END-START).days*SCALE+45
ROWS=[
 ('Desktop/core/UI baseline','2026-10-05','2026-10-17','done'),
 ('Android + iPhone first installs','2026-10-16',None,'done'),
 ('Electrical: native completion','2026-10-19','2026-11-02','feature'),
 ('Android + iOS CI/CD','2026-10-19','2026-10-24','parallel'),
 ('Distribution / update qualification','2026-10-19','2027-02-01','parallel'),
 ('Engine','2026-11-02','2026-11-16','feature'),
 ('Route','2026-11-16','2026-11-30','feature'),
 ('Authentication / authorization','2026-11-30','2026-12-14','feature'),
 ('Review / holiday contingency','2026-12-14','2027-01-04','buffer'),
 ('Route story','2027-01-04','2027-01-18','feature'),
 ('Integration / beta-readiness review','2027-01-18','2027-02-01','feature'),
]
COLORS={'done':'#258568','feature':'#285d99','parallel':'#7a58a1','buffer':'#b18a37'}

def x(day):return LEFT+(date.fromisoformat(day)-START).days*SCALE

def main():
    top=180;rowheight=63;height=top+len(ROWS)*rowheight+130
    s=[f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {WIDTH} {height}" role="img" aria-labelledby="title desc"><title id="title">aiNavLog implementation Gantt chart, October 2026 through January 2027</title><desc id="desc">Completed baseline and first installs precede native Electrical and parallel CI/CD. Engine, Route and Authentication follow. Holiday contingency precedes January Route Story and beta-readiness review.</desc><style>text{{font-family:Arial,sans-serif;fill:#193149}}.h{{font-size:26px;font-weight:bold}}.label{{font-size:16px}}.small{{font-size:13px;fill:#52697f}}</style><rect width="{WIDTH}" height="{height}" fill="white"/><text x="25" y="40" class="h">aiNavLog · revised implementation schedule</text><text x="25" y="70" class="label">Planning baseline: October 16, 2026 · America/Vancouver · dates are targets, not release guarantees</text>']
    for i,(key,label) in enumerate([('done','Completed evidence'),('feature','Planned feature'),('parallel','Parallel delivery'),('buffer','Holiday / review buffer')]):
        a=25+i*300;s.append(f'<rect x="{a}" y="94" width="16" height="16" rx="3" fill="{COLORS[key]}"/><text x="{a+24}" y="108" class="small">{label}</text>')
    # Weekly cells; the bar end dates below are exclusive.
    day=START
    while day<END:
        xx=LEFT+(day-START).days*SCALE
        s.append(f'<path d="M{xx} 150V{top+len(ROWS)*rowheight}" stroke="#e0e7ef"/><text transform="translate({xx+4},160) rotate(-35)" class="small">{day.strftime("%b %d")}</text>')
        day+=timedelta(days=7)
    for i,(label,start,end,kind) in enumerate(ROWS):
        y=top+i*rowheight
        if i%2==0:s.append(f'<rect x="15" y="{y}" width="{WIDTH-30}" height="{rowheight}" fill="#f5f8fb"/>')
        s.append(f'<text x="25" y="{y+33}" class="label">{escape(label)}</text>')
        color=COLORS[kind]
        if end is None:
            xx=x(start);s.append(f'<path d="M{xx} {y+20}l10 10-10 10-10-10Z" fill="{color}"/><text x="{xx+18}" y="{y+35}" class="small">Oct 16 · achieved</text>')
        else:
            w=x(end)-x(start);s.append(f'<rect x="{x(start)}" y="{y+17}" width="{w}" height="27" rx="5" fill="{color}"/>')
            last=date.fromisoformat(end)-timedelta(days=1)
            caption=f'{date.fromisoformat(start).strftime("%b %d")}–{last.strftime("%b %d")}'
            if w>100:s.append(f'<text x="{x(start)+8}" y="{y+36}" style="font:13px Arial;fill:white">{caption}</text>')
            else:s.append(f'<text x="{x(end)+8}" y="{y+35}" class="small">{caption}</text>')
    # Checkpoints sit on their relevant task bar; short symbols avoid crowded labels.
    for row,day in [(2,'2026-10-23'),(2,'2026-10-30'),(5,'2026-11-13'),(6,'2026-11-27'),(7,'2026-12-11'),(9,'2027-01-15'),(10,'2027-01-29')]:
        xx=x(day);yy=top+row*rowheight+30
        s.append(f'<path d="M{xx} {yy-7}l7 7-7 7-7-7Z" fill="white" stroke="#193149"><title>Review {day}</title></path>')
    y=top+len(ROWS)*rowheight+30
    s.append(f'<text x="25" y="{y}" class="small">Diamonds on bars = reviews: Oct 23 / Oct 30; Nov 13 / Nov 27; Dec 11; Jan 15 / Jan 29.</text><text x="25" y="{y+28}" class="small">Distribution continues across slices; its bar does not assume full holiday staffing. CI/CD competes for next week’s native-integration capacity.</text><text x="25" y="{y+56}" class="small">Jan 29 is a readiness review, not public publication. Physical hardware, update/persistence qualification and external dependencies can change dates.</text></svg>')
    output=ROOT/'images/project-plan-gantt.svg';output.write_text(''.join(s));print(output)


if __name__=='__main__':main()
