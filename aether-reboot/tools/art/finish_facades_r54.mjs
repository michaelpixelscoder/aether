// Final GLB assembly preserves every non-architectural batch byte-for-byte.
import fs from 'node:fs';import path from 'node:path';import assert from 'node:assert/strict';import crypto from 'node:crypto';import {fileURLToPath} from 'node:url';
const root=fileURLToPath(new URL('../../',import.meta.url)),sha=b=>crypto.createHash('sha256').update(b).digest('hex');
const read=file=>{const bytes=fs.readFileSync(file),n=bytes.readUInt32LE(12);return{bytes,doc:JSON.parse(bytes.subarray(20,20+n)),bin:bytes.subarray(28+n)};};
const ncomp={SCALAR:1,VEC2:2,VEC3:3,VEC4:4},width={5121:1,5123:2,5125:4,5126:4};
const report=[],manifestFile=path.join(root,'assets/world/manifest.json'),manifest=JSON.parse(fs.readFileSync(manifestFile));
for(const kind of ['dawn','dawn-watch']){
 const file=path.join(root,'assets/world',kind+'.glb'),candidate=read(file),original=read(path.join(root,'tools/art/architecture-source/archive',kind+'.glb'));
 assert.equal(candidate.doc.asset.extras.aether_architecture_r54.assembled_original_payloads,undefined,'regenerate before reassembling');
 assert.deepEqual(candidate.doc.materials,original.doc.materials);assert.deepEqual(candidate.doc.images,original.doc.images);
 const doc=structuredClone(candidate.doc),views=[],accessors=[],chunks=[];let offset=0;
 const cache=new Map();
 function copyAccessor(source,index){
  const key=(source===original?'old:':'new:')+index;if(cache.has(key))return cache.get(key);
  const a=source.doc.accessors[index],v=source.doc.bufferViews[a.bufferView];assert.equal(a.sparse,undefined);const stride=ncomp[a.type]*width[a.componentType],start=(v.byteOffset??0)+(a.byteOffset??0),data=Buffer.alloc(stride*a.count);
  for(let i=0;i<a.count;i++)source.bin.copy(data,i*stride,start+i*(v.byteStride??stride),start+i*(v.byteStride??stride)+stride);
  const pad=(4-offset%4)%4;if(pad){chunks.push(Buffer.alloc(pad));offset+=pad;}
  const bufferView=views.length;views.push({buffer:0,byteOffset:offset,byteLength:data.length,...(v.target?{target:v.target}:{})});chunks.push(data);offset+=data.length;
  const next={...structuredClone(a),bufferView,byteOffset:0};const result=accessors.length;accessors.push(next);cache.set(key,result);return result;
 }
 const kept=[];
 doc.meshes=doc.meshes.map((mesh,index)=>{
  const modified=['03 |','04 |','05 |','07 |','10 |'].some(p=>mesh.name.startsWith(p));const source=modified?candidate:original;const result=structuredClone(source.doc.meshes[index]);assert.equal(result.name,mesh.name);
  result.primitives=result.primitives.map(p=>{assert.equal(p.targets,undefined);return {...p,indices:copyAccessor(source,p.indices),attributes:Object.fromEntries(Object.entries(p.attributes).map(([name,a])=>[name,copyAccessor(source,a)]))};});
  if(!modified)kept.push(result.name);return result;
 });
 doc.accessors=accessors;doc.bufferViews=views;const bin=Buffer.concat(chunks);doc.buffers=[{byteLength:bin.length}];
 Object.assign(doc.asset.extras.aether_architecture_r54,{assembled_original_payloads:kept,original_glb_sha256:sha(original.bytes),assembly_sha256:sha(fs.readFileSync(fileURLToPath(import.meta.url)))});
 let text=Buffer.from(JSON.stringify(doc));text=Buffer.concat([text,Buffer.alloc((4-text.length%4)%4,32)]);const binary=Buffer.concat([bin,Buffer.alloc((4-bin.length%4)%4)]);
 const head=Buffer.alloc(20),tail=Buffer.alloc(8);head.writeUInt32LE(0x46546c67);head.writeUInt32LE(2,4);head.writeUInt32LE(28+text.length+binary.length,8);head.writeUInt32LE(text.length,12);head.writeUInt32LE(0x4e4f534a,16);tail.writeUInt32LE(binary.length);tail.writeUInt32LE(0x004e4942,4);
 const bytes=Buffer.concat([head,text,tail,binary]);fs.writeFileSync(file,bytes);report.push({kind,sha256:sha(bytes),bytes:bytes.length,original_sha256:sha(original.bytes),preserved_batches:kept});
 manifest[kind].high={file:kind+'.glb',triangles:doc.meshes.flatMap(m=>m.primitives).reduce((n,p)=>n+doc.accessors[p.indices].count/3,0),materials:doc.materials.length,bytes:bytes.length,sha256:sha(bytes)};
 manifest[kind].architecture_r54=doc.asset.extras.aether_architecture_r54;
}
fs.writeFileSync(manifestFile,JSON.stringify(manifest,null,2));
const evidence=path.join(root,'tools/art/architecture-source/evidence');fs.mkdirSync(evidence,{recursive:true});fs.writeFileSync(path.join(evidence,'facades-assembly.json'),JSON.stringify({islands:report},null,2)+'\n');console.log(JSON.stringify(report,null,2));
