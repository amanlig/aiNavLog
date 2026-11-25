// Regression: sharing the Engine viewer must not restore removed Electrical POC controls.
const {chromium}=require('playwright');
const assert=require('node:assert/strict');
(async()=>{
 const browser=await chromium.launch({headless:true,executablePath:process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH,args:['--no-sandbox']});
 try {
  const page=await browser.newPage({viewport:{width:1100,height:900}});
  const errors=[];page.on('pageerror',e=>errors.push(e.message));
  const origin=process.env.ENGINE_TEST_ORIGIN||'http://127.0.0.1:49380';
  const button=name=>page.getByRole('button',{name,exact:true});
  const field=name=>page.getByRole('textbox',{name,exact:true});
  await page.goto(`${origin}/elec/maintenance`);
  if(process.env.POC_VERIFY_REOPEN==='1'){
   await button('1. House battery').click();
   await page.getByText('Model: POC-REGRESSION',{exact:true}).waitFor();
   await page.getByText('Description: Existing Electrical service must survive editing',{exact:true}).waitFor();
   await button('Edit equipment details').click();await field('Model').waitFor();
   assert.equal(await field('Model').inputValue(),'POC-REGRESSION');
   await button('Save changes').waitFor();assert.equal(await button('Create separate equipment').count(),0);
   assert.deepEqual(errors,[]);
   console.log('PASS: saved equipment changes and service history persist after host process restart; editor save remains available.');
   return;
  }
  await button('Add diagram').waitFor();
  const config=await page.evaluate(()=>globalThis.__AINAVLOG_DESKTOP__);
  async function call(data){const response=await page.request.post(`${origin}/maintenance`,{headers:{Origin:origin,'Content-Type':'application/json','X-AiNavLog-Token':config.token},data});const result=await response.json();assert(!result.error,JSON.stringify(result));return result;}
  const image='data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+/l9sAAAAASUVORK5CYII=';
  await call({op:'diagram_put',imageId:'poc-image',index:0,body:image});
  const diagram={revision:'poc-electrical',imageId:'poc-image',imageChunks:1,width:640,height:1500,areas:[{equipmentId:'house-battery',x:.1,y:.03,width:.2,height:.08},{equipmentId:'starter-battery',x:.7,y:.91,width:.2,height:.08}]};
  await call({op:'diagram_save',diagram,equipment:[],baseRevision:null});
  await call({op:'save',entry:{id:'poc-preserve-service',equipmentId:'house-battery',system:'electrical',date:'2026-11-24',kind:'Inspection',description:'Existing Electrical service must survive editing',provider:'',hours:null,parts:'',costMinor:null,currency:'',reference:'',notes:'',nextDate:null,nextHours:null}});
  await call({op:'save',entry:{id:'poc-starter-service',equipmentId:'starter-battery',system:'electrical',date:'2026-11-24',kind:'Inspection',description:'Latest starter battery service',provider:'',hours:null,parts:'',costMinor:null,currency:'',reference:'',notes:'',nextDate:null,nextHours:null}});
  await page.reload();
  async function assertHover(name,description,review=false){
   await page.getByRole('button',{name:review?`Review ${name} ${name==='House battery'?1:2}`:`Service details for ${name}`,exact:true}).hover();
   const popup=page.getByLabel('Latest equipment service');await popup.waitFor();
   await popup.getByText(`Description: ${description}`,{exact:true}).waitFor();
   await page.waitForTimeout(100);
   const box=await popup.boundingBox(),viewport=page.viewportSize();assert(box);
   assert(box.x>=0&&box.y>=0&&box.x+box.width<=viewport.width&&box.y+box.height<=viewport.height,`${name} hover popup must be on-screen: ${JSON.stringify(box)}`);
   await page.mouse.move(5,5);await popup.waitFor({state:'hidden'});
  }
  await assertHover('House battery','Existing Electrical service must survive editing');
  await assertHover('Starter battery','Latest starter battery service');
  await button('1. House battery').click();await button('Edit equipment details').click();
  await field('Model').waitFor();
  async function assertNoLinking(){
   assert.equal(await button('Create separate equipment').count(),0);
   assert.equal(await page.getByRole('button',{name:/^Link /}).count(),0);
   assert.equal(await page.getByText('Link a component already recorded on another view, or create a separate component.',{exact:true}).count(),0);
  }
  await assertNoLinking();
  const session=await page.context().newCDPSession(page);
  async function assertSaveAccessible(){
   const box=await button('Save changes').boundingBox();assert(box);
   const visible=await page.evaluate(()=>({left:visualViewport?.offsetLeft??0,top:visualViewport?.offsetTop??0,width:visualViewport?.width??innerWidth,height:visualViewport?.height??innerHeight}));
   assert(box.x>=visible.left-.5&&box.y>=visible.top-.5&&box.x+box.width<=visible.left+visible.width+.5&&box.y+box.height<=visible.top+visible.height+.5,`Save must fit visible window: ${JSON.stringify({box,visible})}`);
   const heading=await page.getByRole('heading',{name:'Equipment details',exact:true}).boundingBox();assert(heading);
   assert(box.y<heading.y+heading.height+12,'Save changes belongs in the editor top bar');
  }
  // Resize an already-open editor, preserving pending inputs and its top action.
  await field('Model').fill('RESIZE-DRAFT');
  for(const [width,height] of [[1000,650],[800,500],[640,480],[420,360],[320,480],[1100,900]]){
   await page.setViewportSize({width,height});await page.waitForTimeout(150);
   await assertSaveAccessible();assert.equal(await field('Model').inputValue(),'RESIZE-DRAFT');
  }
  // Save must remain visible when the user scrolls to lower equipment fields.
  await field('Model').hover();await page.mouse.wheel(0,1800);await page.waitForTimeout(150);
  const saveBox=await button('Save changes').boundingBox();assert(saveBox);
  assert(saveBox.y>=0&&saveBox.y+saveBox.height<=page.viewportSize().height,'Editor Save changes must remain visible while scrolling');
  await field('Model').fill('POC-REGRESSION');await button('Close review').click();
  await assertHover('House battery','Existing Electrical service must survive editing',true);
  await assertHover('Starter battery','Latest starter battery service',true);
  // The editor has its own save action; the user need not find a button below
  // a tall image. Close/reopen review first to retain the hover regression.
  await page.getByRole('button',{name:'Review House battery 1',exact:true}).click();
  await field('Model').waitFor();assert.equal(await field('Model').inputValue(),'POC-REGRESSION');
  // Exercise the actual save while the window is small and zoomed, not just bounds.
  await page.setViewportSize({width:800,height:500});
  await session.send('Emulation.setPageScaleFactor',{pageScaleFactor:1.5});await page.waitForTimeout(150);
  await assertSaveAccessible();
  await page.screenshot({path:require('node:path').resolve(__dirname,'../../docs/images/equipment-editor-compact.png')});
  await button('Save changes').click();
  await page.getByText('Diagram and reviewed equipment saved on this device.',{exact:true}).waitFor();
  await session.send('Emulation.setPageScaleFactor',{pageScaleFactor:1});await page.setViewportSize({width:1100,height:900});
  await field('Model').waitFor({state:'hidden'});
  const snapshot=await call({op:'list'});
  assert.equal(snapshot.equipment.find(e=>e.id==='house-battery').model,'POC-REGRESSION');
  assert.equal(snapshot.entries.filter(e=>e.id==='poc-preserve-service').length,1);
  assert.equal((await call({op:'diagram_read'})).diagram.areas[0].equipmentId,'house-battery');
  // Confirm the controls stay absent after saving and reloading, not just first use.
  await page.reload();await button('1. House battery').click();await button('Edit equipment details').click();
  await field('Model').waitFor();assert.equal(await field('Model').inputValue(),'POC-REGRESSION');await assertNoLinking();
  await button('Close review').click();await button('Cancel diagram changes').click();
  await assertHover('Starter battery','Latest starter battery service');
  // The same omission applies when creating a new Electrical component.
  await button('Edit equipment links').click();await button('Add component manually').click();await field('Equipment name').waitFor();await assertNoLinking();await button('Close review').click();await button('Cancel diagram changes').click();
  // Engine still supports one component shared between multiple views.
  await call({op:'diagram_save',system:'engine',diagramId:'upper',name:'Upper view',baseRevision:null,equipment:[],diagram:{...diagram,revision:'poc-engine',areas:[{...diagram.areas[0],equipmentId:'main-engine'}]}});
  await page.goto(`${origin}/mech/maintenance`);await button('1. Main engine').click();await button('Edit equipment details').click();
  await button('Create separate equipment').waitFor();assert(await page.getByRole('button',{name:/^Link /}).count()>0);
  assert.deepEqual(errors,[]);
  console.log('PASS: Electrical hover shows latest service on-screen for upper/lower components of tall diagrams, after closing an editor before save, and dismisses on exit; linking controls absent for existing/new components and after reload; top-bar Save changes stays accessible on resize/scroll/zoom, preserves drafts and persists metadata from a small scaled window; saved equipment identity and service history retained; Engine linking preserved.');
 } finally {await browser.close();}
})().catch(e=>{console.error(e);process.exit(1);});
