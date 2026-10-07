import {test,expect} from '@playwright/test';
import fs from 'node:fs/promises';
const snapshot=page=>page.evaluate(()=>window.aetherProbe);
const phase=(page,value)=>expect.poll(async()=>(await snapshot(page))?.phase,{timeout:value==='Menu'?60000:20000}).toBe(value);
async function click(page,action){await expect.poll(async()=>(await snapshot(page))?.ui_targets.some(t=>t.action===action)).toBe(true);const t=(await snapshot(page)).ui_targets.find(t=>t.action===action);await page.mouse.click(t.x,t.y);}
async function ready(page){await page.goto('/?probe=1');await phase(page,'Menu');await page.keyboard.press('Enter');await phase(page,'Playing');await expect.poll(async()=>(await snapshot(page)).mesh_pending).toBe(0);}

test('boutons au clavier, focus visible et commandes de jeu isolées',async({page})=>{
  await page.goto('/?probe=1');await phase(page,'Menu');await page.keyboard.press('Tab');
  await expect.poll(async()=>(await snapshot(page)).keyboard_action).toBe('NewGame');
  await page.keyboard.press('Enter');await phase(page,'Playing');
  await page.keyboard.press('F6');await expect.poll(async()=>(await snapshot(page)).keyboard_ui).toBe(true);
  await page.keyboard.press('f');await page.waitForTimeout(300);expect((await snapshot(page)).docked).toBe(true);
  await page.keyboard.press('ArrowDown');await expect.poll(async()=>(await snapshot(page)).keyboard_action).toBe('Dock');
  await page.screenshot({path:'../docs/evidence/web-keyboard-focus.png'});
  await page.keyboard.press('Enter');await expect.poll(async()=>(await snapshot(page)).docked).toBe(false);
  await page.keyboard.press('F6');await expect.poll(async()=>(await snapshot(page)).keyboard_ui).toBe(true);
  await page.keyboard.press('Escape');await page.waitForTimeout(300);expect((await snapshot(page)).phase).toBe('Playing');
  await page.keyboard.press('Escape');await phase(page,'Paused');await page.keyboard.press('Tab');
  await expect.poll(async()=>(await snapshot(page)).keyboard_action).toBe('Pause');await page.keyboard.press('Enter');await phase(page,'Playing');
});

test('perte réelle du focus canvas : pause et touche maintenue libérée',async({page})=>{
  await ready(page);await page.keyboard.press('f');await expect.poll(async()=>(await snapshot(page)).docked).toBe(false);
  await page.keyboard.down('e');await page.waitForTimeout(350);await page.locator('#fullscreen').focus();await page.waitForTimeout(300);
  const before=await snapshot(page);await page.waitForTimeout(1200);const hidden=await snapshot(page);
  expect(hidden.tick-before.tick).toBeLessThan(3);await page.keyboard.up('e');await page.locator('canvas').focus();await page.waitForTimeout(500);
  const after=await snapshot(page);expect(after.tick).toBeGreaterThan(hidden.tick+10);
  expect(Math.abs(after.navigation.target_altitude-hidden.navigation.target_altitude)).toBeLessThan(.1);
});

