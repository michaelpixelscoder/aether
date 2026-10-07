import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import {fileURLToPath,pathToFileURL} from 'node:url';
const root = path.dirname(fileURLToPath(import.meta.url));
const assets=fs.existsSync(path.join(root,'assets/world'))?path.join(root,'assets/world'):path.resolve(root,'../../../assets/world');
const sha = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const groundRoot=path.resolve(root,'../ground-r60-source'),groundModule=path.join(groundRoot,'validate-ground-delta.mjs'),groundURI='textures/087deb4c68767e876653edc1b1fd0890b0ce5cb051beb5a29d03de718806b365.png';let groundAPI=null;
if(fs.existsSync(groundModule)){assert.equal(sha(fs.readFileSync(groundModule)),'6ff191df9ff34a8d5c5283ce111cb5db1c95088ba78cbb1827abc465130f6c8c','independent R60 API exact');groundAPI=await import(pathToFileURL(groundModule));}
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
// R60 provenance is exempted from the historical R56 manifest comparison
// only after the independent validator authenticates the entire final world,
// actual source PNG, complete metadata and recovered historical manifest.
let groundWorldProof=null;
const groundWorldModule=path.join(groundRoot,'validate-world.mjs');
if(fs.existsSync(groundWorldModule)){
 assert.equal(sha(fs.readFileSync(groundWorldModule)),'40bc03e3e67246ab966a064b8f721ad1aa3cf1a6ca86155a872f42f7d75ac4aa','independent whole R60 world API exact');
 assert.equal(sha(fs.readFileSync(path.join(groundRoot,'history/r59-portable-world/manifest.json'))),'5fe969215a7e20c2e61050485a5a934781335bc76cb7ab9c426e4a6072d901c9','whole pre-R60 manifest authenticated');
 const api=await import(pathToFileURL(groundWorldModule));groundWorldProof=api.validateGroundR60World({root:path.resolve(root,'../../..'),world:assets});
}
if(['dawn','dawn-watch'].some(k=>Object.hasOwn(manifest[k],'ground_r60')))assert(groundWorldProof?.applied,'R60 manifest metadata requires full-world authentication');
const originalManifest=JSON.parse(fs.readFileSync(path.join(root,'baseline/assets/world/manifest.json')));
const authoring=JSON.parse(fs.readFileSync(path.join(root,'evidence/authoring.json')));
assert.equal(authoring.source_sha256,sha(fs.readFileSync(path.join(root,'build_terraces.py'))));
assert.equal(authoring.frozen_helpers_sha256,sha(fs.readFileSync(path.join(root,'baseline/tools/art/build_world.py'))));
function inspect(kind,bytes) {
 const g=glb(bytes),b=glb(fs.readFileSync(path.join(root,'baseline/assets/world',kind+'.glb'))),record=manifest[kind],provenance=g.doc.asset.extras.aether_terraces_r56;
 let surfaceDoc=g.doc,groundProof=null;
 if(g.doc.images.some(i=>i.uri===groundURI)){
  assert(groundAPI,'R60 material requires independent source contract');const authoredR59=JSON.parse(fs.readFileSync(path.join(root,'../cliff-source/assembly-evidence.json'))).islands.find(i=>i.kind===kind);assert(authoredR59);
  const beforeRaw=fs.readFileSync(path.join(groundRoot,'history/r59-portable-world',kind+'.glb')),nativePNG=fs.readFileSync(path.join(assets,groundURI));
  groundProof=groundAPI.validateGroundDelta(beforeRaw,bytes,{expectedInputSha:authoredR59.sha256,nativePNG});surfaceDoc=groundProof.projectedDoc;
 }
 assert.equal(sha(bytes),record.high.sha256,'final GLB SHA');assert.equal(bytes.length,record.high.bytes);
 assert.deepEqual(provenance,record.terraces_r56);assert.equal(provenance.baseline_glb_sha256,sha(b.bytes));
 assert.equal(provenance.author_sha256,authoring.source_sha256);assert.equal(provenance.frozen_helpers_sha256,authoring.frozen_helpers_sha256);
 assert.equal(provenance.additions_glb_sha256,sha(fs.readFileSync(path.join(root,'source',kind+'-additions.glb'))));
 assert.equal(provenance.assembly_sha256,sha(fs.readFileSync(path.join(root,'assemble_terraces.mjs'))));
 let cliffDelta=0;const laterCliff=g.doc.asset.extras.aether_cliff_r59;
 if(laterCliff){
  assert.deepEqual(laterCliff,record.cliff_r59,'R59 provenance manifest');
  const source=path.resolve(root,'../cliff-source'),authored=glb(fs.readFileSync(path.join(source,'output',kind+'-candidate.glb'))),frozen=glb(fs.readFileSync(path.join(source,'baseline/assets/world',kind+'.glb')));
  assert.equal(sha(authored.bytes),laterCliff.candidate_rock_payload_sha256);assert.equal(sha(frozen.bytes),laterCliff.baseline_glb_sha256);
  assert.equal(sha(fs.readFileSync(path.join(source,'build_cliff.py'))),laterCliff.author_sha256);assert.equal(sha(fs.readFileSync(path.join(source,'assemble.mjs'))),laterCliff.assembly_sha256);
  const p=g.doc.meshes.find(m=>m.name.startsWith('01 |')).primitives[0],q=authored.doc.meshes[0].primitives[0],fp=frozen.doc.meshes.find(m=>m.name.startsWith('01 |')).primitives[0];
  for(const field of Object.keys(p.attributes))assert(raw(g,p.attributes[field]).equals(raw(authored,q.attributes[field])),'actual R59 rock '+field);assert.deepEqual(rows(g,p.indices),rows(authored,q.indices),'actual R59 corners');
  cliffDelta=g.doc.accessors[p.indices].count/3-frozen.doc.accessors[fp.indices].count/3;
 }
 for(const field of ['materials','images','textures','samplers','nodes','scenes','scene','extensionsUsed','extensionsRequired'])assert.deepEqual(surfaceDoc[field],b.doc[field],field+' exact after authenticated material12 R60 projection');
 assert.equal(g.doc.meshes.length,12);assert.equal(g.doc.materials.length,12);assert.equal(g.doc.nodes.length,12);
 const preserved={...record};delete preserved.high;delete preserved.terraces_r56;if(laterCliff)delete preserved.cliff_r59;const oldPreserved={...originalManifest[kind]};delete oldPreserved.high;
 if(groundWorldProof?.applied)delete preserved.ground_r60;
 if(lodChecks.find(i=>i.file===kind+'-lod.glb')?.material12_r60_validated)preserved.lod={...preserved.lod,bytes:oldPreserved.lod.bytes,sha256:oldPreserved.lod.sha256};
 assert.deepEqual(preserved,oldPreserved,'all non-HD manifest metadata unchanged except authenticated R60 LOD bytes/SHA');
 let triangles=0,added=0,maxNormalError=0,maxTangentError=0,degenerate=0,inverted=0,addedFoliageArea=0;const batches=[];
 const bounds=g=>{const result=[[Infinity,-Infinity],[Infinity,-Infinity],[Infinity,-Infinity]];for(const mesh of g.doc.meshes)for(const p of mesh.primitives)for(const xyz of rows(g,p.attributes.POSITION))for(let j=0;j<3;j++){result[j][0]=Math.min(result[j][0],xyz[j]);result[j][1]=Math.max(result[j][1],xyz[j]);}return result;};
 assert.deepEqual(bounds(g),bounds(b),'whole island AABB exact');
 for(let m=0;m<g.doc.meshes.length;m++){
  const mesh=g.doc.meshes[m],old=b.doc.meshes[m];assert.equal(mesh.name,old.name);assert.equal(mesh.primitives.length,1);
  const p=mesh.primitives[0],q=old.primitives[0],prefix=mesh.name.slice(0,2),modified=['06','08','09'].includes(prefix);
  assert.equal(p.material,q.material);assert.equal(p.mode,q.mode);assert.deepEqual(Object.keys(p.attributes),Object.keys(q.attributes));
  if(laterCliff && prefix==='01'){
   // Exact authored R59 payload was compared above. No other historical
   // terrain/architecture batch gets a replacement exception.
   const rockTriangles=g.doc.accessors[p.indices].count/3;triangles+=rockTriangles;batches.push({mesh:mesh.name,later_replacement:'R59',triangle_delta:cliffDelta,authored_r59_payload_byte_exact:true});continue;
  }
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
  triangles+=ids.length/3;if(modified)added+=(ids.length-oldIds.length)/3;batches.push({mesh:mesh.name,added_triangles:(ids.length-oldIds.length)/3,original_attributes_and_corners_exact:true});
 }
 assert.equal(triangles,record.high.triangles);assert(triangles<=200000);assert.equal(added,provenance.added_triangles);assert.equal(degenerate,0);assert.equal(inverted,0);assert(maxNormalError<1e-4);assert(maxTangentError<1e-3);
 const authored=authoring.islands.find(i=>i.kind===kind);assert.equal(authored.added_triangles,added);assert.equal(authored.combined_triangles+cliffDelta,triangles);assert.equal(authored.collision_changes,0);assert.equal(authored.lod_changes,0);
 for(const image of g.doc.images){const texture=fs.readFileSync(path.join(assets,image.uri));assert.equal(sha(texture),path.basename(image.uri).split('.')[0]);if(groundProof&&image.uri===groundURI)continue;assert(texture.equals(fs.readFileSync(path.join(root,'baseline/assets/world',image.uri))));}
 return {kind,sha256:sha(bytes),triangles,added_triangles:added,later_r59_rock_delta:cliffDelta,material12_r60_validated:!!groundProof,meshes:12,nodes:12,materials:12,batches,added_foliage_surface_area_m2:addedFoliageArea,max_normal_length_error:maxNormalError,max_tangent_length_error:maxTangentError,degenerate_added_triangles:degenerate,inverted_added_triangles:inverted,architecture_and_practicable_terrain_exact:true,rock_replaced_by_r59:!!laterCliff,whole_island_aabb_exact:true,lod_collision_landmark_routes_exact:true};
}
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
const corruptionTests=[];
if(process.argv.includes('--self-test'))for(const kind of ['dawn','dawn-watch']){
 const original=fs.readFileSync(path.join(assets,kind+'.glb'));
 for(const [label,mutate] of [['payload',b=>{b[b.length-16]^=1;}],['header-length',b=>{b.writeUInt32LE(b.length-4,8);}],['json',b=>{b[20]=0;}]]){
  const corrupted=Buffer.from(original);mutate(corrupted);assert.throws(()=>inspect(kind,corrupted),kind+' '+label+' rejected');corruptionTests.push({kind,case:label,rejected:true});
 }
}
const report={schema:1,islands,lod:lodChecks,ground_r60_whole_world_provenance:groundWorldProof,corruption_tests:corruptionTests};
fs.writeFileSync(path.join(root,'evidence/terraces-gate.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report,null,2));
