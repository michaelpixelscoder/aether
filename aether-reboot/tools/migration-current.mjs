// Real legacy/current binaries on one isolated origin; storage is only read.
// Construction, refit and writes pass through the player's keys and buttons.
import {chromium,expect} from '@playwright/test';
import fs from 'node:fs/promises';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
const base=process.env.AETHER_MIGRATION_URL||'http://127.0.0.1:4194';
const browser=await chromium.launch({channel:process.env.AETHER_BROWSER||'chrome',headless:true,args:['--use-angle=d3d11','--enable-unsafe-webgpu']});
const page=await browser.newPage({viewport:{width:1280,height:720}});
const report={started_at:new Date().toISOString(),base,errors:[],visits:[],finished:false};
const sha=b=>crypto.createHash('sha256').update(b).digest('hex');
const snap=()=>page.evaluate(()=>window.aetherProbe);
const phase=p=>expect.poll(async()=>(await snap())?.phase,{timeout:p==='Menu'?90000:25000}).toBe(p);
page.on('pageerror',e=>report.errors.push(e.message));
page.on('console',m=>{if(m.type()==='error')report.errors.push(m.text());});
async function payload(){return page.evaluate(()=>new Promise((resolve,reject)=>{
 const r=indexedDB.open('aether-isles-reboot',1);r.onerror=()=>reject(r.error);
 r.onsuccess=()=>{const db=r.result,tx=db.transaction('saves','readonly');let data;
  const get=tx.objectStore('saves').get('session');get.onsuccess=()=>{data=get.result;};
  tx.oncomplete=()=>{db.close();resolve(data);};tx.onabort=()=>{db.close();reject(tx.error);};};
}));}
async function save(){
 const requested=(await snap()).tick;await page.keyboard.press('Control+s',{delay:80});
 const released=(await snap()).tick;await expect.poll(async()=>(await snap()).tick).toBeGreaterThan(released);
 await expect.poll(async()=>(await snap()).pending_storage).toBe(false);
 await expect.poll(async()=>(await snap()).notice).toContain('enregistrée');
 await expect.poll(async()=>JSON.parse(await payload()).tick).toBeGreaterThanOrEqual(requested);
 return JSON.parse(await payload());
}
async function visit(version){
 await page.goto(`${base}/${version}/?probe=1`);await phase('Menu');
 const resources=await page.evaluate(()=>performance.getEntriesByType('resource').map(r=>r.name));
 assert(resources.filter(u=>u.includes('/assets/')||u.includes('/build/')).every(u=>new URL(u).pathname.startsWith('/'+version+'/')),'No assets from another binary');
 const wasmUrl=resources.find(u=>u.endsWith('/aether_game_bg.wasm'));assert(wasmUrl,'Served WASM recorded');
 const response=await page.request.get(wasmUrl);assert(response.ok());
 report.visits.push({version,url:page.url(),wasm_sha256:sha(await response.body()),resources});
}
try {
 await visit('legacy');await page.keyboard.press('Enter',{delay:80});await phase('Playing');
 const initial=(await snap()).cells;await page.keyboard.press('Tab',{delay:80});await phase('Editing');
 await expect.poll(async()=>(await snap()).mesh_pending).toBe(0);await page.waitForTimeout(400);
 const point=(await snap()).edit_point;assert(point?.length===2);await page.mouse.click(...point);
 await expect.poll(async()=>(await snap()).cells).toBe(initial+1);
 await page.keyboard.press('Tab',{delay:80});await phase('Playing');
 const old=await save();assert.equal(old.version,1);report.legacy={initial_cells:initial,edited_cells:initial+1,save_sha256:sha(await payload()),active_id:old.active};
 const original=old.vessels.find(v=>v.id===old.active);assert(original);
 await visit('current');await page.keyboard.press('l',{delay:80});await phase('Playing');
 await expect.poll(async()=>(await snap()).mesh_pending).toBe(0);assert.equal((await snap()).cells,initial+1);
 const cargo=(await snap()).expedition.cargo;
 await page.keyboard.press('m',{delay:80});await phase('Atlas');
 await expect.poll(async()=>(await snap()).ui_targets.some(t=>t.action==='MotorRefit'&&t.width>0)).toBe(true);
 const target=(await snap()).ui_targets.find(t=>t.action==='MotorRefit');await page.mouse.click(target.x,target.y);
 await expect.poll(async()=>(await snap()).notice).toContain('Deux hélices');
 await page.keyboard.press('m',{delay:80});await phase('Playing');
 const converted=await save();await save(); // Both main and backup must be v2.
 assert.equal(converted.version,2);assert.equal(converted.active,old.active);
 const refit=converted.vessels.find(v=>v.id===old.active);assert(refit);
 assert.deepEqual(refit.blueprint.cells,original.blueprint.cells,'User construction preserved');
 for(const part of original.blueprint.parts)assert.deepEqual(refit.blueprint.parts.find(p=>p.id===part.id),part,'Original equipment ID and pose preserved');
 const engines=refit.blueprint.parts.filter(p=>p.kind==='Propeller');assert.equal(engines.length,2);
 assert.equal(new Set(refit.blueprint.parts.map(p=>p.id)).size,refit.blueprint.parts.length);
 const expectedCargo=cargo.map((n,i)=>n-[4,4,2,0,0,0][i]);assert.deepEqual(converted.expedition.cargo,expectedCargo);
 const v2Bytes=await payload();report.refit={version:2,active_id:converted.active,cells:refit.blueprint.cells.length,engine_ids:engines.map(p=>p.id),cargo_before:cargo,cargo_after:expectedCargo,save_sha256:sha(v2Bytes)};
 await visit('legacy');await page.keyboard.press('l',{delay:80});
 await expect.poll(async()=>!((await snap()).pending_storage)&&/Version|version|incompatible|invalide|illisible/.test((await snap()).notice)).toBe(true);
 assert.equal((await snap()).phase,'Menu');assert.equal(await payload(),v2Bytes,'Old reader never overwrites incompatible data');
 report.legacy_refusal={phase:(await snap()).phase,notice:(await snap()).notice,stored_bytes_unchanged:true};
 await visit('current');await page.keyboard.press('l',{delay:80});await phase('Playing');
 assert.equal((await snap()).cells,initial+1);assert.deepEqual((await snap()).expedition.cargo,expectedCargo);
 assert.equal(await payload(),v2Bytes);report.restored=await snap();report.finished=true;
 assert.deepEqual(report.errors,[]);await page.screenshot({path:'docs/evidence/migration-current.png'});
} catch(error){report.failure=String(error);process.exitCode=1;report.last=await snap().catch(()=>null);await page.screenshot({path:'docs/evidence/migration-current-failure.png'}).catch(()=>{});}
finally {await fs.writeFile('docs/evidence/migration-current.json',JSON.stringify(report,null,2));await browser.close();}
console.log(JSON.stringify({finished:report.finished,visits:report.visits.map(v=>({version:v.version,wasm_sha256:v.wasm_sha256})),refit:report.refit,failure:report.failure,errors:report.errors},null,2));
