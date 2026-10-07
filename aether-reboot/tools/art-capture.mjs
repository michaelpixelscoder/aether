import {chromium,expect} from '@playwright/test';
import fs from 'node:fs/promises';
const browser=await chromium.launch({channel:'chrome',headless:true,args:['--use-angle=d3d11','--enable-unsafe-webgpu']});
const page=await browser.newPage({viewport:{width:1672,height:941}});
const probe=()=>page.evaluate(()=>window.aetherProbe);
const phase=p=>expect.poll(async()=>(await probe())?.phase,{timeout:p==='Menu'?60000:30000}).toBe(p);
try {
 await page.goto((process.env.AETHER_TEST_URL||'http://127.0.0.1:4173')+'/?probe=1');await phase('Menu');
 await page.screenshot({path:'../.dream-loop/latest-menu.png'});
 await page.keyboard.press('Enter');await phase('Playing');await expect.poll(async()=>(await probe()).mesh_pending).toBe(0);
 const held=new Set();
 const set=async(key,on)=>{if(on&&!held.has(key)){await page.keyboard.down(key);held.add(key);}else if(!on&&held.has(key)){await page.keyboard.up(key);held.delete(key);}};
 await page.keyboard.press('f');
 await page.keyboard.down('e');await page.waitForTimeout(1400);await page.keyboard.up('e');await page.waitForTimeout(12500);
 await set('s',true);
 const turnStart=Date.now();
 while(Date.now()-turnStart<8000){
  const nav=(await probe()).navigation;
  const error=.85-nav.yaw,turn=error*2-nav.angular_y*1.6;
  await set('ArrowLeft',turn>.06);await set('ArrowRight',turn<-.06);
  if(Math.abs(error)<.035&&Math.abs(nav.angular_y)<.04)break;
  await page.waitForTimeout(50);
 }
 for(const key of held)await page.keyboard.up(key);
 await page.keyboard.up('s');
 const c=(await probe()).camera;
 await page.keyboard.down('x');await page.waitForTimeout(440);await page.keyboard.up('x');
 await page.mouse.move(600,450);await page.mouse.down({button:'right'});
 await page.mouse.move(600-(-1.1-c.yaw)/.005,450+(.14-c.pitch)/.004,{steps:12});await page.mouse.up({button:'right'});
 // Wheel events in winit use opposite signs to DOM deltaY, with physical pixels.
 await page.mouse.wheel(0,-(c.distance-7.8)/1.1);await page.waitForTimeout(600);
 await page.keyboard.press('Escape');await phase('Paused');await page.keyboard.press('F8');
 await page.waitForTimeout(500);await page.screenshot({path:'../.dream-loop/latest-voyage.png'});
 await fs.writeFile('../.dream-loop/latest-view.json',JSON.stringify(await probe(),null,2));
 await page.keyboard.press('F8');await page.keyboard.press('Escape');await phase('Playing');await page.keyboard.press('r');
 await expect.poll(async()=>(await probe()).docked).toBe(true);await page.keyboard.press('Tab');await phase('Editing');await page.waitForTimeout(600);
 await page.screenshot({path:'../.dream-loop/latest-atelier.png'});
}finally {await browser.close();}
