import assert from 'node:assert/strict';
import test from 'node:test';
import {costMinor,validDate,optionalNumber,validate,inSystem,recordSystem} from '../src/maintenance/model.ts';
test('manual maintenance retains exact cents and optional unknown hours',()=>{
 assert.equal(costMinor('123.45'),12345);assert.equal(costMinor('0'),0);assert.equal(costMinor(''),null);assert.throws(()=>costMinor('1.234'));assert.throws(()=>costMinor('-1'));
 assert.equal(optionalNumber('','Hours'),null);assert.equal(optionalNumber('0','Hours'),0);assert.throws(()=>optionalNumber('NaN','Hours'));assert.throws(()=>optionalNumber('-1','Hours'));
 assert(validDate('2028-02-29'));assert(!validDate('2026-02-29'));assert(!validDate('2026-13-01'));
 const entry={equipmentId:'engine',date:'2026-10-30',description:'Changed oil',hours:100,nextHours:200,nextDate:'2027-10-30',costMinor:100,currency:'CAD'};
 assert.doesNotThrow(()=>validate(entry));assert.throws(()=>validate({...entry,nextHours:99}));assert.throws(()=>validate({...entry,currency:''}));
 assert(inSystem({system:'both'},'engine'));assert(inSystem({system:'both'},'electrical'));assert(!inSystem({system:'engine'},'electrical'));
});

test('service log ownership is independent of shared equipment and preserves older entries',()=>{
 const equipment={id:'alternator',name:'Alternator',system:'both'};
 assert.equal(recordSystem({system:'electrical'},equipment),'electrical');
 assert.equal(recordSystem({system:'engine'},equipment),'engine');
 assert.equal(recordSystem({},equipment),'legacy');
 assert.equal(recordSystem({},{...equipment,system:'engine'}),'engine');
});
