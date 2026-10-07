import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import {validateGroundDelta,parseGroundGlb,hasGroundR60,GROUND_R60_PNG_SHA,GROUND_R60_IMAGE_URI} from './validate-ground-delta.mjs';
const sha=b=>crypto.createHash('sha256').update(b).digest('hex');
const assemblySha='e6035440828c3c45753ff6c1c46feee4d6f263a2752a2fb4897f58ff27e575cd';
const lodSha={'dawn-lod.glb':'727e814e40a920ad23b0d7280566a8b40b0419f5e13bd232331992db4e646dc9','dawn-watch-lod.glb':'448ea68f0496c1e4d69346f53c618b8b04a9f0e751ac9dfd91e3f5af9f6e8902'};
export function validateGroundR60World({root,world=path.join(root,'assets/world'),sources=path.join(root,'tools/art')}={}){
 assert(root,'Explicit source root required');
 const author=path.join(sources,'ground-r60-source'),files=['dawn.glb','dawn-lod.glb','dawn-watch.glb','dawn-watch-lod.glb'];
 const assets=files.map(file=>({file,raw:fs.readFileSync(path.join(world,file))}));
 const flagged=assets.filter(a=>hasGroundR60(parseGroundGlb(a.raw).doc));
 if(!flagged.length)return{passed:true,applied:false,models:[]};
 assert.equal(flagged.length,4,'R60 must cover all4 intended HD/LOD models together');
 const native=fs.readFileSync(path.join(author,'source/ground-r60-native.png'));
 assert.equal(sha(native),GROUND_R60_PNG_SHA);assert.equal(sha(fs.readFileSync(path.join(world,GROUND_R60_IMAGE_URI))),GROUND_R60_PNG_SHA);
 const assemblyRaw=fs.readFileSync(path.join(author,'source/r59-assembly-evidence.json'));
 assert.equal(sha(assemblyRaw),assemblySha,'Frozen R59 caller evidence, not an R60-recentered baseline');
 const assembly=JSON.parse(assemblyRaw),manifest=JSON.parse(fs.readFileSync(path.join(world,'manifest.json'))),originalManifest=JSON.parse(fs.readFileSync(path.join(author,'history/r59-portable-world/manifest.json')));
 const models=[];
 for(const {file,raw} of assets){
  const kind=file.replace(/-lod\.glb$|\.glb$/,''),level=file.endsWith('-lod.glb')?'lod':'high';
  const expected=level==='lod'?lodSha[file]:assembly.islands.find(i=>i.kind===kind)?.sha256;
  const before=fs.readFileSync(path.join(author,'history/r59-portable-world',file));
  const result=validateGroundDelta(before,raw,{expectedInputSha:expected,nativePNG:native});
  assert.equal(manifest[kind][level].sha256,result.output_sha256);assert.equal(manifest[kind][level].bytes,raw.length);
  models.push({file,input_sha256:result.input_sha256,output_sha256:result.output_sha256,bin_byte_exact:true,all_unrelated_json_exact:true,target_material_index:result.target_material_index});
 }
 const recoveredManifest=structuredClone(manifest);
 for(const kind of ['dawn','dawn-watch']){
  const recovered=structuredClone(manifest[kind]),old=originalManifest[kind];
  const high=models.find(m=>m.file===kind+'.glb'),lod=models.find(m=>m.file===kind+'-lod.glb');
  const expectedMetadata={source_png_sha256:GROUND_R60_PNG_SHA,runtime_texture_uri:GROUND_R60_IMAGE_URI,
   native_dimensions:[1254,1254],base_color_factor:[1,1,1,1],uv_transform_scale:[2.1/(4*.11),2.1/(4*.11)],
   primary_instance_scale:2.1,primary_physical_tile_m:4,input_high_sha256:high.input_sha256,input_lod_sha256:lod.input_sha256,
   author:'tools/art/ground-r60-source/patch_ground.py',author_sha256:sha(fs.readFileSync(path.join(author,'patch_ground.py'))),
   delta_validator:'tools/art/ground-r60-source/validate-ground-delta.mjs',delta_validator_sha256:'6ff191df9ff34a8d5c5283ce111cb5db1c95088ba78cbb1827abc465130f6c8c',
   scope:'Material12 albedo binding/factor/texture transform only; BIN/geometry/UV/all unrelated material/images/history JSON exact'};
  assert.equal(sha(fs.readFileSync(path.join(author,'validate-ground-delta.mjs'))),expectedMetadata.delta_validator_sha256);
  assert.deepEqual(recovered.ground_r60,expectedMetadata,'R60 manifest provenance exactly matches authenticated caller and actual sources');
  delete recovered.ground_r60;
  for(const level of ['high','lod']){recovered[level].sha256=old[level].sha256;recovered[level].bytes=old[level].bytes;}
  assert.deepEqual(recovered,old,'Dawn/Watch R54/R56/R59/collision/history metadata exact; only output bytes/SHA change');
  recoveredManifest[kind]=recovered;
 }
 assert.deepEqual(recoveredManifest,originalManifest,'Entire manifest/history/unrelated islands exact after removing only four high/LOD hash/bytes pairs and ground_r60');
 const filtering=JSON.parse(fs.readFileSync(path.join(author,'source/native-runtime-filtering-study.json')));
 assert.equal(filtering.native_original_sha256,GROUND_R60_PNG_SHA);
 assert.equal(filtering.runtime_level_zero_sha256,'c0f56f9e7a9b15f8f39fe1024cc161e861db8108ba548c716d256f6b991b4c59');
 assert.equal(filtering.decoded_rgba_sha256,filtering.runtime_level_zero_sha256);
 assert.deepEqual(filtering.dimensions,[1254,1254]);assert.equal(filtering.mip_count,11);
 assert.equal(filtering.gpu_texel_bytes,8384072);assert.equal(filtering.linear_min_mag_mip_and_anisotropy8,true);
 assert.equal(filtering.glb_metadata_pre_portable_r59,true,'Earlier fixture must not become a final portable capture');
 return{passed:true,applied:true,models,source_png_sha256:GROUND_R60_PNG_SHA,r59_caller_evidence_sha256:assemblySha,geometry_uv_bin_exact:true,historical_metadata_exact:true,native_compile:'Separate production verify.mjs checks actual native captures; this material API does not',performance:'UNMEASURED',source_mip_count:1,prior_fixture_runtime_mip_count:11,prior_fixture_runtime_level0_exact:true,final_portable_runtime_filtering:'Separate production verify.mjs checks final surface report and level0 dump',runtime_filtering_evidence_sha256:sha(fs.readFileSync(path.join(author,'source/native-runtime-filtering-study.json')))};
}
