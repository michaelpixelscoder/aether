import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import zlib from 'node:zlib';
import assert from 'node:assert/strict';
import {fileURLToPath} from 'node:url';
const defaultRoot=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../../..');
const root=process.argv[2]==='--root'?path.resolve(process.argv[3]):defaultRoot;
const author=path.join(root,'tools/art/flow-source');
const sha=b=>crypto.createHash('sha256').update(b).digest('hex');
const read=f=>fs.readFileSync(path.join(root,f));
const readArt=f=>fs.readFileSync(path.join(author,f));
const proof=JSON.parse(readArt('proofs/mip.json'));
const png=readArt('source/world-flow-native.png'),ktx=read('assets/textures/world-flow.ktx2');
assert.equal(sha(png),'b0925b91ad5c3a8e3f17099b8262963620e806476c493453c0a358722134fc3b','Pinned native ImageGen art');
assert.equal(sha(png),proof.native_png_sha256);assert.equal(sha(ktx),proof.derived_ktx2_sha256);
assert.equal(sha(readArt('build_mips.py')),proof.generator_sha256);
assert(png.subarray(0,8).equals(Buffer.from([137,80,78,71,13,10,26,10])));
let cursor=8,width,height;const idat=[];
while(cursor<png.length){const n=png.readUInt32BE(cursor),kind=png.toString('ascii',cursor+4,cursor+8),data=png.subarray(cursor+8,cursor+8+n);if(kind==='IHDR'){width=data.readUInt32BE(0);height=data.readUInt32BE(4);assert.deepEqual([...data.subarray(8)],[8,6,0,0,0]);}if(kind==='IDAT')idat.push(data);cursor+=n+12;}
assert.equal(cursor,png.length);assert.equal(width,1254);assert.equal(height,1254);
const filtered=zlib.inflateSync(Buffer.concat(idat)),stride=width*4,decoded=Buffer.alloc(width*height*4);
assert.equal(filtered.length,(stride+1)*height);
for(let y=0;y<height;y++){
 const base=y*stride,filter=filtered[y*(stride+1)];assert(filter<=4);
 for(let x=0;x<stride;x++){
  const left=x>=4?decoded[base+x-4]:0,up=y?decoded[base+x-stride]:0,ul=y&&x>=4?decoded[base+x-stride-4]:0;
  let add=0;if(filter===1)add=left;else if(filter===2)add=up;else if(filter===3)add=Math.floor((left+up)/2);else if(filter===4){const p=left+up-ul,a=Math.abs(p-left),b=Math.abs(p-up),c=Math.abs(p-ul);add=a<=b&&a<=c?left:b<=c?up:ul;}
  decoded[base+x]=(filtered[y*(stride+1)+1+x]+add)&255;
 }
}
assert.equal(sha(decoded),proof.native_rgba_sha256);
assert(ktx.subarray(0,12).equals(Buffer.from([171,75,84,88,32,50,48,187,13,10,26,10])));
assert.deepEqual(Array.from({length:9},(_,i)=>ktx.readUInt32LE(12+i*4)),[43,1,1254,1254,0,0,1,11,0]);
const dfdOffset=ktx.readUInt32LE(48),dfdLength=ktx.readUInt32LE(52),dfd=ktx.subarray(dfdOffset,dfdOffset+dfdLength);
assert.equal(dfdOffset,80+24*11);assert.equal(dfdLength,92);assert.equal(dfd.readUInt32LE(0),92);
assert.deepEqual([...dfd.subarray(12,28)],[1,1,2,0,0,0,0,0,4,0,0,0,0,0,0,0]);
for(let c=0;c<4;c++){assert.equal(dfd.readUInt16LE(28+c*16),c*8);assert.equal(dfd[30+c*16],7);assert.equal(dfd[31+c*16],[0,1,2,31][c]);}
let total=0;const levels=[];
for(let i=0;i<11;i++){
 const offset=Number(ktx.readBigUInt64LE(80+i*24)),length=Number(ktx.readBigUInt64LE(88+i*24)),unpacked=Number(ktx.readBigUInt64LE(96+i*24));
 const size=Math.max(1,Math.floor(1254/2**i)),payload=ktx.subarray(offset,offset+length);
 assert.equal(offset%8,0);assert(offset>=dfdOffset+dfdLength);assert.equal(length,size*size*4);assert.equal(length,unpacked);assert.equal(payload.length,length);
 assert.deepEqual(proof.levels[i].dimensions,[size,size]);assert.equal(sha(payload),proof.levels[i].sha256);
 if(i===0)assert(payload.equals(decoded),'Level zero must be exact decoded native PNG');
 levels.push({level:i,dimensions:[size,size],bytes:length,offset});total+=length;
}
const ordered=[...levels].sort((a,b)=>a.offset-b.offset);
for(let i=1;i<ordered.length;i++)assert(ordered[i-1].offset+ordered[i-1].bytes<=ordered[i].offset);
assert.equal(ordered.at(-1).offset+ordered.at(-1).bytes,ktx.length);assert.equal(total,8384072);assert.equal(total,proof.gpu_bytes);
assert.equal(proof.tests_passed.length,4);
const shader=read('assets/shaders/world-flow.wgsl').toString(),old=readArt('history/prior-r52.wgsl').toString();
assert.equal(sha(Buffer.from(old)),'69078509c08b01ca11e31fdbde632902624daca7bee34c30e7558d65dd74f7fc','Pinned prior shader, no baseline recentering');
for(const binding of [0,1,2,3])assert.equal(shader.split('@binding('+binding+')').length-1,1);
const reserved=JSON.parse(readArt('source/naga-wgsl-reserved-29.0.4.json'));
const forbidden=new Set(reserved.words);
const checkIdentifiers=source=>{
 const code=source.replace(/\/\*[\s\S]*?\*\//g,'').replace(/\/\/[^\n]*/g,'').replace(/^\s*#.*$/gm,'');
 for(const token of code.match(/[A-Za-z_][A-Za-z_0-9]*/g)??[])assert(!forbidden.has(token),`Naga ${reserved.naga_version} reserved word: ${token}`);
};
checkIdentifiers(shader);
assert.throws(()=>checkIdentifiers(shader.replaceAll('let ridge=','let patch=')),'Reserved-word guard must catch the actual rejected identifier');
const normalized=s=>s.replace(/\/\/[^\n]*/g,'').replace(/\s/g,'');
assert.equal(normalized(shader.slice(shader.indexOf('let p=in.world_position.xz;'))),normalized(old.slice(old.indexOf('let p=in.world_position.xz;'))));
assert(shader.includes('let t=settings.w*settings.y;')&&shader.includes('alpha*=in.color.a;')&&!shader.includes('in.color.r'));
assert.equal((shader.match(/textureSampleGrad\(/g)??[]).length,2);
const shaderProof=JSON.parse(readArt('proofs/shader.json'));
const materialProof=JSON.parse(readArt('proofs/material.json'));
const medium=readArt('history/reviewed-medium.wgsl').toString();
assert.equal(sha(Buffer.from(shader)),'f3f2efc413e71ae5e941325d59c2a457b6863cfa79dd1478a44af475f678fb41','Final reviewed runtime shader');
assert.equal(sha(Buffer.from(shader)),shaderProof.shader_sha256);
assert.equal(sha(readArt('build_shader.py')),shaderProof.generator_sha256);
assert.equal(sha(Buffer.from(medium)),shaderProof.medium_source_sha256);
assert.equal(sha(Buffer.from(medium)),'3d9432e720abcae6ab54c3c0ebe7fa9081d2394d178f30ee879502ffa0c85d61');
const violetBlock=`        // The native red/green ratio selects a few authored crest portions.
        // A ridge-only gate leaves the water body and all alpha unchanged.
        let violet_ink=smoothstep(.18,.32,tex.r/max(tex.g,.0005));
        let violet_weight=violet_ink*smoothstep(.035,.095,ridge)*.95;
        let crest_hue=mix(hue,vec3<f32>(.58,.12,1.0),violet_weight);
        rgb=tex.rgb*.40+crest_hue*ridge*14.0/opacity;`;
const lfShader=shader.replace(/\r\n/g,'\n');
assert(lfShader.includes(violetBlock),'Exact narrow crest hue patch');
assert.equal(normalized(lfShader.replace(violetBlock,'        rgb=tex.rgb*.40+hue*ridge*14.0/opacity;')),normalized(medium),'No changes outside current hue block');
assert.equal(materialProof.shader_sha256,shaderProof.shader_sha256);
assert.equal(materialProof.review_source_sha256,sha(readArt('review_material.py')));
assert.equal(materialProof.sampling_source_sha256,sha(readArt('liquid_probe.py')));
assert.equal(materialProof.energy_source_sha256,sha(readArt('energy_probe.py')));
assert(materialProof.body_alpha_uv_cascade_unchanged);
assert.equal(materialProof.mips.length,9);
for(const mip of materialProof.mips){assert(mip.alpha_exact_medium);assert(mip.water_body_rgb_exact);}
const report={schema:2,passed:true,native_png_sha256:sha(png),shader_sha256:sha(Buffer.from(shader)),derived_ktx2_sha256:sha(ktx),native_level0_byte_exact:true,rgba_srgb_straight_alpha_dfd:true,levels,gpu_bytes:total,clock_vertex_alpha_pool_haze_preserved:true,only_current_hue_changed_from_medium:true,material_proof_sha256:sha(readArt('proofs/material.json')),naga_reserved_words_checked:reserved.words.length,naga_version:reserved.naga_version,reserved_patch_negative_test:true,native_shader_compilation:'Parent capture world-r53-violet.png was reviewed and accepted; static keyword guard itself is not a complete Naga parser.'};
fs.writeFileSync(path.join(author,'proofs/container.json'),JSON.stringify(report,null,2)+'\n');
console.log('Native PNG independently decoded; KTX2 level0 exact,11 sRGB/straight-alpha levels,8,384,072 GPU bytes; shader contracts preserved.');
