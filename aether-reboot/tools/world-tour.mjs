import {chromium,expect} from '@playwright/test';
import fs from 'node:fs/promises';
import crypto from 'node:crypto';
const catalog=JSON.parse(await fs.readFile('docs/evidence/world-catalog.json','utf8'));
const browser=await chromium.launch({channel:'chrome',headless:true,args:['--use-angle=d3d11','--enable-unsafe-webgpu']});
const page=await browser.newPage({viewport:{width:1672,height:941}}),held=new Set();
const report={started:new Date().toISOString(),errors:[],trace:[],completed:[]};
await page.addInitScript(()=>{
 window.inputAudit=[];
 for(const type of ['keydown','keyup','focus','blur'])window.addEventListener(type,e=>{
  window.inputAudit.push({type,code:e.code,key:e.key,at:performance.now(),focus:document.activeElement?.tagName});
  if(window.inputAudit.length>3000)window.inputAudit.shift();
 },true);
});
report.wasm_sha256=crypto.createHash('sha256').update(await fs.readFile(process.env.AETHER_WASM||'target/bevy_web/web/aether_game/build/aether_game_bg.wasm')).digest('hex');
page.on('pageerror',e=>report.errors.push(e.message));page.on('console',m=>{if(m.type()==='error')report.errors.push(m.text());});
const snap=()=>page.evaluate(()=>window.aetherProbe);
// Separate successive toggles across real input frames. Two complete F taps
// within one winit frame collapse into a single ButtonInput::just_pressed.
async function tap(key){
 await page.keyboard.press(key,{delay:80});
 const released=await snap();
 if(released?.phase==='Playing')await expect.poll(async()=>(await snap()).tick,{timeout:10000}).toBeGreaterThan(released.tick);
 else await page.waitForTimeout(80);
}
async function save(){
 const requested=(await snap()).tick;await tap('Control+s');
 await expect.poll(async()=>(await snap()).pending_storage).toBe(false);
 await expect.poll(async()=>(await snap()).notice).toContain('enregistrée');
 const stored=()=>page.evaluate(()=>new Promise((resolve,reject)=>{
  const request=indexedDB.open('aether-isles-reboot',1);request.onerror=()=>reject(request.error);
  request.onsuccess=()=>{const db=request.result,tx=db.transaction('saves','readonly');let tick=-1;
   const get=tx.objectStore('saves').get('session');get.onsuccess=()=>{try{tick=JSON.parse(get.result).tick;}catch(e){reject(e);}};
   tx.oncomplete=()=>{db.close();resolve(tick);};tx.onabort=()=>{db.close();reject(tx.error);};};
 }));
 await expect.poll(stored).toBeGreaterThanOrEqual(requested);
 report.save={requested_tick:requested,committed_tick:await stored()};
}
async function key(k,on){if(on&&!held.has(k)){await page.keyboard.down(k);held.add(k);}else if(!on&&held.has(k)){await page.keyboard.up(k);held.delete(k);}}
async function release(){for(const k of [...held])await key(k,false);}
async function navigate(points,name,{stop=true,timeout=240000,range=9,speed=18,corner_range=28,vertical_range=4}={}){
 const start=Date.now();let waypoint=0,done=false,lastRecord=0,lastWritten=0,lastInputSeen=Date.now();
 try{while(Date.now()-start<timeout){
  const s=await snap(),n=s.navigation,p=n.position,v=n.velocity;let goal=points[waypoint];
  let dx=goal[0]-p[0],dz=goal[2]-p[2],distance=Math.hypot(dx,dz),vertical=Math.abs(goal[1]-p[1]);
  if(waypoint<points.length-1&&distance<corner_range&&vertical<25){waypoint++;continue;}
  const final=waypoint===points.length-1;
  const error=((Math.atan2(-dx,-dz)-n.yaw+Math.PI)%(2*Math.PI)+2*Math.PI)%(2*Math.PI)-Math.PI;
  const turn=error*2-n.angular_y*1.6,velocity=Math.hypot(v[0],v[2]);
  const desired=final&&stop?Math.min(speed,distance*.35):speed;
  await key('ArrowLeft',distance>range&&turn>.09);await key('ArrowRight',distance>range&&turn<-.09);
  await key('ArrowUp',distance>range&&velocity<desired+.3&&Math.abs(error)<.75);
  await key('s',velocity>desired+1||(distance<range)||(Math.abs(error)>1.0&&velocity>4));
  await key('e',goal[1]-n.target_altitude>.4);await key('c',goal[1]-n.target_altitude<-.4);
  if(Date.now()-lastRecord>750){report.trace.push({leg:name,seconds:(Date.now()-start)/1000,waypoint,...n,distance,vertical,fuel:s.fuel,held:[...held],physical:s.physical_keys,focus:s.window_focused,ui:s.keyboard_ui});lastRecord=Date.now();}
  if(!held.size||s.physical_keys?.length)lastInputSeen=Date.now();
  if(Date.now()-lastInputSeen>3000)throw Error('Held keys missing from Bevy: '+JSON.stringify(report.trace.at(-1)));
  if(Date.now()-lastWritten>8000){await fs.writeFile('docs/evidence/world-tour.json',JSON.stringify(report,null,2));lastWritten=Date.now();}
  // Finish collection approaches after vertical braking as well: crossing the
  // altitude window at -7 m/s can leave the visible resource outside reach at B.
  if(final&&distance<range&&vertical<vertical_range&&(!stop||Math.hypot(...v)<3.4)){done=true;break;}
  await page.waitForTimeout(150);
 }}finally{await release();await fs.writeFile('docs/evidence/world-tour.json',JSON.stringify(report,null,2));}
 expect(done,`${name}: ${JSON.stringify(report.trace.at(-1))}`).toBe(true);report.completed.push(name);console.log(name,(await snap()).navigation);
}
try {
 await page.goto((process.env.AETHER_TEST_URL||'http://127.0.0.1:4180/')+'?probe=1');await page.waitForFunction(()=>window.aetherProbe?.phase==='Menu',null,{timeout:90000});
 const servedWasm=await page.evaluate(()=>performance.getEntriesByType('resource').map(r=>r.name).find(u=>u.endsWith('/aether_game_bg.wasm')));
 expect(servedWasm).toBeTruthy();const response=await page.request.get(servedWasm);expect(response.ok()).toBe(true);
 report.served_wasm_sha256=crypto.createHash('sha256').update(await response.body()).digest('hex');
 expect(report.served_wasm_sha256).toBe(report.wasm_sha256);
 await tap('Enter');await page.waitForFunction(()=>window.aetherProbe?.phase==='Playing');
 await expect.poll(async()=>(await snap()).mesh_pending).toBe(0);await tap('f');await expect.poll(async()=>(await snap()).docked).toBe(false);
 await navigate([[-260,47.3,-501.5]],'Dawn harbour',{speed:13});await tap('f');await expect.poll(async()=>(await snap()).checkpoint).toBe(100);
 await expect.poll(async()=>(await snap()).pending_storage).toBe(false);
 await tap('f');await expect.poll(async()=>(await snap()).docked).toBe(false);
 await navigate([[-285.2,92.6,-544]],'Crystal harvest',{speed:9,range:3});await tap('b');await page.waitForTimeout(500);
 expect((await snap()).expedition.cargo[2]).toBe(9);report.completed.push('visible crystal harvested');
 await page.screenshot({path:'docs/evidence/world-tour-harvest.png'});
 await navigate([[-340,94,-523],[-340,65,-532]],'Water harvest',{speed:6,range:3,corner_range:6,vertical_range:2});await tap('b');await page.waitForTimeout(500);
 expect((await snap()).expedition.cargo[4]).toBe(17);report.completed.push('visible water harvested');
 await page.screenshot({path:'docs/evidence/world-tour-water.png'});
 if(process.argv.includes('--harvest-only')){report.harvest_finished=true;report.final=await snap();}
 else {
 // Join the current in motion. Its entrance is a broad flight corridor, not a
 // docking target; trying to park on its centre creates a brake/flow limit cycle.
 await navigate([[-340,160,-500],catalog.routes[3].points[0]],'Join underground current',{speed:13,range:16,stop:false});
 await navigate(catalog.routes[3].points.slice(1),'Dawn to Hollow current',{timeout:360000,speed:24,range:18});
 await page.screenshot({path:'docs/evidence/world-tour-hollow-arrival.png'});
 const hollow=catalog.islands.find(i=>i.id===800), [hx,hy,hz]=hollow.center;
 await navigate([[hx,hollow.dock[1],hz+320],hollow.dock],'Hollow harbour',{speed:10});
 await tap('f');await expect.poll(async()=>(await snap()).checkpoint).toBe(800);
 await expect.poll(async()=>(await snap()).pending_storage).toBe(false);
 await tap('f');await expect.poll(async()=>(await snap()).docked).toBe(false);
 await navigate([[hx-70,hy+7,hz+137]],'Guardian observation',{speed:7});
 await tap('b');await expect.poll(async()=>(await snap()).expedition.observed_species).toContain('Guardian');
 await navigate([[hx,hy+42,hz+140],[hx,hy+42,hz+95]],'Cavewing observation',{speed:7,range:4});
 for(let n=0;n<35&&!(await snap()).expedition.observed_species.includes('Cavewing');n++){await tap('b');await page.waitForTimeout(1000);}
 expect((await snap()).expedition.observed_species).toContain('Cavewing');
 await page.screenshot({path:'docs/evidence/world-tour-fauna.png'});
 await save();
 report.final=await snap();
 await expect.poll(async()=>(await snap()).pending_storage).toBe(false);
 report.flight_input_events=await page.evaluate(()=>window.inputAudit);
 await page.reload();await page.waitForFunction(()=>window.aetherProbe?.phase==='Menu',null,{timeout:90000});
 await tap('l');await page.waitForFunction(()=>window.aetherProbe?.phase==='Playing');
 report.reloaded=await snap();
 expect(report.reloaded.expedition.cargo).toEqual(report.final.expedition.cargo);
 expect(report.reloaded.expedition.observed_species).toEqual(report.final.expedition.observed_species);
 expect(report.reloaded.checkpoint).toBe(800);
 report.finished=true;expect(report.errors).toEqual([]);
 }
}catch(e){report.failure=String(e);report.final=await snap().catch(()=>null);process.exitCode=1;await page.screenshot({path:'docs/evidence/world-tour-failure.png'}).catch(()=>{});}
finally{await release();report.input_events=await page.evaluate(()=>window.inputAudit).catch(()=>[]);await fs.writeFile('docs/evidence/world-tour.json',JSON.stringify(report,null,2));await browser.close();}
