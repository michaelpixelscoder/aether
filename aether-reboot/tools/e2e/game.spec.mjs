import {test,expect} from '@playwright/test';
import fs from 'node:fs/promises';
const snapshot=page=>page.evaluate(()=>window.aetherProbe);

test('traversée au clavier avec la physique de production',async({page})=>{
  const errors=await ready(page);await page.keyboard.press('Enter');await phase(page,'Playing');
  await expect.poll(async()=>(await snapshot(page)).mesh_pending).toBe(0);
  await page.keyboard.press('f');
  await expect.poll(async()=>(await snapshot(page)).docked).toBe(false);
  // The expedition hull is 16.5 m long. Approach the west side parallel to
  // the jetty rather than driving its bow into the centre of the platform.
  const route=[[5,14,-65],[28,18,-115],[55,20,-132],[57,20,-154]];let waypoint=0;const held=new Set();const trace=[];
  async function setKey(key,value){if(value&&!held.has(key)){await page.keyboard.down(key);held.add(key);}else if(!value&&held.has(key)){await page.keyboard.up(key);held.delete(key);}}
  const started=Date.now();
  while(Date.now()-started<85000){
    const s=await snapshot(page);if(s.progress===4)break;
    const n=s.navigation;if(!n){await page.waitForTimeout(200);continue;}
    const p=n.position;let goal=route[waypoint];const distance=Math.hypot(p[0]-65,p[1]-20,p[2]+154);
    const speed=Math.hypot(...n.velocity);
    const reached=Math.hypot(p[0]-goal[0],p[2]-goal[2])<(waypoint===2?2:14);
    if(waypoint<3&&reached&&(waypoint!==2||speed<1.5)){waypoint++;goal=route[waypoint];}
    const yaw=Math.atan2(-(goal[0]-p[0]-n.velocity[0]*1.5),-(goal[2]-p[2]-n.velocity[2]*1.5));const error=((yaw-n.yaw+Math.PI)%(2*Math.PI)+2*Math.PI)%(2*Math.PI)-Math.PI;
    const turn=error*2-n.angular_y*1.5;
    await setKey('ArrowLeft',turn>0.13);await setKey('ArrowRight',turn<-.13);
    const segment=Math.hypot(p[0]-goal[0],p[2]-goal[2]);
    const desired=Math.min(11,segment*.3);
    await setKey('ArrowUp',speed<desired+.3&&Math.abs(error)<(segment<30?.2:.7));
    await setKey('e',goal[1]-n.target_altitude>.4);await setKey('c',goal[1]-n.target_altitude<-.4);await setKey('s',speed>desired+.25||segment<1.8||(segment<30&&Math.abs(error)>.25));
    if(distance<9&&Math.hypot(...n.velocity)<3.5)await page.keyboard.press('f');
    trace.push({seconds:(Date.now()-started)/1000,...n,speed,fuel:s.fuel,distance,waypoint,segment});
    await page.waitForTimeout(200);
  }
  for(const key of held)await page.keyboard.up(key);
  await fs.writeFile('../docs/evidence/browser-crossing.json',JSON.stringify(trace,null,2));
  expect((await snapshot(page)).progress).toBe(4);expect((await snapshot(page)).docked).toBe(true);
  await page.screenshot({path:'../docs/evidence/web-crossing.png'});expect(errors).toEqual([]);
});

