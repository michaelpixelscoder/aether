// No-argument portable build gate; recompute CPU proof with verify_cloud_cache.py.
// The old .035 regional gate remains archived and explicitly failed in evidence.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import {fileURLToPath} from 'node:url';
export function verify(root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../..')) {
 const hash=file=>crypto.createHash('sha256').update(fs.readFileSync(path.join(root,file))).digest('hex');
 const read=file=>JSON.parse(fs.readFileSync(path.join(root,file),'utf8'));
 const contract=read('tools/art/cloud-r47/contract.json');
 assert.equal(contract.recipe,'r47-regional-plume-crown-fold-r2');
 assert.deepEqual(contract.limits,{gpu_bytes:59768832,density_mean:.006,regional_density_p99:.05,high_density_p99:.035,analytic_derivative_p99_degrees:.05,opaque_display_rmse:.025,edge_opacity_p99:.08});
 for(const entry of contract.archive)assert.equal(hash(entry.file),entry.sha256,`${entry.file}: immutable historical evidence`);
 const manifest=read('assets/atmosphere/cloud-cache-manifest.json'),config=read('assets/atmosphere/cloud-banks.json');
 assert.equal(manifest.generator,'tools/art/build_cloud_cache.py');
 assert.equal(hash(manifest.generator),manifest.generator_sha256);
 assert.equal(manifest.generator_sha256,contract.generator_sha256);
 assert.equal(hash('assets/atmosphere/cloud-banks.json'),manifest.config_sha256);
 assert.deepEqual(config,contract.config,'Centers/extents/padding/seeds/floor/resolution must match the reviewed contract');
 assert.equal(manifest.recipe,'r47 connected regional plume/crown/fold union; high banks exact legacy');
 assert.equal(manifest.format,'RGBA8Unorm 3D');
 assert.equal(config.length,6);assert.equal(manifest.banks.length,6);
 const history=read('tools/art/cloud-r47/archive/baseline/assets/atmosphere/cloud-cache-manifest.json');
 let total=0;
 const banks=config.map((bank,index)=>{
  const regional=index<2,entry=manifest.banks[index];
  assert.equal(entry.key,bank.key);assert.equal(entry.recipe,regional?'regional-r47':'unchanged-high-bank');
  const file=`assets/atmosphere/${bank.key}-shape.ktx2`;assert.equal(entry.file,file);
  const bytes=fs.readFileSync(path.join(root,file));assert.equal(hash(file),entry.sha256);
  assert.equal(entry.sha256,contract.reviewed_cache_sha256[bank.key]);
  if(!regional)assert.equal(entry.sha256,history.banks[index].sha256);
  assert(bytes.subarray(0,12).equals(Buffer.from([171,75,84,88,32,50,48,187,13,10,26,10])));
  assert.deepEqual([12,16,32,36,40,44].map(n=>bytes.readUInt32LE(n)),[37,1,0,1,1,0]);
  const size=[20,24,28].map(n=>bytes.readUInt32LE(n));
  assert.deepEqual(size,bank.resolution);assert.deepEqual(size,entry.dimensions);
  assert.deepEqual(size,regional?[256,96,256]:[96,64,96]);
  const offset=Number(bytes.readBigUInt64LE(80)),length=Number(bytes.readBigUInt64LE(88));
  assert.equal(length,size.reduce((a,b)=>a*b,4));assert.equal(length,entry.gpu_bytes);
  assert.equal(Number(bytes.readBigUInt64LE(96)),length);assert.equal(offset+length,bytes.length);
  let positive=0,min=255,max=0;
  for(let n=offset;n<bytes.length;n+=4){const v=bytes[n];positive+=Number(v>0);min=Math.min(min,v);max=Math.max(max,v);}
  assert.equal(positive,entry.positive_voxels);assert.equal(min,0);assert(max>=250);
  assert(positive>size.reduce((a,b)=>a*b)*.02);total+=length;
  return {file,sha256:entry.sha256,dimensions:size,gpu_bytes:length,positive_voxels:positive,recipe:entry.recipe};
 });
 assert.equal(total,manifest.gpu_bytes);assert.equal(total,59768832);
 const proof=read('docs/evidence/cloud-cache-fidelity.json');
 assert.equal(proof.passed,true);assert.equal(proof.contract,'r47-regional-accepted-with-image-and-derivative-guards');
 assert.equal(proof.contract_sha256,hash('tools/art/cloud-r47/contract.json'));
 assert.equal(proof.generator_sha256,manifest.generator_sha256);assert.equal(proof.config_sha256,manifest.config_sha256);
 for(const [file,expected] of Object.entries(proof.source_hashes))assert.equal(hash('tools/art/'+file),expected,`${file}: stale fidelity proof`);
 for(const file of ['build_cloud_cache.py','cloud_cache_sampling.py','audit_cloud_field.py','review_cloud_field_cpu.py','compare_cloud_rays_cpu.py','verify_cloud_cache.py'])assert(file in proof.source_hashes);
 assert.equal(proof.audit_sha256,hash('docs/evidence/cloud-r47-audit.json'));assert.equal(proof.ray_comparison_sha256,hash('docs/evidence/cloud-r47-ray-comparison.json'));
 assert.deepEqual(proof.banks,read('docs/evidence/cloud-r47-audit.json').banks);
 assert.deepEqual(proof.views,read('docs/evidence/cloud-r47-ray-comparison.json').views);
 assert.equal(proof.gpu_bytes,total);assert.equal(proof.legacy_failure_preserved,true);assert.equal(proof.legacy_pointwise_gate_pass,false);
 assert.equal(proof.banks.length,6);
 proof.banks.forEach((b,i)=>{
  assert.equal(b.key,config[i].key);assert.equal(b.sha256,banks[i].sha256);
  assert(b.all.mean<.006);assert(b.all.p50_p95_p99_max[2]<(i<2?.05:.035));
  assert.equal(b.exact_quantized_texel_centers,2048);assert(b.advection_in_bounds&&b.center_extent_padding_shape_exact);
  if(i<2){assert.equal(b.analytic_normal_vs_finite_difference_degrees.count,3000);assert(b.analytic_normal_vs_finite_difference_degrees.p50_p95_p99_max[2]<.05);}
  else assert(b.high_bank_byte_identical&&b.legacy_interpolation_gate_pass);
 });
 assert.equal(proof.views.length,4);
 assert.deepEqual(proof.views.map(v=>v.bank+'/'+v.view),['hollow/oblique','hollow/grazing','underforge/oblique','underforge/grazing']);
 for(const v of proof.views){
  assert.equal(v.images.length,3);
  assert.deepEqual(v.images.map(im=>im.mode),['analytic','cache','cache-density-analytic-normal']);
  for(const im of v.images){
   assert.equal(im.bank,v.bank);assert.deepEqual(im.dimensions,[240,150]);assert.equal(im.steps,128);assert.equal(im.lighting_refresh_steps,4);
   assert.deepEqual(im.origin_local,v.view==='oblique'?[3250,2200,3600]:[3300,550,3800]);
   assert.deepEqual(im.target_local,v.view==='oblique'?[0,-90,0]:[0,-140,0]);
   assert.equal(im.orthographic_span,v.view==='oblique'?6200:5600);
   assert.equal(im.live_noise,'held at R=.70,G/B=.65,A=.50 for both fields');
  }
  assert(v.comparisons.cache_vs_analytic.opaque_alpha_gt_095.display_rmse<.025);
  assert(v.comparisons.cache_vs_analytic.edge_alpha_005_to_095.opacity_absolute.p50_p95_p99_max[2]<.08);
 }
 const report={passed:true,contract:contract.recipe,legacy_regional_pointwise_gate_pass:false,banks,gpu_bytes:total,fidelity_sha256:hash('docs/evidence/cloud-cache-fidelity.json')};
 fs.mkdirSync(path.join(root,'docs/evidence'),{recursive:true});
 fs.writeFileSync(path.join(root,'docs/evidence/cloud-cache-assets.json'),JSON.stringify(report,null,2)+'\n');
 console.log('Six exact 3D cloud caches, accepted r47 fidelity, historical high banks and 57 MiB GPU verified; old regional .035 gate remains failed.');
 return report;
}
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url))verify();
