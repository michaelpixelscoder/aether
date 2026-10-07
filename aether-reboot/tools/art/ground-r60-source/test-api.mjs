import fs from 'node:fs';import path from 'node:path';import assert from 'node:assert/strict';import {fileURLToPath} from 'node:url';
import {validateGroundDelta,parseGroundGlb,GROUND_R60_IMAGE_URI} from './validate-ground-delta.mjs';
const author=path.dirname(fileURLToPath(import.meta.url)),root=path.resolve(author,'../../..');
const native=fs.readFileSync(path.join(author,'source/ground-r60-native.png'));
const before=fs.readFileSync(path.join(author,'history/r59-portable-world/dawn.glb'));
const after=fs.readFileSync(path.join(root,'assets/world/dawn.glb'));
const expectedInputSha='77c38267abdbce3208f891b1e472a6bc522eccdee7fcf5e62402a7721449acc4';
const call=bytes=>validateGroundDelta(before,bytes,{expectedInputSha,nativePNG:native});
const actual=call(after);assert.deepEqual(actual.projectedDoc,parseGroundGlb(before).doc);assert(actual.bin.equals(parseGroundGlb(after).bin));
function repack(doc,bin){let json=Buffer.from(JSON.stringify(doc));json=Buffer.concat([json,Buffer.alloc((4-json.length%4)%4,32)]);const header=Buffer.alloc(20),chunk=Buffer.alloc(8);header.writeUInt32LE(0x46546c67);header.writeUInt32LE(2,4);header.writeUInt32LE(28+json.length+bin.length,8);header.writeUInt32LE(json.length,12);header.writeUInt32LE(0x4e4f534a,16);chunk.writeUInt32LE(bin.length);chunk.writeUInt32LE(0x004e4942,4);return Buffer.concat([header,json,chunk,bin]);}
const jsonMutation=change=>{const {doc,bin}=parseGroundGlb(after);change(doc);return repack(doc,bin);};
const cases=[
 ['caller SHA authentication',()=>validateGroundDelta(before,after,{expectedInputSha:'0'.repeat(64),nativePNG:native}),/authenticated SHA/],
 ['mandatory caller authority',()=>validateGroundDelta(before,after,{nativePNG:native}),/mandatory/],
 ['original PNG',()=>{const changed=Buffer.from(native);changed[changed.length-1]^=1;validateGroundDelta(before,after,{expectedInputSha,nativePNG:changed});},/PNG exact/],
 ['BIN geometry',()=>{const {doc,bin}=parseGroundGlb(after);const changed=Buffer.from(bin);changed[0]^=1;call(repack(doc,changed));},/BIN byte-exact/],
 ['other material',()=>call(jsonMutation(d=>d.materials[0].doubleSided=!d.materials[0].doubleSided)),/All other/],
 ['soil roughness',()=>call(jsonMutation(d=>d.materials[11].pbrMetallicRoughness.roughnessFactor=.5)),/roughness/],
 ['hidden texture-info extension',()=>call(jsonMutation(d=>d.materials[11].pbrMetallicRoughness.baseColorTexture.extensions.forbidden={x:1})),/albedo index/],
 ['texCoord override',()=>call(jsonMutation(d=>d.materials[11].pbrMetallicRoughness.baseColorTexture.texCoord=1)),/albedo index/],
 ['normal map',()=>call(jsonMutation(d=>d.materials[11].normalTexture={index:0})),/roughness/],
 ['existing image mutation',()=>call(jsonMutation(d=>d.images[0].name='mutated')),/Expected values/],
 ['existing UV metadata',()=>call(jsonMutation(d=>d.accessors[57].count-=1)),/All other/],
 ['new image URI',()=>call(jsonMutation(d=>d.images.at(-1).uri=GROUND_R60_IMAGE_URI+'wrong')),/Expected values/],
 ['sampler override',()=>call(jsonMutation(d=>d.textures.at(-1).sampler=999)),/Expected values/],
 ['added extras',()=>call(jsonMutation(d=>d.asset.extras.unapproved='mutated')),/All other/]
];
const rejected=[];for(const [name,test,reason] of cases){assert.throws(test,reason,name+' must fail at intended delta boundary');rejected.push(name);}
const report={passed:true,real_r59_caller_authenticated:true,full_candidate_bin_exact:true,projection_only_after_full_delta_validation:true,corruptions_rejected:rejected};
fs.writeFileSync(path.join(author,'proofs/api-negative-tests.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
