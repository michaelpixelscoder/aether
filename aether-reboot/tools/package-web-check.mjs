import {chromium} from '@playwright/test';
import fs from 'node:fs/promises';
import assert from 'node:assert/strict';
const release=JSON.parse(await fs.readFile('dist/latest.json','utf8'));
const base=`http://127.0.0.1:4174/${release.id}/`;
const browser=await chromium.launch({channel:'chrome',headless:true,args:['--use-angle=d3d11','--enable-unsafe-webgpu']});
const results={id:release.id,browser:browser.version(),base,network:{download_mbit_per_second:50,upload_mbit_per_second:10,latency_ms:40},runs:[]};
try{
 const context=await browser.newContext({viewport:{width:1280,height:720}});const page=await context.newPage();
 const errors=[];page.on('pageerror',e=>errors.push(e.message));page.on('console',m=>{if(m.type()==='error')errors.push(m.text());});
 const cdp=await context.newCDPSession(page);await cdp.send('Network.enable');
 let savedCells=0;
 await cdp.send('Network.emulateNetworkConditions',{offline:false,latency:40,downloadThroughput:50_000_000/8,uploadThroughput:10_000_000/8});
 for(const phase of ['cold','warm']){
   const started=performance.now();await page.goto(base+'?probe=1');
   await page.waitForFunction(()=>window.aetherProbe?.phase==='Menu',null,{timeout:90000});
   const seconds=(performance.now()-started)/1000;
   const measured=await page.evaluate(()=>({resources:performance.getEntriesByType('resource').map(r=>({url:r.name,transfer_bytes:r.transferSize,decoded_bytes:r.decodedBodySize,duration_ms:r.duration})),memory:window.aetherProbe.wasm_linear_memory_bytes}));
   assert(page.url().includes(`/r/${release.id}/`));assert.equal(await page.locator('#loading-screen').isVisible(),false);
   results.runs.push({phase,first_interactive_seconds:seconds,wasm_linear_memory_bytes:measured.memory,resources:measured.resources});
   if(phase==='cold'){
     await page.keyboard.press('Enter');await page.waitForFunction(()=>window.aetherProbe?.phase==='Playing');
     savedCells=await page.evaluate(()=>window.aetherProbe.cells);
     await page.keyboard.press('Control+s');await page.waitForFunction(()=>window.aetherProbe?.notice.includes('enregistrée'));
   }else{
     await page.keyboard.press('l');await page.waitForFunction(()=>window.aetherProbe?.phase==='Playing');
     assert.equal(await page.evaluate(()=>window.aetherProbe.cells),savedCells);
   }
 }
 const response=await page.request.get(base+`r/${release.id}/build/aether_game_bg.wasm`,{headers:{'accept-encoding':'br'}});
 results.wasm_headers=response.headers();assert.equal(response.status(),200);assert.match(results.wasm_headers['cache-control'],/immutable/);
 assert.equal(results.wasm_headers['content-encoding'],'br');assert.equal(results.wasm_headers['content-type'],'application/wasm');
 results.errors=errors;assert.deepEqual(errors,[]);results.saved_session_after_launcher_reload=true;
 // A v2 save cannot be rolled back into the v1 reader. Exercise the actual
 // forward migration, explicit refusal by the old binary and intact v2 reload.
 const candidates=[];
 for(const e of await fs.readdir('dist/web',{withFileTypes:true})){
   if(!e.isDirectory()||!/^0\.1\.0-/.test(e.name))continue;
   try{const build=JSON.parse(await fs.readFile(`dist/web/${e.name}/r/${e.name}/BUILD.json`,'utf8'));if(build.save_format===1)candidates.push(build);}catch{}
 }
 const older=candidates.sort((a,b)=>a.built_at.localeCompare(b.built_at)).at(-1)?.id;
 assert(older,'A real v1 distribution is required for migration validation');
 const legacyContext=await browser.newContext({viewport:{width:1440,height:900}});
 try{
   const legacy=await legacyContext.newPage();const seen=[];
   legacy.on('pageerror',e=>errors.push(e.message));
   async function visit(id){
     await legacy.goto(`http://127.0.0.1:4174/${id}/?probe=1`);
     await legacy.waitForFunction(()=>window.aetherProbe?.phase==='Menu',null,{timeout:90000});
     assert(legacy.url().includes(`/r/${id}/`));
     const resources=await legacy.evaluate(()=>performance.getEntriesByType('resource').map(r=>r.name));
     assert(resources.filter(u=>u.includes('/assets/')||u.includes('/build/')).every(u=>u.includes(`/r/${id}/`)));
     seen.push(id);
   }
   async function save(){await legacy.keyboard.press('Control+s');await legacy.waitForFunction(()=>!window.aetherProbe.pending_storage&&window.aetherProbe.notice.includes('enregistrée'));}
   await visit(older);await legacy.keyboard.press('Enter');await legacy.waitForFunction(()=>window.aetherProbe?.phase==='Playing');
   const original=await legacy.evaluate(()=>window.aetherProbe.cells);await save();
   await visit(release.id);await legacy.keyboard.press('l');await legacy.waitForFunction(()=>window.aetherProbe?.phase==='Playing');
   assert.equal(await legacy.evaluate(()=>window.aetherProbe.cells),original);
   await legacy.keyboard.press('m');await legacy.waitForFunction(()=>window.aetherProbe?.phase==='Atlas');
   const motor=await legacy.evaluate(()=>window.aetherProbe.ui_targets.find(t=>t.action==='MotorRefit'));
   assert(motor);await legacy.mouse.click(motor.x,motor.y);
   await legacy.waitForFunction(()=>window.aetherProbe.notice.includes('Deux hélices'));
   await legacy.keyboard.press('m');await legacy.waitForFunction(()=>window.aetherProbe?.phase==='Playing');
   await save();await legacy.waitForTimeout(500);await save();
   const cargo=await legacy.evaluate(()=>window.aetherProbe.expedition.cargo);
   await visit(older);await legacy.keyboard.press('l');
   await legacy.waitForFunction(()=>!window.aetherProbe.pending_storage&&/Version|version|incompatible|invalide|illisible/.test(window.aetherProbe.notice));
   const refusal=await legacy.evaluate(()=>({phase:window.aetherProbe.phase,notice:window.aetherProbe.notice}));
   assert.equal(refusal.phase,'Menu');
   await visit(release.id);await legacy.keyboard.press('l');await legacy.waitForFunction(()=>window.aetherProbe?.phase==='Playing');
   assert.equal(await legacy.evaluate(()=>window.aetherProbe.cells),original);
   assert.deepEqual(await legacy.evaluate(()=>window.aetherProbe.expedition.cargo),cargo);
   results.migration={visited_builds:seen,old_save_format:1,new_save_format:2,restored_cells:original,legacy_refusal:refusal,refit_persisted:true,mixed_asset_versions:false};
 }finally{await legacyContext.close();}
 assert.deepEqual(errors,[]);
 await page.screenshot({path:'docs/evidence/web-packaged.png'});
}finally{await browser.close();await fs.writeFile('docs/evidence/web-package-check.json',JSON.stringify(results,null,2));}
console.log(JSON.stringify({id:results.id,browser:results.browser,runs:results.runs.map(({phase,first_interactive_seconds,wasm_linear_memory_bytes})=>({phase,first_interactive_seconds,wasm_linear_memory_bytes})),errors:results.errors},null,2));
