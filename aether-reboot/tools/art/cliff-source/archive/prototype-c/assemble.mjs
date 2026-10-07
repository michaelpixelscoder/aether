import fs from 'node:fs';import path from 'node:path';import crypto from 'node:crypto';import assert from 'node:assert/strict';import {fileURLToPath} from 'node:url';
const root=path.dirname(fileURLToPath(import.meta.url)),sha=b=>crypto.createHash('sha256').update(b).digest('hex');
const load=file=>{const bytes=fs.readFileSync(file),n=bytes.readUInt32LE(12);return {bytes,doc:JSON.parse(bytes.subarray(20,20+n)),bin:bytes.subarray(28+n)};};
const widths={SCALAR:1,VEC2:2,VEC3:3,VEC4:4},sizes={5126:4,5125:4,5123:2};
function raw(g,i){const a=g.doc.accessors[i],v=g.doc.bufferViews[a.bufferView],w=widths[a.type]*sizes[a.componentType],b=Buffer.alloc(a.count*w);for(let j=0;j<a.count;j++)g.bin.copy(b,j*w,(v.byteOffset??0)+(a.byteOffset??0)+j*(v.byteStride??w),(v.byteOffset??0)+(a.byteOffset??0)+j*(v.byteStride??w)+w);return b;}
function values(g,i){const a=g.doc.accessors[i],b=raw(g,i),n=widths[a.type],s=sizes[a.componentType];return Array.from({length:a.count},(_,j)=>Array.from({length:n},(_,k)=>a.componentType===5126?b.readFloatLE((j*n+k)*s):s===4?b.readUInt32LE((j*n+k)*s):b.readUInt16LE((j*n+k)*s)));}
function bounds(g){const b=[[Infinity,-Infinity],[Infinity,-Infinity],[Infinity,-Infinity]];for(const m of g.doc.meshes)for(const p of m.primitives)for(const q of values(g,p.attributes.POSITION))for(let k=0;k<3;k++){b[k][0]=Math.min(b[k][0],q[k]);b[k][1]=Math.max(b[k][1],q[k]);}return b;}
function triangleSignatures(g,p){const fields=Object.keys(p.attributes).sort(),attributes=fields.map(k=>{const a=g.doc.accessors[p.attributes[k]];return {bytes:raw(g,p.attributes[k]),width:widths[a.type]*sizes[a.componentType]};}),corners=values(g,p.indices).flat(),signatures=new Map();for(let i=0;i<corners.length;i+=3){const hex=corners.slice(i,i+3).map(index=>Buffer.concat(attributes.map(({bytes,width})=>bytes.subarray(index*width,(index+1)*width))).toString('hex')),rotations=[0,1,2].map(j=>[...hex.slice(j),...hex.slice(0,j)].join(':')),key=rotations.sort()[0];signatures.set(key,(signatures.get(key)??0)+1);}return signatures;}
fs.mkdirSync(path.join(root,'candidate/assets/world/textures'),{recursive:true});
const report=[];
const build=JSON.parse(fs.readFileSync(path.join(root,'build-evidence.json')));
for(const kind of ['dawn','dawn-watch']){
 const base=load(path.join(root,'baseline/assets/world',kind+'.glb')),reg=load(path.join(root,'output',kind+'-regenerated-baseline.glb')),rock=load(path.join(root,'output',kind+'-candidate.glb'));
 const before=base.doc.meshes.find(m=>m.name.startsWith('01 |')).primitives[0],legacy=reg.doc.meshes[0].primitives[0],after=rock.doc.meshes[0].primitives[0];
 const metric=build.records.find(r=>r.kind===kind&&r.candidate).metrics[0];
 for(const field of Object.keys(before.attributes))assert.ok(raw(base,before.attributes[field]).equals(raw(reg,legacy.attributes[field])),kind+' regenerated baseline '+field);
 assert.deepEqual(values(base,before.indices),values(reg,legacy.indices),kind+' baseline corners');
 const doc=structuredClone(base.doc),views=[],accessors=[],chunks=[];let offset=0;
 const append=(data,def,target)=>{const pad=(4-offset%4)%4;if(pad){chunks.push(Buffer.alloc(pad));offset+=pad;}const vi=views.length;views.push({buffer:0,byteOffset:offset,byteLength:data.length,target});chunks.push(data);offset+=data.length;const ai=accessors.length;accessors.push({...structuredClone(def),bufferView:vi,byteOffset:0});return ai;};
 const unchanged=[];
 for(let j=0;j<doc.meshes.length;j++){
  const m=doc.meshes[j],p=m.primitives[0],changed=m.name.startsWith('01 |'),source=changed?rock:base,sp=changed?after:base.doc.meshes[j].primitives[0];
  for(const field of Object.keys(p.attributes)){const index=sp.attributes[field];assert(index!==undefined);p.attributes[field]=append(raw(source,index),source.doc.accessors[index],34962);}
  p.indices=append(raw(source,sp.indices),source.doc.accessors[sp.indices],34963);
  if(!changed)unchanged.push({name:m.name,attributes_sha256:Object.fromEntries(Object.entries(sp.attributes).map(([k,i])=>[k,sha(raw(source,i))])),indices_sha256:sha(raw(source,sp.indices))});
 }
 doc.bufferViews=views;doc.accessors=accessors;const bin=Buffer.concat(chunks);doc.buffers=[{byteLength:bin.length}];
 doc.asset.extras.aether_cliff_r59={baseline_glb_sha256:sha(base.bytes),baseline_rock_payload_sha256:sha(reg.bytes),candidate_rock_payload_sha256:sha(rock.bytes),author_sha256:sha(fs.readFileSync(path.join(root,'build_cliff.py'))),assembly_sha256:sha(fs.readFileSync(fileURLToPath(import.meta.url))),scope:'Replace 01 decorative cliff geometry only; all 11 other batch payloads, material/nodes/images/LOD and gameplay authority unchanged'};
 let json=Buffer.from(JSON.stringify(doc));json=Buffer.concat([json,Buffer.alloc((4-json.length%4)%4,32)]);const binary=Buffer.concat([bin,Buffer.alloc((4-bin.length%4)%4)]),head=Buffer.alloc(20),tail=Buffer.alloc(8);head.writeUInt32LE(0x46546c67);head.writeUInt32LE(2,4);head.writeUInt32LE(28+json.length+binary.length,8);head.writeUInt32LE(json.length,12);head.writeUInt32LE(0x4e4f534a,16);tail.writeUInt32LE(binary.length);tail.writeUInt32LE(0x004e4942,4);const bytes=Buffer.concat([head,json,tail,binary]);
 fs.writeFileSync(path.join(root,'candidate/assets/world',kind+'.glb'),bytes);
 const assembled={doc,bin};
 for(const field of ['materials','images','textures','samplers','nodes','scenes','scene','extensionsUsed','extensionsRequired'])assert.deepEqual(doc[field],base.doc[field],field+' byte-equivalent definitions');
 for(let j=0;j<doc.meshes.length;j++)if(!doc.meshes[j].name.startsWith('01 |')){
  const p=doc.meshes[j].primitives[0],q=base.doc.meshes[j].primitives[0];
  for(const field of Object.keys(p.attributes))assert.ok(raw(assembled,p.attributes[field]).equals(raw(base,q.attributes[field])),kind+' final non-rock '+field);
  assert.ok(raw(assembled,p.indices).equals(raw(base,q.indices)),kind+' final non-rock corners');
 }
 const roots=load(path.join(root,'output',kind+'-protected-roots.glb')),rootSignatures=triangleSignatures(roots,roots.doc.meshes[0].primitives[0]),oldSignatures=triangleSignatures(base,before),nextSignatures=triangleSignatures(rock,after);
 assert.equal(roots.doc.accessors[roots.doc.meshes[0].primitives[0].indices].count/3,metric.protected_root_rock_triangles);
 for(const [signature,count] of rootSignatures){assert((oldSignatures.get(signature)??0)>=count,'baseline root triangle all-corner attributes');assert((nextSignatures.get(signature)??0)>=count,'candidate root triangle all-corner attributes');}
 for(const img of doc.images)fs.copyFileSync(path.join(root,'baseline/assets/world',img.uri),path.join(root,'candidate/assets/world',img.uri));
 const result={kind,sha256:sha(bytes),bytes:bytes.length,triangles:doc.meshes.reduce((s,m)=>s+doc.accessors[m.primitives[0].indices].count/3,0),rock_triangles:rock.doc.accessors[after.indices].count/3,baseline_regeneration_payload_exact:true,unchanged_batches:unchanged,before_bounds:bounds(base),after_bounds:bounds({doc,bin})};result.bounds_exact=JSON.stringify(result.before_bounds)===JSON.stringify(result.after_bounds);assert.ok(result.triangles<=200000);report.push(result);
 const pp=values(rock,after.attributes.POSITION),nn=values(rock,after.attributes.NORMAL),tt=values(rock,after.attributes.TANGENT),ii=values(rock,after.indices).flat();let degenerate=0,inverted=0,worstN=0,worstT=0,minDot=1;
 for(const n of nn)worstN=Math.max(worstN,Math.abs(Math.hypot(...n)-1));for(const t of tt)worstT=Math.max(worstT,Math.abs(Math.hypot(...t.slice(0,3))-1));
 for(let i=0;i<ii.length;i+=3){const ids=ii.slice(i,i+3),[a,b,c]=ids.map(j=>pp[j]),u=b.map((v,j)=>v-a[j]),v=c.map((v,j)=>v-a[j]),x=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]],area=Math.hypot(...x);if(area<1e-10){degenerate++;continue;}const n=[0,1,2].map(j=>ids.reduce((s,k)=>s+nn[k][j],0)),dot=x.reduce((s,v,j)=>s+v*n[j],0)/(area*Math.hypot(...n));minDot=Math.min(minDot,dot);if(dot<-.001)inverted++;}
 let maxUv=0;const uv=values(rock,after.attributes.TEXCOORD_0);for(let i=0;i<pp.length;i++){assert(pp[i].every(Number.isFinite));assert(nn[i].every(Number.isFinite));assert(tt[i].every(Number.isFinite));assert.equal(Math.abs(tt[i][3]),1);assert(uv[i].every(Number.isFinite));if(Math.abs(nn[i][1])<.4&&Math.max(Math.abs(nn[i][0]),Math.abs(nn[i][2]))>.9)maxUv=Math.max(maxUv,Math.abs(uv[i][1]-(1-pp[i][1]/16)));}
 Object.assign(result,{degenerate_triangles:degenerate,inverted_triangles:inverted,max_normal_length_error:worstN,max_tangent_length_error:worstT,min_normal_dot:minDot,max_metric_uv_height_error:maxUv,protected_root_triangles:metric.protected_root_rock_triangles,protected_root_attributes_and_corners_exact:true,substrate_connected_components:metric.substrate_connected_components,original_terrain_components:metric.original_terrain_components});
 assert(result.bounds_exact);assert.equal(degenerate,0);assert.equal(inverted,0);assert(worstN<1e-4);assert(worstT<1e-3);assert(maxUv<.001);
}
const manifest=JSON.parse(fs.readFileSync(path.join(root,'baseline/assets/world/manifest.json')));
for(const r of report){const file=path.join(root,'candidate/assets/world',r.kind+'.glb'),doc=load(file).doc;manifest[r.kind].high={...manifest[r.kind].high,bytes:r.bytes,triangles:r.triangles,sha256:r.sha256};manifest[r.kind].cliff_r59=doc.asset.extras.aether_cliff_r59;}
fs.writeFileSync(path.join(root,'candidate/assets/world/manifest.json'),JSON.stringify(manifest,null,2)+'\n');
for(const file of ['collisions.json','landmarks.json','dawn-lod.glb','dawn-watch-lod.glb'])fs.copyFileSync(path.join(root,'baseline/assets/world',file),path.join(root,'candidate/assets/world',file));
fs.writeFileSync(path.join(root,'assembly-evidence.json'),JSON.stringify({schema:1,islands:report},null,2)+'\n');console.log(JSON.stringify(report.map(({unchanged_batches,...r})=>r),null,2));
