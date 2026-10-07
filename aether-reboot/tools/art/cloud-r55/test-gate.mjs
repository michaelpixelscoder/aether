// Deliberate corruptions are confined to the caller's separate scratch root.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {fileURLToPath} from 'node:url';
import {verify} from '../verify-cloud-lighting.mjs';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../../..');
const scratch=process.argv[2]&&path.resolve(process.argv[2]);
assert(scratch&&scratch!==root&&!root.startsWith(scratch+path.sep),'Explicit separate scratch root required');
fs.mkdirSync(scratch,{recursive:true});
for(const sub of ['assets/atmosphere','assets/shaders/clouds.wgsl','tools/art','docs/evidence']) {
 const dst=path.join(scratch,sub);fs.mkdirSync(path.dirname(dst),{recursive:true});
 fs.cpSync(path.join(root,sub),dst,{recursive:true,filter:p=>!p.includes('__pycache__')});
}
const original=new Map(),read=f=>JSON.parse(fs.readFileSync(path.join(scratch,f)));
const write=(f,v)=>{const p=path.join(scratch,f);if(!original.has(f))original.set(f,fs.readFileSync(p));fs.writeFileSync(p,Buffer.isBuffer(v)?v:JSON.stringify(v,null,2)+'\n');};
const tests=[];
function reject(test,mutate) {
 try{mutate();assert.throws(()=>verify(scratch,{write:false}));tests.push({test,rejected:true});}
 finally{for(const [f,b] of original)fs.writeFileSync(path.join(scratch,f),b);original.clear();}
}
const edit=(file,fn)=>{const r=read(file);fn(r);write(file,r);};
const rays='docs/evidence/cloud-r55-ray-comparison.json',contract='tools/art/cloud-r55/contract.json';
verify(scratch,{write:false});
reject('opaque RMSE must remain below .025',()=>edit(rays,r=>r.views[0].comparisons.cache_vs_analytic.opaque_alpha_gt_095.display_rmse=.026));
reject('edge alpha p99 must remain below .08',()=>edit(rays,r=>r.views[0].comparisons.cache_vs_analytic.edge_alpha_005_to_095.opacity_absolute.p50_p95_p99_max[2]=.081));
reject('current two-step cadence cannot be omitted',()=>edit(rays,r=>r.views=r.views.slice(0,4)));
reject('comparison camera cannot drift',()=>edit(rays,r=>r.views[0].images[0].origin_local[0]+=1));
reject('old sunlight cannot be relabelled R55',()=>edit(rays,r=>r.views[0].images[0].sun_direction=[-.5,.8,.3]));
reject('bounce path cannot be omitted',()=>edit(rays,r=>r.views[0].images[0].bounce_direction=[0,0,0]));
reject('silently loosened contract limits are rejected',()=>edit(contract,r=>r.limits.opaque_display_rmse=.03));
reject('lighting may not alter opacity even within the old tolerance',()=>edit(rays,r=>r.views[0].comparisons.cache_vs_analytic.all.opacity_absolute.mean+=.000001));
reject('R47 evidence cannot be rewritten to imply R55 coverage',()=>edit('docs/evidence/cloud-r47-ray-comparison.json',r=>r.note='Now validates R55'));
for(const f of ['assets/shaders/clouds.wgsl','tools/art/review_cloud_light_cpu.py','tools/art/build_cloud_lighting.mjs'])reject('stale input '+f,()=>write(f,Buffer.concat([fs.readFileSync(path.join(scratch,f)),Buffer.from('\n// deliberate drift\n')])));
verify(scratch,{write:false});
const out={schema:1,passed:true,negative_cases:tests.length,tests};
fs.writeFileSync(path.join(root,'docs/evidence/cloud-r55-negative-tests.json'),JSON.stringify(out,null,2)+'\n');
console.log(`${tests.length} R55 deliberate corruptions rejected; source package and R47 history untouched.`);
