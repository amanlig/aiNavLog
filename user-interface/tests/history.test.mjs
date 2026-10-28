import { test } from 'node:test';
import assert from 'node:assert/strict';
import { dateRange, shiftDay, parseHistory, series, nearest, metricRange, demoHistory } from '../src/electrical/history.ts';

test('history validates sorted actual samples, presence bits and range boundaries', () => {
  const rows = [[1000,15,12.8,-5.25,77.5,12.9,1],[2000,9,12.7,0,0,12.8,2]];
  const points = parseHistory(rows, 1000, 3000);
  assert.equal(points[0].receivedAt,1000);
  assert.equal(points[0].houseCurrent,-5.25);
  assert.equal(points[1].houseCharge,null);
  assert.equal(points[1].starterVoltage,12.8);
  for (const value of [null, [[1000,16,1,2,3,4,1]], [[1000,15,NaN,2,3,4,1]], [rows[1],rows[0]], [[3000,15,1,2,3,4,1]]]) assert.throws(()=>parseHistory(value,1000,3000));
});
test('chart does not connect segments or unavailable metrics; summaries exclude missing values', () => {
  const points=parseHistory([[1000,15,12.8,-5,77,12.9,1],[2000,15,12.7,-4,76,12.8,1],[9000,9,12.6,0,0,12.7,2],[10000,15,12.5,12,78,14.2,3]],0,20000);
  assert.deepEqual(series(points,'houseCharge').map(run=>run.map(p=>p.receivedAt)),[[1000,2000],[10000]]);
  assert.deepEqual(metricRange(points,'houseCharge'),{low:76,high:78});
  assert.equal(nearest(points,'houseCharge',9100).receivedAt,10000);
  assert.equal(metricRange([], 'starterVoltage'),null);
});
test('date selection uses local calendar boundaries, including DST', () => {
  const original=process.env.TZ;
  process.env.TZ='America/Vancouver';
  try {
    const anchor=new Date(2025,10,2,12);
    const {start,end}=dateRange(anchor,1,new Date(2025,10,3,12).getTime());
    assert.equal(end-start,25*3600000);
    assert.equal(new Date(start).getHours(),0);
    assert.equal(shiftDay(anchor,-1).getDate(),1);
    const now=new Date(2025,10,2,12).getTime();
    assert.equal(dateRange(anchor,1,now).end,now+1);
  } finally { if(original===undefined) delete process.env.TZ; else process.env.TZ=original; }
});
test('illustrative demo history stays consistent when a timestamp is re-read at finer zoom', () => {
  const start=new Date(2026,9,28).getTime();
  const end=new Date(2026,9,29).getTime();
  const points=demoHistory(start,end);
  assert(points.every(p=>p.receivedAt>=start && p.receivedAt<end));
  assert.equal(series(points,'houseCharge').length,2);
  const selected=points[Math.floor(points.length*.6)];
  const fine=demoHistory(selected.receivedAt,selected.receivedAt+300000);
  assert.deepEqual(fine[0],selected,'zoom must not invent different values at the same receipt time');
  const gapStart=new Date(2026,9,28,8,0).getTime();
  assert.deepEqual(demoHistory(gapStart,gapStart+300000),[]);
});
