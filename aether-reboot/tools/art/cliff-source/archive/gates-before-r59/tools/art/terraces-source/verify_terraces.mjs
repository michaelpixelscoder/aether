import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import {fileURLToPath} from 'node:url';
const root = path.dirname(fileURLToPath(import.meta.url));
const assets=fs.existsSync(path.join(root,'assets/world'))?path.join(root,'assets/world'):path.resolve(root,'../../../assets/world');
const sha = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const sizes = {5121:1,5123:2,5125:4,5126:4}, components = {SCALAR:1,VEC2:2,VEC3:3,VEC4:4};
function glb(bytes) {
 assert.equal(bytes.readUInt32LE(0),0x46546c67); assert.equal(bytes.readUInt32LE(4),2); assert.equal(bytes.readUInt32LE(8),bytes.length);
 const n=bytes.readUInt32LE(12); assert.equal(bytes.readUInt32LE(16),0x4e4f534a); assert.equal(bytes.readUInt32LE(24+n),0x004e4942);
 return {bytes,doc:JSON.parse(bytes.subarray(20,20+n)),bin:bytes.subarray(28+n)};
}
function raw(g,index) {
 const a=g.doc.accessors[index],v=g.doc.bufferViews[a.bufferView],width=sizes[a.componentType]*components[a.type]; assert(!a.sparse); assert(width>0);
 const b=Buffer.alloc(a.count*width); for(let i=0;i<a.count;i++){const at=(v.byteOffset??0)+(a.byteOffset??0)+i*(v.byteStride??width);assert(at+width<=g.bin.length);g.bin.copy(b,i*width,at,at+width);}return b;
}
function rows(g,index) {
 const a=g.doc.accessors[index],b=raw(g,index),w=sizes[a.componentType],c=components[a.type];
 return Array.from({length:a.count},(_,i)=>Array.from({length:c},(_,j)=>{const at=(i*c+j)*w;return a.componentType===5126?b.readFloatLE(at):w===4?b.readUInt32LE(at):w===2?b.readUInt16LE(at):b[at];}));
}
const baseline=JSON.parse(fs.readFileSync(path.join(root,'baseline-manifest.json')));
for(const f of baseline.files){const bytes=fs.readFileSync(path.join(root,'baseline',f.file));assert.equal(bytes.length,f.bytes,f.file);assert.equal(sha(bytes),f.sha256,f.file);}
const manifest=JSON.parse(fs.readFileSync(path.join(assets,'manifest.json')));
const originalManifest=JSON.parse(fs.readFileSync(path.join(root,'baseline/assets/world/manifest.json')));
const authoring=JSON.parse(fs.readFileSync(path.join(root,'evidence/authoring.json')));
assert.equal(authoring.source_sha256,sha(fs.readFileSync(path.join(root,'build_terraces.py'))));
assert.equal(authoring.frozen_helpers_sha256,sha(fs.readFileSync(path.join(root,'baseline/tools/art/build_world.py'))));
function inspect(kind,bytes) {
 const g=glb(bytes),b=glb(fs.readFileSync(path.join(root,'baseline/assets/world',kind+'.glb'))),record=manifest[kind],provenance=g.doc.asset.extras.aether_terraces_r56;
 assert.equal(sha(bytes),record.high.sha256,'final GLB SHA');assert.equal(bytes.length,record.high.bytes);
 assert.deepEqual(provenance,record.terraces_r56);assert.equal(provenance.baseline_glb_sha256,sha(b.bytes));
 assert.equal(provenance.author_sha256,authoring.source_sha256);assert.equal(provenance.frozen_helpers_sha256,authoring.frozen_helpers_sha256);
 assert.equal(provenance.additions_glb_sha256,sha(fs.readFileSync(path.join(root,'source',kind+'-additions.glb'))));
 assert.equal(provenance.assembly_sha256,sha(fs.readFileSync(path.join(root,'assemble_terraces.mjs'))));
 for(const field of ['materials','images','textures','samplers','nodes','scenes','scene','extensionsUsed','extensionsRequired'])assert.deepEqual(g.doc[field],b.doc[field],field+' exact');
 assert.equal(g.doc.meshes.length,12);assert.equal(g.doc.materials.length,12);assert.equal(g.doc.nodes.length,12);
 const preserved={...record};delete preserved.high;delete preserved.terraces_r56;const oldPreserved={...originalManifest[kind]};delete oldPreserved.high;assert.deepEqual(preserved,oldPreserved,'all non-HD manifest metadata unchanged');
 let triangles=0,added=0,maxNormalError=0,maxTangentError=0,degenerate=0,inverted=0,addedFoliageArea=0;const batches=[];
 const bounds=g=>{const result=[[Infinity,-Infinity],[Infinity,-Infinity],[Infinity,-Infinity]];for(const mesh of g.doc.meshes)for(const p of mesh.primitives)for(const xyz of rows(g,p.attributes.POSITION))for(let j=0;j<3;j++){result[j][0]=Math.min(result[j][0],xyz[j]);result[j][1]=Math.max(result[j][1],xyz[j]);}return result;};
 assert.deepEqual(bounds(g),bounds(b),'whole island AABB exact');
 for(let m=0;m<g.doc.meshes.length;m++){
  const mesh=g.doc.meshes[m],old=b.doc.meshes[m];assert.equal(mesh.name,old.name);assert.equal(mesh.primitives.length,1);
  const p=mesh.primitives[0],q=old.primitives[0],prefix=mesh.name.slice(0,2),modified=['06','08','09'].includes(prefix);
  assert.equal(p.material,q.material);assert.equal(p.mode,q.mode);assert.deepEqual(Object.keys(p.attributes),Object.keys(q.attributes));
  const oldCount=b.doc.accessors[q.attributes.POSITION].count;
  for(const [field,index] of Object.entries(p.attributes)){
   const next=raw(g,index),before=raw(b,q.attributes[field]);assert(next.subarray(0,before.length).equals(before),mesh.name+' '+field+' original bytes');
   if(!modified)assert(next.equals(before),mesh.name+' '+field+' untouched batch');
  }
  const ids=rows(g,p.indices).flat(),oldIds=rows(b,q.indices).flat();assert.deepEqual(ids.slice(0,oldIds.length),oldIds,'all original corners exact');
  if(!modified)assert.deepEqual(ids,oldIds);else assert(ids.slice(oldIds.length).every(i=>i>=oldCount),'added geometry cannot use old vertices');
  const pos=rows(g,p.attributes.POSITION),normal=rows(g,p.attributes.NORMAL),tan=rows(g,p.attributes.TANGENT);
  for(let i=oldCount;i<pos.length;i++){
   assert(pos[i].every(Number.isFinite));maxNormalError=Math.max(maxNormalError,Math.abs(Math.hypot(...normal[i])-1));maxTangentError=Math.max(maxTangentError,Math.abs(Math.hypot(...tan[i].slice(0,3))-1));assert.equal(Math.abs(tan[i][3]),1);
  }
  for(let i=oldIds.length;i<ids.length;i+=3){
   const [a,c,d]=ids.slice(i,i+3).map(j=>pos[j]);assert(a&&c&&d);const u=c.map((v,j)=>v-a[j]),v=d.map((v,j)=>v-a[j]),cross=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]],area=Math.hypot(...cross);if(area<1e-10){degenerate++;continue;}
   if(['08','09'].includes(prefix))addedFoliageArea+=area*.5;
   const n=[0,1,2].map(j=>ids.slice(i,i+3).reduce((s,k)=>s+normal[k][j],0));if(cross.reduce((s,v,j)=>s+v*n[j],0)/(area*Math.hypot(...n))<-.001)inverted++;
  }
  triangles+=ids.length/3;added+=(ids.length-oldIds.length)/3;batches.push({mesh:mesh.name,added_triangles:(ids.length-oldIds.length)/3,original_attributes_and_corners_exact:true});
 }
 assert.equal(triangles,record.high.triangles);assert(triangles<=200000);assert.equal(added,provenance.added_triangles);assert.equal(degenerate,0);assert.equal(inverted,0);assert(maxNormalError<1e-4);assert(maxTangentError<1e-3);
 const authored=authoring.islands.find(i=>i.kind===kind);assert.equal(authored.added_triangles,added);assert.equal(authored.combined_triangles,triangles);assert.equal(authored.collision_changes,0);assert.equal(authored.lod_changes,0);
 for(const image of g.doc.images){const texture=fs.readFileSync(path.join(assets,image.uri));assert.equal(sha(texture),path.basename(image.uri).split('.')[0]);assert(texture.equals(fs.readFileSync(path.join(root,'baseline/assets/world',image.uri))));}
 return {kind,sha256:sha(bytes),triangles,added_triangles:added,meshes:12,nodes:12,materials:12,batches,added_foliage_surface_area_m2:addedFoliageArea,max_normal_length_error:maxNormalError,max_tangent_length_error:maxTangentError,degenerate_added_triangles:degenerate,inverted_added_triangles:inverted,architecture_geometry_and_all_existing_terrain_exact:true,whole_island_aabb_exact:true,lod_collision_landmark_routes_exact:true};
}
for(const file of ['collisions.json','landmarks.json','dawn-lod.glb','dawn-watch-lod.glb'])assert(fs.readFileSync(path.join(assets,file)).equals(fs.readFileSync(path.join(root,'baseline/assets/world',file))),file+' exact');
const islands=['dawn','dawn-watch'].map(kind=>inspect(kind,fs.readFileSync(path.join(assets,kind+'.glb'))));
const corruptionTests=[];
if(process.argv.includes('--self-test'))for(const kind of ['dawn','dawn-watch']){
 const original=fs.readFileSync(path.join(assets,kind+'.glb'));
 for(const [label,mutate] of [['payload',b=>{b[b.length-16]^=1;}],['header-length',b=>{b.writeUInt32LE(b.length-4,8);}],['json',b=>{b[20]=0;}]]){
  const corrupted=Buffer.from(original);mutate(corrupted);assert.throws(()=>inspect(kind,corrupted),kind+' '+label+' rejected');corruptionTests.push({kind,case:label,rejected:true});
 }
}
const report={schema:1,islands,corruption_tests:corruptionTests};
fs.writeFileSync(path.join(root,'evidence/terraces-gate.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report,null,2));
