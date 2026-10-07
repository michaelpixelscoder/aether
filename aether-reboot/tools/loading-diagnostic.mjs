import {chromium} from '@playwright/test';
import fs from 'node:fs/promises';
const browser=await chromium.launch({channel:'chrome',headless:true,args:['--use-angle=d3d11','--enable-unsafe-webgpu']});
const page=await browser.newPage({viewport:{width:1280,height:720}}),start=Date.now();
const report={errors:[],requests:[],samples:[]};
page.on('pageerror',e=>report.errors.push(e.message));
page.on('console',m=>{if(m.type()==='error')report.errors.push(m.text());});
page.on('requestfinished',async r=>{if(r.url().includes('/assets/'))report.requests.push({url:r.url(),seconds:(Date.now()-start)/1000});});
page.on('requestfailed',r=>report.errors.push(`${r.url()}: ${r.failure()?.errorText}`));
try {
 await page.goto((process.env.AETHER_TEST_URL||'http://127.0.0.1:4181/')+'?probe=1');
 while(Date.now()-start<90000){
  const state=await page.evaluate(()=>window.aetherProbe);
  report.samples.push({seconds:(Date.now()-start)/1000,phase:state?.phase,notice:state?.notice});
  if(['Menu','Error'].includes(state?.phase)){report.final=state;break;}
  await page.waitForTimeout(1000);
 }
 await page.screenshot({path:'.dream-loop/loading-r40.png'});
 await fs.writeFile('.dream-loop/loading-r40.json',JSON.stringify(report,null,2));
 console.log(JSON.stringify({last:report.samples.at(-1),errors:report.errors,requests:report.requests.length,latest:report.requests.slice(-3),loading:report.final?.loading}));
}finally{await browser.close();}
