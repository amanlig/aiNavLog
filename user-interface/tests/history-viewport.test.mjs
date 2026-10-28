import { test } from 'node:test';
import assert from 'node:assert/strict';
import { buttonZoom, clampWindow, HOUR, MINUTE, MIN_WINDOW, panWindow, pinchWindow, touchPair, windowLabel, zoomWindow } from '../src/electrical/history-viewport.ts';
import { calendarRange } from '../src/electrical/history.ts';
const day={start:0,end:24*HOUR};

test('each outward pinch halves 24 -> 12 -> 6 -> 3 hours; inward pinch doubles',()=>{
  let view=day;
  for(const hours of [12,6,3]) {
    view=pinchWindow(view,day,1.4,.5);
    assert.equal(view.end-view.start,hours*HOUR);
  }
  view=pinchWindow(view,day,.7,.5);
  assert.equal(view.end-view.start,6*HOUR);
  assert.deepEqual(pinchWindow(view,day,1.05,.5),view,'small finger jitter does not zoom');
});
test('pinch focal time stays under fingers until constrained by period edges',()=>{
  const view={start:6*HOUR,end:18*HOUR};
  const next=pinchWindow(view,day,1.4,.25,.4);
  const focal=9*HOUR;
  assert(Math.abs(next.start+.4*(next.end-next.start)-focal)<=1);
  assert.deepEqual(zoomWindow(day,12*HOUR,day,0),{start:0,end:12*HOUR});
  assert.deepEqual(zoomWindow(day,12*HOUR,day,24*HOUR),{start:12*HOUR,end:24*HOUR});
});
test('zoom buttons use one-hour steps, then five-minute steps near the minimum',()=>{
  const bounds={start:0,end:48*HOUR};
  const view={start:12*HOUR,end:36*HOUR};
  const next=buttonZoom(view,bounds,'in',24*HOUR);
  assert.equal(next.end-next.start,23*HOUR);
  assert.equal((next.start+next.end)/2,24*HOUR);
  const hour={start:10*HOUR,end:11*HOUR};
  const finer=buttonZoom(hour,day,'in');
  assert.equal(finer.end-finer.start,55*MINUTE);
  assert.equal(buttonZoom(finer,day,'out').end-buttonZoom(finer,day,'out').start,HOUR);
});
test('zoom/pan clamp to selected period and never go below five minutes or fractional query milliseconds',()=>{
  let view=day;
  for(let i=0;i<20;i++)view=pinchWindow(view,day,1.5,.37);
  assert.equal(view.end-view.start,MIN_WINDOW);
  assert(Number.isSafeInteger(view.start)&&Number.isSafeInteger(view.end));
  assert.equal(buttonZoom(view,day,'in').end-buttonZoom(view,day,'in').start,MIN_WINDOW);
  assert.deepEqual(panWindow(view,-100*HOUR,day),{start:0,end:MIN_WINDOW});
  assert.deepEqual(panWindow(view,100*HOUR,day),{start:day.end-MIN_WINDOW,end:day.end});
  assert.deepEqual(clampWindow({start:-HOUR,end:50*HOUR},day),day);
  assert.deepEqual(pinchWindow(day,day,.5,.5),day);
});
test('visible labels retain hour, minute and sub-minute resolution',()=>{
  assert.equal(windowLabel(day),'24 hours');
  assert.equal(windowLabel({start:0,end:1.5*HOUR}),'1 h 30 min');
  assert.equal(windowLabel({start:0,end:45*MINUTE}),'45 minutes');
  assert.equal(windowLabel({start:0,end:22.5*MINUTE}),'22 min 30 s');
  assert.equal(windowLabel({start:0,end:MIN_WINDOW}),'5 minutes');
});
test('two-finger geometry uses distance and midpoint; single touch is not a pinch',()=>{
  assert.equal(touchPair([{pageX:0,pageY:0}]),null);
  assert.deepEqual(touchPair([{pageX:0,pageY:0},{pageX:30,pageY:40}]),{distance:50,midpoint:15});
});
test('full calendar viewport includes the whole selected day instead of shrinking to elapsed time',()=>{
  const original=process.env.TZ;process.env.TZ='America/Vancouver';
  try {
    const range=calendarRange(new Date(2026,9,28,8),1);
    assert.equal(range.end-range.start,24*HOUR);
    assert.equal(new Date(range.start).getHours(),0);
    assert.equal(new Date(range.end).getDate(),29);
  } finally {if(original===undefined)delete process.env.TZ;else process.env.TZ=original;}
});
