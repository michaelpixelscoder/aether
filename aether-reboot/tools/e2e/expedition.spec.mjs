import {test,expect} from '@playwright/test';
import fs from 'node:fs/promises';
const snapshot=page=>page.evaluate(()=>window.aetherProbe);
async function phase(page,name){await expect.poll(async()=>(await snapshot(page))?.phase,{timeout:name==='Menu'?60000:20000}).toBe(name);}
async function action(page,name){await expect.poll(async()=>!!(await snapshot(page))?.ui_targets?.find(t=>t.action===name&&t.width>0)).toBe(true);const t=(await snapshot(page)).ui_targets.find(t=>t.action===name);await page.mouse.click(t.x,t.y);await page.waitForTimeout(250);}
async function save(page){
 // Hold the real chord through input frames, then fence its key release before
 // another gameplay command. An old notice is not a new save acknowledgement.
 const requestedAt=(await snapshot(page)).tick;
 await page.keyboard.press('Control+s',{delay:80});
 const releasedAt=(await snapshot(page)).tick;
 await expect.poll(async()=>(await snapshot(page)).tick).toBeGreaterThan(releasedAt);
 await expect.poll(async()=>(await snapshot(page)).pending_storage).toBe(false);
 await expect.poll(async()=>(await snapshot(page)).notice).toContain('enregistrée');
 const storedTick=()=>page.evaluate(()=>new Promise((resolve,reject)=>{
  const request=indexedDB.open('aether-isles-reboot',1);
  request.onerror=()=>reject(request.error);
  request.onsuccess=()=>{
   const db=request.result,tx=db.transaction('saves','readonly');let tick=-1;
   const get=tx.objectStore('saves').get('session');
   get.onsuccess=()=>{try{tick=JSON.parse(get.result).tick;}catch(error){reject(error);}};
   tx.oncomplete=()=>{db.close();resolve(tick);};
   tx.onabort=()=>{db.close();reject(tx.error);};
  };
 }));
 await expect.poll(storedTick).toBeGreaterThanOrEqual(requestedAt);
 return {requested_tick:requestedAt,committed_tick:await storedTick(),key_release_tick:releasedAt};
}
// The probe is read-only. All navigation, commerce and portal activation use
// the same keyboard and buttons as a player; no injected save/pose state.
async function navigate(page,target,range=8,timeout=115000){
 const held=new Set(),trace=[],start=Date.now();let arrived=false;
 async function set(k,v){if(v&&!held.has(k)){held.add(k);await page.keyboard.down(k);}else if(!v&&held.has(k)){held.delete(k);await page.keyboard.up(k);}}
 try{while(Date.now()-start<timeout){
  const s=await snapshot(page),n=s.navigation;if(!n){await page.waitForTimeout(150);continue;}
  const p=n.position,v=n.velocity,dx=target[0]-p[0],dz=target[2]-p[2],distance=Math.hypot(dx,dz);
  const error=((Math.atan2(-dx,-dz)-n.yaw+Math.PI)%(2*Math.PI)+2*Math.PI)%(2*Math.PI)-Math.PI;
  const turn=error*2-n.angular_y*1.6,speed=Math.hypot(v[0],v[2]),desired=Math.min(13,distance*.4);
  await set('ArrowLeft',turn>.08);await set('ArrowRight',turn<-.08);await set('ArrowUp',speed<desired+.3&&Math.abs(error)<.7);await set('s',speed>desired+1||distance<range);
  await set('e',target[1]-n.target_altitude>.4);await set('c',target[1]-n.target_altitude<-.4);
  trace.push({seconds:(Date.now()-start)/1000,...n,distance});
  if(distance<range&&Math.abs(p[1]-target[1])<4&&speed<3.3){arrived=true;break;}
  await page.waitForTimeout(150);
 }}finally{for(const k of held)await page.keyboard.up(k);}
 expect(arrived,JSON.stringify(trace.at(-1))).toBe(true);return trace;
}
test('expédition réelle : capitale, cargaison, contrat, portail et reprise',async({page})=>{
 test.setTimeout(240000);await page.setViewportSize({width:1672,height:941});
 const errors=[];page.on('pageerror',e=>errors.push(e.message));page.on('console',m=>{if(m.type()==='error')errors.push(m.text());});
 const report={started_at:new Date().toISOString(),errors,finished:false};
 await page.addInitScript(()=>{
  window.expeditionInputs=[];
  for(const type of ['keydown','keyup','focus','blur'])window.addEventListener(type,e=>{
   window.expeditionInputs.push({type,key:e.key,code:e.code,ctrl:e.ctrlKey,at:performance.now(),focus:document.activeElement?.tagName});
   if(window.expeditionInputs.length>4000)window.expeditionInputs.shift();
  },true);
 });
 try {
 await page.goto('./?probe=1');await phase(page,'Menu');await page.keyboard.press('Enter');await phase(page,'Playing');
 await expect.poll(async()=>(await snapshot(page)).mesh_pending).toBe(0);
 await page.keyboard.press('f',{delay:80});await expect.poll(async()=>(await snapshot(page)).docked).toBe(false);
 report.outward=await navigate(page,[-260,47.3,-501.5]);await page.keyboard.press('f',{delay:80});await expect.poll(async()=>(await snapshot(page)).checkpoint).toBe(100);
 expect((await snapshot(page)).expedition.visited).toContain(100);
 await expect.poll(async()=>(await snapshot(page)).pending_storage).toBe(false);
 await page.keyboard.press('m');await phase(page,'Atlas');const initial=(await snapshot(page)).expedition;
 for(let n=0;n<4;n++)await action(page,'Trade(Crystal, true)');
 await expect.poll(async()=>(await snapshot(page)).expedition.cargo[2]).toBe(initial.cargo[2]+4);
 await action(page,'Deliver');await expect.poll(async()=>(await snapshot(page)).expedition.contracts).toContain(100);
 const delivered=(await snapshot(page)).expedition;expect(delivered.cargo[2]).toBe(0);
 await action(page,'Deliver');expect((await snapshot(page)).expedition.credits).toBe(delivered.credits);
 await action(page,'Destination(0)');await page.screenshot({path:'../docs/evidence/web-expedition-atlas.png'});
 await page.keyboard.press('m',{delay:80});await phase(page,'Playing');report.capital_save=await save(page);
 report.before_departure=await snapshot(page);
 await page.keyboard.press('f',{delay:80});await expect.poll(async()=>(await snapshot(page)).docked).toBe(false);
 report.portal=await navigate(page,[-260,70.2,-566.6],12,65000);const before=(await snapshot(page)).fuel;
 await page.keyboard.press('b',{delay:80});await expect.poll(async()=>(await snapshot(page)).notice).toContain('Passage vers');
 const passed=await snapshot(page);expect(passed.fuel).toBeLessThan(before-79);expect(Math.hypot(passed.navigation.position[0],passed.navigation.position[2]-45)).toBeLessThan(3);
 report.passed=passed;report.portal_save=await save(page);
 await page.reload();await phase(page,'Menu');await page.keyboard.press('l');await phase(page,'Playing');
 const loaded=await snapshot(page);expect(loaded.expedition.contracts).toContain(100);expect(loaded.expedition.destination).toBe(0);expect(loaded.expedition.cargo[2]).toBe(0);
 report.loaded=loaded;report.finished=true;
 expect(errors).toEqual([]);
 } finally {
  report.last=await snapshot(page).catch(()=>null);
  report.inputs=await page.evaluate(()=>window.expeditionInputs).catch(()=>[]);
  await fs.writeFile('../docs/evidence/browser-expedition.json',JSON.stringify(report,null,2));
 }
});
