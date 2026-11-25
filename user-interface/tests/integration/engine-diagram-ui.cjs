const {chromium}=require('playwright');
const assert=require('node:assert/strict');
const path=require('node:path');
(async()=>{
 const browser=await chromium.launch({headless:true,executablePath:process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH,args:['--no-sandbox']});
 try{
 const context=await browser.newContext({viewport:{width:390,height:844},hasTouch:true,isMobile:true});
 let page=await context.newPage();page.setDefaultTimeout(15000);
 const errors=[];page.on('pageerror',e=>errors.push(e.message));
 const origin=process.env.ENGINE_TEST_ORIGIN||'http://127.0.0.1:18081';
 const api=process.env.ENGINE_TEST_API||'http://127.0.0.1:18787';
 const button=name=>page.getByRole('button',{name,exact:true});
 const field=name=>page.getByRole('textbox',{name,exact:true});
 const call=async data=>{const config=await page.evaluate(()=>globalThis.__AINAVLOG_DESKTOP__);const r=await page.request.post(`${api}/maintenance`,{headers:{Origin:origin,'Content-Type':'application/json',...(config?{'X-AiNavLog-Token':config.token}:{})},data});const result=await r.json();assert(!result.error,JSON.stringify(result));return result;};
 await page.goto(`${origin}/mech/maintenance`);
 await button('Add diagram').waitFor();
 assert.equal(await page.getByRole('img',{name:'Saved electrical system diagram'}).count(),0);
 const fixture=await page.evaluate(()=>{const c=document.createElement('canvas');c.width=640;c.height=360;const g=c.getContext('2d');g.fillStyle='#eef3f6';g.fillRect(0,0,640,360);g.fillStyle='#234354';g.font='24px sans-serif';g.fillText('ENGINE VIEW — TEST FIXTURE',40,48);g.strokeStyle='#234354';g.lineWidth=4;g.strokeRect(160,90,320,210);g.fillStyle='#16887f';g.fillRect(272,153,96,54);g.fillStyle='#fff';g.font='15px sans-serif';g.fillText('Oil filter',287,185);return c.toDataURL('image/png');});
 const electrical={revision:'electrical-before',imageId:'electrical-before',imageChunks:Math.ceil(fixture.length/16000),width:640,height:360,areas:[{equipmentId:'house-battery',x:.1,y:.1,width:.2,height:.2}]};
 for(let index=0;index<electrical.imageChunks;index++)await call({op:'diagram_put',imageId:electrical.imageId,index,body:fixture.slice(index*16000,(index+1)*16000)});
 await call({op:'diagram_save',diagram:electrical,equipment:[],baseRevision:null});
 async function pick(label){const pending=page.waitForEvent('filechooser');await button(await button('Add/update diagram').count()?'Add/update diagram':'Add diagram').click();const chooser=await pending;await chooser.setFiles({name:'engine-view.png',mimeType:'image/png',buffer:Buffer.from(fixture.split(',')[1],'base64')});await field('View name').waitFor();if(label==='Replace view image')await button('Replace current view').click();}
 async function saved(){await page.getByText('Diagram and reviewed equipment saved on this device.',{exact:true}).waitFor();await button('Add/update diagram').waitFor();}
 const views=['Upper view','Port side view','Starboard side view','Bottom view'];
 for(let i=0;i<views.length;i++){
  await pick('Add engine view');await field('View name').fill(views[i]);
  await button('Add component manually').click();
  if(i===0){await field('Equipment name').fill('Oil filter');await button('Choose common type').click();await button('Oil filter').click();await field('Model').fill('FILTER-TEST');}
  else await button('Link Oil filter').click();
  await button('Close review').click();await button('Save diagram and equipment').click();await saved();
 }
 let list=await call({op:'diagram_list',system:'engine'});assert.equal(list.diagrams.length,4);
 let snapshot=await call({op:'list'});const oil=snapshot.equipment.find(e=>e.name==='Oil filter');assert(oil);assert.equal(oil.system,'engine');
 for(const view of list.diagrams){const r=await call({op:'diagram_read',system:'engine',diagramId:view.id});assert.equal(r.diagram.areas[0].equipmentId,oil.id);}
 assert.equal(snapshot.equipment.filter(e=>e.name==='Oil filter').length,1);
 assert.deepEqual((await call({op:'diagram_read'})).diagram,electrical);
 // Create real maintenance via the exact same service-history form as Electrical.
 await button('1. Oil filter').click();await page.getByText('Model: FILTER-TEST',{exact:true}).waitFor();await button('View service history').click();
 assert(page.url().includes('/mech/service-history'));
 await button('Record completed work').click();await button('Oil filter').click();
 await field('Work performed *').fill('Replaced engine oil filter');await button('Save service record').click();
 await page.getByText('Replaced engine oil filter',{exact:true}).waitFor();
 await page.reload();await page.getByText('Replaced engine oil filter',{exact:true}).waitFor();
 await page.goto(`${origin}/mech/maintenance`);await page.getByRole('heading',{name:'Upper view',exact:true}).waitFor();
 const active=name=>page.getByRole('heading',{name,exact:true});
 await active('Upper view').waitFor();
 // Saved views survive reload; touch tap, long hold and horizontal swipe remain distinct.
 let session=await context.newCDPSession(page);
 async function drag(dx,dy=0,multi=false,cancel=false){
  const area=page.getByTestId('maintenance-diagram-image');await area.scrollIntoViewIfNeeded();const b=await area.boundingBox();
  const x=b.x+b.width*.5,y=b.y+b.height*.5;
  const points=(px,py)=>multi?[{x:px,y:py,id:1},{x:px+25,y:py+15,id:2}]:[{x:px,y:py,id:1}];
  await session.send('Input.dispatchTouchEvent',{type:'touchStart',touchPoints:points(x,y)});
  for(let n=1;n<=8;n++){await session.send('Input.dispatchTouchEvent',{type:'touchMove',touchPoints:points(x+dx*n/8,y+dy*n/8)});await page.waitForTimeout(20);}
  await session.send('Input.dispatchTouchEvent',{type:cancel?'touchCancel':'touchEnd',touchPoints:[]});
  await page.waitForTimeout(350);
 }
 async function goToView(name){for(let tries=0;tries<8;tries++){const current=(await page.getByRole('heading').allTextContents()).find(text=>views.includes(text));if(current===name)return;assert(current);const direction=views.indexOf(current)<views.indexOf(name)?1:-1;const next=views[views.indexOf(current)+direction];await drag(direction===1?-110:110);await active(next).waitFor();}throw Error(`Could not swipe to ${name}`);}
 await drag(-110);await active('Port side view').waitFor();assert.equal(await button('Close equipment details').count(),0);
 await drag(-110);await active('Starboard side view').waitFor();
 await drag(-110);await active('Bottom view').waitFor();
 await drag(-110);await active('Bottom view').waitFor(); // no wrap at last view
 await drag(110);await active('Starboard side view').waitFor();
 await drag(-20);await active('Starboard side view').waitFor();
 // Every view opens the same completed-work summary and history route.
 for(const name of views){await goToView(name);await active(name).waitFor();await button('1. Oil filter').tap();await page.getByText('Description: Replaced engine oil filter',{exact:true}).waitFor();await button('Close equipment details').click();}
 await goToView('Upper view');await active('Upper view').waitFor();
 const marker=page.getByRole('button',{name:'Service details for Oil filter',exact:true});await marker.scrollIntoViewIfNeeded();const box=await marker.boundingBox();
 await session.send('Input.dispatchTouchEvent',{type:'touchStart',touchPoints:[{x:box.x+box.width/2,y:box.y+box.height/2,id:1}]});await page.waitForTimeout(650);await session.send('Input.dispatchTouchEvent',{type:'touchEnd',touchPoints:[]});
 await field('Model').waitFor();await field('Model').fill('UNSAVED');await button('Close review').click();
 await drag(-110); // reviewing never navigates to another saved view
 await field('View name').waitFor();assert.equal(await field('View name').inputValue(),'Upper view');
 await button('Cancel diagram changes').click();await button('1. Oil filter').click();await page.getByText('Model: FILTER-TEST',{exact:true}).waitFor();await button('Close equipment details').click();
 // Replacement cancellation and invalid uploads cannot overwrite other views.
 await pick('Replace view image');await button('Cancel diagram changes').click();
 assert.equal((await call({op:'diagram_list',system:'engine'})).diagrams.length,4);
 assert.deepEqual((await call({op:'diagram_read'})).diagram,electrical);
 snapshot=await call({op:'list'});assert.equal(snapshot.entries.length,1);assert.equal(snapshot.entries[0].system,'engine');
 // Mouse hover still gives the Electrical-style service summary; drag is not swipe.
 const desktop=await browser.newPage({viewport:{width:1200,height:1000}});await desktop.goto(`${origin}/mech/maintenance`);await desktop.getByRole('heading',{name:'Upper view',exact:true}).waitFor();
 const target=desktop.getByRole('button',{name:'Service details for Oil filter',exact:true});await target.hover();await desktop.getByLabel('Latest equipment service').waitFor();
 await desktop.getByText('Description: Replaced engine oil filter',{exact:true}).waitFor();
 const db=await desktop.getByTestId('maintenance-diagram-image').boundingBox();await desktop.mouse.move(db.x+db.width*.8,db.y+db.height*.7);await desktop.mouse.down();await desktop.mouse.move(db.x+db.width*.2,db.y+db.height*.7,{steps:8});await desktop.mouse.up();await desktop.getByRole('heading',{name:'Upper view',exact:true}).waitFor();
 await desktop.close();
 // Electrical still uses its original single diagram and history scope.
 await page.goto(`${origin}/elec/maintenance`);await button('1. House battery').click();await page.getByText('Description: No service history',{exact:true}).waitFor();await button('View service history').click();assert(page.url().includes('/elec/service-history'));assert.equal(await page.getByText('Replaced engine oil filter',{exact:true}).count(),0);
 await page.goto(`${origin}/mech/maintenance`);await active('Upper view').waitFor();
 assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth));
 await page.getByRole('heading',{name:'Engine Maintenance',exact:true}).scrollIntoViewIfNeeded();
 const visibleDiagram=await page.getByTestId('maintenance-diagram-image').boundingBox();
 assert(visibleDiagram.y+visibleDiagram.height<=844,'Engine diagram should fit the initial phone viewport using the Electrical baseline.');
 await page.screenshot({path:path.resolve(__dirname,'../../../docs/images/engine-maintenance-views.png'),fullPage:true});
 // Chromium CDP can retain synthetic contacts after multi-touch/cancel/scroll.
 // Use a separate touch context for each special gesture so stale injected
 // contacts cannot alter the next test's input (real UI/store remain shared).
 await page.close();
 for(const [dx,dy,multi,cancel] of [[0,-70,false,false],[-110,0,true,false],[-110,0,false,true]]){
   const isolated=await browser.newContext({viewport:{width:390,height:844},hasTouch:true,isMobile:true});
   page=await isolated.newPage();page.on('pageerror',e=>errors.push(e.message));
   session=await isolated.newCDPSession(page);
   await page.goto(`${origin}/mech/maintenance`);await active('Upper view').waitFor();
   await drag(dx,dy,multi,cancel);await active('Upper view').waitFor();
   await isolated.close();
 }
 assert.deepEqual(errors,[]);
 console.log('PASS: four engine views, real image uploads, shared equipment/history, reload persistence, isolated Electrical, taps/holds/hover, left/right swipe, edges, short/vertical/multi-touch/cancelled gestures, review cancellation and mobile layout.');
 }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exit(1);});
