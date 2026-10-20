// Browser integration check. Requires Playwright, an exported UI configured for
// http://127.0.0.1:18787/electrical, and the servers described in the root README.
const assert = require('node:assert/strict');
const { spawn } = require('node:child_process');
const path = require('node:path');
const { chromium } = require('playwright');
const root = path.resolve(__dirname, '../../..');
const database = process.env.BMV_TEST_DATABASE || '/tmp/ainavlog-bmv-ui.sqlite3';
const replay = () => spawn(path.join(root, 'device-interface/target/debug/examples/replay_bmv'), [database, path.join(root, 'simulation/bmv712/integration-packets.log')], {stdio:'inherit'});
(async () => {
  const browser = await chromium.launch({headless:true});
  let source;
  try {
    const page = await browser.newPage({viewport:{width:1200,height:900}});
    const errors = [];
    page.on('pageerror', error => errors.push(error.message));
    await page.goto('http://127.0.0.1:18081/elec');
    await page.getByText('House battery', {exact:true}).waitFor();
    source = replay();
    await page.getByText('Simulated BMV readings · device interface', {exact:true}).waitFor({timeout:15000});
    await page.getByText('77.5', {exact:false}).waitFor();
    let data = await (await fetch('http://127.0.0.1:18787/electrical')).json();
    assert.equal(data.source,'simulated');
    assert.equal(data.houseVoltage,12.76); assert.equal(data.houseCurrent,-5.25);
    assert.equal(data.houseCharge,77.5); assert.equal(data.starterVoltage,12.8);
    await page.getByText('77.3', {exact:false}).waitFor({timeout:18000});
    await page.screenshot({path:path.join(root,'docs/images/bmv-device-ui-integration.png'),fullPage:true});
    const body = await page.locator('body').innerText();
    assert(body.includes('Discharging · supplying onboard loads'));
    assert(body.includes('-5.3')); assert(body.includes('12.8'));
    source.kill(); source = null;
    await page.getByText('Updates stopped. Latest readings are hidden until a fresh sample arrives.',{exact:true}).waitFor({timeout:10000});
    assert(!(await page.locator('body').innerText()).includes('77.3'));
    source = replay();
    await page.getByText('Simulated BMV readings · device interface',{exact:true}).waitFor({timeout:10000});
    assert.deepEqual(errors,[]);
    console.log('PASS: simulator ciphertext → Rust monitor/decoder → SQLite → HTTP → rendered UI; changing SOC, stale hiding and recovery verified.');
  } finally { source?.kill(); await browser.close(); }
})().catch(error => { console.error(error); process.exitCode=1; });