test('scène de performance 10k et 32 corps à 1080p',async({page})=>{
  await page.setViewportSize({width:1920,height:1080});
  const errors=[];page.on('pageerror',e=>errors.push(e.message));page.on('console',m=>{if(m.type()==='error')errors.push(m.text());});
  await page.goto('./?bench=1');
  await expect.poll(async()=>(await snapshot(page))?.qa_finished,{timeout:75000}).toBe(true);
  const s=await snapshot(page);await fs.writeFile('../docs/evidence/web-benchmark.json',JSON.stringify(s,null,2));
  await page.screenshot({path:'../docs/evidence/web-benchmark.png'});
  expect(s.qa_failure).toBeNull();expect(s.cells).toBe(10000);expect(s.tick).toBeGreaterThan(1500);expect(s.mesh_pending).toBe(0);expect(errors).toEqual([]);
});
async function phase(page, expected){await expect.poll(async()=>(await snapshot(page))?.phase,{timeout:expected==='Menu'?60000:20000}).toBe(expected);}
async function click(page,action){
  await expect.poll(async()=>!!(await snapshot(page))?.ui_targets?.find(t=>t.action===action&&t.width>0)).toBe(true);
  const target=(await snapshot(page)).ui_targets.find(t=>t.action===action);
  await page.mouse.click(target.x,target.y);
}
async function ready(page,query='?probe=1'){
  const errors=[];
  page.on('pageerror',e=>errors.push(e.message));
  page.on('console',m=>{if(m.type()==='error')errors.push(m.text());});
  await page.goto('./'+query);
  await phase(page,'Menu');
  await expect(page.locator('#loading-screen')).toBeHidden();
  return errors;
}
test('parcours réel : clavier, édition souris, disque navigateur, export et menus',async({page})=>{
  const errors=await ready(page);
  await page.screenshot({path:'../docs/evidence/web-menu.png'});
  await page.keyboard.press('Enter'); await phase(page,'Playing');
  const original=(await snapshot(page)).cells;
  await page.keyboard.press('Tab'); await phase(page,'Editing');
  await expect.poll(async()=>(await snapshot(page)).mesh_pending).toBe(0);
  await page.waitForTimeout(400);
  const p=(await snapshot(page)).edit_point;
  await page.mouse.click(p[0],p[1]);
  await expect.poll(async()=>(await snapshot(page)).cells).toBe(original+1);
  await page.keyboard.press('Control+z');
  await expect.poll(async()=>(await snapshot(page)).cells).toBe(original);
  await page.keyboard.press('Control+y');
  await expect.poll(async()=>(await snapshot(page)).cells).toBe(original+1);
  // Clicking interface controls must not also place a voxel in the scene.
  await click(page,'SelectBlock(Metal)');
  expect((await snapshot(page)).cells).toBe(original+1);
  await page.keyboard.press('Control+s');
  await expect.poll(async()=>(await snapshot(page)).notice).toContain('enregistrée');
  const downloadPromise=page.waitForEvent('download');
  await click(page,'Export');
  const download=await downloadPromise;
  await download.saveAs('../docs/evidence/exported-blueprint.json');
  const exported=JSON.parse(await fs.readFile('../docs/evidence/exported-blueprint.json','utf8'));
  expect(exported.blueprint.cells.length).toBe(original+1);
  await page.screenshot({path:'../docs/evidence/web-atelier.png'});
  await page.reload(); await phase(page,'Menu');
  await page.keyboard.press('KeyL'); await phase(page,'Playing');
  expect((await snapshot(page)).cells).toBe(original+1);
  await page.keyboard.press('Tab'); await phase(page,'Editing');
  const chooserPromise=page.waitForEvent('filechooser'); await click(page,'Import');
  const chooser=await chooserPromise; await chooser.setFiles({name:'invalide.json',mimeType:'application/json',buffer:Buffer.from('{"version":999}')});
  await expect.poll(async()=>(await snapshot(page)).notice).toContain('impossible');
  expect((await snapshot(page)).cells).toBe(original+1);
  const importPromise=page.waitForEvent('filechooser'); await click(page,'Import');
  await (await importPromise).setFiles('../docs/evidence/exported-blueprint.json');
  await expect.poll(async()=>(await snapshot(page)).notice).toContain('importée');
  await page.keyboard.press('Escape'); await phase(page,'Paused');
  const tick=(await snapshot(page)).tick; await page.waitForTimeout(600);
  expect((await snapshot(page)).tick).toBe(tick);
  await click(page,'Settings');await phase(page,'Settings');await click(page,'Pause');await phase(page,'Paused');
  await page.keyboard.press('Escape');await phase(page,'Editing');await page.keyboard.press('Escape');await phase(page,'Paused');
  await click(page,'Menu'); await phase(page,'Menu');
  await click(page,'Settings'); await phase(page,'Settings');
  await click(page,'Volume'); await click(page,'Motion'); await click(page,'Pause'); await phase(page,'Menu');
  await page.setViewportSize({width:960,height:640}); await page.keyboard.press('Enter'); await phase(page,'Playing');
  await page.keyboard.press('Tab'); await phase(page,'Editing'); await page.waitForTimeout(500);
  for(const button of (await snapshot(page)).ui_targets){expect(button.x+button.width/2).toBeLessThanOrEqual(961);expect(button.y+button.height/2).toBeLessThanOrEqual(641);}
  await page.screenshot({path:'../docs/evidence/web-small-window.png'});
  expect(errors).toEqual([]);
});
test('scénario intégré avec IndexedDB, câble, scission et vingt reprises',async({page})=>{
  const errors=[];page.on('pageerror',e=>errors.push(e.message));page.on('console',m=>{if(m.type()==='error')errors.push(m.text());});
  await page.goto('./?qa=1');
  try {
    await page.waitForFunction(()=>window.aetherProbe?.qa_finished,{},{timeout:110000});
  } finally {
    await fs.writeFile('../docs/evidence/web-qa.json',JSON.stringify(await snapshot(page),null,2));
  }
  const report=await snapshot(page);
  await fs.writeFile('../docs/evidence/web-qa.json',JSON.stringify(report,null,2));
  expect(report.qa_failure).toBeNull();expect(report.entity_samples_20_cycles).toHaveLength(20);
  expect(errors).toEqual([]);
});
test('capacité GPU absente : erreur explicite et bouton de reprise',async({page})=>{
  await page.addInitScript(()=>Object.defineProperty(navigator,'gpu',{value:undefined}));
  await page.goto('./');
  await expect(page.locator('#loading-message')).toContainText('WebGPU est indisponible');
  await expect(page.getByRole('button',{name:'Réessayer'})).toBeVisible();
  await page.screenshot({path:'../docs/evidence/web-no-gpu.png'});
});
test('asset critique absent : erreur récupérable puis rechargement',async({page})=>{
  test.setTimeout(180000);
  const attempts=[];
  // Inspect the actual served candidate, including portable validation runners
  // which intentionally have no second copy of the canonical asset tree.
  const modelResponse=await page.request.get('./assets/world/dawn.glb');
  expect(modelResponse.ok()).toBe(true);
  const glb=await modelResponse.body();
  const model=JSON.parse(glb.subarray(20,20+glb.readUInt32LE(12)));
  const normal=model.images.find(image=>image.name==='dressed-masonry-r42-normal');
  expect(normal?.uri).toBeTruthy();
  try {
   // Cedar is a direct barrier; masonry is discovered through glTF materials.
   for(const texture of ['textures/cedar.png','world/'+normal.uri]){
    let rejectedRequests=0;
    const routePattern='**/assets/'+texture;
    await page.route(routePattern,route=>{rejectedRequests++;return route.abort();});
    await page.goto('./?probe=1'); await phase(page,'Error');
    const initialRequests=rejectedRequests;
    await click(page,'Retry');
    // A genuinely new failure must remain visible, even when the old failure
    // is fenced out while AssetServer starts its asynchronous retry.
    await expect.poll(()=>rejectedRequests).toBeGreaterThan(initialRequests);
    await phase(page,'Error');
    await page.unroute(routePattern);
    await click(page,'Retry'); await phase(page,'Menu');
    attempts.push({texture,rejected_requests:rejectedRequests,phase:(await snapshot(page)).phase});
   }
  } finally {
    await fs.writeFile('../docs/evidence/asset-retry.json',JSON.stringify({attempts,last:await snapshot(page)},null,2));
  }
});
test('stockage refusé : pas de faux succès, export toujours possible',async({page})=>{
  await page.addInitScript(()=>Object.defineProperty(window,'indexedDB',{value:{open(){throw new DOMException('Quota de test','QuotaExceededError');}}}));
  await ready(page);await page.keyboard.press('Enter');await phase(page,'Playing');
  await page.keyboard.press('Control+s');
  await expect.poll(async()=>(await snapshot(page)).notice).toContain('impossible');
  expect((await snapshot(page)).docked).toBe(true);
});

