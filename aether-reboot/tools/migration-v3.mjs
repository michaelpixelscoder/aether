// Genuine V1/V2/V3 bundles on one fresh origin. IndexedDB access is read-only;
// construction, refit, circuit installation and saves use real player inputs.
import {chromium,expect} from '@playwright/test';
import fs from 'node:fs/promises';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
const base=process.env.AETHER_MIGRATION_URL||'http://127.0.0.1:4199';
const browser=await chromium.launch({channel:process.env.AETHER_BROWSER||'chrome',headless:true,args:['--use-angle=d3d11','--enable-unsafe-webgpu']});
const page=await browser.newPage({viewport:{width:1672,height:941}});
const report={started_at:new Date().toISOString(),base,errors:[],visits:[],finished:false};
const sha=b=>crypto.createHash('sha256').update(b).digest('hex');
const snap=()=>page.evaluate(()=>window.aetherProbe);
const phase=p=>expect.poll(async()=>(await snap())?.phase,{timeout:p==='Menu'?90000:25000}).toBe(p);
const vessel=s=>s.vessels.find(v=>v.id===s.active);
page.on('pageerror',e=>report.errors.push(e.message));
page.on('console',m=>{if(m.type()==='error')report.errors.push(m.text());});
async function payload(key='session'){return page.evaluate(key=>new Promise((resolve,reject)=>{
 const r=indexedDB.open('aether-isles-reboot',1);r.onerror=()=>reject(r.error);
 r.onsuccess=()=>{const db=r.result,tx=db.transaction('saves','readonly');let data;
  const get=tx.objectStore('saves').get(key);get.onsuccess=()=>{data=get.result;};
  tx.oncomplete=()=>{db.close();resolve(data);};tx.onabort=()=>{db.close();reject(tx.error);};};
}),key);}
async function action(name){
 await expect.poll(async()=>(await snap())?.ui_targets.some(t=>t.action===name&&t.width>0)).toBe(true);
 const t=(await snap()).ui_targets.find(t=>t.action===name);await page.mouse.click(t.x,t.y);await page.waitForTimeout(250);
}
async function save(predicate){
 if((await snap()).phase==='Atlas')await action('Save');
 else {
  const requested=(await snap()).tick;await page.keyboard.press('Control+s',{delay:80});
  const released=(await snap()).tick;await expect.poll(async()=>(await snap()).tick).toBeGreaterThan(released);
  await expect.poll(async()=>JSON.parse(await payload()).tick).toBeGreaterThanOrEqual(requested);
 }
 await expect.poll(async()=>(await snap()).pending_storage).toBe(false);
 await expect.poll(async()=>(await snap()).notice).toContain('enregistrée');
 await expect.poll(async()=>predicate(JSON.parse(await payload()))).toBe(true);
 return JSON.parse(await payload());
}
async function visit(version,expectedSha){
 await page.goto(`${base}/${version}/?probe=1`);await phase('Menu');
 const resources=await page.evaluate(()=>performance.getEntriesByType('resource').map(r=>r.name));
 assert(resources.filter(u=>u.includes('/assets/')||u.includes('/build/')).every(u=>new URL(u).pathname.startsWith('/'+version+'/')),'Each binary loads its own assets');
 const wasmUrl=resources.find(u=>u.endsWith('/aether_game_bg.wasm'));assert(wasmUrl);
 const response=await page.request.get(wasmUrl);assert(response.ok());
 const digest=sha(await response.body());assert.equal(digest,expectedSha);
 report.visits.push({version,url:page.url(),wasm_sha256:digest,resources});
}
const hashes={legacy:'e20f3e7e9d2b76acfec66911104418c38c2274cd1764a4a2e7fad6b88734caf8',v2:'dd321132d076b722e5f5f9c9e8a4e95260d84db797928cbdae96577db65196ae',current:'b42485f3a7a1f43bf50e6e1e4386d4046ab3760b7552ecb8ef4433484877a54b'};
try {
 await visit('legacy',hashes.legacy);await page.keyboard.press('Enter',{delay:80});await phase('Playing');
 const initial=(await snap()).cells;await page.keyboard.press('Tab',{delay:80});await phase('Editing');
 await expect.poll(async()=>(await snap()).mesh_pending).toBe(0);await page.waitForTimeout(400);
 const point=(await snap()).edit_point;assert(point?.length===2);await page.mouse.click(...point);
 await expect.poll(async()=>(await snap()).cells).toBe(initial+1);
 await page.keyboard.press('Tab',{delay:80});await phase('Playing');
 const old=await save(s=>s.version===1);report.legacy={initial_cells:initial,edited_cells:initial+1,save_sha256:sha(await payload()),active_id:old.active};
 const original=vessel(old);assert(original);
 await visit('v2',hashes.v2);await page.keyboard.press('l',{delay:80});await phase('Playing');
 assert.equal((await snap()).cells,initial+1);const cargo=(await snap()).expedition.cargo;
 await page.keyboard.press('m',{delay:80});await phase('Atlas');await action('MotorRefit');
 await expect.poll(async()=>(await snap()).notice).toContain('Deux hélices');
 await page.keyboard.press('m',{delay:80});await phase('Playing');
 const upgraded=await save(s=>s.version===2);await save(s=>s.version===2);
 const refit=vessel(upgraded);assert.deepEqual(refit.blueprint.cells,original.blueprint.cells);
 for(const part of original.blueprint.parts)assert.deepEqual(refit.blueprint.parts.find(p=>p.id===part.id),part);
 const engines=refit.blueprint.parts.filter(p=>p.kind==='Propeller');assert.equal(engines.length,2);
 const expectedCargo=cargo.map((n,i)=>n-[4,4,2,0,0,0][i]);assert.deepEqual(upgraded.expedition.cargo,expectedCargo);
 report.v2={active:upgraded.active,engine_ids:engines.map(p=>p.id),cargo:expectedCargo,save_sha256:sha(await payload())};
 await visit('current',hashes.current);await page.keyboard.press('l',{delay:80});await phase('Playing');
 assert.equal((await snap()).cells,initial+1);assert.deepEqual((await snap()).expedition.cargo,expectedCargo);
 await page.keyboard.press('m',{delay:80});await phase('Atlas');await action('Engineering');
 const plain=await save(s=>s.version===3&&!vessel(s).circuit);assert.deepEqual(vessel(plain).blueprint,refit.blueprint);
 await action('CircuitEnable');
 const wired=await save(s=>s.version===3&&vessel(s).circuit?.network.circuit.links.length>0);
 await save(s=>s.version===3&&!!vessel(s).circuit); // Main AND backup must be V3.
 const backupBytes=await payload('backup');assert.equal(JSON.parse(backupBytes).version,3);
 assert.deepEqual(vessel(JSON.parse(backupBytes)).circuit.network.circuit,vessel(wired).circuit.network.circuit);
 assert.deepEqual(vessel(wired).blueprint.parts,refit.blueprint.parts);
 assert.deepEqual(vessel(wired).blueprint.cells,refit.blueprint.cells);
 assert.deepEqual(vessel(wired).blueprint.circuit_design,vessel(wired).circuit.network.circuit);
 assert.deepEqual(wired.expedition.cargo,expectedCargo);
 const bytes=await payload();report.v3={active:wired.active,plain,wired,save_sha256:sha(bytes)};
 for(const version of ['v2','legacy']){
  await visit(version,hashes[version]);await page.keyboard.press('l',{delay:80});
  await expect.poll(async()=>!(await snap()).pending_storage&&/Version|version|incompatible|invalide|illisible/.test((await snap()).notice)).toBe(true);
  assert.equal((await snap()).phase,'Menu');assert.equal(await payload(),bytes,'An older reader never overwrites V3');
  assert.equal(await payload('backup'),backupBytes,'An older reader never overwrites the V3 backup');
  report[version+'_refusal']={notice:(await snap()).notice,stored_bytes_unchanged:true,backup_bytes_unchanged:true};
 }
 await visit('current',hashes.current);await page.keyboard.press('l',{delay:80});await phase('Playing');
 assert.equal((await snap()).cells,initial+1);assert.deepEqual((await snap()).expedition.cargo,expectedCargo);assert.equal(await payload(),bytes);
 await page.keyboard.press('m',{delay:80});await phase('Atlas');await action('Engineering');
 report.restored=await save(s=>s.version===3&&!!vessel(s).circuit);
 assert.deepEqual(vessel(report.restored).blueprint,vessel(wired).blueprint);
 assert.deepEqual(vessel(report.restored).circuit.network.circuit,vessel(wired).circuit.network.circuit);
 assert.deepEqual(report.errors,[]);report.finished=true;await page.screenshot({path:'docs/evidence/migration-v3.png'});
} catch(error){report.failure=String(error);process.exitCode=1;report.last=await snap().catch(()=>null);await page.screenshot({path:'docs/evidence/migration-v3-failure.png'}).catch(()=>{});}
finally {await fs.writeFile('docs/evidence/migration-v3.json',JSON.stringify(report,null,2));await browser.close();}
console.log(JSON.stringify({finished:report.finished,visits:report.visits.map(v=>({version:v.version,wasm_sha256:v.wasm_sha256})),failure:report.failure,errors:report.errors},null,2));