test('remappage persistant, conflit explicite, anciennes touches libérées et tutoriel rejouable',async({page})=>{
  await ready(page);await page.keyboard.press('Escape');await phase(page,'Paused');await click(page,'Settings');await phase(page,'Settings');
  await click(page,'Rebind(Dock)');await page.keyboard.press('g');
  await expect.poll(async()=>(await snapshot(page)).binding_message).toContain('déjà affectée');
  await page.keyboard.press('h');
  await expect.poll(async()=>(await snapshot(page)).bindings.Dock?.Letter).toBe('h');
  await page.screenshot({path:'../docs/evidence/web-keybindings.png'});
  await page.reload();await phase(page,'Menu');await page.keyboard.press('Enter');await phase(page,'Playing');
  await expect.poll(async()=>(await snapshot(page)).mesh_pending).toBe(0);
  await page.keyboard.press('f');await page.waitForTimeout(300);expect((await snapshot(page)).docked).toBe(true);
  await page.keyboard.press('h');await expect.poll(async()=>(await snapshot(page)).docked).toBe(false);
  await page.keyboard.down('x');await page.waitForTimeout(500);await page.keyboard.up('x');
  await expect.poll(async()=>((await snapshot(page)).tutorial_flags&14)).toBe(14);
  await page.keyboard.press('Escape');await phase(page,'Paused');await click(page,'Tutorial');await phase(page,'Playing');
  expect((await snapshot(page)).tutorial_flags&1).toBe(0);
  await page.keyboard.press('Escape');await phase(page,'Paused');await click(page,'Settings');await phase(page,'Settings');
  await click(page,'ResetBindings');await expect.poll(async()=>(await snapshot(page)).bindings).toEqual({});
});

test('réattribution : Maj, touche réservée visible et échange persistant des commandes',async({page})=>{
  await ready(page);await page.keyboard.press('Escape');await phase(page,'Paused');await click(page,'Settings');await phase(page,'Settings');
  await click(page,'Rebind(Ascend)');
  await expect.poll(async()=>(await snapshot(page)).binding_message).toContain('Monter : appuyez');
  await page.keyboard.press('Shift');
  await expect.poll(async()=>(await snapshot(page)).bindings.Ascend).toBe('Shift');
  await click(page,'Rebind(Forward)');await page.keyboard.press('F8');
  await expect.poll(async()=>(await snapshot(page)).binding_message).toContain('réservée');
  expect((await snapshot(page)).bindings.Forward).toBeUndefined();
  await page.keyboard.press('q');
  await expect.poll(async()=>(await snapshot(page)).binding_message).toContain('déjà affectée');
  await page.keyboard.press('Enter');
  await expect.poll(async()=>(await snapshot(page)).bindings.Forward?.Letter).toBe('q');
  expect((await snapshot(page)).bindings.Left?.Letter).toBe('z');
  await click(page,'Rebind(TrimLeft)');
  await expect.poll(async()=>(await snapshot(page)).binding_message).toContain('Voile − : appuyez');
  await page.keyboard.press('x');
  await expect.poll(async()=>(await snapshot(page)).binding_message).toContain('déjà affectée');
  await click(page,'SwapBinding');
  await expect.poll(async()=>(await snapshot(page)).bindings.TrimLeft?.Letter).toBe('x');
  expect((await snapshot(page)).bindings.TrimRight?.Letter).toBe('j');
  await page.screenshot({path:'../docs/evidence/web-rebind-swap-shift.png'});
  await page.reload();await phase(page,'Menu');await page.keyboard.press('Enter');await phase(page,'Playing');
  await expect.poll(async()=>(await snapshot(page)).mesh_pending).toBe(0);
  expect((await snapshot(page)).bindings.Ascend).toBe('Shift');
  expect((await snapshot(page)).bindings.Forward?.Letter).toBe('q');
  expect((await snapshot(page)).bindings.Left?.Letter).toBe('z');
  expect((await snapshot(page)).bindings.TrimLeft?.Letter).toBe('x');
  expect((await snapshot(page)).bindings.TrimRight?.Letter).toBe('j');
  await page.keyboard.press('f');await expect.poll(async()=>(await snapshot(page)).docked).toBe(false);
  const initial=(await snapshot(page)).navigation.target_altitude;
  await page.keyboard.down('Shift');await page.waitForTimeout(650);await page.keyboard.up('Shift');
  await expect.poll(async()=>(await snapshot(page)).navigation.target_altitude).toBeGreaterThan(initial+1);
  // The full probe is published at5Hz: wait until the actual release reaches
  // the game before using its altitude as the old-key baseline.
  await expect.poll(async()=>(await snapshot(page)).navigation.intent.climb).toBe(0);
  const after=(await snapshot(page)).navigation.target_altitude;
  await page.keyboard.down('e');await page.waitForTimeout(400);
  expect((await snapshot(page)).navigation.intent.climb).toBe(0);
  await page.keyboard.up('e');
  expect(Math.abs((await snapshot(page)).navigation.target_altitude-after)).toBeLessThan(.1);
  await page.keyboard.down('q');
  await expect.poll(async()=>(await snapshot(page)).navigation.intent.throttle).toBeGreaterThan(.5);
  await page.keyboard.up('q');
  await expect.poll(async()=>(await snapshot(page)).navigation.intent.throttle).toBe(0);
});

