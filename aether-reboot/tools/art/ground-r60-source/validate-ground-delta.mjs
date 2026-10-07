// Independent JS structural audit, no Python generator imports or mutation.
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
export const GROUND_R60_PNG_SHA='087deb4c68767e876653edc1b1fd0890b0ce5cb051beb5a29d03de718806b365';
export const GROUND_R60_IMAGE_URI=`textures/${GROUND_R60_PNG_SHA}.png`;
export const GROUND_R60_UV_SCALE=2.1/(4*.11);
const sha=b=>crypto.createHash('sha256').update(b).digest('hex');
export function parseGroundGlb(raw){
 assert(Buffer.isBuffer(raw)||raw instanceof Uint8Array,'GLB bytes required');
 const bytes=Buffer.from(raw);assert(bytes.length>=28);
 assert.equal(bytes.readUInt32LE(0),0x46546c67);assert.equal(bytes.readUInt32LE(4),2);assert.equal(bytes.readUInt32LE(8),bytes.length);
 const n=bytes.readUInt32LE(12);assert.equal(n%4,0);assert(20+n+8<=bytes.length);assert.equal(bytes.readUInt32LE(16),0x4e4f534a);
 assert.equal(bytes.readUInt32LE(24+n),0x004e4942);assert.equal(bytes.readUInt32LE(20+n)+28+n,bytes.length);
 return{raw:bytes,doc:JSON.parse(bytes.subarray(20,20+n)),bin:bytes.subarray(28+n)};
}
export function hasGroundR60(doc){
 return(doc.materials??[]).some(m=>m.name?.startsWith('12 | Surface biome')&&doc.images?.[doc.textures?.[m.pbrMetallicRoughness?.baseColorTexture?.index]?.source]?.uri===GROUND_R60_IMAGE_URI);
}
export function validateGroundDelta(beforeRaw,afterRaw,{expectedInputSha,nativePNG}={}){
 assert(typeof expectedInputSha==='string'&&/^[a-f0-9]{64}$/.test(expectedInputSha),'Authenticated caller expectedInputSha is mandatory; no baseline inference');
 assert(Buffer.isBuffer(nativePNG)||nativePNG instanceof Uint8Array,'Original native PNG bytes required');
 assert.equal(sha(nativePNG),GROUND_R60_PNG_SHA,'Original ImageGen PNG exact');
 const before=parseGroundGlb(beforeRaw),after=parseGroundGlb(afterRaw);
 assert.equal(sha(before.raw),expectedInputSha,'Caller R59 GLB must match independently authenticated SHA');
 assert(after.bin.equals(before.bin),'Entire BIN byte-exact; no UV/geometry/index/COLOR_0 waiver');
 const original=before.doc,doc=after.doc;
 const ids=original.materials.map((m,i)=>m.name?.startsWith('12 | Surface biome')?i:-1).filter(i=>i>=0);
 assert.equal(ids.length,1,'Exactly one material12');const target=ids[0];
 assert.equal(doc.materials.length,original.materials.length);
 const material=structuredClone(doc.materials[target]),oldMaterial=original.materials[target];
 const pbr=material.pbrMetallicRoughness,oldPbr=oldMaterial.pbrMetallicRoughness;
 assert.deepEqual(pbr.baseColorFactor,[1,1,1,1]);
 const info=pbr.baseColorTexture;assert.equal(info.index,original.textures.length);
 assert.deepEqual(info.extensions?.KHR_texture_transform,{scale:[GROUND_R60_UV_SCALE,GROUND_R60_UV_SCALE]});
 // Reconstruct the exact permitted texture-info patch, preserving all other
 // fields and refusing hidden changes in other extensions or texCoord.
 const expectedInfo=structuredClone(oldPbr.baseColorTexture);
 assert(!expectedInfo.extensions?.KHR_texture_transform,'Caller already transformed; explicit migration required');
 expectedInfo.index=original.textures.length;
 expectedInfo.extensions={...(expectedInfo.extensions??{}),KHR_texture_transform:{scale:[GROUND_R60_UV_SCALE,GROUND_R60_UV_SCALE]}};
 assert.deepEqual(info,expectedInfo,'Only albedo index and explicit uniform texture transform');
 const expectedTexture=structuredClone(original.textures[oldPbr.baseColorTexture.index]);
 assert(!expectedTexture.extensions,'Unexpected caller texture extensions');
 expectedTexture.source=original.images.length;expectedTexture.name='R60 grass moss original PNG';
 assert.equal(doc.textures.length,original.textures.length+1);assert.deepEqual(doc.textures.slice(0,-1),original.textures);assert.deepEqual(doc.textures.at(-1),expectedTexture);
 assert.equal(doc.images.length,original.images.length+1);assert.deepEqual(doc.images.slice(0,-1),original.images);
 assert.deepEqual(doc.images.at(-1),{mimeType:'image/png',name:'R60 natural grass moss native',uri:GROUND_R60_IMAGE_URI});
 const expectedExtensions=[...(original.extensionsUsed??[])];if(!expectedExtensions.includes('KHR_texture_transform'))expectedExtensions.push('KHR_texture_transform');
 assert.deepEqual(doc.extensionsUsed,expectedExtensions);
 pbr.baseColorTexture=structuredClone(oldPbr.baseColorTexture);
 if('baseColorFactor' in oldPbr)pbr.baseColorFactor=structuredClone(oldPbr.baseColorFactor);else delete pbr.baseColorFactor;
 assert.deepEqual(material,oldMaterial,'Material12 roughness/normal/metallic/alpha/other properties exact');
 const projectedDoc=structuredClone(doc);
 projectedDoc.materials[target]=structuredClone(oldMaterial);
 projectedDoc.images=structuredClone(original.images);
 projectedDoc.textures=structuredClone(original.textures);
 if('extensionsUsed' in original)projectedDoc.extensionsUsed=structuredClone(original.extensionsUsed);else delete projectedDoc.extensionsUsed;
 assert.deepEqual(projectedDoc,original,'All other materials/meshes/accessors/nodes/images/history JSON exact');
 return{passed:true,input_sha256:sha(before.raw),output_sha256:sha(after.raw),native_png_sha256:GROUND_R60_PNG_SHA,target_material_index:target,bin_byte_exact:true,all_unrelated_json_exact:true,projectedDoc,bin:after.bin};
}
