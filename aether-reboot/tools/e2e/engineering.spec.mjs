import {test,expect} from '@playwright/test';
import fs from 'node:fs/promises';
const snapshot=page=>page.evaluate(()=>window.aetherProbe);
const stored=page=>page.evaluate(()=>new Promise((resolve,reject)=>{
  const r=indexedDB.open('aether-isles-reboot',1);
  r.onerror=()=>reject(r.error);
  r.onsuccess=()=>{const db=r.result,tx=db.transaction('saves','readonly');let data;
    const get=tx.objectStore('saves').get('session');get.onsuccess=()=>{data=get.result;};
    tx.oncomplete=()=>{db.close();try{resolve(data?JSON.parse(data):null);}catch(e){reject(e);}};
    tx.onabort=()=>{db.close();reject(tx.error);};
  };
}));
async function phase(page,name){await expect.poll(async()=>(await snapshot(page))?.phase,{timeout:name==='Menu'?60000:20000}).toBe(name);}
async function action(page,name){
  await expect.poll(async()=>!!(await snapshot(page))?.ui_targets?.find(t=>t.action===name&&t.width>0)).toBe(true);
  const t=(await snapshot(page)).ui_targets.find(t=>t.action===name);
  await page.mouse.click(t.x,t.y);await page.waitForTimeout(250);
}
const linkAction=e=>`CircuitValve(PartId(${e.a}), PartId(${e.b}))`;
const vessel=s=>s?.vessels?.find(v=>v.id===s.active);
const design=s=>vessel(s)?.circuit?.network?.circuit;
async function save(page,predicate){
  await action(page,'Save');
  await expect.poll(async()=>(await snapshot(page)).pending_storage).toBe(false);
  await expect.poll(async()=>predicate(await stored(page))).toBe(true);
  return stored(page);
}
test('atelier réel : distribution, vannes, annulation, sauvegarde V3 et reprise',async({page})=>{
  test.setTimeout(180000);await page.setViewportSize({width:1672,height:941});
  const errors=[];page.on('pageerror',e=>errors.push(e.message));page.on('console',m=>{if(m.type()==='error')errors.push(m.text());});
  const report={started_at:new Date().toISOString(),errors,finished:false};
  try{
    await page.goto('./?probe=1');await phase(page,'Menu');await page.keyboard.press('Enter',{delay:80});await phase(page,'Playing');
    await expect.poll(async()=>(await snapshot(page)).mesh_pending).toBe(0);
    await page.keyboard.press('m',{delay:80});await phase(page,'Atlas');await action(page,'Engineering');await action(page,'CircuitEnable');
    const initial=await save(page,s=>s?.version===3&&design(s)?.links?.length>0);
    report.initial=initial;
    expect(vessel(initial).blueprint.circuit_design).toEqual(design(initial));
    const engines=vessel(initial).blueprint.parts.filter(p=>p.kind==='Propeller').map(p=>p.id);
    expect(engines.length).toBeGreaterThan(0);
    const engineLinks=design(initial).links.filter(e=>engines.includes(e.a)||engines.includes(e.b));
    expect(engineLinks.length).toBe(engines.length);
    const first=engineLinks[0];await action(page,linkAction(first));
    report.closed=await save(page,s=>design(s)?.links.some(e=>e.a===first.a&&e.b===first.b&&!e.open));
    await action(page,'Undo');report.undo=await save(page,s=>design(s)?.links.some(e=>e.a===first.a&&e.b===first.b&&e.open));
    expect(vessel(report.undo).circuit.network.contents).toEqual(vessel(initial).circuit.network.contents);
    await action(page,'Undo');
    report.uninstalled=await save(page,s=>!!vessel(s)&&!vessel(s).circuit&&!vessel(s).blueprint.circuit_design);
    expect(vessel(report.uninstalled).fuel).toBeLessThanOrEqual(vessel(initial).fuel);
    await action(page,'Redo');
    report.reinstalled=await save(page,s=>design(s)?.links.some(e=>e.a===first.a&&e.b===first.b&&e.open));
    expect(vessel(report.reinstalled).circuit.network.contents).toEqual(vessel(initial).circuit.network.contents);
    await action(page,'Redo');report.redo=await save(page,s=>design(s)?.links.some(e=>e.a===first.a&&e.b===first.b&&!e.open));
    for(const e of engineLinks.slice(1))await action(page,linkAction(e));
    report.saved=await save(page,s=>design(s)?.links.filter(e=>engines.includes(e.a)||engines.includes(e.b)).every(e=>!e.open));
    await page.screenshot({path:'../docs/evidence/web-engineering-workshop.png'});
    await page.reload();await phase(page,'Menu');await page.keyboard.press('l',{delay:80});await phase(page,'Playing');
    await page.keyboard.press('m',{delay:80});await phase(page,'Atlas');await action(page,'Engineering');
    report.reloaded=await save(page,s=>JSON.stringify(design(s))===JSON.stringify(design(report.saved)));
    expect(vessel(report.reloaded).blueprint).toEqual(vessel(report.saved).blueprint);
    expect(design(report.reloaded)).toEqual(design(report.saved));
    expect(errors).toEqual([]);report.finished=true;
    await page.screenshot({path:'../docs/evidence/web-engineering-reloaded.png'});
  }finally{report.last=await snapshot(page).catch(()=>null);await fs.writeFile('../docs/evidence/web-engineering.json',JSON.stringify(report,null,2));}
});
