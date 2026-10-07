// Deliberate corruptions only in an explicit independent scratch fixture.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {verify} from '../verify-cloud-lighting.mjs';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../../..');
const scratch=process.argv[2]&&path.resolve(process.argv[2]);
assert(scratch&&scratch!==root&&!root.startsWith(scratch+path.sep));
fs.mkdirSync(scratch,{recursive:true});
for(const sub of ['assets/atmosphere','assets/shaders/clouds.wgsl','tools/art','docs/evidence']) {
 const dst=path.join(scratch,sub);fs.mkdirSync(path.dirname(dst),{recursive:true});
 fs.cpSync(path.join(root,sub),dst,{recursive:true,filter:p=>!p.includes('__pycache__')});
}
const original=new Map(),read=f=>JSON.parse(fs.readFileSync(path.join(scratch,f)));
const write=(f,v)=>{const p=path.join(scratch,f);if(!original.has(f))original.set(f,fs.readFileSync(p));fs.writeFileSync(p,Buffer.isBuffer(v)?v:JSON.stringify(v,null,2)+'\n');};
const edit=(f,fn)=>{const r=read(f);fn(r);write(f,r);};
const replace=(f,from,to)=>write(f,Buffer.from(fs.readFileSync(path.join(scratch,f),'utf8').replace(from,to)));
const tests=[];
function reject(test,mutate) {
 try{mutate();assert.throws(()=>verify(scratch,{write:false}));tests.push({test,rejected:true});}
 finally{for(const [f,b] of original)fs.writeFileSync(path.join(scratch,f),b);original.clear();}
}
const rays='docs/evidence/cloud-r57-ray-comparison.json',contract='tools/art/cloud-r57/contract.json';
verify(scratch,{write:false});
reject('R57 opaque RMSE .026 rejected',()=>edit(rays,r=>r.views[0].comparisons.cache_vs_analytic.opaque_alpha_gt_095.display_rmse=.026));
reject('R57 edge opacity p99 .081 rejected',()=>edit(rays,r=>r.views[0].comparisons.cache_vs_analytic.edge_alpha_005_to_095.opacity_absolute.p50_p95_p99_max[2]=.081));
reject('two-step cadence cannot be omitted',()=>edit(rays,r=>r.views=r.views.slice(0,4)));
reject('ray camera cannot drift',()=>edit(rays,r=>r.views[0].images[0].origin_local[0]+=1));
reject('R55 cannot be relabelled current proof',()=>edit(rays,r=>r.recipe='r55-directional-reflected-light'));
reject('sun direction must remain exact',()=>edit(rays,r=>r.views[0].images[0].sun_direction=[0,1,0]));
reject('bounce cannot be omitted',()=>edit(rays,r=>r.views[0].images[0].bounce_direction=[0,0,0]));
reject('opacity may not change even inside tolerance',()=>edit(rays,r=>r.views[0].comparisons.cache_vs_analytic.all.opacity_absolute.mean+=.000001));
reject('limits may not be loosened',()=>edit(contract,r=>r.limits.opaque_display_rmse=.03));
reject('wrong incident gains rejected',()=>edit(contract,r=>r.lighting.ambient_rgb_gain=[2,2,2]));
reject('old shader is not current shader',()=>write('assets/shaders/clouds.wgsl',fs.readFileSync(path.join(scratch,'tools/art/cloud-r57/archive/clouds-r55.wgsl'))));
reject('CPU gains cannot drift',()=>replace('tools/art/review_cloud_light_r57_cpu.py','[4.,4.5,5.]','[2.,2.,2.]'));
reject('CPU direct radiance cannot drift',()=>replace('tools/art/review_cloud_light_r57_cpu.py','direct=np.array([3.,1.86,.96])','direct=np.array([3.,2.55,1.95])'));
reject('CPU integration cannot drift',()=>replace('tools/art/review_cloud_light_r57_cpu.py','for k in range(128):','for k in range(64):'));
reject('shader density and integration cannot drift',()=>replace('assets/shaders/clouds.wgsl','fn sunlight(','// unknown radiance drift\nfn sunlight('));
reject('stored ray floats cannot be rescaled',()=>write('tools/art/cloud-r57/review/cloud-ray-floats-hollow-oblique-refresh4.npz',Buffer.from('stale floats')));
reject('historical R55 comparison must stay exact',()=>edit('docs/evidence/cloud-r55-ray-comparison.json',r=>r.note='Now validates R57'));
reject('historical R47 comparison must stay exact',()=>edit('docs/evidence/cloud-r47-ray-comparison.json',r=>r.note='Now validates R57'));
reject('historical R55 verifier source remains exact',()=>write('tools/art/cloud-r57/archive/verify-cloud-lighting-r55.mjs.txt',Buffer.from('relabelled verifier')));
reject('compatibility acceptance logic cannot drift even with refreshed source SHA',()=>{
 const f='tools/art/cloud-r57/verify-r55-historical.mjs';replace(f,'opaque_display_rmse:.025','opaque_display_rmse:.03');
 edit('tools/art/cloud-r57/sources.json',r=>r[f]=crypto.createHash('sha256').update(fs.readFileSync(path.join(scratch,f))).digest('hex'));
});
verify(scratch,{write:false});
const out={schema:1,passed:true,recipe:'r57-blue-fill-warm-direct',negative_cases:tests.length,tests};
fs.writeFileSync(path.join(root,'docs/evidence/cloud-r57-negative-tests.json'),JSON.stringify(out,null,2)+'\n');
console.log(`${tests.length} deliberate R57 corruptions rejected; R47/R55 evidence untouched.`);
