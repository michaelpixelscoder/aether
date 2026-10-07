// Portable build gate. Optional authoring accuracy study: verify_cloud_cache.py.
import fs from 'node:fs';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
const hash=file=>crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
const manifest=JSON.parse(fs.readFileSync('assets/atmosphere/cloud-cache-manifest.json','utf8'));
const config=JSON.parse(fs.readFileSync('assets/atmosphere/cloud-banks.json','utf8'));
assert.equal(hash(manifest.generator),manifest.generator_sha256);
assert.equal(hash('assets/atmosphere/cloud-banks.json'),manifest.config_sha256);
assert.equal(config.length,6);assert.equal(new Set(config.map(x=>x.key)).size,6);
assert.equal(manifest.banks.length,config.length);
let total=0;
const banks=config.map((bank,index)=>{
 for(const [key,n] of [['center',3],['extent',3],['padding',3],['shape',4],['resolution',3]]){
  assert.equal(bank[key].length,n);assert(bank[key].every(Number.isFinite));
 }
 assert(bank.extent.every(v=>v>0));assert(bank.padding[0]>=192&&bank.padding[2]>=192);
 assert(bank.resolution.every(v=>Number.isInteger(v)&&v>=64&&v<=256));
 const entry=manifest.banks[index];assert.equal(entry.key,bank.key);
 const file=`assets/atmosphere/${bank.key}-shape.ktx2`;assert.equal(entry.file,file);
 const bytes=fs.readFileSync(file);assert.equal(hash(file),entry.sha256);
 assert(bytes.subarray(0,12).equals(Buffer.from([171,75,84,88,32,50,48,187,13,10,26,10])));
 assert.deepEqual([12,16,32,36,40].map(n=>bytes.readUInt32LE(n)),[37,1,0,1,1]);
 assert.equal(bytes.readUInt32LE(44),0);
 const size=[20,24,28].map(n=>bytes.readUInt32LE(n));
 assert.deepEqual(size,bank.resolution);assert.deepEqual(size,entry.dimensions);
 const offset=Number(bytes.readBigUInt64LE(80)),length=Number(bytes.readBigUInt64LE(88));
 assert.equal(length,size.reduce((a,b)=>a*b,4));assert.equal(length,entry.gpu_bytes);
 assert.equal(Number(bytes.readBigUInt64LE(96)),length);assert.equal(offset+length,bytes.length);
 let positive=0,minimum=255,maximum=0;
 for(let n=offset;n<bytes.length;n+=4){const v=bytes[n];positive+=Number(v>0);minimum=Math.min(minimum,v);maximum=Math.max(maximum,v);}
 assert.equal(positive,entry.positive_voxels);assert.equal(minimum,0);assert(maximum>=250);
 assert(positive>size.reduce((a,b)=>a*b)*.02);
 total+=length;
 return {file,sha256:entry.sha256,dimensions:size,gpu_bytes:length,positive_voxels:positive};
});
assert.equal(total,manifest.gpu_bytes);assert(total<=32*1024*1024);
const report={passed:true,banks,gpu_bytes:total};
fs.writeFileSync('docs/evidence/cloud-cache-assets.json',JSON.stringify(report,null,2));
console.log(`Six caches nuageux 3D vérifiés, ${(total/1024/1024).toFixed(1)} Mio GPU.`);
