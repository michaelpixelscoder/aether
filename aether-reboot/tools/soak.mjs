// A real rendered browser session. The probe is read-only; every action is input.
import {chromium,expect} from '@playwright/test';
import fs from 'node:fs/promises';
import crypto from 'node:crypto';
import path from 'node:path';
const seconds=Number(process.env.AETHER_SOAK_SECONDS||1800);
const channel=process.env.AETHER_BROWSER||'chrome';
const url=process.env.AETHER_TEST_URL||'http://127.0.0.1:4173';
const browser=await chromium.launch({channel,headless:true,args:['--use-angle=d3d11','--enable-unsafe-webgpu']});
const page=await browser.newPage({viewport:{width:1440,height:900}});
const report={started:new Date().toISOString(),requested_seconds:seconds,channel,url,cycles:[],errors:[],finished:false};
const bytes=await(await fetch(url+'/build/aether_game_bg.wasm')).arrayBuffer();
report.wasm_sha256=crypto.createHash('sha256').update(new Uint8Array(bytes)).digest('hex');
const hash=bytes=>crypto.createHash('sha256').update(bytes).digest('hex');
async function assetEntries(dir,prefix=''){
 const entries=[];
 for(const entry of (await fs.readdir(dir,{withFileTypes:true})).sort((a,b)=>a.name.localeCompare(b.name))){
  const name=prefix+entry.name,file=path.join(dir,entry.name);
  if(entry.isDirectory())entries.push(...await assetEntries(file,name+'/'));
  else {const data=await fs.readFile(file);entries.push({path:name,sha256:hash(data)});}
 }
 return entries;
}
report.asset_files=await assetEntries('../assets');
report.assets_sha256=hash(JSON.stringify(report.asset_files));
for(const asset of report.asset_files){
 const response=await fetch(url+'/assets/'+asset.path.split('/').map(encodeURIComponent).join('/'));
 if(!response.ok||hash(new Uint8Array(await response.arrayBuffer()))!==asset.sha256)throw Error('Asset servi différent : '+asset.path);
}
page.on('pageerror',e=>report.errors.push(e.message));
page.on('console',m=>{if(m.type()==='error')report.errors.push(m.text());});
const snap=()=>page.evaluate(()=>window.aetherProbe);
const phase=value=>expect.poll(async()=>(await snap())?.phase,{timeout:value==='Menu'?60000:25000}).toBe(value);
const settle=()=>expect.poll(async()=>{const s=await snap();return s.mesh_pending===0&&!s.pending_storage;},{timeout:20000}).toBe(true);
async function click(action){await expect.poll(async()=>(await snap()).ui_targets.some(t=>t.action===action&&t.width>0)).toBe(true);const p=(await snap()).ui_targets.find(t=>t.action===action);await page.mouse.click(p.x,p.y);}
const save=()=>fs.writeFile('../docs/evidence/rendered-soak.json',JSON.stringify(report,null,2));
const timer=Date.now();
try {
  await page.goto(url+'/?probe=1');await phase('Menu');await page.keyboard.press('Enter');await phase('Playing');
  const original=(await snap()).cells;
  while(Date.now()-timer<seconds*1000){
    await settle();await page.keyboard.press('Tab');await phase('Editing');await page.waitForTimeout(500);
    const point=(await snap()).edit_point;await page.mouse.click(...point);
    await expect.poll(async()=>(await snap()).cells).toBe(original+1);
    await page.keyboard.press('Control+z');await expect.poll(async()=>(await snap()).cells).toBe(original);
    await page.keyboard.press('Control+y');await expect.poll(async()=>(await snap()).cells).toBe(original+1);
    await settle();await page.keyboard.press('Control+s');await expect.poll(async()=>(await snap()).notice).toContain('enregistrée');
    await page.keyboard.press('Tab');await phase('Playing');await page.keyboard.press('f');await expect.poll(async()=>(await snap()).docked).toBe(false);
    await page.keyboard.down('e');await page.waitForTimeout(1600);await page.keyboard.up('e');
    await page.keyboard.down('x');await page.waitForTimeout(250);await page.keyboard.up('x');
    await page.waitForTimeout(15000);await page.keyboard.press('g');await page.waitForTimeout(1200);await page.keyboard.press('g');
    await page.keyboard.press('t');await expect.poll(async()=>(await snap()).walker).toBe(true);
    await page.keyboard.press('Space');await page.waitForTimeout(700);await page.keyboard.press('t');
    await expect.poll(async()=>(await snap()).walker).toBe(false);
    await page.keyboard.press('Control+s');await expect.poll(async()=>(await snap()).notice).toContain('enregistrée');
    await settle();await page.keyboard.press('Control+o');await settle();await page.waitForTimeout(700);
    const flight=await snap();expect(flight.navigation.position.every(Number.isFinite)).toBe(true);expect(flight.cells).toBe(original+1);
    await page.keyboard.press('r');await expect.poll(async()=>(await snap()).docked).toBe(true);await settle();
    await page.keyboard.press('Escape');await phase('Paused');await click('Settings');await phase('Settings');
    await click('Motion');await page.waitForTimeout(250);await click('Motion');await page.keyboard.press('Escape');await phase('Paused');
    await click('Menu');await phase('Menu');await settle();await page.keyboard.press('Enter');await phase('Playing');await settle();await page.waitForTimeout(1000);
    const s=await snap();report.cycles.push({seconds:(Date.now()-timer)/1000,entities:s.entities,meshes:s.mesh_assets,materials:s.material_assets,memory_bytes:s.wasm_linear_memory_bytes,tick:s.tick,flight:flight.navigation,frame_p95_ms:s.frame_ms_p50_p95_p99[1]});
    if(report.cycles.length%10===0)await page.screenshot({path:`../docs/evidence/soak-${report.cycles.length}.png`});
    expect(report.errors).toEqual([]);report.elapsed_seconds=(Date.now()-timer)/1000;await save();
    console.log(`cycle ${report.cycles.length}, ${report.elapsed_seconds.toFixed(1)} s, ${s.entities} entités, ${s.mesh_assets} meshes`);
    // Fail early on sustained growth; retain the partial report for diagnosis.
    if(report.cycles.length>=12){
      const baseline=report.cycles[2];
      expect(s.entities<=baseline.entities+8&&s.mesh_assets<=baseline.meshes+8&&s.material_assets<=baseline.materials+2).toBe(true);
    }
  }
  const stable=report.cycles.slice(2);const first=stable[0];
  expect(stable.every(s=>s.entities<=first.entities+8&&s.meshes<=first.meshes+8&&s.materials<=first.materials+2)).toBe(true);
  report.finished=true;report.elapsed_seconds=(Date.now()-timer)/1000;
}catch(error){report.failure=String(error);process.exitCode=1;await page.screenshot({path:'../docs/evidence/soak-failure.png'}).catch(()=>{});}
finally {await save();await browser.close();}
