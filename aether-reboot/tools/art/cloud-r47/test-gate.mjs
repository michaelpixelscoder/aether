// Mutation tests use an isolated scratch copy; the reviewed promotion is read-only.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {verify} from '../verify-cloud-cache.mjs';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../../..');
const scratch=process.argv[2]&&path.resolve(process.argv[2]);
assert(scratch&&scratch!==root&&!root.startsWith(scratch+path.sep),'Explicit separate scratch root required');
fs.mkdirSync(scratch,{recursive:true});
const copy=sub=>{const dst=path.join(scratch,sub);fs.mkdirSync(path.dirname(dst),{recursive:true});fs.cpSync(path.join(root,sub),dst,{recursive:true});};
for(const sub of ['assets/atmosphere','tools/art/cloud-r47/archive','tools/art/cloud-r47/contract.json'])copy(sub);
for(const f of ['build_cloud_cache.py','cloud_cache_sampling.py','audit_cloud_field.py','review_cloud_field_cpu.py','compare_cloud_rays_cpu.py','verify_cloud_cache.py','verify-cloud-cache.mjs'])copy('tools/art/'+f);
for(const f of ['cloud-r47-audit.json','cloud-r47-ray-comparison.json','cloud-cache-fidelity.json','cloud-cache-assets.json'])copy('docs/evidence/'+f);
const original=new Map(),read=f=>JSON.parse(fs.readFileSync(path.join(scratch,f))),digest=f=>crypto.createHash('sha256').update(fs.readFileSync(path.join(scratch,f))).digest('hex');
const write=(f,value)=>{const p=path.join(scratch,f);if(!original.has(f))original.set(f,fs.readFileSync(p));fs.writeFileSync(p,Buffer.isBuffer(value)?value:JSON.stringify(value,null,2)+'\n');};
const results=[];
function reject(name,mutate){
 try{mutate();assert.throws(()=>verify(scratch));results.push({test:name,rejected:true});}
 finally{for(const [f,b] of original)fs.writeFileSync(path.join(scratch,f),b);original.clear();}
}
verify(scratch);
reject('regional memory/resolution cannot drift',()=>{const x=read('assets/atmosphere/cloud-banks.json');x[0].resolution[0]=257;write('assets/atmosphere/cloud-banks.json',x);const m=read('assets/atmosphere/cloud-cache-manifest.json');m.config_sha256=digest('assets/atmosphere/cloud-banks.json');write('assets/atmosphere/cloud-cache-manifest.json',m);});
reject('continuous cave roof floor cannot be disabled',()=>{const x=read('assets/atmosphere/cloud-banks.json');x[0].shape[3]=0;write('assets/atmosphere/cloud-banks.json',x);const m=read('assets/atmosphere/cloud-cache-manifest.json');m.config_sha256=digest('assets/atmosphere/cloud-banks.json');write('assets/atmosphere/cloud-cache-manifest.json',m);});
for(const index of [0,2])reject((index?'historical high':'reviewed regional')+' cache cannot change even with updated manifest hash',()=>{const m=read('assets/atmosphere/cloud-cache-manifest.json'),f=m.banks[index].file,b=fs.readFileSync(path.join(scratch,f));b[b.length-17]^=1;write(f,b);m.banks[index].sha256=digest(f);write('assets/atmosphere/cloud-cache-manifest.json',m);});
reject('unreviewed generator cannot be relabelled in manifest',()=>{const f='tools/art/build_cloud_cache.py';write(f,Buffer.concat([fs.readFileSync(path.join(scratch,f)),Buffer.from('\n# drift\n')]));const m=read('assets/atmosphere/cloud-cache-manifest.json');m.generator_sha256=digest(f);write('assets/atmosphere/cloud-cache-manifest.json',m);});
reject('historical evidence cannot be overwritten',()=>{const f='tools/art/cloud-r47/archive/verify_cloud_cache.py';write(f,Buffer.concat([fs.readFileSync(path.join(scratch,f)),Buffer.from('\n# drift\n')]));});
function scalarProof(mutator){const a=read('docs/evidence/cloud-r47-audit.json'),p=read('docs/evidence/cloud-cache-fidelity.json');mutator(a);write('docs/evidence/cloud-r47-audit.json',a);p.banks=a.banks;p.audit_sha256=digest('docs/evidence/cloud-r47-audit.json');write('docs/evidence/cloud-cache-fidelity.json',p);}
reject('regional p99 must stay below accepted .05',()=>scalarProof(a=>a.banks[0].all.p50_p95_p99_max[2]=.051));
reject('high-bank p99 still obeys historical .035',()=>scalarProof(a=>a.banks[2].all.p50_p95_p99_max[2]=.036));
reject('analytic derivative fidelity is independently bounded',()=>scalarProof(a=>a.banks[0].analytic_normal_vs_finite_difference_degrees.p50_p95_p99_max[2]=.051));
function rayProof(mutator){const a=read('docs/evidence/cloud-r47-ray-comparison.json'),p=read('docs/evidence/cloud-cache-fidelity.json');mutator(a);write('docs/evidence/cloud-r47-ray-comparison.json',a);p.views=a.views;p.ray_comparison_sha256=digest('docs/evidence/cloud-r47-ray-comparison.json');write('docs/evidence/cloud-cache-fidelity.json',p);}
reject('opaque image RMSE is independently bounded',()=>rayProof(a=>a.views[0].comparisons.cache_vs_analytic.opaque_alpha_gt_095.display_rmse=.026));
reject('edge opacity is independently bounded',()=>rayProof(a=>a.views[0].comparisons.cache_vs_analytic.edge_alpha_005_to_095.opacity_absolute.p50_p95_p99_max[2]=.081));
reject('proof cannot quietly change comparison camera',()=>rayProof(a=>a.views[0].images[0].origin_local[0]+=1));
reject('stale renderer proof is rejected',()=>{const f='tools/art/review_cloud_field_cpu.py';write(f,Buffer.concat([fs.readFileSync(path.join(scratch,f)),Buffer.from('\n# drift\n')]));});
const report={schema:1,passed:true,negative_cases:results.length,tests:results};
fs.writeFileSync(path.join(root,'docs/evidence/cloud-r47-negative-tests.json'),JSON.stringify(report,null,2)+'\n');
console.log(`${results.length} deliberate contract/fidelity corruptions rejected; originals intact.`);