test('personnage absent : représentation visible et marche restent utilisables',async({page})=>{
  const errors=[];page.on('pageerror',e=>errors.push(e.message));
  await page.route('**/assets/characters/Knight.glb',route=>route.abort());
  await page.goto('./?probe=1');await phase(page,'Menu');await page.keyboard.press('Enter');await phase(page,'Playing');
  await expect.poll(async()=>(await snapshot(page)).mesh_pending).toBe(0);
  await expect.poll(async()=>(await snapshot(page)).avatar_fallbacks).toBeGreaterThan(0);
  await page.keyboard.press('t');await expect.poll(async()=>(await snapshot(page)).walker).toBe(true);
  await page.keyboard.press('Space');await page.waitForTimeout(700);
  await page.keyboard.press('Control+s');await expect.poll(async()=>(await snapshot(page)).notice).toContain('enregistrée');
  expect((await snapshot(page)).navigation.position.every(Number.isFinite)).toBe(true);
  expect(errors).toEqual([]);await page.screenshot({path:'../docs/evidence/web-character-fallback.png'});
});

test('backup IndexedDB et transaction interrompue conservent une partie valide',async({page})=>{
  const errors=await ready(page);await page.keyboard.press('Enter');await phase(page,'Playing');
  const cells=(await snapshot(page)).cells;
  await page.keyboard.press('Control+s');await expect.poll(async()=>(await snapshot(page)).notice).toContain('enregistrée');
  await page.keyboard.press('Tab');await phase(page,'Editing');await page.waitForTimeout(500);
  const p=(await snapshot(page)).edit_point;await page.mouse.click(p[0],p[1]);
  await expect.poll(async()=>(await snapshot(page)).cells).toBe(cells+1);
  await page.keyboard.press('Control+s');await expect.poll(async()=>(await snapshot(page)).notice).toContain('enregistrée');
  await page.evaluate(()=>new Promise((resolve,reject)=>{
    const r=indexedDB.open('aether-isles-reboot',1);r.onerror=()=>reject(r.error);r.onsuccess=()=>{
      const db=r.result,t=db.transaction('saves','readwrite');t.objectStore('saves').put('{tronqué','session');
      t.oncomplete=()=>{db.close();resolve();};t.onabort=()=>reject(t.error);
    };
  }));
  await page.keyboard.press('Control+o');await phase(page,'Playing');
  await expect.poll(async()=>(await snapshot(page)).cells).toBe(cells);
  await expect.poll(async()=>(await snapshot(page)).notice).toContain('secours');
  await page.evaluate(()=>{
    const original=IDBDatabase.prototype.transaction;
    IDBDatabase.prototype.transaction=function(...args){const tx=original.apply(this,args);if(args[1]==='readwrite')queueMicrotask(()=>tx.abort());return tx;};
  });
  await page.keyboard.press('Control+s');await expect.poll(async()=>(await snapshot(page)).notice).toContain('impossible');
  await page.reload();await phase(page,'Menu');await page.keyboard.press('l');await phase(page,'Playing');
  await expect.poll(async()=>(await snapshot(page)).cells).toBe(cells);expect(errors).toEqual([]);
});
