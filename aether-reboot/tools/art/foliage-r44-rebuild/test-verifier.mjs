import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {fileURLToPath} from 'node:url';
import {normalizeTangentRounding} from '../verify-foliage-materials.mjs';
const dir=path.dirname(fileURLToPath(import.meta.url)),root=path.resolve(dir,'../../..');
const evidence=JSON.parse(fs.readFileSync(path.join(dir,'../foliage-r44-tangent-rounding.json')));
const results=[];
const nextFloat=x=>{if(x===0)return 2**-149;const b=Buffer.alloc(4);b.writeFloatLE(x);b.writeUInt32LE(b.readUInt32LE()+(x>0?1:-1));return b.readFloatLE();};
function load(e) {
 const raw=fs.readFileSync(path.join(root,'assets/world',e.file)),length=raw.readUInt32LE(12),doc=JSON.parse(raw.subarray(20,20+length)),bin=raw.subarray(28+length);
 const p=doc.meshes.find(m=>m.name===e.mesh).primitives.find(p=>doc.materials[p.material].name===e.material);
 const read=index=>{const a=doc.accessors[index],v=doc.bufferViews[a.bufferView],n=a.type==='SCALAR'?1:4,b=a.componentType===5123?2:4;return Array.from({length:a.count},(_,i)=>Array.from({length:n},(_,k)=>{const at=(v.byteOffset??0)+(a.byteOffset??0)+i*(v.byteStride??n*b)+k*b;return a.componentType===5126?bin.readFloatLE(at):b===2?bin.readUInt16LE(at):bin.readUInt32LE(at);}));};
 const values=read(p.attributes.TANGENT),indices=read(p.indices).flat();return {ordered:indices.map(i=>values[i].slice()),material:doc.materials[p.material]};
}
for(const e of evidence.cases) {
 const {ordered,material}=load(e),historical=ordered.map(v=>v.slice());
 assert.equal(normalizeTangentRounding(ordered,e,material).changed_components,e.points.length);
 for(const p of e.points)historical[p.corner][p.component]=p.historical;
 assert.equal(normalizeTangentRounding(historical,e,material).changed_components,0);
 const rejects=(name,mutate,mat=material)=>{const copy=historical.map(v=>v.slice());mutate(copy);assert.throws(()=>normalizeTangentRounding(copy,e,mat));results.push({file:e.file,mesh:e.mesh,test:name,passed:true});};
 const p=e.points[0];
 rejects('unlisted one-ULP change',a=>{a[0][0]=nextFloat(a[0][0]);});
 rejects('unmeasured value inside apparent epsilon',a=>{a[p.corner][p.component]=nextFloat(p.historical);});
 rejects('two quantization steps',a=>{a[p.corner][p.component]=Math.fround(p.historical+0.0002);});
 rejects('handedness flip',a=>{a[p.corner][3]*=-1;});
 rejects('normal-mapped foliage',()=>{},{...material,normalTexture:{index:0}});
 rejects('non-foliage material',()=>{},{...material,name:'03 | Dressed masonry'});
 rejects('anisotropic foliage',()=>{},{...material,extensions:{KHR_materials_anisotropy:{anisotropyStrength:1}}});
}
fs.writeFileSync(path.join(dir,'negative-tests.json'),JSON.stringify({historical_and_rebuilt_accepted:4,rejection_tests:results},null,2)+'\n');
console.log(`Four authentic historical/rebuilt pairs accepted; ${results.length} meaningful mutation tests rejected.`);
