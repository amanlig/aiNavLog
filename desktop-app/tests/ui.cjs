const {chromium}=require('playwright');const assert=require('node:assert/strict');
(async()=>{const browser=await chromium.launch({headless:true,executablePath:process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH,args:['--no-sandbox']});try{
 const page=await browser.newPage({viewport:{width:1200,height:1000}});const failures=[];page.on('pageerror',e=>failures.push(e.message));
 const remote=[];page.on('request',r=>{if(!r.url().startsWith('http://127.0.0.1:49380/')&&!r.url().startsWith('data:'))remote.push(r.url());});
 await page.goto('http://127.0.0.1:49380/elec/maintenance');
 if(process.argv[2]==='record'){
  await page.getByRole('button',{name:'Record completed work',exact:true}).click();
  await page.getByRole('button',{name:'Alternator',exact:true}).click();
  await page.getByRole('textbox',{name:'Work performed *',exact:true}).fill('Desktop embedded UI service test');
  await page.getByRole('textbox',{name:'Equipment hours (optional, manual)',exact:true}).fill('0');
  await page.getByRole('button',{name:'Save service record',exact:true}).click();
 }
 await page.getByText('Desktop embedded UI service test',{exact:true}).waitFor();
 await page.goto('http://127.0.0.1:49380/mech/maintenance');
 await page.getByRole('heading',{name:'Engine Maintenance',exact:true}).waitFor();
 assert.equal(await page.getByText('Desktop embedded UI service test',{exact:true}).count(),0);
 if(process.argv[2]==='record'){
  await page.getByRole('button',{name:'Record completed work',exact:true}).click();
  await page.getByRole('button',{name:'Alternator',exact:true}).click();
  await page.getByRole('textbox',{name:'Work performed *',exact:true}).fill('Engine alternator service');
  await page.getByRole('button',{name:'Save service record',exact:true}).click();
 }
 await page.getByText('Engine alternator service',{exact:true}).waitFor();
 await page.goto('http://127.0.0.1:49380/');await page.getByTestId('fire-safety-card').waitFor();
 await page.getByRole('button',{name:'Enlarge fire safety boat layout'}).hover();await page.getByTestId('fire-safety-hover-preview').waitFor();
 assert.deepEqual(failures,[]);assert.deepEqual(remote,[]);console.log('Embedded desktop UI: save/separate logs, reload, home hover, zero hours and offline asset routing pass');
}finally{await browser.close();}})().catch(e=>{console.error(e);process.exit(1)});
