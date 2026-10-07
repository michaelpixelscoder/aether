import {test,expect} from '@playwright/test';
import fs from 'node:fs/promises';
const snapshot=page=>page.evaluate(()=>window.aetherProbe);
const phase=(page,value)=>expect.poll(async()=>(await snapshot(page))?.phase,{timeout:60000}).toBe(value);
async function click(page,action){await expect.poll(async()=>(await snapshot(page))?.ui_targets.some(t=>t.action===action)).toBe(true);const t=(await snapshot(page)).ui_targets.find(t=>t.action===action);await page.mouse.click(t.x,t.y);}
async function ready(page){await page.goto('/?probe=1');await phase(page,'Menu');await page.keyboard.press('Enter');await phase(page,'Playing');await expect.poll(async()=>(await snapshot(page)).mesh_pending).toBe(0);}
async function travel(page){await page.keyboard.press('Escape');await phase(page,'Paused');await click(page,'Travel');await phase(page,'Travel');}
async function tab(page,category){const action=`TravelTab(${category})`;await click(page,action);await expect.poll(async()=>(await snapshot(page)).ui_targets.find(t=>t.action===action)?.selected).toBe(true);}

test('préférences stables : même ligne sélectionnée et mêmes boutons après attribution',async({page})=>{
  await ready(page);await page.keyboard.press('Escape');await phase(page,'Paused');await click(page,'Settings');await phase(page,'Settings');
  const rows=(await snapshot(page)).ui_targets.filter(t=>t.action.startsWith('Rebind('));
  await click(page,'Rebind(Ascend)');
  await expect.poll(async()=>(await snapshot(page)).binding_active).toBe('Ascend');
  expect((await snapshot(page)).ui_targets.find(t=>t.action==='Rebind(Ascend)').selected).toBe(true);
  await page.keyboard.press('Shift');
  await expect.poll(async()=>(await snapshot(page)).bindings.Ascend).toBe('Shift');
  expect((await snapshot(page)).binding_selected).toBe('Ascend');
  expect((await snapshot(page)).ui_targets.filter(t=>t.action.startsWith('Rebind(')).map(t=>({...t,selected:false}))).toEqual(rows.map(t=>({...t,selected:false})));
  expect((await snapshot(page)).ui_targets.find(t=>t.action==='Rebind(Ascend)').selected).toBe(true);
  await click(page,'Rebind(Forward)');await page.keyboard.press('q');
  await expect.poll(async()=>(await snapshot(page)).binding_message).toContain('déjà affectée');
  expect((await snapshot(page)).ui_targets.filter(t=>t.action.startsWith('Rebind(')).map(t=>({...t,selected:false}))).toEqual(rows.map(t=>({...t,selected:false})));
  await page.keyboard.press('Enter');await expect.poll(async()=>(await snapshot(page)).bindings.Forward?.Letter).toBe('q');
  expect((await snapshot(page)).ui_targets.find(t=>t.action==='Rebind(Forward)').selected).toBe(true);
  await page.screenshot({path:'../docs/evidence/web-bindings-stable-r75.png'});
});

