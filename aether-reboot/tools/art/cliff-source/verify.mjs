// Independent R59 geometry contracts. No Blender or renderer is required.
import fs from 'node:fs';import path from 'node:path';import crypto from 'node:crypto';import assert from 'node:assert/strict';import {fileURLToPath,pathToFileURL} from 'node:url';
const root=path.dirname(fileURLToPath(import.meta.url)),sha=b=>crypto.createHash('sha256').update(b).digest('hex');
const assets=path.resolve(root,'../../../assets/world');
const manifest=JSON.parse(fs.readFileSync(path.join(assets,'manifest.json'))),build=JSON.parse(fs.readFileSync(path.join(root,'build-evidence.json')));
const groundRoot=path.resolve(root,'../ground-r60-source'),groundModule=path.join(groundRoot,'validate-ground-delta.mjs'),groundURI='textures/087deb4c68767e876653edc1b1fd0890b0ce5cb051beb5a29d03de718806b365.png';let groundAPI=null;
if(fs.existsSync(groundModule)){assert.equal(sha(fs.readFileSync(groundModule)),'6ff191df9ff34a8d5c5283ce111cb5db1c95088ba78cbb1827abc465130f6c8c','independent R60 API exact');groundAPI=await import(pathToFileURL(groundModule));}
// Authenticate material provenance against the complete historical manifest,
// in addition to the per-GLB geometry/material deltas below.
let groundWorldProof=null;const groundWorldModule=path.join(groundRoot,'validate-world.mjs');
if(fs.existsSync(groundWorldModule)){
 assert.equal(sha(fs.readFileSync(groundWorldModule)),'40bc03e3e67246ab966a064b8f721ad1aa3cf1a6ca86155a872f42f7d75ac4aa','independent whole R60 world API exact');
 assert.equal(sha(fs.readFileSync(path.join(groundRoot,'history/r59-portable-world/manifest.json'))),'5fe969215a7e20c2e61050485a5a934781335bc76cb7ab9c426e4a6072d901c9','whole pre-R60 manifest authenticated');
 const api=await import(pathToFileURL(groundWorldModule));groundWorldProof=api.validateGroundR60World({root:path.resolve(root,'../../..'),world:assets});
}
if(['dawn','dawn-watch'].some(k=>Object.hasOwn(manifest[k],'ground_r60')))assert(groundWorldProof?.applied,'R60 manifest metadata requires full-world authentication');
const sizes={5126:4,5125:4,5123:2},widths={SCALAR:1,VEC2:2,VEC3:3,VEC4:4};
function glb(bytes){assert.equal(bytes.readUInt32LE(0),0x46546c67,'GLB magic');assert.equal(bytes.readUInt32LE(4),2,'GLB version');assert.equal(bytes.readUInt32LE(8),bytes.length,'GLB size');const n=bytes.readUInt32LE(12);assert.equal(bytes.readUInt32LE(16),0x4e4f534a);assert.equal(bytes.readUInt32LE(24+n),0x004e4942);return {bytes,doc:JSON.parse(bytes.subarray(20,20+n)),bin:bytes.subarray(28+n)};}
const load=f=>glb(fs.readFileSync(f));
function raw(g,index){const a=g.doc.accessors[index],v=g.doc.bufferViews[a.bufferView],w=widths[a.type]*sizes[a.componentType];assert(w>0);assert(!a.sparse);const b=Buffer.alloc(a.count*w);for(let i=0;i<a.count;i++){const o=(v.byteOffset??0)+(a.byteOffset??0)+i*(v.byteStride??w);assert(o+w<=g.bin.length,'accessor outside binary');g.bin.copy(b,i*w,o,o+w);}return b;}
function rows(g,index){const a=g.doc.accessors[index],b=raw(g,index),n=widths[a.type],s=sizes[a.componentType];return Array.from({length:a.count},(_,i)=>Array.from({length:n},(_,k)=>a.componentType===5126?b.readFloatLE((i*n+k)*s):s===4?b.readUInt32LE((i*n+k)*s):b.readUInt16LE((i*n+k)*s)));}
function bounds(g){const result=[[Infinity,-Infinity],[Infinity,-Infinity],[Infinity,-Infinity]];for(const m of g.doc.meshes)for(const p of m.primitives)for(const xyz of rows(g,p.attributes.POSITION))for(let k=0;k<3;k++){result[k][0]=Math.min(result[k][0],xyz[k]);result[k][1]=Math.max(result[k][1],xyz[k]);}return result;}
function signatures(g,p){const attrs=Object.keys(p.attributes).sort().map(k=>{const a=g.doc.accessors[p.attributes[k]];return {bytes:raw(g,p.attributes[k]),width:widths[a.type]*sizes[a.componentType]};}),ids=rows(g,p.indices).flat(),result=new Map();for(let i=0;i<ids.length;i+=3){const hex=ids.slice(i,i+3).map(id=>Buffer.concat(attrs.map(a=>a.bytes.subarray(id*a.width,(id+1)*a.width))).toString('hex')),key=[0,1,2].map(j=>[...hex.slice(j),...hex.slice(0,j)].join(':')).sort()[0];result.set(key,(result.get(key)??0)+1);}return result;}
function verifyTopology(g,p,witness,colliders){
 const pos=rows(g,p.attributes.POSITION),ids=rows(g,p.indices).flat(),actualPoint=q=>q.join(','),cycle=v=>[0,1,2].map(i=>[...v.slice(i),...v.slice(0,i)].join(':')).sort()[0],triangle=(a,b,c)=>{const u=b.map((v,k)=>v-a[k]),v=c.map((v,k)=>v-a[k]);return [u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]];};
 const bins=new Map(),unique=new Map(pos.map(q=>[actualPoint(q),q])),snapped=new Map();for(const [key,q] of unique){const b=q.map(v=>Math.floor(v*10000)).join(',');if(!bins.has(b))bins.set(b,[]);bins.get(b).push({key,q});}
 const point=q=>{const key=q.join(',');if(snapped.has(key))return snapped.get(key);const b=q.map(v=>Math.floor(v*10000));let closest=null,distance=Infinity;for(let x=-1;x<=1;x++)for(let y=-1;y<=1;y++)for(let z=-1;z<=1;z++)for(const v of bins.get([b[0]+x,b[1]+y,b[2]+z].join(','))??[]){const d=Math.hypot(...q.map((t,k)=>t-v.q[k]));if(d<distance){distance=d;closest=v.key;}}assert(distance<3e-5,'closed-body witness vertex present in actual fp32 mesh');snapped.set(key,closest);return closest;};
 const actual=new Map(),incident=new Map();
 for(let i=0;i<ids.length;i+=3){const corners=ids.slice(i,i+3).map(id=>pos[id]),keys=corners.map(actualPoint),key=cycle(keys);if(actual.has(key))continue;const t={keys,corners,key};actual.set(key,t);for(const k of keys){if(!incident.has(k))incident.set(k,[]);incident.get(k).push(t);}}
 const facesPresent=(points,faces)=>{
  const edges=new Map();
  for(const face of faces){
   for(let i=0;i<face.length;i++){const a=face[i],b=face[(i+1)%face.length],key=[a,b].sort((a,b)=>a-b).join(',');if(!edges.has(key))edges.set(key,[]);edges.get(key).push([a,b]);}
   const q=face.map(i=>points[i]),keys=q.map(point),has=t=>actual.has(cycle(t.map(i=>keys[i])));
   if(q.length===3)assert(has([0,1,2]),'actual closed-body triangular face');
   else if(q.length===4)assert((has([0,1,2])&&has([0,2,3]))||(has([0,1,3])&&has([1,2,3])),'actual closed-body quadrilateral face');
   else{
    const allowed=new Set(keys),candidates=new Map();for(const k of keys)for(const t of incident.get(k)??[])if(t.keys.every(v=>allowed.has(v)))candidates.set(t.key,t);
    const normal=triangle(q[0],q[1],q[2]),oriented=[...candidates.values()].filter(t=>triangle(...t.corners).reduce((s,v,k)=>s+v*normal[k],0)>0);
    assert.equal(oriented.length,q.length-2,'actual closed-body n-gon triangulation');
    const expected=q.slice(1,-1).reduce((s,_,i)=>s+Math.hypot(...triangle(q[0],q[i+1],q[i+2]))*.5,0),area=oriented.reduce((s,t)=>s+Math.hypot(...triangle(...t.corners))*.5,0);
    assert(Math.abs(area-expected)<Math.max(.002,expected*1e-4),'actual closed-body n-gon coverage');
   }
  }
  for(const pair of edges.values())assert(pair.length===2&&pair[0][0]===pair[1][1]&&pair[0][1]===pair[1][0],'closed-body edge multiplicity and winding');
 };
 const boxFaces=[[0,3,2,1],[4,5,6,7],[0,4,7,3],[1,2,6,5],[3,7,6,2],[0,1,5,4]];
 for(const [xmin,xmax,ymin,ymax,zmin,zmax] of witness.cores){const points=[[xmin,ymin,zmin],[xmax,ymin,zmin],[xmax,ymax,zmin],[xmin,ymax,zmin],[xmin,ymin,zmax],[xmax,ymin,zmax],[xmax,ymax,zmax],[xmin,ymax,zmax]];facesPresent(points,boxFaces);}
 for(const plate of witness.plates)facesPresent(plate.points,plate.faces);
 const remaining=new Set(witness.cores.map((_,i)=>i));let connected=0;
 while(remaining.size){connected++;const pending=[remaining.values().next().value];remaining.delete(pending[0]);while(pending.length){const a=witness.cores[pending.pop()];for(const j of [...remaining]){const b=witness.cores[j],overlap=[0,2,4].map(k=>Math.min(a[k+1],b[k+1])-Math.max(a[k],b[k]));if(Math.min(...overlap)>=-1e-8&&overlap.filter(v=>v>1e-5).length>=2){remaining.delete(j);pending.push(j);}}}}
 const cellSet=new Set(witness.cells.map(([x,z])=>[x,z].join(','))),cells=new Set(cellSet);let original=0;
 while(cells.size){original++;const first=cells.values().next().value,pending=[first];cells.delete(first);while(pending.length){const [x,z]=pending.pop().split(',').map(Number);for(const [dx,dz] of [[1,0],[-1,0],[0,1],[0,-1]]){const q=[x+dx,z+dz].join(',');if(cells.delete(q))pending.push(q);}}}
 assert.equal(connected,original,'actual substrate connected like original terrain');
 for(const [ix,iz,h,d] of witness.cells){const spacing=witness.spacing,size=spacing===8?8.04:6,center=[ix*spacing,(h-d)/2,iz*spacing],dimensions=[size,h+d,size];assert(colliders.some(c=>c.center.every((v,k)=>Math.abs(v-center[k])<1e-4)&&c.size.every((v,k)=>Math.abs(v-dimensions[k])<1e-4)),'topology witness matches actual physical terrain cell');}
 return {closed_substrate_solids:witness.cores.length,closed_fracture_solids:witness.plates.length,actual_mesh_faces_verified:true,original_physical_cells:witness.cells.length,actual_connected_components:connected,original_terrain_components:original};
}
function inspect(kind,bytes,record=manifest[kind]){
 assert.equal(sha(bytes),record.high.sha256,'current GLB sha');assert.equal(bytes.length,record.high.bytes,'current GLB bytes');
 const g=glb(bytes),baseline=load(path.join(root,'baseline/assets/world',kind+'.glb')),provenance=g.doc.asset.extras.aether_cliff_r59;
 let surfaceDoc=g.doc,groundProof=null;
 if(g.doc.images.some(i=>i.uri===groundURI)){
  assert(groundAPI,'R60 material requires independent source contract');const authoredR59=JSON.parse(fs.readFileSync(path.join(root,'assembly-evidence.json'))).islands.find(i=>i.kind===kind);assert(authoredR59);
  const beforeRaw=fs.readFileSync(path.join(groundRoot,'history/r59-portable-world',kind+'.glb')),nativePNG=fs.readFileSync(path.join(assets,groundURI));
  groundProof=groundAPI.validateGroundDelta(beforeRaw,bytes,{expectedInputSha:authoredR59.sha256,nativePNG});surfaceDoc=groundProof.projectedDoc;
 }
 assert.deepEqual(provenance,record.cliff_r59,'provenance manifest');assert.equal(provenance.baseline_glb_sha256,sha(baseline.bytes),'baseline provenance');
 assert.equal(provenance.author_sha256,sha(fs.readFileSync(path.join(root,'build_cliff.py'))),'actual author');assert.equal(provenance.assembly_sha256,sha(fs.readFileSync(path.join(root,'assemble.mjs'))),'actual assembly');
 for(const field of ['nodes','materials','images','textures','samplers','scenes','scene','extensionsUsed','extensionsRequired'])assert.deepEqual(surfaceDoc[field],baseline.doc[field],field+' unchanged after authenticated material12 R60 projection');
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
 const topologyBytes=fs.readFileSync(path.join(root,'output',kind+'-topology-witness.json'));assert.equal(sha(topologyBytes),provenance.topology_witness_sha256);assert.equal(sha(topologyBytes),metric.topology_witness_sha256);const topology=verifyTopology(g,p,JSON.parse(topologyBytes),JSON.parse(fs.readFileSync(path.join(root,'baseline/assets/world/collisions.json')))[kind]);
 const groundSummary=groundProof?{passed:groundProof.passed,input_sha256:groundProof.input_sha256,output_sha256:groundProof.output_sha256,native_png_sha256:groundProof.native_png_sha256,bin_byte_exact:groundProof.bin_byte_exact,all_unrelated_json_exact:groundProof.all_unrelated_json_exact}:null;
 return {kind,sha256:sha(bytes),triangles,rock_triangles:generated.doc.accessors[gp.indices].count/3,non_rock_batches_byte_exact:11,physical_root_triangles_exact:metric.protected_root_rock_triangles,bounds_exact:true,max_normal_length_error:maxNormal,max_tangent_length_error:maxTangent,max_metric_uv_height_error:maxUv,min_normal_dot:minDot,closed_substrate_components:metric.substrate_connected_components,original_terrain_components:metric.original_terrain_components,closed_geological_joint_solids:true,actual_mesh_topology:topology,material12_r60_independent_contract:groundSummary,terrain_soil_collision_lod_landmarks_routes_unchanged:true};
}
const inventory=JSON.parse(fs.readFileSync(path.join(root,'baseline-inventory.json'),'utf8').replace(/^\uFEFF/,''));for(const f of inventory.files){const bytes=fs.readFileSync(path.join(root,f.path));assert.equal(bytes.length,f.bytes);assert.equal(sha(bytes),f.sha256,'frozen input '+f.path);}
assert.equal(build.author_sha256,sha(fs.readFileSync(path.join(root,'build_cliff.py'))));
for(const file of ['collisions.json','landmarks.json'])assert(fs.readFileSync(path.join(assets,file)).equals(fs.readFileSync(path.join(root,'baseline/assets/world',file))),file+' exact');
const lodChecks=[];
for(const kind of ['dawn','dawn-watch']){
 const file=kind+'-lod.glb',bytes=fs.readFileSync(path.join(assets,file)),frozen=fs.readFileSync(path.join(root,'baseline/assets/world',file));
 assert.equal(sha(bytes),manifest[kind].lod.sha256,'current LOD SHA');assert.equal(bytes.length,manifest[kind].lod.bytes,'current LOD bytes');
 if(glb(bytes).doc.images.some(i=>i.uri===groundURI)){
  assert(groundAPI,'R60 LOD material requires independent source contract');const proof=groundAPI.validateGroundDelta(fs.readFileSync(path.join(groundRoot,'history/r59-portable-world',file)),bytes,{expectedInputSha:sha(frozen),nativePNG:fs.readFileSync(path.join(assets,groundURI))});assert.deepEqual(proof.projectedDoc,glb(frozen).doc);assert(proof.bin.equals(glb(frozen).bin));lodChecks.push({file,material12_r60_validated:true,all_geometry_bin_and_projected_json_exact:true});
 }else{assert(bytes.equals(frozen),file+' exact');lodChecks.push({file,material12_r60_validated:false,file_byte_exact:true});}
}
const islands=['dawn','dawn-watch'].map(kind=>inspect(kind,fs.readFileSync(path.join(assets,kind+'.glb'))));
const corruptions=[];
if(process.argv.includes('--self-test'))for(const kind of ['dawn','dawn-watch']){
 const original=fs.readFileSync(path.join(assets,kind+'.glb'));
 const mutateAttribute=(bytes,prefix,semantic,component,value)=>{const g=glb(bytes),p=g.doc.meshes.find(m=>m.name.startsWith(prefix)).primitives[0],a=g.doc.accessors[p.attributes[semantic]],v=g.doc.bufferViews[a.bufferView],n=bytes.readUInt32LE(12);bytes.writeFloatLE(value,28+n+(v.byteOffset??0)+(a.byteOffset??0)+component*4);};
 for(const [label,mutate,expected] of [
  ['header',b=>b.writeUInt32LE(b.length-4,8),'GLB size'],
  ['architecture-position',b=>mutateAttribute(b,'03 |','POSITION',0,10000),'non-rock'],
  ['soil-position',b=>mutateAttribute(b,'12 |','POSITION',1,10000),'non-rock'],
  ['rock-normal-nan',b=>mutateAttribute(b,'01 |','NORMAL',0,NaN),'finite normal'],
  ['rock-tangent-zero',b=>{mutateAttribute(b,'01 |','TANGENT',0,0);mutateAttribute(b,'01 |','TANGENT',1,0);mutateAttribute(b,'01 |','TANGENT',2,0);},'unit tangents'],
  ['rock-bounds',b=>mutateAttribute(b,'01 |','POSITION',0,-10000),'bounds'],
 ]){const bytes=Buffer.from(original);mutate(bytes);const record=structuredClone(manifest[kind]);record.high.sha256=sha(bytes);let reason='';try{inspect(kind,bytes,record);}catch(e){reason=String(e.message);}assert(reason,label+' rejected');const expectedReason=label!=='header'&&glb(original).doc.images.some(i=>i.uri===groundURI)?'Entire BIN byte-exact':expected;assert(reason.includes(expectedReason),label+' rejected at intended contract, got '+reason);corruptions.push({kind,case:label,rejected:true,reason});}
}
const report={schema:1,islands,lod:lodChecks,ground_r60_whole_world_provenance:groundWorldProof,corruptions};fs.writeFileSync(path.join(root,'gate-evidence.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report,null,2));
