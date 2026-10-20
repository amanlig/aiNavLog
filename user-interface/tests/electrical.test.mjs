import { test } from 'node:test';
import assert from 'node:assert/strict';
import { simulate, isFresh, reading, starterAlert, parseThreshold } from '../src/electrical/model.ts';
test('BMV demo follows Windows discharge/charge cycle and includes starter voltage', () => {
  const a = simulate(0, 1000); assert.equal(a.houseVoltage, 12.76); assert.equal(a.houseCurrent, -5.25); assert.equal(a.houseCharge, 77.5); assert.equal(a.starterVoltage, 12.8);
  const b = simulate(60, 2000); assert.equal(b.houseCurrent, 12); assert.equal(b.starterVoltage, 14.2); assert.equal(b.houseCharge, 76.3);
});
test('stale/missing data never reports an active voltage alert', () => {
  const a = simulate(0, 1000);
  assert.equal(isFresh(a, 6000), true); assert.equal(isFresh(a, 6001), false);
  assert.equal(starterAlert(a, 6001, 13), 'Unavailable');
  assert.equal(starterAlert({...a, starterVoltage:null}, 1000, 13), 'Unavailable');
  assert.equal(starterAlert(null, 1000, 13), 'Unavailable');
  assert.equal(starterAlert(a, 1000, null), 'Not configured');
  assert.equal(starterAlert(a, 1000, 13), 'Low voltage');
  assert.equal(starterAlert(a, 1000, 12.8), 'At or above threshold');
});
test('missing readings and invalid threshold remain explicit', () => {
  for (const v of [null, undefined, NaN, Infinity]) assert.equal(reading(v), '—');
  assert.equal(reading(-5.25, 2, true), '-5.25'); assert.equal(reading(12, 2, true), '+12.00');
  for (const text of ['', '0', '-1', 'NaN', '70.01', '12abc']) assert.equal(parseThreshold(text), null);
  assert.equal(parseThreshold('12.2'), 12.2);
});
