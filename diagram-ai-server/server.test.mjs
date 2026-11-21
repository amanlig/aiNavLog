import test from 'node:test';
import assert from 'node:assert/strict';
import {once} from 'node:events';
import {mkdtempSync,writeFileSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {createScanServer,analyze,validateImage,validateComponents,runtimeSecret,decodeComponents} from './server.mjs';
const image='data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+/l9sAAAAASUVORK5CYII=';
const result={components:[{name:'House battery',category:'Unknown / custom',model:'',serialNumber:'',x:.1,y:.2,width:.2,height:.3}]};
test('gateway rejects unauthorized, invalid, disallowed-origin requests before AI; handles valid and upstream failure',async()=>{
  let calls=0;const errors=[];const originalError=console.error;console.error=(...args)=>errors.push(args.join(' '));
  const server=createScanServer({apiKey:'test-not-a-real-key',token:'test-token-at-least-24-characters',origins:['http://localhost:8081'],maxPerMinute:20,analyzeImpl:async()=>{calls++;if(calls===2)throw Error('private provider details');return result;}});
  server.listen(0,'127.0.0.1');await once(server,'listening');
  const url=`http://127.0.0.1:${server.address().port}/scan-diagram`;
  const headers={Authorization:'Bearer test-token-at-least-24-characters','Content-Type':'application/json'};
  try {
    const health=await fetch(url.replace('/scan-diagram','/healthz'));
    assert.equal(health.status,200);assert.deepEqual(await health.json(),{status:'ok'});
    assert.equal(calls,0);
    assert.equal((await fetch(url,{method:'POST',body:'{}'})).status,401);
    assert.equal((await fetch(url,{method:'POST',headers:{...headers,Origin:'https://untrusted.example'},body:JSON.stringify({image})})).status,403);
    assert.equal((await fetch(url,{method:'POST',headers,body:JSON.stringify({image:'data:image/png;base64,AAAA'})})).status,400);
    assert.equal(calls,0);
    const ok=await fetch(url,{method:'POST',headers:{...headers,Origin:'http://localhost:8081'},body:JSON.stringify({image})});
    assert.equal(ok.headers.get('access-control-allow-origin'),'http://localhost:8081');
    assert.deepEqual(await ok.json(),result);
    const bad=await fetch(url,{method:'POST',headers,body:JSON.stringify({image})});
    assert.equal(bad.status,502);assert.doesNotMatch(await bad.text(),/private provider/);
    assert.equal(JSON.parse(errors[0]).code,'analysis_failed');assert.doesNotMatch(errors.join(' '),/private provider|test-not-a-real-key|iVBORw/);
  }finally{console.error=originalError;server.closeAllConnections();await new Promise(resolve=>server.close(resolve));}
});
test('real Responses request uses schema, image, no stored response; parses completed output and rejects bad output',async()=>{
  let request;
  const answer=await analyze(image,{apiKey:'test',fetchImpl:async(url,options)=>{request={url,...JSON.parse(options.body)};return {ok:true,json:async()=>({status:'completed',output:[{type:'message',content:[{type:'output_text',text:JSON.stringify({components:[{name:'House battery',category:'Unknown / custom',model:'',serialNumber:'',left:.1,top:.2,right:.3,bottom:.5}]})}]}]})};}});
  assert.deepEqual(answer,result);assert.equal(request.store,false);assert.equal(request.text.format.strict,true);
  assert.equal(request.input[0].content[1].image_url,image);
  assert.throws(()=>validateImage('data:image/png;base64,AAAA'));
  assert.throws(()=>validateComponents({components:[{...result.components[0],x:.95}]}));
  await assert.rejects(analyze(image,{apiKey:'test',fetchImpl:async()=>({ok:true,json:async()=>({status:'incomplete',output:[]})})}));
});

test('runtime secret configuration does not print or expose secret file contents',()=>{
  assert.equal(runtimeSecret('TEST',{TEST:'private-value'}),'private-value');
  const directory=mkdtempSync(join(tmpdir(),'diagram-secret-test-'));
  try {const file=join(directory,'secret');writeFileSync(file,'file-secret-value\n',{mode:0o600});assert.equal(runtimeSecret('TEST',{TEST_FILE:file}),'file-secret-value');}
  finally{rmSync(directory,{recursive:true});}
  assert.throws(()=>runtimeSecret('TEST',{TEST:'private-value',TEST_FILE:'/unused'}),/Configure only TEST/);
  assert.throws(()=>runtimeSecret('TEST',{TEST_FILE:'/path-that-does-not-exist'}),/^Error: Could not read TEST_FILE\.$/);
});

test('corner-coordinate contract converts to bounded UI regions and rejects malformed boxes',()=>{
 const corner={name:'Controller',category:'MPPT solar charge controller',model:'',serialNumber:'',left:.72,top:.4,right:.98,bottom:.8};
 const component=decodeComponents({components:[corner]}).components[0];
 assert.deepEqual({x:component.x,y:component.y,width:component.width,height:component.height},{x:.72,y:.4,width:.26,height:.4});
 assert.throws(()=>decodeComponents({components:[{...corner,right:1.02}]}),/bounds/);
 assert.throws(()=>decodeComponents({components:[{...corner,right:.6}]}),/bounds/);
 assert.throws(()=>decodeComponents({components:[{...corner,bottom:corner.top}]}),/bounds/);
 assert.throws(()=>decodeComponents({components:[{...corner,left:720}]}),/bounds/);
});