test('suspension JS 65 s : reprise bornée, sauvegarde et historique arrière/avant',async({page,context})=>{
  test.setTimeout(240000);
  await ready(page);await page.keyboard.press('f');await expect.poll(async()=>(await snapshot(page)).docked).toBe(false);
  await page.keyboard.press('Control+s');await expect.poll(async()=>(await snapshot(page)).notice).toContain('enregistrée');
  const before=await snapshot(page);const cdp=await context.newCDPSession(page);
  await cdp.send('Debugger.enable');
  const pausedEvent=new Promise(resolve=>cdp.once('Debugger.paused',resolve));
  await cdp.send('Debugger.pause');await pausedEvent;
  await new Promise(resolve=>setTimeout(resolve,65000));
  await cdp.send('Debugger.resume');await cdp.send('Debugger.disable');
  await page.bringToFront();await page.locator('canvas').focus();
  await page.waitForTimeout(500);const after=await snapshot(page);
  expect(after.tick-before.tick).toBeLessThan(80);
  expect(after.navigation.position.every(Number.isFinite)).toBe(true);
  expect(Math.hypot(...after.navigation.position.map((v,i)=>v-before.navigation.position[i]))).toBeLessThan(25);
  await page.keyboard.press('Escape');await phase(page,'Paused');
  const paused=(await snapshot(page)).tick;await page.waitForTimeout(1200);expect((await snapshot(page)).tick-paused).toBeLessThan(7);
  await page.goto('/index.html?history-test=1&probe=1');await phase(page,'Menu');
  await page.goBack();await expect.poll(async()=>['Menu','Paused'].includes((await snapshot(page))?.phase),{timeout:60000}).toBe(true);
  await page.goForward();await phase(page,'Menu');await page.keyboard.press('l');await phase(page,'Playing');
  expect((await snapshot(page)).cells).toBe(before.cells);
  await fs.writeFile(`../docs/evidence/lifecycle-${process.env.AETHER_BROWSER||'chromium'}.json`,JSON.stringify({suspension_method:'CDP Debugger.pause / resume, real JS execution suspension',freeze_seconds:65,before_tick:before.tick,after_tick:after.tick,resume_ticks:after.tick-before.tick,history_save_restored:true,bfcache_claimed:false,os_suspend_claimed:false},null,2));
});

test('plein écran par geste utilisateur, retour et redimensionnement des préférences',async({page})=>{
  await ready(page);
  await page.locator('#fullscreen').click();
  await expect.poll(()=>page.evaluate(()=>!!document.fullscreenElement)).toBe(true);
  await page.locator('#fullscreen').click();
  await expect.poll(()=>page.evaluate(()=>!!document.fullscreenElement)).toBe(false);
  await page.setViewportSize({width:960,height:640});await page.keyboard.press('Escape');await phase(page,'Paused');
  await click(page,'Settings');await phase(page,'Settings');await page.waitForTimeout(250);
  const targets=(await snapshot(page)).ui_targets;
  expect(targets.every(t=>t.x-t.width/2>=0&&t.x+t.width/2<=961&&t.y-t.height/2>=0&&t.y+t.height/2<=641)).toBe(true);
  await page.screenshot({path:'../docs/evidence/web-settings-small.png'});
});
