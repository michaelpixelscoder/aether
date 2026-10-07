// Pure read-only succession API. Validates ACTUAL runtime before restoring R53.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
const sha=b=>crypto.createHash('sha256').update(b).digest('hex');
export const AUTHORITIES=Object.freeze({
 r53:'f3f2efc413e71ae5e941325d59c2a457b6863cfa79dd1478a44af475f678fb41',
 r62:'f5f7517a5123232cb2b6ee9f02527a9753840a3ebec152f72557289cd4869cc1',
 r64:'0861bdc4197dab68fb10826a6165fa6ef5e325fa52ad6d42331a6b65e30c5cf5',
 r66:'b76e67027f1b34cf6f9d56192bcdf315ad43810ff32c4e64e075297ab9a16c31',
 png:'eaf534efc2a2e30632fcfee7f3d5b731efe77dcd19f0849216e6c74c662497c3',
 currentKtx:'68755a59545f61bc047975e7af6c301bd98164468a6f293a1336e9bdbb5b1f05',
 waterfallKtx:'8ecd62eb7df01a02d9864521e63b1c29a128ff18a267f369a1d7d96cd73b4973',
 manifests:Object.freeze({r62:'255f6db336b8c1b25c71fd7c1344eaa89161215d0eb5d087bc83ba0b737effc1',r64:'afab9be09e7851fbeb0a8523196aae877a0551df65dc5970b92ff550301aaeb4',r66:'8a7c69d43300133365968ee2f7ebc6d4d0d9b6beeef3bfe16be58179515d63c7'})
});
function replaceOnce(text,before,after,label){assert.equal(text.split(before).length,2,label+' exact occurrence');return text.replace(before,after);}
export function createFlowValidator(root){
 root=path.resolve(root);const author=path.join(root,'tools/art/flow-current-source');const read=f=>fs.readFileSync(path.join(author,f));
 const legacyRaw=read('source/r53-authority.json');assert.equal(sha(legacyRaw),'a87afc92ef545ae719b2af0921ea555df234ca3937989d7a7a9c365d3a2bec8d','Entire original R53 metadata/source inventory');
 const legacy=JSON.parse(legacyRaw);
 for(const f of legacy.unchanged_historical_author_files){assert(!path.isAbsolute(f.file)&&!f.file.split(/[\\/]/).includes('..'),'legacy confinement');const bytes=fs.readFileSync(path.join(root,f.file));assert.equal(bytes.length,f.bytes);assert.equal(sha(bytes),f.sha256,'Historical R53 author metadata/source unchanged '+f.file);}
 assert.equal(sha(read('history/r53-verify.mjs')),legacy.original_verify_sha256,'Pristine R53 gate archived');assert.equal(sha(read('history/r53-container.json')),legacy.original_container_sha256,'Pristine R53 derived report archived');
 assert.equal(sha(fs.readFileSync(path.join(root,'tools/art/flow-source/verify.mjs'))),legacy.current_adapter_sha256,'Exact minimal R53 adapter');
 for(const [rev,digest] of Object.entries(AUTHORITIES.manifests)){
  const bytes=read(`archive/${rev}/package-manifest.json`);assert.equal(sha(bytes),digest,rev+' frozen manifest');
  for(const entry of JSON.parse(bytes).files){
   const relative=entry.path??'portable/'+entry.file;assert(!path.isAbsolute(relative)&&!relative.split(/[\\/]/).includes('..'),'archive path confinement');
   const data=read(`archive/${rev}/${relative}`);assert.equal(data.length,entry.bytes,rev+' archived bytes '+relative);assert.equal(sha(data),entry.sha256,rev+' archived authority '+relative);
  }
 }
 const r53=read('history/world-flow-r53.wgsl'),r62=read('archive/r62/portable/assets/shaders/world-flow.wgsl'),r64=read('archive/r64/portable/assets/shaders/world-flow.wgsl'),r66=read('archive/r66/portable/assets/shaders/world-flow.wgsl');
 for(const [rev,b] of Object.entries({r53,r62,r64,r66}))assert.equal(sha(b),AUTHORITIES[rev],rev+' entire shader preimage');
 const original=r53.toString('utf8').replaceAll('\r\n','\n');
 const currentBlock=read('archive/r62/portable/tools/art/flow-r62-source/source/current-block.wgsl').toString('utf8').replaceAll('\r\n','\n');
 const helpers=read('archive/r62/portable/tools/art/flow-r62-source/source/current-sampling.wgsl').toString('utf8').replaceAll('\r\n','\n');
 const begin=original.indexOf('    if settings.x < 0.5 {'),end=original.indexOf('    } else if settings.x < 1.5 {');assert(begin>=0&&end>begin,'R53 branch bounds');
 let forward=original.slice(0,begin)+currentBlock+original.slice(end);
 forward=replaceOnce(forward,'@fragment\n',helpers+'\n@fragment\n','R62 helper insertion');
 forward=replaceOnce(forward,'// R53 energy calibration medium; same native image, geometry and clock.','// R62 isolated coherent current transport; authentic current alpha delta, unchanged native texture/geometry/clock.','R62 label');
 assert(Buffer.from(forward).equals(r62),'True R53→R62 reconstruction, alpha/sampling delta explicit');
 // Independently reverse the actual R62 payload. No ignored metadata/branches.
 let recovered=replaceOnce(r62.toString('utf8'),helpers+'\n','','R62 helper restoration');
 recovered=replaceOnce(recovered,currentBlock,original.slice(begin,end),'R62 current restoration');
 recovered=replaceOnce(recovered,'// R62 isolated coherent current transport; authentic current alpha delta, unchanged native texture/geometry/clock.',original.split('\n')[0],'R62 header restoration');
 assert.equal(recovered,original,'Every non-current R53 source byte exact after documented LF normalization');
 const artAdaptations=[
  ['// R62 isolated coherent current transport; authentic current alpha delta, unchanged native texture/geometry/clock.','// R64 fresh coherent current art; separate texture handle, unchanged R62 optics and historical waterfalls/pools.'],
  ['        // Broad paths:~7 strong native crest groups per transverse section,\n        // versus~20 in R61; the compact thermal keeps its R61 crop scale.','        // The new native asset has three coherent unequal bands.\n        // Both broad paths and the thermal sample the full central42% crop.'],
  ['let crop=mix(.16,.42,thermal);','let crop=.42;'],['let crop_start=mix(.42,.29,thermal);','let crop_start=.29;'],
  ['// R62: true authored color samples are classified BEFORE either blend.','// R64 uses unchanged R62 optical sampling with the fresh current-only image.']
 ];
 for(const [before,after] of artAdaptations)forward=replaceOnce(forward,before,after,'R64 native adaptation');
 assert(Buffer.from(forward).equals(r64),'True R62→R64 reconstruction; only native matching UV/comments');
 forward=replaceOnce(forward,'let optical_profile=.12+1.88*middle;','let optical_profile=1.0;','R66 optical profile');
 assert(Buffer.from(forward).equals(r66),'True R64→R66 reconstruction; one radiance expression');
 const oldTail=original.slice(end);const actualTail=forward.slice(forward.indexOf('    } else if settings.x < 1.5 {'));
 assert.equal(actualTail,oldTail,'Cascades/pools/haze entire branches exact to R53');
 const oldSampler=original.slice(original.indexOf('fn liquid_sample('),original.indexOf('@fragment'));
 assert(forward.slice(forward.indexOf('fn liquid_sample(')).startsWith(oldSampler.trimEnd()),'Historical liquid_sample exact');
 const prefix=t=>t.slice(t.indexOf('@fragment'),t.indexOf('    if settings.x < 0.5 {'));
 assert.equal(prefix(forward),prefix(original),'Clock/uniform/vertex-alpha source exact to R53');
 for(const [relative,digest] of [['assets/textures/world-flow.ktx2',AUTHORITIES.waterfallKtx],['assets/textures/world-flow-current-r64.ktx2',AUTHORITIES.currentKtx]])assert.equal(sha(fs.readFileSync(path.join(root,relative))),digest,'Actual runtime texture '+relative);
 assert.equal(sha(read('source/world-flow-current-native.png')),AUTHORITIES.png,'Actual author PNG unchanged');
 assert(read('source/world-flow-current-native.png').equals(read('archive/r64/portable/tools/art/flow-r64-source/source/world-flow-current-native.png')),'Actual new native PNG authentic');
 for(const file of ['native-r64.json','prompt.txt','prompt-invocation.txt'])assert(read('source/'+file).equals(read('archive/r64/portable/tools/art/flow-r64-source/source/'+file)),'Actual new author metadata/tool string exact '+file);
 const reserved=new Set(JSON.parse(read('archive/r64/portable/tools/art/flow-r64-source/source/naga-wgsl-reserved-29.0.4.json')).words);
 return function validateRuntime(runtimeRaw){
  assert(Buffer.isBuffer(runtimeRaw)&&runtimeRaw.equals(r66),'Only full authentic final R66 runtime accepted');
  const code=runtimeRaw.toString('utf8').replace(/\/\*[\s\S]*?\*\//g,'').replace(/\/\/[^\n]*/g,'').replace(/^\s*#.*$/gm,'');for(const word of code.match(/[A-Za-z_][A-Za-z_0-9]*/g)??[])assert(!reserved.has(word),'Naga reserved identifier '+word);
  assert(!/\b(?:dpdx|dpdy|fwidth|discard)\s*\(/.test(helpers.replace(/\/\/[^\n]*/g,'')),'No helper derivative hazard');assert(!/\b(?:for|while|loop)\b/.test(helpers.replace(/\/\/[^\n]*/g,'')),'No helper loops');
  for(const n of [0,1,2,3])assert.equal(runtimeRaw.toString().split('@binding('+n+')').length-1,1);
  return {r53Raw:Buffer.from(r53),report:{schema:1,passed:true,runtime_shader_sha256:sha(runtimeRaw),restored_entire_r53_sha256:sha(r53),chain_shader_sha256:{r53:AUTHORITIES.r53,r62:AUTHORITIES.r62,r64:AUTHORITIES.r64,r66:AUTHORITIES.r66},current_alpha_delta_from_r53_explicit:true,current_uv_texture_sampling_radiance_deltas_explicit:true,legacy_r53_metadata_retained:true,noncurrent_r53_exact_after_explicit_lf_normalization:true,cascade_pool_sampler_clock_vertex_alpha_exact:true,original_waterfall_texture_sha256:AUTHORITIES.waterfallKtx,new_current_texture_sha256:AUTHORITIES.currentKtx,new_native_png_sha256:AUTHORITIES.png,current_fetches:5,cascade_fetches:2,coverage_exact_to_r64:true,r66_radiance_profile:'uniform1.0',production_binding_proof:'REQUIRED ROOT RUST AND NATIVE CAPTURE; THIS ART API DOES NOT PROVE HANDLE WIRING',gpu_performance:'UNMEASURED',visual_tier2:'NOT ESTABLISHED'}};
 };
}
export function validateAndRecoverR53({root,runtimeShader}){return createFlowValidator(root)(runtimeShader);}
