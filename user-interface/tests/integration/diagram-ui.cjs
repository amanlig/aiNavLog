const {chromium}=require('playwright');
const assert=require('node:assert/strict');
const path=require('node:path');
(async()=>{
 const browser=await chromium.launch({headless:true,executablePath:process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH,args:['--no-sandbox']});
 try {
 const page=await browser.newPage({viewport:{width:390,height:844}});
 page.setDefaultTimeout(15000);
 const errors=[];page.on('pageerror',e=>errors.push(e.message));
 const button=name=>page.getByRole('button',{name,exact:true});
 const field=name=>page.getByRole('textbox',{name,exact:true});
 const maintenance=async request=>{
   const response=await page.request.post('http://127.0.0.1:18787/maintenance',{headers:{Origin:'http://127.0.0.1:18081','Content-Type':'application/json'},data:request});
   const result=await response.json();assert(!result.error,JSON.stringify(result));return result;
 };
 await page.goto('http://127.0.0.1:18081/elec/maintenance');
 await button('Use example diagram').waitFor();
 await button('Use example diagram').click();
 await field('AI scan server URL').fill('http://127.0.0.1:8788');
 await field('Scan server access token').fill('integration-only-session-token');
 let rescan=false;
 await page.route('http://127.0.0.1:8788/scan-diagram',async route=>{
   const body=route.request().postDataJSON();assert(body.image.startsWith('data:image/png;base64,'));assert(!body.equipment);
   await route.fulfill({json:{components:[{name:rescan?'Renamed image label':'House battery 1',category:'Unknown / custom',model:'',serialNumber:rescan?'EXAMPLE-123':'',x:.18,y:.76,width:.17,height:.18}]}});
 });
 await button('Scan with AI').click();
 await field('Equipment name').fill('House battery 1');
 await button('Choose common type').click();
 await button('LiFePO₄ battery').click();
 await field('Model').fill('Test battery model');await field('Serial number').fill('EXAMPLE-123');
 await button('Save diagram and equipment').click();
 await page.getByText('Diagram and reviewed equipment saved on this device.',{exact:true}).waitFor();
 let snapshot=await maintenance({op:'list'});
 const battery=snapshot.equipment.find(e=>e.name==='House battery 1');assert(battery);assert.equal(battery.category,'LiFePO₄ battery');assert.equal(battery.serialNumber,'EXAMPLE-123');
 await maintenance({op:'save',entry:{id:'diagram-service',equipmentId:battery.id,system:'electrical',date:'2026-11-01',kind:'Inspection',description:'Checked battery terminals',provider:'Owner',hours:null,parts:'',costMinor:null,currency:'',reference:'',notes:'',nextDate:null,nextHours:null}});
 await page.reload();
 await button('1. House battery 1').click();
 await page.getByText('Model: Test battery model',{exact:true}).waitFor();await page.getByText('Serial number: EXAMPLE-123',{exact:true}).waitFor();await page.getByText('Date: 2026-11-01',{exact:true}).waitFor();
 await button('Close equipment details').click();
 await button('View service history').waitFor({state:'hidden'});
 await button('1. House battery 1').click();await button('View service history').click();
 await page.getByRole('heading',{name:'Equipment Service History',exact:true}).waitFor();await page.getByText('Checked battery terminals',{exact:true}).filter({visible:true}).waitFor();
 assert(page.url().includes(encodeURIComponent(battery.id)));
 // A direct refresh must retain the route's equipment filter.
 await page.reload();await page.getByText('Checked battery terminals',{exact:true}).filter({visible:true}).waitFor();
 await page.goto('http://127.0.0.1:18081/elec/maintenance');await button('Edit equipment links').click();
 await field('AI scan server URL').fill('http://127.0.0.1:8788');await field('Scan server access token').fill('integration-only-session-token');
 rescan=true;await button('Rescan with AI (replaces review)').click();
 await button('Link House battery 1').waitFor();
 await button('Save diagram and equipment').click();await page.getByText('Diagram and reviewed equipment saved on this device.',{exact:true}).waitFor();
 snapshot=await maintenance({op:'list'});assert.equal(snapshot.equipment.filter(e=>e.serialNumber==='EXAMPLE-123').length,1);assert.equal(snapshot.entries.length,1);assert.equal(snapshot.entries[0].equipmentId,battery.id);
 // Local manual editing works when scan service is unavailable; cancel preserves current diagram.
 await button('Edit equipment links').click();await button('Add component manually').click();await field('Equipment name').last().fill('Unconfirmed panel');await button('Cancel diagram changes').click();
 snapshot=await maintenance({op:'list'});assert(!snapshot.equipment.some(e=>e.name==='Unconfirmed panel'));
 // Exercise the real browser image-picker upload and cancel without replacing saved data.
 const chooserEvent=page.waitForEvent('filechooser');await button('Add/update diagram').click();
 const chooser=await chooserEvent;
 const example=require('../../src/diagrams/example.json');
 await chooser.setFiles({name:'electrical-diagram.png',mimeType:'image/png',buffer:Buffer.from(example.uri.split(',')[1],'base64')});
 await page.getByRole('heading',{name:'Review diagram equipment',exact:true}).waitFor();
 await button('Cancel diagram changes').click();
 // Desktop hover opens the same dismissible dialog.
 await page.setViewportSize({width:1200,height:900});
 await page.getByRole('button',{name:'Service details for House battery 1',exact:true}).hover();await button('Close equipment details').waitFor();await button('Close equipment details').click();await button('Close equipment details').waitFor({state:'hidden'});await page.mouse.move(5,5);
 assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth));
 await page.setViewportSize({width:390,height:844});
 await page.getByRole('heading',{name:'Electrical Maintenance',exact:true}).scrollIntoViewIfNeeded();
 await page.screenshot({path:path.resolve(__dirname,'../../../docs/images/electrical-diagram-implemented.png'),fullPage:true});
 assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth));assert.deepEqual(errors,[]);
 console.log('Diagram UI passed: example, stubbed AI review, LiFePO4 choice, real SQLite image/equipment persistence, modal/close, filtered history/reload, identity preservation on rescan, manual cancel, desktop hover and mobile layout. Live AI and native devices not tested.');
 }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exit(1);});
