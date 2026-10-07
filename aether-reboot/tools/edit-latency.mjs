// A bounded command-to-captured-image measurement, including browser IPC/readback.
// This is deliberately not described as display latency or isolated GPU upload.
import {chromium,expect} from '@playwright/test';
import fs from 'node:fs/promises';
import crypto from 'node:crypto';
const channel=process.env.AETHER_BROWSER||'chrome';
const browser=await chromium.launch({channel,headless:true,args:['--use-angle=d3d11','--enable-unsafe-webgpu']});
const page=await browser.newPage({viewport:{width:1280,height:720}});
const errors=[];page.on('pageerror',e=>errors.push(e.message));
const probe=()=>page.evaluate(()=>window.aetherProbe);
const editProbe=()=>page.evaluate(()=>window.aetherEditProbe);
const settle=()=>expect.poll(async()=>(await editProbe())?.mesh_pending,{intervals:[5,10,20]}).toBe(0);
const samples=[];
const stages=[];
try {
 await page.goto((process.env.AETHER_TEST_URL||'http://127.0.0.1:4173')+'/?probe=1');
 await expect.poll(async()=>(await probe())?.phase,{timeout:60000}).toBe('Menu');
 await page.keyboard.press('Enter');await expect.poll(async()=>(await probe()).phase).toBe('Playing');await settle();
 await page.keyboard.press('Tab');await expect.poll(async()=>(await probe()).phase).toBe('Editing');await page.waitForTimeout(500);
 const count=(await probe()).cells;
 for(let i=0;i<23;i++){
  const point=(await probe()).edit_point;
  const start=performance.now();await page.mouse.click(...point);
  await expect.poll(async()=>(await editProbe()).cells,{intervals:[5,10,20]}).toBe(count+1);await settle();
  const ready=performance.now()-start;
  // Read the edited region, avoiding whole-frame PNG compression in the timed path.
  await page.screenshot({clip:{x:Math.max(0,Math.min(1216,point[0]-32)),y:Math.max(0,Math.min(656,point[1]-32)),width:64,height:64}});
  const elapsed=performance.now()-start;
  if(i>=3){samples.push(elapsed);stages.push({validated_and_mesh_ready_ms:ready,readback_and_png_ms:elapsed-ready});}
  if(i===22)await page.screenshot({path:'../docs/evidence/edit-latency.png'});
  await page.keyboard.press('Control+z');await expect.poll(async()=>(await editProbe()).cells,{intervals:[5,10,20]}).toBe(count);await settle();
 }
 expect(errors).toEqual([]);
 const sorted=[...samples].sort((a,b)=>a-b),pct=p=>sorted[Math.min(sorted.length-1,Math.floor(sorted.length*p))];
 const wasm=await(await fetch((process.env.AETHER_TEST_URL||'http://127.0.0.1:4173')+'/build/aether_game_bg.wasm')).arrayBuffer();
 const report={channel,samples_ms:samples,stages,p50_p95_p99_ms:[pct(.5),pct(.95),pct(.99)],warmup_edits:3,method:'Input dispatch → validated cell + mesh queue complete → 64×64 PNG render readback around edited point. Includes polling, browser IPC, screenshot encoding; upper bound, not photon latency or GPU upload time. Full screenshot is outside timing.',wasm_sha256:crypto.createHash('sha256').update(new Uint8Array(wasm)).digest('hex'),errors};
 await fs.writeFile('../docs/evidence/edit-latency.json',JSON.stringify(report,null,2));console.log(JSON.stringify(report));
} finally {await browser.close();}
