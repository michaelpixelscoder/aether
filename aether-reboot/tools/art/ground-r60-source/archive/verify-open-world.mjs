// Run from the workspace root. Asset contracts are checked without Blender.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
const hash=file=>crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
const read=file=>JSON.parse(fs.readFileSync(file,'utf8'));
for(const tool of ['verify-islands','verify-variants','verify-vault','verify-sky','verify-sky-packing','verify-ibl','verify-rock-materials','verify-masonry-materials','verify-crystal-materials','verify-cloud-cache','verify-cloud-lighting','verify-foliage-materials','verify-ship-batches','verify_facades_r54','terraces-source/verify_terraces','flow-source/verify']){
 const r=spawnSync(process.execPath,[`tools/art/${tool}.mjs`],{encoding:'utf8'});
 fs.writeFileSync(`docs/evidence/${tool.replaceAll('/','-')}.log`,r.stdout+r.stderr);
 assert.equal(r.status,0,`${tool}: ${r.stderr}`);
}
function glb(file){
 const bytes=fs.readFileSync(file),length=bytes.readUInt32LE(12);
 assert.equal(bytes.readUInt32LE(0),0x46546c67);assert.equal(bytes.readUInt32LE(8),bytes.length);
 const doc=JSON.parse(bytes.subarray(20,20+length)),bin=bytes.subarray(28+length);
 for(const image of doc.images??[]){
  if(image.uri){
   const texture=path.resolve(path.dirname(file),image.uri);
   assert(texture.startsWith(path.resolve('assets')+path.sep));assert(fs.statSync(texture).size>0);
  }
 }
 let triangles=0;
 for(const mesh of doc.meshes)for(const p of mesh.primitives){
  assert.equal(p.mode??4,4);const indices=doc.accessors[p.indices];triangles+=indices.count/3;
  for(const name of ['POSITION','NORMAL','TEXCOORD_0']){
   const a=doc.accessors[p.attributes[name]],v=doc.bufferViews[a.bufferView];assert(a);
   assert.equal(a.componentType,5126);const components=name==='TEXCOORD_0'?2:3;
   for(let n=0;n<a.count;n++)for(let k=0;k<components;k++){
    assert(Number.isFinite(bin.readFloatLE((v.byteOffset??0)+(a.byteOffset??0)+n*(v.byteStride??components*4)+k*4)));
   }
  }
 }
 return {doc,triangles,bytes:bytes.length};
}
const kits=[];
for(const [file,key] of [['assets/ship/manifest.json','modules'],['assets/fauna/manifest.json','assets'],['assets/fauna/trader-manifest.json','assets']]){
 const m=read(file);assert.equal(hash(m.generator),m.generator_sha256);
 for(const texture of m.shared_images)assert.equal(hash(texture.file),texture.sha256);
 for(const [name,a] of Object.entries(m[key])){
  assert.equal(hash(a.file),a.sha256);assert.equal(hash(a.source),a.source_sha256);
  const {doc,triangles,bytes}=glb(a.file);
  assert.equal(triangles,a.triangles);assert.equal(bytes,a.bytes);
  if(name.startsWith('trader'))for(const side of ['L','R']){
   const n=doc.nodes.find(n=>n.name==='MerchantPropeller'+side);assert(n);
   assert(n.translation.every((v,i)=>Math.abs(v-m.pivots_game_xyz_m['MerchantPropeller'+side][i])<0.00001));
   assert(n.rotation===undefined||n.rotation.every((v,i)=>v===(i===3?1:0)));
  }
  kits.push({file:a.file,sha256:a.sha256,triangles,bytes});
 }
}
for(const lod of ['', '-lod']){
 const {doc}=glb(`assets/world/underforge${lod}.glb`);
 const gears=doc.nodes.filter(n=>n.name?.startsWith('UnderforgeGear'));
 assert.equal(gears.length,5);for(const gear of gears)assert.equal(gear.extras.aether_animation_axis,'local Z');
}
const collision=read('assets/fauna/trader-collisions.json'),m=collision.metadata;
assert.equal(hash(m.generator),m.generator_sha256);
assert.equal(hash(m.model_generator),m.model_generator_sha256);
assert.equal(collision.trader.length,m.box_count);
for(const [name,digest] of Object.entries(m.asset_sha256))assert.equal(hash('assets/fauna/'+name),digest);
for(const box of collision.trader){
 assert(box.center.every(Number.isFinite)&&box.size.every(v=>Number.isFinite(v)&&v>0));
 for(const x of [-1,1])for(const y of [-1,1])for(const z of [-1,1]){
  assert(Math.hypot(...box.center.map((v,i)=>v+[x,y,z][i]*box.size[i]/2))<=12.001);
 }
}
const report={checked_at:new Date().toISOString(),islands:read('docs/evidence/world-art-validation.json'),variants:read('docs/evidence/world-art-variants-validation.json'),vault:read('docs/evidence/world-art-vault-validation.json'),sky:read('docs/evidence/sky-assets.json'),kits,trader_boxes:collision.trader.length,underforge_gears_per_lod:5};
fs.writeFileSync('docs/evidence/open-world-assets.json',JSON.stringify(report,null,2));
console.log(`${report.islands.glbs.length+report.variants.glbs.length+report.vault.glbs.length+kits.length} GLB vérifiés, ${collision.trader.length} boîtes de dirigeable, cinq engrenages par LOD.`);
