// Authored kit contract consumed by ship::prepare. Fail on unsupported changes
// instead of silently dropping a hierarchy, animation or additional surface.
import fs from 'node:fs';
import assert from 'node:assert/strict';
for (const name of ['rail','trim','cabin-detail','lantern']) {
 const file=`assets/ship/${name}.glb`, bytes=fs.readFileSync(file);
 const doc=JSON.parse(bytes.subarray(20,20+bytes.readUInt32LE(12)));
 assert.equal(doc.scenes.length,1,file);
 assert.deepEqual(doc.scenes[0].nodes,[0,1,2],file);
 assert.equal(doc.nodes.length,3,file);
 assert.equal(doc.meshes.length,3,file);
 assert.equal(doc.materials.length,3,file);
 assert.equal(doc.animations?.length??0,0,file);
 assert.equal(doc.skins?.length??0,0,file);
 for(let slot=0;slot<3;slot++) {
  const node=doc.nodes[slot], mesh=doc.meshes[slot];
  assert.equal(node.mesh,slot,file);
  for(const field of ['matrix','translation','rotation','scale','children','skin','weights'])
   assert.equal(node[field],undefined,`${file}: baked identity nodes required (${field})`);
  assert.equal(mesh.primitives.length,1,file);
  const p=mesh.primitives[0];
  assert.equal(p.material,slot,file);
  assert.equal(p.mode??4,4,file);
  assert.equal(p.targets,undefined,file);
  assert.deepEqual(Object.keys(p.attributes).sort(),['NORMAL','POSITION','TANGENT','TEXCOORD_0'],file);
  assert.equal(doc.materials[slot].alphaMode??'OPAQUE','OPAQUE',`${file}: transparent primitives require individual sorting`);
 }
}
console.log('4 authored hull kits: static identity nodes, 12 opaque material batches, UV/tangent layout verified.');
