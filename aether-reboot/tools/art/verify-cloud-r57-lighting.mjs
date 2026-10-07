// Independent current R57 fixed-ray proof; never rewrite or relabel R55 evidence.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import {fileURLToPath} from 'node:url';
const rootDefault=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../..');
const recipe='r57-blue-fill-warm-direct';
const limits={gpu_bytes:59768832,density_mean:.006,regional_density_p99:.05,high_density_p99:.035,analytic_derivative_p99_degrees:.05,opaque_display_rmse:.025,edge_opacity_p99:.08};
export function verify(root=rootDefault,{write=true}={}) {
 const bytes=f=>fs.readFileSync(path.join(root,f)),text=f=>bytes(f).toString();
 const hash=f=>crypto.createHash('sha256').update(bytes(f)).digest('hex'),read=f=>JSON.parse(text(f));
 const contract=read('tools/art/cloud-r57/contract.json');
 assert.equal(contract.recipe,recipe);assert.equal(contract.schema,1);
 assert.deepEqual(contract.limits,limits,'Inherited acceptance limits must remain unchanged');
 assert.deepEqual(contract.lighting,{ambient_rgb_gain:[4,4.5,5],direct_radiance:[3,1.86,.96],bounce_radiance:[3,1.86,.96],bounce_strength:.35,phase_g:.45,bounce_elevation_degrees:42,shadow_taps_per_lobe:4,lighting_refresh_steps:[4,2],daylight_factor:1});
 assert.equal(contract.shader_sha256,'1e586a174543ea72cae64fa7399b5ecd863b8785e12b2a65b177366063375f54');
 assert.equal(hash('assets/shaders/clouds.wgsl'),contract.shader_sha256);
 // The compatibility adapter may change paths, never historical acceptance logic.
 let adapter=text('tools/art/cloud-r57/archive/verify-cloud-lighting-r55.mjs.txt').replaceAll('\r\n','\n');
 for(const [before,after] of [
  ["from './build_cloud_lighting.mjs'","from '../build_cloud_lighting.mjs'"],
  ["from './verify-cloud-cache.mjs'","from '../verify-cloud-cache.mjs'"],
  ["),'../..');","),'../../..');"],
  ["const shaderFile='assets/shaders/clouds.wgsl';","const shaderFile='tools/art/cloud-r57/archive/clouds-r55.wgsl';"],
  ["hash(f),expected,`${f}: stale R55 source proof`","hash(f==='tools/art/verify-cloud-lighting.mjs'?'tools/art/cloud-r57/archive/verify-cloud-lighting-r55.mjs.txt':f),expected,`${f}: stale R55 source proof`"],
  ['// Distinct R55 proof. R47 remains an immutable, separately named historical gate.',
   '// Compatibility adapter validates R55 only against archived R55 shader/verifier bytes.\n// Its contract, rays, renderer and all historical evidence remain unchanged.']]) {
  assert.equal(adapter.split(before).length,2,'Exactly one historical path/comment adaptation');adapter=adapter.replace(before,after);
 }
 assert.equal(text('tools/art/cloud-r57/verify-r55-historical.mjs').replaceAll('\r\n','\n'),adapter,'Historical R55 acceptance logic cannot change');
 const frozen='tools/art/cloud-r57/archive/clouds-r55.wgsl';
 assert.equal(hash(frozen),'ffcc09fbaea8702ddcab1938c3ee1e2c3c8c5431a765a1c00002b2f7ef9119b6');
 let restored=text('assets/shaders/clouds.wgsl');
 for(const [after,before] of [['h)*.65*vec3<f32>(4.0,4.5,5.0)','h)*.65'],['let direct=vec3<f32>(3.0,1.86,.96)*scattering','let direct=vec3<f32>(3.0,2.55,1.95)*scattering']]) {
  assert.equal(restored.split(after).length,2,'Exactly one reviewed radiance substitution');restored=restored.replace(after,before);
 }
 assert.equal(restored,text(frozen),'All shader bytes except two radiance substitutions must remain R55 exact');
 let renderer=text('tools/art/review_cloud_light_r57_cpu.py');
 for(const [after,before] of [
  ['R57 freshly recomputed matched CPU volumetric inspection','R55 matched CPU volumetric inspection'],
  ["OUT=ROOT/'tools/art/cloud-r57/review'","OUT=ROOT/'tools/art/cloud-r55/review'"],
  [')*.65*np.array([4.,4.5,5.])',')*.65'],
  ['direct=np.array([3.,1.86,.96])','direct=np.array([3.,2.55,1.95])'],
  ["'lighting_recipe':'r57-blue-fill-warm-direct'","'lighting_recipe':'r55-directional-reflected-light'"]]) {
  assert.equal(renderer.split(after).length,2,'Exactly one CPU radiance/metadata substitution');renderer=renderer.replace(after,before);
 }
 assert.equal(renderer.replaceAll('\r\n','\n'),text('tools/art/review_cloud_light_cpu.py').replaceAll('\r\n','\n'),'CPU executable text except radiance/metadata and Windows newline serialization must remain exact');
 const sources=read('tools/art/cloud-r57/sources.json');
 for(const [f,expected] of Object.entries(sources))assert.equal(hash(f),expected,`${f}: stale independent R57 source/artifact`);
 for(const f of ['tools/art/review_cloud_light_r57_cpu.py','tools/art/compare_cloud_light_r57_rays_cpu.py','tools/art/verify-cloud-r57-lighting.mjs','tools/art/cloud-r57/verify-r55-historical.mjs','tools/art/verify-cloud-lighting.mjs','tools/art/cloud-r57/test-gate.mjs'])assert(f in sources);
 const rays=read('docs/evidence/cloud-r57-ray-comparison.json');
 assert.equal(rays.schema,1);assert.equal(rays.recipe,recipe);
 for(const [key,file] of Object.entries({generator_sha256:'tools/art/build_cloud_cache.py',renderer_sha256:'tools/art/review_cloud_light_r57_cpu.py',comparison_sha256:'tools/art/compare_cloud_light_r57_rays_cpu.py',shader_sha256:'assets/shaders/clouds.wgsl',r47_contract_sha256:'tools/art/cloud-r47/contract.json',historical_ray_comparison_sha256:'docs/evidence/cloud-r47-ray-comparison.json',r55_ray_comparison_sha256:'docs/evidence/cloud-r55-ray-comparison.json',config_sha256:'assets/atmosphere/cloud-banks.json'}))assert.equal(rays[key],hash(file),`${key}: stale R57 ray computation`);
 assert(rays.note.includes('Fresh CPU recomputation')&&rays.note.includes('no R55 pixel rescaling or relabelling'));
 assert.deepEqual(rays.dimensions,[240,150]);assert.equal(rays.views.length,8);
 assert.deepEqual(rays.views.map(v=>`${v.bank}/${v.view}/${v.lighting_refresh_steps}`),[4,2].flatMap(refresh=>['hollow','underforge'].flatMap(bank=>['oblique','grazing'].map(view=>`${bank}/${view}/${refresh}`))));
 const historical=read('docs/evidence/cloud-r55-ray-comparison.json'),summaries=[];
 for(const v of rays.views) {
  assert.deepEqual(v.images.map(im=>im.mode),['analytic','cache','cache-density-analytic-normal']);
  const old=historical.views.find(h=>h.bank===v.bank&&h.view===v.view&&h.lighting_refresh_steps===v.lighting_refresh_steps);assert(old);
  v.images.forEach((im,i)=>{
   const prior=old.images[i];
   assert.equal(im.lighting_recipe,recipe);assert.equal(im.bank,v.bank);assert.deepEqual(im.dimensions,[240,150]);
   assert.equal(im.steps,128);assert.equal(im.lighting_refresh_steps,v.lighting_refresh_steps);
   for(const k of ['origin_local','target_local','orthographic_span','sun_direction','bounce_direction','shadow_taps_per_lobe','daylight_factor','live_noise','alpha_mean'])assert.deepEqual(im[k],prior[k],`${k}: only radiance may change`);
  });
  const c=v.comparisons.cache_vs_analytic;
  for(const category of ['all','opaque_alpha_gt_095','edge_alpha_005_to_095','empty_alpha_lte_005']) {
   assert(c[category]);assert(Number.isFinite(c[category].display_rmse));
   assert(c[category].opacity_absolute.p50_p95_p99_max.every(Number.isFinite));
   assert.deepEqual(c[category].opacity_absolute,old.comparisons.cache_vs_analytic[category].opacity_absolute,'All ray opacity error distributions must remain R55 exact');
  }
  const rmse=c.opaque_alpha_gt_095.display_rmse,alpha=c.edge_alpha_005_to_095.opacity_absolute.p50_p95_p99_max[2];
  assert(rmse<limits.opaque_display_rmse,`${v.bank}/${v.view}: opaque RMSE exceeds .025`);
  assert(alpha<limits.edge_opacity_p99,`${v.bank}/${v.view}: edge alpha p99 exceeds .08`);
  summaries.push({bank:v.bank,view:v.view,lighting_refresh_steps:v.lighting_refresh_steps,opaque_display_rmse:rmse,edge_opacity_p99:alpha});
 }
 for(const [f,expected] of Object.entries(contract.historical_unchanged))assert.equal(hash(f),expected,`${f}: historical evidence changed`);
 const report={schema:1,passed:true,recipe,shader_sha256:contract.shader_sha256,contract_sha256:hash('tools/art/cloud-r57/contract.json'),ray_comparison_sha256:hash('docs/evidence/cloud-r57-ray-comparison.json'),historical_evidence_preserved:true,shader_density_integration_exact_r55:true,cpu_density_rays_phase_integration_exact_r55:true,legacy_regional_pointwise_gate_pass:false,limits,views:summaries,maximum_cpu_threads:4,cpu_wall_seconds:rays.duration_seconds,limitations:['Fixed-noise orthographic CPU model, not native scene fidelity or performance.','R55 display transform retained; native ACES/bloom/opaque depth/live advection not modeled.','Freshly recomputed R57 values, no rescaling or relabelling of historical pixels.']};
 if(write)fs.writeFileSync(path.join(root,'docs/evidence/cloud-r57-lighting-fidelity.json'),JSON.stringify(report,null,2)+'\n');
 console.log('R57 freshly recomputed analytic/cache rays: 8 views pass inherited .025 RMSE/.08 opacity limits; R47/R55 history preserved.');return report;
}
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url))verify();
