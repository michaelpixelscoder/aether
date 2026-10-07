// Independent R59 geometry contracts. No Blender or renderer is required.
import fs from 'node:fs';import path from 'node:path';import crypto from 'node:crypto';import assert from 'node:assert/strict';import {fileURLToPath} from 'node:url';
const root=path.dirname(fileURLToPath(import.meta.url)),sha=b=>crypto.createHash('sha256').update(b).digest('hex');
const manifest=JSON.parse(fs.readFileSync(path.join(root,'candidate/assets/world/manifest.json'))),build=JSON.parse(fs.readFileSync(path.join(root,'build-evidence.json')));
const sizes={5126:4,5125:4,5123:2},widths={SCALAR:1,VEC2:2,VEC3:3,VEC4:4};
function glb(bytes){assert.equal(bytes.readUInt32LE(0),0x46546c67,'GLB magic');assert.equal(bytes.readUInt32LE(4),2,'GLB version');assert.equal(bytes.readUInt32LE(8),bytes.length,'GLB size');const n=bytes.readUInt32LE(12);assert.equal(bytes.readUInt32LE(16),0x4e4f534a);assert.equal(bytes.readUInt32LE(24+n),0x004e4942);return {bytes,doc:JSON.parse(bytes.subarray(20,20+n)),bin:bytes.subarray(28+n)};}
const load=f=>glb(fs.readFileSync(f));
function raw(g,index){const a=g.doc.accessors[index],v=g.doc.bufferViews[a.bufferView],w=widths[a.type]*sizes[a.componentType];assert(w>0);assert(!a.sparse);const b=Buffer.alloc(a.count*w);for(let i=0;i<a.count;i++){const o=(v.byteOffset??0)+(a.byteOffset??0)+i*(v.byteStride??w);assert(o+w<=g.bin.length,'accessor outside binary');g.bin.copy(b,i*w,o,o+w);}return b;}
function rows(g,index){const a=g.doc.accessors[index],b=raw(g,index),n=widths[a.type],s=sizes[a.componentType];return Array.from({length:a.count},(_,i)=>Array.from({length:n},(_,k)=>a.componentType===5126?b.readFloatLE((i*n+k)*s):s===4?b.readUInt32LE((i*n+k)*s):b.readUInt16LE((i*n+k)*s)));}
function bounds(g){const result=[[Infinity,-Infinity],[Infinity,-Infinity],[Infinity,-Infinity]];for(const m of g.doc.meshes)for(const p of m.primitives)for(const xyz of rows(g,p.attributes.POSITION))for(let k=0;k<3;k++){result[k][0]=Math.min(result[k][0],xyz[k]);result[k][1]=Math.max(result[k][1],xyz[k]);}return result;}
function signatures(g,p){const attrs=Object.keys(p.attributes).sort().map(k=>{const a=g.doc.accessors[p.attributes[k]];return {bytes:raw(g,p.attributes[k]),width:widths[a.type]*sizes[a.componentType]};}),ids=rows(g,p.indices).flat(),result=new Map();for(let i=0;i<ids.length;i+=3){const hex=ids.slice(i,i+3).map(id=>Buffer.concat(attrs.map(a=>a.bytes.subarray(id*a.width,(id+1)*a.width))).toString('hex')),key=[0,1,2].map(j=>[...hex.slice(j),...hex.slice(0,j)].join(':')).sort()[0];result.set(key,(result.get(key)??0)+1);}return result;}
function inspect(kind,bytes,record=manifest[kind]){
 assert.equal(sha(bytes),record.high.sha256,'current GLB sha');assert.equal(bytes.length,record.high.bytes,'current GLB bytes');
 const g=glb(bytes),baseline=load(path.join(root,'baseline/assets/world',kind+'.glb')),provenance=g.doc.asset.extras.aether_cliff_r59;
 assert.deepEqual(provenance,record.cliff_r59,'provenance manifest');assert.equal(provenance.baseline_glb_sha256,sha(baseline.bytes),'baseline provenance');
 assert.equal(provenance.author_sha256,sha(fs.readFileSync(path.join(root,'build_cliff.py'))),'actual author');assert.equal(provenance.assembly_sha256,sha(fs.readFileSync(path.join(root,'assemble.mjs'))),'actual assembly');
 for(const field of ['nodes','materials','images','textures','samplers','scenes','scene','extensionsUsed','extensionsRequired'])assert.deepEqual(g.doc[field],baseline.doc[field],field+' unchanged');
 assert.deepEqual(g.doc.asset.extras.aether_architecture_r54,baseline.doc.asset.extras.aether_architecture_r54,'R54 history unchanged');assert.deepEqual(g.doc.asset.extras.aether_terraces_r56,baseline.doc.asset.extras.aether_terraces_r56,'R56 history unchanged');
 assert.equal(g.doc.meshes.length,12);assert.equal(g.doc.nodes.length,12);assert.equal(g.doc.materials.length,12);
 let triangles=0,maxNormal=0,maxTangent=0,maxUv=0,minDot=1;
 for(let j=0;j<12;j++){
  const m=g.doc.meshes[j],old=baseline.doc.meshes[j],p=m.primitives[0],q=old.primitives[0];assert.equal(m.name,old.name);assert.equal(m.primitives.length,1);assert.equal(p.material,q.material);assert.equal(p.mode,q.mode);assert.deepEqual(Object.keys(p.attributes),Object.keys(q.attributes));
  if(!m.name.startsWith('01 |')){for(const field of Object.keys(p.attributes))assert(raw(g,p.attributes[field]).equals(raw(baseline,q.attributes[field])),'non-rock '+m.name+' '+field);assert(raw(g,p.indices).equals(raw(baseline,q.indices)),'non-rock corners '+m.name);}
  const pos=rows(g,p.attributes.POSITION),normal=rows(g,p.attributes.NORMAL),tangent=rows(g,p.attributes.TANGENT),uv=rows(g,p.attributes.TEXCOORD_0),ids=rows(g,p.indices).flat();assert.equal(ids.length%3,0);assert.equal(pos.length,normal.length);assert.equal(pos.length,tangent.length);assert.equal(pos.length,uv.length);
  for(let i=0;i<pos.length;i++){assert(pos[i].every(Number.isFinite),'finite position');assert(normal[i].every(Number.isFinite),'finite normal');assert(tangent[i].every(Number.isFinite),'finite tangent');assert(uv[i].every(Number.isFinite),'finite UV');maxNormal=Math.max(maxNormal,Math.abs(Math.hypot(...normal[i])-1));maxTangent=Math.max(maxTangent,Math.abs(Math.hypot(...tangent[i].slice(0,3))-1));assert.equal(Math.abs(tangent[i][3]),1,'tangent handedness');if(m.name.startsWith('01 |')&&Math.abs(normal[i][1])<.4&&Math.max(Math.abs(normal[i][0]),Math.abs(normal[i][2]))>.9)maxUv=Math.max(maxUv,Math.abs(uv[i][1]-(1-pos[i][1]/16)));}
  for(let i=0;i<ids.length;i+=3){const indices=ids.slice(i,i+3);assert(indices.every(id=>id>=0&&id<pos.length),'valid corners');const [a,b,c]=indices.map(id=>pos[id]),u=b.map((v,k)=>v-a[k]),v=c.map((v,k)=>v-a[k]),x=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]],area=Math.hypot(...x);assert(area>=1e-10,'nondegenerate triangle');const n=[0,1,2].map(k=>indices.reduce((s,id)=>s+normal[id][k],0)),dot=x.reduce((s,v,k)=>s+v*n[k],0)/(area*Math.hypot(...n));assert(dot>=-.001,'triangle orientation');minDot=Math.min(minDot,dot);}
  triangles+=ids.length/3;
 }
 assert(maxNormal<1e-4,'unit normals');assert(maxTangent<1e-3,'unit tangents');assert(maxUv<.001,'metric cliff UV');assert.deepEqual(bounds(g),bounds(baseline),'accepted island bounds');assert.equal(triangles,record.high.triangles);assert(triangles<=200000);
 const p=g.doc.meshes.find(m=>m.name.startsWith('01 |')).primitives[0],generated=load(path.join(root,'output',kind+'-candidate.glb')),gp=generated.doc.meshes[0].primitives[0];assert.equal(sha(generated.bytes),provenance.candidate_rock_payload_sha256);
 for(const field of Object.keys(p.attributes))assert(raw(g,p.attributes[field]).equals(raw(generated,gp.attributes[field])),'exact authored rock '+field);assert(raw(g,p.indices).equals(raw(generated,gp.indices)),'exact authored rock corners');
 const roots=load(path.join(root,'output',kind+'-protected-roots.glb')),rootTriangles=signatures(roots,roots.doc.meshes[0].primitives[0]),currentTriangles=signatures(g,p);for(const [s,n] of rootTriangles)assert((currentTriangles.get(s)??0)>=n,'protected physical root all-corner attributes');
 const metric=build.records.find(r=>r.kind===kind&&r.candidate).metrics[0];assert.equal(metric.substrate_connected_components,metric.original_terrain_components);assert(metric.closed_plates_edge_multiplicity_and_winding_verified);assert(metric.walk_faces_recess_only);assert(metric.wall_site_field_shared_across_cells);
 return {kind,sha256:sha(bytes),triangles,rock_triangles:generated.doc.accessors[gp.indices].count/3,non_rock_batches_byte_exact:11,physical_root_triangles_exact:metric.protected_root_rock_triangles,bounds_exact:true,max_normal_length_error:maxNormal,max_tangent_length_error:maxTangent,max_metric_uv_height_error:maxUv,min_normal_dot:minDot,closed_substrate_components:metric.substrate_connected_components,original_terrain_components:metric.original_terrain_components,closed_geological_joint_solids:true,terrain_soil_collision_lod_landmarks_routes_unchanged:true};
}
const inventory=JSON.parse(fs.readFileSync(path.join(root,'baseline-inventory.json'),'utf8').replace(/^\uFEFF/,''));for(const f of inventory.files){const bytes=fs.readFileSync(path.join(root,f.path));assert.equal(bytes.length,f.bytes);assert.equal(sha(bytes),f.sha256,'frozen input '+f.path);}
assert.equal(build.author_sha256,sha(fs.readFileSync(path.join(root,'build_cliff.py'))));
for(const file of ['collisions.json','landmarks.json','dawn-lod.glb','dawn-watch-lod.glb'])assert(fs.readFileSync(path.join(root,'candidate/assets/world',file)).equals(fs.readFileSync(path.join(root,'baseline/assets/world',file))),file+' exact');
const islands=['dawn','dawn-watch'].map(kind=>inspect(kind,fs.readFileSync(path.join(root,'candidate/assets/world',kind+'.glb'))));
const corruptions=[];
if(process.argv.includes('--self-test'))for(const kind of ['dawn','dawn-watch']){
 const original=fs.readFileSync(path.join(root,'candidate/assets/world',kind+'.glb'));
 const mutateAttribute=(bytes,prefix,semantic,component,value)=>{const g=glb(bytes),p=g.doc.meshes.find(m=>m.name.startsWith(prefix)).primitives[0],a=g.doc.accessors[p.attributes[semantic]],v=g.doc.bufferViews[a.bufferView],n=bytes.readUInt32LE(12);bytes.writeFloatLE(value,28+n+(v.byteOffset??0)+(a.byteOffset??0)+component*4);};
 for(const [label,mutate,expected] of [
  ['header',b=>b.writeUInt32LE(b.length-4,8),'GLB size'],
  ['architecture-position',b=>mutateAttribute(b,'03 |','POSITION',0,10000),'non-rock'],
  ['soil-position',b=>mutateAttribute(b,'12 |','POSITION',1,10000),'non-rock'],
  ['rock-normal-nan',b=>mutateAttribute(b,'01 |','NORMAL',0,NaN),'finite normal'],
  ['rock-tangent-zero',b=>{mutateAttribute(b,'01 |','TANGENT',0,0);mutateAttribute(b,'01 |','TANGENT',1,0);mutateAttribute(b,'01 |','TANGENT',2,0);},'unit tangents'],
  ['rock-bounds',b=>mutateAttribute(b,'01 |','POSITION',0,-10000),'bounds'],
 ]){const bytes=Buffer.from(original);mutate(bytes);const record=structuredClone(manifest[kind]);record.high.sha256=sha(bytes);let reason='';try{inspect(kind,bytes,record);}catch(e){reason=String(e.message);}assert(reason,label+' rejected');assert(reason.includes(expected),label+' rejected at intended contract, got '+reason);corruptions.push({kind,case:label,rejected:true,reason});}
}
const report={schema:1,islands,corruptions};fs.writeFileSync(path.join(root,'gate-evidence.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report,null,2));
