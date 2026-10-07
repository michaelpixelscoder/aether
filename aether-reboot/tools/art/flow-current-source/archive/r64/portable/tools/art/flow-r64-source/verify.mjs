// Independent PNG/container/source and historical-shader gate. No GPU.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import zlib from 'node:zlib';
import assert from 'node:assert/strict';
import {fileURLToPath} from 'node:url';
const author=path.dirname(fileURLToPath(import.meta.url)),root=path.resolve(author,'../../..');
const sha=b=>crypto.createHash('sha256').update(b).digest('hex'),read=f=>fs.readFileSync(path.join(author,f));
const source=read('source/world-flow-current-native.png'),metadata=JSON.parse(read('source/native-r64.json'));
const nativeSha='eaf534efc2a2e30632fcfee7f3d5b731efe77dcd19f0849216e6c74c662497c3';
const historicalKtxSha='8ecd62eb7df01a02d9864521e63b1c29a128ff18a267f369a1d7d96cd73b4973';
const current=fs.readFileSync(path.join(root,'assets/textures/world-flow-current-r64.ktx2')),historical=fs.readFileSync(path.join(root,'assets/textures/world-flow.ktx2'));
const shader=fs.readFileSync(path.join(root,'assets/shaders/world-flow.wgsl')).toString(),baseline=read('history/world-flow-r62.wgsl').toString();
const mip=JSON.parse(read('proofs/mip.json')),shaderProof=JSON.parse(read('proofs/shader.json'));
const table=Array.from({length:256},(_,i)=>{let c=i;for(let n=0;n<8;n++)c=(c>>>1)^((c&1)?0xedb88320:0);return c>>>0;});
function crc(b){let c=0xffffffff;for(const n of b)c=table[(c^n)&255]^(c>>>8);return(c^0xffffffff)>>>0;}
function png(raw){
 assert(raw.subarray(0,8).equals(Buffer.from([137,80,78,71,13,10,26,10])));
 let cursor=8,w,h;const chunks=[];let ended=false;
 while(cursor<raw.length){const n=raw.readUInt32BE(cursor);assert(cursor+n+12<=raw.length);const kind=raw.toString('ascii',cursor+4,cursor+8),data=raw.subarray(cursor+8,cursor+8+n);
  assert.equal(crc(raw.subarray(cursor+4,cursor+8+n)),raw.readUInt32BE(cursor+8+n),'Native PNG CRC');
  if(kind==='IHDR'){assert.equal(w,undefined);w=data.readUInt32BE(0);h=data.readUInt32BE(4);assert.deepEqual([...data.subarray(8)],[8,6,0,0,0]);}
  if(kind==='IDAT')chunks.push(data);cursor+=n+12;if(kind==='IEND'){ended=true;break;}
 }
 assert(ended&&cursor===raw.length&&w&&h);const stride=w*4,filtered=zlib.inflateSync(Buffer.concat(chunks)),rgba=Buffer.alloc(w*h*4);assert.equal(filtered.length,(stride+1)*h);
 for(let y=0;y<h;y++){const at=y*(stride+1),base=y*stride,filter=filtered[at];assert(filter<=4);
  for(let x=0;x<stride;x++){const l=x>=4?rgba[base+x-4]:0,u=y?rgba[base+x-stride]:0,ul=y&&x>=4?rgba[base+x-stride-4]:0;let add=0;
   if(filter===1)add=l;else if(filter===2)add=u;else if(filter===3)add=Math.floor((l+u)/2);else if(filter===4){const p=l+u-ul,a=Math.abs(p-l),b=Math.abs(p-u),c=Math.abs(p-ul);add=a<=b&&a<=c?l:b<=c?u:ul;}
   rgba[base+x]=(filtered[at+1+x]+add)&255;
  }
 }
 return{w,h,rgba};
}
function container(raw,decoded,proof){
 assert(raw.subarray(0,12).equals(Buffer.from([171,75,84,88,32,50,48,187,13,10,26,10])));
 const count=Math.floor(Math.log2(Math.max(decoded.w,decoded.h)))+1;
 assert.deepEqual(Array.from({length:9},(_,i)=>raw.readUInt32LE(12+i*4)),[43,1,decoded.w,decoded.h,0,0,1,count,0]);
 assert.equal(proof.mip_count,count);assert.deepEqual(proof.native_dimensions,[decoded.w,decoded.h]);
 const offset=raw.readUInt32LE(48),length=raw.readUInt32LE(52),dfd=raw.subarray(offset,offset+length);
 assert.equal(offset,80+count*24);assert.equal(length,92);assert.equal(dfd.readUInt32LE(0),92);
 assert.deepEqual([...dfd.subarray(12,28)],[1,1,2,0,0,0,0,0,4,0,0,0,0,0,0,0]);
 for(let c=0;c<4;c++){assert.equal(dfd.readUInt16LE(28+c*16),c*8);assert.equal(dfd[30+c*16],7);assert.equal(dfd[31+c*16],[0,1,2,31][c]);}
 const ranges=[];let bytes=0;
 for(let i=0;i<count;i++){const off=Number(raw.readBigUInt64LE(80+i*24)),n=Number(raw.readBigUInt64LE(88+i*24)),unpacked=Number(raw.readBigUInt64LE(96+i*24));
  const w=Math.max(1,Math.floor(decoded.w/2**i)),h=Math.max(1,Math.floor(decoded.h/2**i));assert.equal(off%8,0);assert(off>=offset+length&&off+n<=raw.length);assert.equal(n,w*h*4);assert.equal(n,unpacked);
  const payload=raw.subarray(off,off+n);assert.deepEqual(proof.levels[i].dimensions,[w,h]);assert.equal(proof.levels[i].gpu_bytes,n);assert.equal(proof.levels[i].offset,off);assert.equal(sha(payload),proof.levels[i].sha256);
  if(i===0)assert(payload.equals(decoded.rgba),'Native level0 byte exact, including hidden RGB and original alpha');ranges.push([off,off+n]);bytes+=n;
 }
 ranges.sort((a,b)=>a[0]-b[0]);for(let i=1;i<ranges.length;i++)assert(ranges[i-1][1]<=ranges[i][0],'No mip overlap');assert.equal(ranges.at(-1)[1],raw.length);assert.equal(bytes,proof.gpu_bytes);
 return{mips:count,dimensions:[decoded.w,decoded.h],gpu_bytes:bytes,level0_rgba_sha256:sha(decoded.rgba)};
}
const substitutions=[
 ['// R64 fresh coherent current art; separate texture handle, unchanged R62 optics and historical waterfalls/pools.','// R62 isolated coherent current transport; authentic current alpha delta, unchanged native texture/geometry/clock.'],
 ['        // The new native asset has three coherent unequal bands.\n        // Both broad paths and the thermal sample the full central42% crop.','        // Broad paths:~7 strong native crest groups per transverse section,\n        // versus~20 in R61; the compact thermal keeps its R61 crop scale.'],
 ['let crop=.42;','let crop=mix(.16,.42,thermal);'],['let crop_start=.29;','let crop_start=mix(.42,.29,thermal);'],
 ['// R64 uses unchanged R62 optical sampling with the fresh current-only image.','// R62: true authored color samples are classified BEFORE either blend.']
];
function checkShader(s){
 let projected=s;for(const [a,b] of substitutions){assert.equal(projected.split(a).length-1,1,'Only exact new-art UV adaptation permitted');projected=projected.replace(a,b);}assert.equal(projected,baseline,'No random R62 shader knob or historical branch change');
 const old=read('history/world-flow-r53.wgsl').toString().replaceAll('\r\n','\n');const tail=t=>t.slice(t.indexOf('    } else if settings.x < 1.5 {'));assert.equal(tail(s),tail(old),'All cascade/pool/haze source exact to R53');
 const sampling=t=>t.slice(t.indexOf('fn liquid_sample('),t.indexOf('@fragment'));assert(sampling(s).startsWith(sampling(old).trim()),'Historical cascade sampler remains exact');
 const reserved=new Set(JSON.parse(read('source/naga-wgsl-reserved-29.0.4.json')).words);const clean=s.replace(/\/\/[^\n]*/g,'').replace(/^\s*#.*$/gm,'');for(const word of clean.match(/[A-Za-z_][A-Za-z_0-9]*/g)??[])assert(!reserved.has(word));
}
function validate(p,k,h,s,meta=metadata,proof=mip){
 const decoded=png(p);assert.equal(sha(p),nativeSha);assert.equal(meta.sha256,nativeSha);assert.deepEqual([meta.width,meta.height],[decoded.w,decoded.h]);assert.equal(meta.channels,4);assert.equal(meta.bit_depth,8);
 assert.equal(sha(h),historicalKtxSha,'Old texture kept for cascade/pool handles');checkShader(s);return container(k,decoded,proof);
}
const measured=validate(source,current,historical,shader);
const cases=[];function reject(label,fn){assert.throws(fn,label);cases.push(label);}
const mutated=(b,offset,value)=>{const out=Buffer.from(b);out[offset]=value??(out[offset]^1);return out;};
reject('native PNG bytes/CRC',()=>validate(mutated(source,150),current,historical,shader));
reject('source SHA authority',()=>validate(source,current,historical,shader,{...metadata,sha256:'0'.repeat(64)}));
reject('native dimension metadata',()=>validate(source,current,historical,shader,{...metadata,width:metadata.width+1}));
reject('KTX Vulkan color format',()=>validate(source,mutated(current,12,37),historical,shader));
const dfdOffset=current.readUInt32LE(48);
reject('DFD sRGB transfer',()=>validate(source,mutated(current,dfdOffset+14,1),historical,shader));
reject('DFD alpha linear qualifier',()=>validate(source,mutated(current,dfdOffset+79,15),historical,shader));
reject('actual complete mip count',()=>validate(source,mutated(current,40,10),historical,shader));
reject('KTX native width',()=>validate(source,mutated(current,20),historical,shader));
const zero=Number(current.readBigUInt64LE(80)),next=Number(current.readBigUInt64LE(104));
reject('native level0 including alpha',()=>{const k=mutated(current,zero+3),proof=structuredClone(mip);proof.levels[0].sha256=sha(k.subarray(zero,zero+mip.levels[0].gpu_bytes));validate(source,k,historical,shader,metadata,proof);});
reject('derived payload byte identity',()=>validate(source,mutated(current,next),historical,shader));
reject('mip index overlap',()=>{const k=Buffer.from(current);k.writeBigUInt64LE(BigInt(next),80);validate(source,k,historical,shader);});
reject('mip index alignment',()=>{const k=Buffer.from(current);k.writeBigUInt64LE(BigInt(zero+1),80);validate(source,k,historical,shader);});
reject('container truncation',()=>validate(source,current.subarray(0,-1),historical,shader));
reject('original waterfall texture',()=>validate(source,current,mutated(historical,600),shader));
reject('project clock',()=>validate(source,current,historical,shader.replace('settings.w*settings.y','settings.w+settings.y')));
reject('waterfall alpha',()=>validate(source,current,historical,shader.replace('let opacity=.012+.62*crest','let opacity=.2+.62*crest')));
reject('pool wave',()=>validate(source,current,historical,shader.replace('sin(p.x*1.8','sin(p.x*3.8')));
reject('new asset UV crop',()=>validate(source,current,historical,shader.replace('let crop=.42;','let crop=.16;')));
reject('historical haze',()=>validate(source,current,historical,shader.replace('haze*0.8','haze*0.2')));
reject('reserved identifier',()=>validate(source,current,historical,shader.replace('let wave=','let patch=')));
assert.equal(cases.length,20);
assert.equal(sha(current),mip.derived_ktx2_sha256);assert.equal(sha(read('build_mips.py')),mip.generator_sha256);
assert.equal(sha(Buffer.from(shader)),shaderProof.shader_sha256);assert.equal(sha(read('build_shader.py')),shaderProof.generator_sha256);
assert.equal(sha(read('history/world-flow-r53.wgsl')),'f3f2efc413e71ae5e941325d59c2a457b6863cfa79dd1478a44af475f678fb41');
assert.equal(sha(read('history/world-flow-r61.wgsl')),'5e2e2c5b377ba30041a0493d0262d0dfc9c778d765548bdcd08c0990683d47d9');
assert.equal(sha(Buffer.from(baseline)),'f5f7517a5123232cb2b6ee9f02527a9753840a3ebec152f72557289cd4869cc1');
assert.equal(sha(read('source/prompt.txt')),metadata.prompt_sha256);assert.equal(sha(read('history/r53/world-flow-native.png')),'b0925b91ad5c3a8e3f17099b8262963620e806476c493453c0a358722134fc3b');
assert.equal(sha(read('source/prompt-invocation.txt')),metadata.invocation_prompt_sha256);
assert(read('source/prompt-invocation.txt').equals(read('source/prompt.txt').subarray(0,-1)),'Remove only the root-added final LF; preserve exact tool string');
assert.deepEqual(metadata.generation_parameters,{transparent_background:true});assert.deepEqual(metadata.omitted_parameters,['referenced_image_paths','num_last_images_to_include']);assert.equal(metadata.tool,'image_gen.imagegen');
const report={passed:true,...measured,native_png_sha256:nativeSha,current_ktx_sha256:sha(current),shader_sha256:sha(Buffer.from(shader)),new_current_only_asset:true,historical_cascade_pool_shader_sampler_texture_exact:true,current_handle_wiring:'PENDING root diagnostic override; bytes alone do not prove runtime binding',negative_cases_rejected:cases,gpu_compile:'PENDING',performance:'UNMEASURED',promotion_authorized:false};
fs.writeFileSync(path.join(author,'proofs/contracts.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
