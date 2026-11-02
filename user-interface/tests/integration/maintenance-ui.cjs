const {chromium}=require('playwright');
const assert=require('node:assert/strict');
const path=require('node:path');
(async()=>{
 const browser=await chromium.launch({headless:true,executablePath:process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH,args:['--no-sandbox']});
 try {
 const page=await browser.newPage({viewport:{width:390,height:844},timezoneId:'America/Vancouver'});
 page.setDefaultTimeout(10000);
 await page.clock.setFixedTime(new Date('2026-10-30T21:00:00Z'));
 const errors=[];page.on('pageerror',e=>errors.push(e.message));
 const button=name=>page.getByRole('button',{name,exact:true});
 const field=name=>page.getByRole('textbox',{name,exact:true});
 await page.goto('http://127.0.0.1:18081/elec/maintenance');
 await page.getByText('No completed work recorded for this selection.',{exact:true}).waitFor();
 await button('Earlier shared records (1)').click();
 await page.getByText('Earlier shared inspection',{exact:true}).waitFor();
 await button('Earlier shared records (1)').click();
 await button('Record completed work').click();await button('Alternator').click();
 await field('Work performed *').fill('Checked alternator wiring 🛥️');
 await field('Work date * (YYYY-MM-DD)').fill('2026-02-29');await button('Save service record').click();
 await page.getByText('Enter a valid service date (YYYY-MM-DD).',{exact:true}).waitFor();
 await field('Work date * (YYYY-MM-DD)').fill('2026-10-30');
 await field('Equipment hours (optional, manual)').fill('312.5');
 await field('Performed by / service provider').fill('Owner');
 await field('Cost (optional)').fill('123.45');
 await field('Invoice / survey reference').fill('Survey A-3');
 await field('Next service date (YYYY-MM-DD)').fill('2027-04-30');
 await field('Next service at equipment hours').fill('500');
 // Server commits, but client loses the acknowledgement: retry must not duplicate.
 let lost=false;
 await page.route('**/maintenance',async route=>{
  const data=route.request().postDataJSON();
  if(data?.op==='save'&&!lost){lost=true;await route.fetch();await route.abort();}else await route.continue();
 });
 await button('Save service record').click();await page.getByRole('alert').waitFor();
 assert.equal(await field('Work performed *').inputValue(),'Checked alternator wiring 🛥️');
 await button('Save service record').click();
 await page.getByText('Service record saved on this device.',{exact:true}).waitFor();
 await page.getByText('Checked alternator wiring 🛥️',{exact:true}).waitFor();
 assert.equal(await page.getByText('Checked alternator wiring 🛥️',{exact:true}).count(),1);
 await page.reload();await page.getByText('Cost: CAD 123.45',{exact:true}).waitFor();
 await page.goto('http://127.0.0.1:18081/mech/maintenance');
 await page.getByRole('heading',{name:'Engine Maintenance',exact:true}).waitFor();
 assert.equal(await page.getByText('Checked alternator wiring 🛥️',{exact:true}).count(),0);
 assert.equal(await button('All systems').count(),0);
 // Create equipment and a separate record with unknown hours, never fabricated zero.
 await button('Record completed work').click();await button('Add equipment').click();
 await field('Equipment name').fill('Port engine');await button('Save equipment').click();
 await field('Work performed *').fill('Replaced oil filter');await button('Replacement').click();
 await button('Save service record').click();await page.getByText('Replaced oil filter',{exact:true}).waitFor();
 assert.equal(await page.getByText(/2026-10-30 · Replacement · 0 h/).count(),0);
 await page.goto('http://127.0.0.1:18081/elec/maintenance');
 await page.getByRole('heading',{name:'Electrical Maintenance',exact:true}).waitFor();
 await page.getByText('Checked alternator wiring 🛥️',{exact:true}).waitFor();
 assert.equal(await page.getByText('Replaced oil filter',{exact:true}).count(),0);
 assert.equal(await button('Show Port engine').count(),0);
 await page.setViewportSize({width:390,height:1800});
 await page.getByRole('heading',{name:'Electrical Maintenance',exact:true}).scrollIntoViewIfNeeded();
 await page.screenshot({path:path.resolve(__dirname,'../../../docs/images/maintenance-implemented.png'),fullPage:true});
 await page.setViewportSize({width:1200,height:900});
 assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth));
 // Reject cross-origin mutation, without exposing a permissive write endpoint.
 const denied=await fetch('http://127.0.0.1:18787/maintenance',{method:'POST',headers:{Origin:'https://untrusted.example','Content-Type':'application/json'},body:JSON.stringify({op:'list'})});assert.equal(denied.status,403);
 assert.deepEqual(errors,[]);
 console.log('Maintenance SQLite/UI: date validation, failed-ack retry/idempotency, reload, separate log ownership on shared equipment, custom equipment, unknown hours, filters and origin protection passed.');
 } finally {await browser.close();}
})().catch(e=>{console.error(e);process.exit(1);});
