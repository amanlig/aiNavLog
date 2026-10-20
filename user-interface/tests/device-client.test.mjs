import { test } from 'node:test';
import assert from 'node:assert/strict';
import { parseDeviceSample } from '../src/electrical/device-client.ts';
import { isFresh } from '../src/electrical/model.ts';
const sample = {source:'live', receivedAt:1000, houseVoltage:12.8, houseCurrent:-5.25, houseCharge:77.5, starterVoltage:null};
test('bridge preserves missing values and original receipt time', () => {
  const result = parseDeviceSample(sample);
  assert.deepEqual(result, sample);
  assert.equal(isFresh(result, 6001), false);
  assert.equal(parseDeviceSample(null), null);
});
test('bridge rejects malformed data rather than showing invented readings', () => {
  for (const value of [{}, {...sample, source:'unknown'}, {...sample, receivedAt:'1000'}, {...sample, houseCurrent:Infinity}, {...sample, starterVoltage:undefined}]) {
    assert.throws(() => parseDeviceSample(value));
  }
});

test('service replay is explicitly labelled simulated', () => { assert.equal(parseDeviceSample({...sample, source:'simulated'}).source, 'simulated'); });
