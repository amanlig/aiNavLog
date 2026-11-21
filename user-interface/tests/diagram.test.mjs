import test from 'node:test';
import assert from 'node:assert/strict';
import {suggestedMatch,buildReviewed,parseDetections,componentTypes} from '../src/diagrams/model.ts';
const component={name:'Panel 1',category:'Solar panel',model:'100W',serialNumber:'',x:.1,y:.1,width:.2,height:.2};
const existing=[{id:'panel-a',name:'Panel 1',system:'electrical',model:'100W'},{id:'panel-b',name:'Panel 2',system:'electrical',model:'100W'}];
const review={...component,key:'area1',newId:'new-panel',equipmentId:'',ignored:false};
test('repeated models do not match by appearance; unique exact serial or name only proposes identity',()=>{
  assert.equal(suggestedMatch({...component,name:'Unidentified panel'},existing),'');
  assert.equal(suggestedMatch(component,existing),'panel-a');
  assert.equal(suggestedMatch({...component,name:'new label',serialNumber:'ABC-123'},[{...existing[0],serialNumber:'ABC123'}]),'panel-a');
  assert.equal(suggestedMatch({...component,serialNumber:'NEW'},[{...existing[0],serialNumber:'OLD'}]),'');
  assert.equal(suggestedMatch(component,[existing[0],{...existing[0],id:'duplicate'}]),'');
});
test('review creates separate unknown records, preserves matched IDs, validates boxes and duplicate metadata',()=>{
  assert.ok(componentTypes.includes('LiFePO₄ battery'));
  const built=buildReviewed([review,{...review,key:'area2',newId:'new-panel2'}],[]);
  assert.equal(built.equipment.length,2);assert.equal(built.areas[1].equipmentId,'new-panel2');
  const matched=buildReviewed([{...review,equipmentId:'panel-a'}],existing);
  assert.equal(matched.equipment[0].id,'panel-a');assert.equal(matched.equipment[0].name,'Panel 1');
  assert.equal(buildReviewed([{...review,ignored:true}],[]).equipment.length,0);
  assert.throws(()=>buildReviewed([{...review,x:.99}],[]));
  assert.throws(()=>buildReviewed([{...review,equipmentId:'missing'}],existing));
  assert.throws(()=>buildReviewed([{...review,equipmentId:'panel-a'},{...review,key:'other',equipmentId:'panel-a',model:'different'}],existing));
  assert.throws(()=>parseDetections({components:[{...component,width:Infinity}]}));
});