test('voyage rapide : toutes les îles, cavernes, courants et reprise persistante',async({page})=>{
  test.setTimeout(240000);
  const errors=[];page.on('pageerror',error=>errors.push(error.message));
  await ready(page);const initial=await snapshot(page);await travel(page);
  await tab(page,'Islands');
  await expect.poll(async()=>(await snapshot(page)).ui_targets.some(t=>t.action==='FastTravel(0)')).toBe(true);
  const seen=new Set();let last;
  for(let pageIndex=0;pageIndex<40;pageIndex++){
    const s=await snapshot(page);const targets=s.ui_targets.filter(t=>t.action.startsWith('FastTravel('));expect(targets.length).toBeGreaterThan(0);
    for(const t of targets){seen.add(Number(t.action.match(/\d+/)[0]));last=t.action;}
    expect(s.ui_targets.every(t=>t.x-t.width/2>=-1&&t.x+t.width/2<=1281&&t.y-t.height/2>=-1&&t.y+t.height/2<=721)).toBe(true);
    if(!s.ui_targets.some(t=>t.action==='TravelPage(true)'))break;
    const first=targets[0].action;await click(page,'TravelPage(true)');
    await expect.poll(async()=>(await snapshot(page)).ui_targets.find(t=>t.action.startsWith('FastTravel('))?.action).not.toBe(first);
  }
  const catalog=(await snapshot(page)).travel_destinations;
  const islandIds=catalog.filter(d=>d.id<10000).map(d=>d.id).sort((a,b)=>a-b);
  expect([...seen].sort((a,b)=>a-b)).toEqual(islandIds);expect(seen.size).toBeGreaterThan(200);
  await page.screenshot({path:'../docs/evidence/web-travel-islands-r75.png'});
  await click(page,last);await phase(page,'Playing');
  await expect.poll(async()=>(await snapshot(page)).pending_storage).toBe(false);
  const lastDestination=catalog.find(d=>`FastTravel(${d.id})`===last);
  expect(Math.hypot(...(await snapshot(page)).navigation.position.map((p,i)=>p-lastDestination.position[i]))).toBeLessThan(220);
  expect((await snapshot(page)).cells).toBe(initial.cells);
  expect((await snapshot(page)).expedition.credits).toBeGreaterThanOrEqual(initial.expedition.credits);
  expect((await snapshot(page)).expedition.cargo).toEqual(initial.expedition.cargo);
  await page.keyboard.press('t');await expect.poll(async()=>(await snapshot(page)).walker).toBe(true);
  await travel(page);await tab(page,'Underground');
  await expect.poll(async()=>(await snapshot(page)).ui_targets.some(t=>t.action.startsWith('FastTravel('))).toBe(true);
  // The category starts with all underground islands. Traverse its pages to
  // reach the authored cavity entry without invoking an action by injection.
  const vault=catalog.find(d=>d.label.startsWith('Grande cavité'));
  for(let i=0;i<40&&!((await snapshot(page)).ui_targets.some(t=>t.action===`FastTravel(${vault.id})`));i++){
    const first=(await snapshot(page)).ui_targets.find(t=>t.action.startsWith('FastTravel(')).action;
    await click(page,'TravelPage(true)');await expect.poll(async()=>(await snapshot(page)).ui_targets.find(t=>t.action.startsWith('FastTravel('))?.action).not.toBe(first);
  }
  await click(page,`FastTravel(${vault.id})`);await phase(page,'Playing');
  await expect.poll(async()=>(await snapshot(page)).walker).toBe(false);
  expect((await snapshot(page)).navigation.position[1]).toBeLessThan(-900);
  await page.keyboard.press('Control+s');await expect.poll(async()=>(await snapshot(page)).notice).toContain('enregistrée');
  const beforeReload=await snapshot(page);await page.screenshot({path:'../docs/evidence/web-travel-cavern-r75.png'});
  await page.reload();await phase(page,'Menu');await page.keyboard.press('l');await phase(page,'Playing');
  await expect.poll(async()=>(await snapshot(page)).mesh_pending).toBe(0);
  expect(Math.hypot(...(await snapshot(page)).navigation.position.map((p,i)=>p-beforeReload.navigation.position[i]))).toBeLessThan(100);
  await page.keyboard.press('m');await phase(page,'Atlas');await click(page,'Travel');await phase(page,'Travel');
  await tab(page,'Currents');await expect.poll(async()=>(await snapshot(page)).ui_targets.some(t=>t.action==='FastTravel(20000)')).toBe(true);
  await click(page,'FastTravel(20000)');await phase(page,'Playing');
  expect((await snapshot(page)).navigation.position.every(Number.isFinite)).toBe(true);
  await travel(page);await page.setViewportSize({width:960,height:640});await page.waitForTimeout(400);
  expect((await snapshot(page)).ui_targets.every(t=>t.x-t.width/2>=-1&&t.x+t.width/2<=961&&t.y-t.height/2>=-1&&t.y+t.height/2<=641)).toBe(true);
  await page.screenshot({path:'../docs/evidence/web-travel-small-r75.png'});
  await page.keyboard.press('Escape');await phase(page,'Playing');expect(errors).toEqual([]);
  await fs.writeFile(`../docs/evidence/travel-r75-${process.env.AETHER_BROWSER}.json`,JSON.stringify({islands:seen.size,catalog:catalog.length,underground: vault.label,real_reload:true,walker_returned_to_helm:true,page_errors:errors},null,2));
});
