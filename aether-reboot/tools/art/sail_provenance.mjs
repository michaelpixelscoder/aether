// Shared by assembly and verification. Fingerprints resolve accessors, including
// sparse morph deltas, so relocating a GLB buffer never changes its identity.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';

export const sha = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
export const fileSha = file => sha(fs.readFileSync(file));
export const sourceRoles = [
  ['canvas', 'tools/art/build_sail_r52.py'],
  ['archived_canvas', 'tools/art/sail-source/archive/r44-canvas.blend'],
  ['spars', 'tools/art/build_sail_rig.py'],
  ['primitive_helpers', 'tools/art/build_ship.py'],
  ['assembly', 'tools/art/merge_sail_rig.mjs'],
  ['provenance_validation', 'tools/art/sail_provenance.mjs'],
];
export const sourceChain = root => sourceRoles.map(([role, file]) => ({role, path: file, sha256: fileSha(path.join(root, file))}));
export function readGlb(file) {
  const bytes=fs.readFileSync(file),length=bytes.readUInt32LE(12);
  assert.equal(bytes.readUInt32LE(0),0x46546c67);
  assert.equal(bytes.readUInt32LE(8),bytes.length);
  return {bytes,doc:JSON.parse(bytes.subarray(20,20+length)),bin:bytes.subarray(28+length)};
}
const components={SCALAR:1,VEC2:2,VEC3:3,VEC4:4,MAT2:4,MAT3:9,MAT4:16};
const widths={5120:1,5121:1,5122:2,5123:2,5125:4,5126:4};
export function accessorBytes(doc,bin,index) {
  const a=doc.accessors[index],size=components[a.type]*widths[a.componentType];
  assert.ok(size>0);const result=Buffer.alloc(a.count*size);
  if(a.bufferView!==undefined){
    const view=doc.bufferViews[a.bufferView];assert.equal(view.buffer??0,0);
    const start=(view.byteOffset??0)+(a.byteOffset??0),stride=view.byteStride??size;
    for(let i=0;i<a.count;i++)bin.copy(result,i*size,start+i*stride,start+i*stride+size);
  }
  if(a.sparse){
    const sparse=a.sparse,indices=doc.bufferViews[sparse.indices.bufferView],values=doc.bufferViews[sparse.values.bufferView];
    const width=widths[sparse.indices.componentType],start=(indices.byteOffset??0)+(sparse.indices.byteOffset??0),data=(values.byteOffset??0)+(sparse.values.byteOffset??0);
    assert.ok([1,2,4].includes(width));
    for(let i=0;i<sparse.count;i++){
      const offset=start+i*width,n=width===1?bin.readUInt8(offset):width===2?bin.readUInt16LE(offset):bin.readUInt32LE(offset);
      assert.ok(n<a.count);bin.copy(result,n*size,data+i*size,data+(i+1)*size);
    }
  }
  return result;
}
export function meshFingerprints(doc,bin,animated) {
  const accessor=index=>{
    const a=doc.accessors[index];
    return {component_type:a.componentType,type:a.type,count:a.count,normalized:a.normalized??false,min:a.min??null,max:a.max??null,sha256:sha(accessorBytes(doc,bin,index))};
  };
  return doc.meshes.filter(m=>m.primitives.some(p=>Boolean(p.targets?.length))===animated).map(mesh=>({
    name:mesh.name,
    nodes:doc.nodes.filter(n=>n.mesh!==undefined&&doc.meshes[n.mesh]===mesh).map(n=>({name:n.name,translation:n.translation??[0,0,0],rotation:n.rotation??[0,0,0,1],scale:n.scale??[1,1,1]})),
    weights:mesh.weights??null,target_names:mesh.extras?.targetNames??null,
    primitives:mesh.primitives.map(p=>({
      material:doc.materials[p.material],mode:p.mode??4,
      indices:accessor(p.indices),
      attributes:Object.fromEntries(Object.entries(p.attributes).sort().map(([key,index])=>[key,accessor(index)])),
      targets:(p.targets??[]).map(t=>Object.fromEntries(Object.entries(t).sort().map(([key,index])=>[key,accessor(index)]))),
    })),
  }));
}
export const fingerprint = records => sha(Buffer.from(JSON.stringify(records)));

export function verifySailProvenance(doc,bin,root,bytes) {
  const pipeline=doc.asset.extras.aether_sail_pipeline;
  assert.ok(pipeline,'sail must carry the composite source chain; run all three authoring stages');
  assert.equal(pipeline.schema,1);
  assert.equal(doc.asset.extras.aether_generator_sha256,undefined,'an old single generator must not claim the new spars');
  assert.deepEqual(pipeline.sources,sourceChain(root),'every participating source must match');
  const evidence=JSON.parse(fs.readFileSync(path.join(root,'tools/art/sail-source/evidence/sail-provenance.json')));
  assert.deepEqual(evidence.pipeline,pipeline);
  assert.equal(evidence.result_sha256,sha(bytes));
  const cloth=meshFingerprints(doc,bin,true),statics=meshFingerprints(doc,bin,false);
  assert.deepEqual(cloth,evidence.cloth_before,'all canvas, seam and flag attributes/morphs must equal the base export');
  assert.equal(fingerprint(cloth),pipeline.cloth_payload_sha256);
  assert.equal(fingerprint(statics),pipeline.static_payload_sha256);
  assert.equal(pipeline.input_sail_sha256,evidence.before.sha256);
  assert.equal(evidence.cloth_exact,true);
  assert.equal(doc.meshes.length,9);assert.equal(doc.materials.length,7);
  const rig=JSON.parse(fs.readFileSync(path.join(root,'tools/art/sail-source/static-manifest.json')));
  assert.equal(rig.generator_sha256,fileSha(path.join(root,'tools/art/build_sail_rig.py')));
  assert.equal(rig.helper_sha256,fileSha(path.join(root,'tools/art/build_ship.py')));
  assert.equal(rig.blend_sha256,fileSha(path.join(root,'tools/art/sail-source/sail-rig.blend')));
  assert.equal(rig.rig_glb_sha256,pipeline.input_rig_sha256);
  const count=doc.meshes.flatMap(m=>m.primitives).reduce((sum,p)=>sum+doc.accessors[p.indices].count/3,0);
  assert.equal(count,evidence.after.triangles);
  assert.ok(count<=80000,'authored sail triangle budget');
  return {sources:pipeline.sources,cloth_payload_sha256:pipeline.cloth_payload_sha256,static_payload_sha256:pipeline.static_payload_sha256,triangles:count};
}
