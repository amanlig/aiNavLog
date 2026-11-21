"""Render the November 21, 2026 provisional project schedule as SVG."""
from datetime import date, timedelta
from html import escape
from pathlib import Path

ROOT = Path(__file__).resolve().parent
START = date(2026, 11, 23)
END = date(2027, 1, 30)  # Exclusive
LEFT = 335
SCALE = 10
WIDTH = LEFT + (END - START).days * SCALE + 50
ROWS = [
    ('Electrical / mobile stabilization', '2026-11-23', '2026-12-05', 'feature', '2026-12-04'),
    ('Engine', '2026-12-07', '2026-12-19', 'feature', '2026-12-18'),
    ('Route', '2026-12-21', '2027-01-02', 'feature', '2026-12-31'),
    ('Authentication / authorization', '2027-01-04', '2027-01-16', 'feature', '2027-01-15'),
    ('Route story', '2027-01-18', '2027-01-30', 'feature', '2027-01-29'),
    ('Distribution / CI follow-up', '2026-11-23', '2027-01-30', 'parallel', '2026-12-04'),
]
COLORS = {'feature':'#285d99', 'parallel':'#148576'}


def x(day):
    return LEFT + (date.fromisoformat(day) - START).days * SCALE


def main():
    top, rowheight = 180, 62
    height = top + len(ROWS) * rowheight + 120
    s = [f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {WIDTH} {height}" role="img" aria-labelledby="title desc"><title id="title">aiNavLog project schedule, November 2026 to January 2027</title><desc id="desc">Provisional schedule starts with electrical stabilization on November 23, followed by engine, route, authentication and route story. Distribution and CI follow-up run in parallel.</desc><style>text{{font-family:Arial,sans-serif;fill:#193149}}.h{{font-size:26px;font-weight:bold}}.label{{font-size:15px}}.small{{font-size:13px;fill:#52697f}}</style><rect width="{WIDTH}" height="{height}" fill="white"/><text x="25" y="40" class="h">aiNavLog · revised project schedule</text><text x="25" y="70" class="label">Rebaselined November 21, 2026 · planning targets, subject to review</text>']
    for i, (key, label) in enumerate([('feature','Planned work'),('parallel','Parallel follow-up')]):
        xx = 25 + i * 300
        s.append(f'<rect x="{xx}" y="94" width="16" height="16" rx="3" fill="{COLORS[key]}"/><text x="{xx+24}" y="107" class="small">{label}</text>')
    day = START
    while day < END:
        xx = LEFT + (day - START).days * SCALE
        s.append(f'<path d="M{xx} 150V{top+len(ROWS)*rowheight}" stroke="#e0e7ef"/><text transform="translate({xx+4},160) rotate(-35)" class="small">{day.strftime("%b %d")}</text>')
        day += timedelta(days=7)
    for i, (label, start, end, kind, review) in enumerate(ROWS):
        y = top + i * rowheight
        if i % 2 == 0:
            s.append(f'<rect x="15" y="{y}" width="{WIDTH-30}" height="{rowheight}" fill="#f5f8fb"/>')
        s.append(f'<text x="25" y="{y+33}" class="label">{escape(label)}</text>')
        w = x(end) - x(start)
        s.append(f'<rect x="{x(start)}" y="{y+17}" width="{w}" height="27" rx="5" fill="{COLORS[kind]}"/>')
        last = date.fromisoformat(end) - timedelta(days=1)
        caption = f'{date.fromisoformat(start).strftime("%b %d")}–{last.strftime("%b %d")}'
        s.append(f'<text x="{x(start)+7}" y="{y+36}" style="font:12px Arial;fill:white">{caption}</text>')
        if review:
            xx = x(review) + SCALE / 2
            yy = y + 30
            s.append(f'<path d="M{xx} {yy-7}l7 7-7 7-7-7Z" fill="white" stroke="#193149"><title>Review {review}</title></path>')
    y = top + len(ROWS) * rowheight + 28
    s.append(f'<text x="25" y="{y}" class="small">Diamonds mark reviews. Distribution / CI is reviewed at each milestone; its first review is December 4.</text><text x="25" y="{y+25}" class="small">Dates are planning targets and may change as scope and capacity are reviewed.</text></svg>')
    output = ROOT / 'images/project-plan-gantt.svg'
    output.write_text(''.join(s))
    print(output)


if __name__ == '__main__':
    main()
