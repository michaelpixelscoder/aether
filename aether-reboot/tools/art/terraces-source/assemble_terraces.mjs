// Append authored landscape vertices into existing material primitives only.
// Every old attribute payload and corner remains exact, including R54 facades.
import fs from 'node:fs';import path from 'node:path';import crypto from 'node:crypto';import assert from 'node:assert/strict';import {fileURLToPath} from 'node:url';
const root=path.dirname(fileURLToPath(import.meta.url));
const assets=fs.existsSync(path.join(root,'assets/world'))?path.join(root,'assets/world'):path.resolve(root,'../../../assets/world');
const sha=b=>crypto.createHash('sha256').update(b).digest('hex');
const read=file=>{const bytes=fs.readFileSync(file),n=bytes.readUInt32LE(12);return {bytes,doc:JSON.parse(bytes.subarray(20,20+n)),bin:bytes.subarray(28+n)};};
const components={SCALAR:1,VEC2:2,VEC3:3,VEC4:4},sizes={5121:1,5123:2,5125:4,5126:4};
function rawData(source,index){const a=source.doc.accessors[index],v=source.doc.bufferViews[a.bufferView],width=components[a.type]*sizes[a.componentType];assert(!a.sparse);const result=Buffer.alloc(width*a.count);for(let i=0;i<a.count;i++){const at=(v.byteOffset??0)+(a.byteOffset??0)+i*(v.byteStride??width);source.bin.copy(result,i*width,at,at+width);}return result;}
function indices(source,index){const a=source.doc.accessors[index],data=rawData(source,index),size=sizes[a.componentType];return Array.from({length:a.count},(_,i)=>size===4?data.readUInt32LE(i*size):size===2?data.readUInt16LE(i*size):data[i]);}
const manifestPath=path.join(assets,'manifest.json'),manifest=JSON.parse(fs.readFileSync(manifestPath));
const report=[];
for(const kind of ['dawn','dawn-watch']){
 const base=read(path.join(root,'baseline/assets/world',kind+'.glb')),added=read(path.join(root,'source',kind+'-additions.glb'));
 const doc=structuredClone(base.doc),views=[],accessors=[],chunks=[];let offset=0,addedTriangles=0;const changes=[];
 const append=(data,definition,target)=>{const pad=(4-offset%4)%4;if(pad){chunks.push(Buffer.alloc(pad));offset+=pad;}const bufferView=views.length;views.push({buffer:0,byteOffset:offset,byteLength:data.length,...(target?{target}:{})});chunks.push(data);offset+=data.length;const index=accessors.length;accessors.push({...structuredClone(definition),bufferView,byteOffset:0});return index;};
 for(const mesh of doc.meshes){
  assert.equal(mesh.primitives.length,1);const prim=mesh.primitives[0],old=base.doc.meshes.find(m=>m.name===mesh.name).primitives[0];
  const prefix=mesh.name.slice(0,2);const extra=['06','08','09'].includes(prefix)?added.doc.meshes.find(m=>added.doc.materials[m.primitives[0].material].name.startsWith(prefix+' |')):undefined;
  const add=extra?.primitives[0];if(add)assert.deepEqual(Object.keys(add.attributes).sort(),Object.keys(old.attributes).sort());
  const baseCount=base.doc.accessors[old.attributes.POSITION].count;
  for(const [semantic,index] of Object.entries(old.attributes)){
   const a=base.doc.accessors[index],data=rawData(base,index),next=structuredClone(a);let combined=data;
   if(add){const ai=add.attributes[semantic],addition=rawData(added,ai),aa=added.doc.accessors[ai];assert.equal(a.type,aa.type);assert.equal(a.componentType,aa.componentType);combined=Buffer.concat([data,addition]);next.count+=aa.count;if(a.min){next.min=a.min.map((x,i)=>Math.min(x,aa.min[i]));next.max=a.max.map((x,i)=>Math.max(x,aa.max[i]));}assert(combined.subarray(0,data.length).equals(data));}
   prim.attributes[semantic]=append(combined,next,34962);
  }
  if(add){const before=indices(base,old.indices),newIndices=indices(added,add.indices).map(i=>i+baseCount),all=[...before,...newIndices],data=Buffer.alloc(all.length*4);all.forEach((v,i)=>data.writeUInt32LE(v,i*4));prim.indices=append(data,{componentType:5125,count:all.length,type:'SCALAR'},34963);addedTriangles+=newIndices.length/3;changes.push({material:mesh.name,old_triangles:before.length/3,added_triangles:newIndices.length/3,old_attributes_byte_exact_prefix:true});}
  else prim.indices=append(rawData(base,old.indices),base.doc.accessors[old.indices],34963);
 }
 doc.bufferViews=views;doc.accessors=accessors;const bin=Buffer.concat(chunks);doc.buffers=[{byteLength:bin.length}];
 assert.deepEqual(doc.materials,base.doc.materials);assert.deepEqual(doc.nodes,base.doc.nodes);assert.deepEqual(doc.images,base.doc.images);
 const provenance={author:'build_terraces.py',author_sha256:sha(fs.readFileSync(path.join(root,'build_terraces.py'))),frozen_helpers_sha256:sha(fs.readFileSync(path.join(root,'baseline/tools/art/build_world.py'))),baseline_glb_sha256:sha(base.bytes),additions_glb_sha256:sha(added.bytes),assembly_sha256:sha(fs.readFileSync(fileURLToPath(import.meta.url))),scope:'Append only06/08/09; all original vertex attributes/corners/materials/nodes/images retained; collision/landmarks/terrain/LOD exact',added_triangles:addedTriangles};
 doc.asset.extras.aether_terraces_r56=provenance;
 let encoded=Buffer.from(JSON.stringify(doc));encoded=Buffer.concat([encoded,Buffer.alloc((4-encoded.length%4)%4,32)]);const binary=Buffer.concat([bin,Buffer.alloc((4-bin.length%4)%4)]);const head=Buffer.alloc(20),tail=Buffer.alloc(8);head.writeUInt32LE(0x46546c67);head.writeUInt32LE(2,4);head.writeUInt32LE(28+encoded.length+binary.length,8);head.writeUInt32LE(encoded.length,12);head.writeUInt32LE(0x4e4f534a,16);tail.writeUInt32LE(binary.length);tail.writeUInt32LE(0x004e4942,4);
 const bytes=Buffer.concat([head,encoded,tail,binary]),output=path.join(assets,kind+'.glb');fs.writeFileSync(output,bytes);
 const triangles=doc.meshes.flatMap(m=>m.primitives).reduce((n,p)=>n+doc.accessors[p.indices].count/3,0);assert(triangles<=200000);
 manifest[kind].high={file:kind+'.glb',triangles,materials:doc.materials.length,bytes:bytes.length,sha256:sha(bytes)};manifest[kind].terraces_r56=provenance;
 report.push({kind,sha256:sha(bytes),triangles,added_triangles:addedTriangles,materials:doc.materials.length,nodes:doc.nodes.length,meshes:doc.meshes.length,changes});
}
fs.writeFileSync(manifestPath,JSON.stringify(manifest,null,2)+'\n');fs.writeFileSync(path.join(root,'evidence/assembly.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
