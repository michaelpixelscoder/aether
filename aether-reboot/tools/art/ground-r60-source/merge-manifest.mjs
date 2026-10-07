// Selective R60 manifest fusion; deliberately has no filesystem or mutation API.
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
const sha=raw=>crypto.createHash('sha256').update(raw).digest('hex');
export function mergeGroundManifest(beforeRaw,patchManifest,{expectedInputSha}={}){
 assert.match(expectedInputSha??'',/^[a-f0-9]{64}$/,'Independent caller manifest SHA required');
 assert.equal(sha(beforeRaw),expectedInputSha,'Caller manifest authentication');
 const before=JSON.parse(beforeRaw.toString('utf8').replace(/^\uFEFF/,'')),merged=structuredClone(before);
 const operations=[];
 for(const kind of ['dawn','dawn-watch']){
  assert(before[kind]&&patchManifest[kind],'Both intended islands required');
  assert(!Object.hasOwn(before[kind],'ground_r60'),'Caller already has ground_r60; require a separate migration');
  const provenance=patchManifest[kind].ground_r60;
  assert.equal(provenance?.input_high_sha256,before[kind].high.sha256);
  assert.equal(provenance?.input_lod_sha256,before[kind].lod.sha256);
  for(const level of ['high','lod'])for(const field of ['sha256','bytes']){
   const value=patchManifest[kind][level][field];
   if(field==='sha256')assert.match(value,/^[a-f0-9]{64}$/);else assert(Number.isSafeInteger(value)&&value>0);
   operations.push({path:[kind,level,field],before:before[kind][level][field],after:value});
   merged[kind][level][field]=value;
  }
  merged[kind].ground_r60=structuredClone(provenance);
  operations.push({path:[kind,'ground_r60'],before_exists:false,after:structuredClone(provenance)});
 }
 // A proposal with unrelated metadata changes is rejected rather than transplanted.
 assert.deepEqual(merged,patchManifest,'Only four high/LOD bytes/SHA pairs and two ground_r60 fields may change');
 const recovered=structuredClone(merged);
 for(const kind of ['dawn','dawn-watch']){
  delete recovered[kind].ground_r60;
  for(const level of ['high','lod'])for(const field of ['sha256','bytes'])recovered[kind][level][field]=before[kind][level][field];
 }
 assert.deepEqual(recovered,before,'All other fields, histories and islands preserved');
 return{manifest:merged,operations,input_sha256:expectedInputSha,all_unrelated_metadata_exact:true};
}
