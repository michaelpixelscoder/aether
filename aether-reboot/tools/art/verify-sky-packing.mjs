// Independent decoder and exhaustive equality gate for the runtime sky format.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
const argument = name => {
  const i=process.argv.indexOf(name);
  if(i<0)return undefined;
  assert(process.argv[i+1]&&!process.argv[i+1].startsWith('--'));
  return process.argv[i+1];
};
const root=path.resolve(argument('--root')??path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../..'));
const resolve=file=>{
  const result=path.resolve(root,file);
  assert(result.startsWith(root+path.sep),'Path escapes workspace');
  return result;
};
const read=file=>fs.readFileSync(resolve(file));
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const manifest=JSON.parse(read('tools/art/sky-source/manifest.json'));
const proof=JSON.parse(read('tools/art/sky-source/runtime-packing.json'));
const original=read(manifest.output), packed=read('assets/textures/sky-world-linear.ktx2');
assert.equal(hash(original),manifest.output_sha256);
assert.equal(hash(original),proof.source_sha256);
assert.equal(hash(packed),proof.output_sha256);
assert.equal(hash(read('tools/art/pack_sky.py')),proof.generator_sha256);
assert.deepEqual(proof.dimensions,manifest.dimensions);
assert.equal(proof.max_absolute_error,0);
assert.equal(proof.lossless,true);
assert.equal(proof.resampled,false);
assert.equal(proof.opaque_alpha_exact,true);
assert.equal(proof.vk_format,123);
assert.equal(proof.runtime_format,'Rgb9e5Ufloat');
assert.equal(proof.source_format,'RGBA16Float');

const half=new Float64Array(65536);
for(let n=0;n<65536;n++){
  const exponent=(n>>10)&31, mantissa=n&1023;
  half[n]=(n&32768?-1:1)*(exponent===31?NaN:exponent===0?mantissa*2**-24:(1+mantissa/1024)*2**(exponent-15));
}
const scales=Array.from({length:32},(_,i)=>2**(i-24));
const signature=Buffer.from([171,75,84,88,32,50,48,187,13,10,26,10]);
function check(runtime){
  assert(runtime.length>=304,'Truncated container');
  assert(runtime.subarray(0,12).equals(signature));
  const [width,height]=manifest.dimensions;
  assert.deepEqual(Array.from({length:9},(_,i)=>runtime.readUInt32LE(12+i*4)),
    [123,4,width,height,0,0,1,1,0],'Native linear RGB9E5 header');
  assert.equal(runtime.readBigUInt64LE(64),0n);
  assert.equal(runtime.readBigUInt64LE(72),0n);
  assert.equal(runtime.readUInt32LE(48),104);
  assert.equal(runtime.readUInt32LE(52),124);
  assert.equal(runtime.readUInt32LE(56),228);
  const dfd=runtime.subarray(104,228);
  assert.equal(dfd.readUInt32LE(0),124);
  assert.equal(dfd.readUInt32LE(4),0);
  assert.equal(dfd.readUInt16LE(8),2);
  assert.equal(dfd.readUInt16LE(10),120);
  assert.deepEqual([...dfd.subarray(12,28)],[1,1,1,0,0,0,0,0,4,0,0,0,0,0,0,0]);
  for(let channel=0;channel<3;channel++){
    const at=28+channel*32;
    assert.equal(dfd.readUInt16LE(at),channel*9);
    assert.equal(dfd[at+2],8);assert.equal(dfd[at+3],channel);
    assert.equal(dfd.readUInt32LE(at+4),0);
    assert.equal(dfd.readUInt32LE(at+8),0);
    assert.equal(dfd.readUInt32LE(at+12),8448);
    assert.equal(dfd.readUInt16LE(at+16),27);
    assert.equal(dfd[at+18],4);assert.equal(dfd[at+19],channel|32);
    assert.equal(dfd.readUInt32LE(at+20),0);
    assert.equal(dfd.readUInt32LE(at+24),15);
    assert.equal(dfd.readUInt32LE(at+28),31);
  }
  const kvdLength=runtime.readUInt32LE(60),kvdEnd=228+kvdLength;
  const metadata={};
  for(let offset=228;offset<kvdEnd;){
    assert(offset+4<=kvdEnd);
    const length=runtime.readUInt32LE(offset);offset+=4;
    assert(length>1&&offset+length<=kvdEnd);
    const item=runtime.subarray(offset,offset+length);
    const zero=item.indexOf(0);assert(zero>0&&item.at(-1)===0);
    const key=item.subarray(0,zero).toString();assert(!(key in metadata));
    metadata[key]=item.subarray(zero+1,-1).toString();
    offset+=Math.ceil(length/4)*4;
    assert(offset<=kvdEnd);
  }
  assert.equal(metadata.KTXorientation,'rd');
  const offset=Number(runtime.readBigUInt64LE(80)),length=Number(runtime.readBigUInt64LE(88));
  assert.equal(offset,Math.ceil(kvdEnd/8)*8);
  assert.equal(length,width*height*4);
  assert.equal(Number(runtime.readBigUInt64LE(96)),length);
  assert.equal(offset+length,runtime.length);
  assert.equal(original.readUInt32LE(12),97);
  assert.equal(original.readUInt32LE(20),width);assert.equal(original.readUInt32LE(24),height);
  const sourceOffset=Number(original.readBigUInt64LE(80));
  assert.equal(sourceOffset+width*height*8,original.length);
  for(let i=0;i<width*height;i++){
    const word=runtime.readUInt32LE(offset+i*4),scale=scales[word>>>27];
    const p=sourceOffset+i*8;
    assert.equal(original.readUInt16LE(p+6),0x3c00,'Opaque source alpha');
    for(let c=0;c<3;c++){
      const value=((word>>>(c*9))&511)*scale;
      if(value!==half[original.readUInt16LE(p+c*2)])throw new Error(`Radiance differs at texel ${i}, component ${c}`);
    }
  }
  return {pixels:width*height,components:width*height*3,source_gpu_bytes:width*height*8,gpu_bytes:length,
    source_sha256:hash(original),runtime_sha256:hash(runtime),max_absolute_error:0,implicit_alpha:1,metadata};
}
const result=check(packed);
assert.equal(proof.components_checked,result.components);
assert.equal(proof.source_gpu_bytes,result.source_gpu_bytes);
assert.equal(proof.gpu_bytes,result.gpu_bytes);
assert.equal(proof.bytes,packed.length);
let negativeCases=0;
if(process.argv.includes('--self-test')){
  assert.throws(()=>check(packed.subarray(0,-1)));negativeCases++;
  for(const offset of [12,20,80,116,134,228,Number(packed.readBigUInt64LE(80))]){
    const previous=packed[offset];packed[offset]^=1;
    try{assert.throws(()=>check(packed));negativeCases++;}finally{packed[offset]=previous;}
  }
  assert.equal(hash(packed),proof.output_sha256);
}
const report={schema:1,passed:true,...result,negative_cases:negativeCases,dimensions:manifest.dimensions,
  format:'RGB9E5',resampled:false,source_manifest_sha256:hash(read('tools/art/sky-source/manifest.json'))};
fs.mkdirSync(resolve('docs/evidence'),{recursive:true});
fs.writeFileSync(resolve('docs/evidence/sky-runtime-packing.json'),JSON.stringify(report,null,2)+'\n');
console.log(`${result.components} components exactly equal; ${result.gpu_bytes} GPU bytes; ${negativeCases} corruptions rejected.`);
