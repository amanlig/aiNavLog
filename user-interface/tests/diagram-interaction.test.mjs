import {test} from 'node:test';
import assert from 'node:assert/strict';
import {componentAction} from '../src/diagrams/interaction.ts';
test('mobile tap shows service for both initial and saved diagrams',()=>{
  assert.equal(componentAction(false,false,false),'show-service');
  assert.equal(componentAction(false,true,false),'show-service');
});
test('editing and desktop first-use behavior remain available',()=>{
  assert.equal(componentAction(true,false,false),'show-service');
  assert.equal(componentAction(true,true,true),'edit-review');
  assert.equal(componentAction(false,false,true),'edit-new');
  assert.equal(componentAction(false,true,true),'show-service');
});
