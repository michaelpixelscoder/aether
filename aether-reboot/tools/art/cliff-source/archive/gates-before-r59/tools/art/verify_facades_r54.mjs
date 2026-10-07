import fs from 'node:fs';import path from 'node:path';import crypto from 'node:crypto';import assert from 'node:assert/strict';import {fileURLToPath} from 'node:url';
const root=fileURLToPath(new URL('../../',import.meta.url)),archive=path.join(root,'tools/art/architecture-source/archive'),sha=b=>crypto.createHash('sha256').update(b).digest('hex');
function glb(file){const bytes=fs.readFileSync(file),n=bytes.readUInt32LE(12);assert.equal(bytes.readUInt32LE(8),bytes.length);return{bytes,doc:JSON.parse(bytes.subarray(20,20+n)),bin:bytes.subarray(28+n)};}
const counts={SCALAR:1,VEC2:2,VEC3:3,VEC4:4};
function read(g,index){const a=g.doc.accessors[index],v=g.doc.bufferViews[a.bufferView],w=a.componentType===5126||a.componentType===5125?4:2,n=counts[a.type],start=(v.byteOffset??0)+(a.byteOffset??0),stride=v.byteStride??w*n;return Array.from({length:a.count},(_,i)=>Array.from({length:n},(_,j)=>{const o=start+i*stride+j*w;return a.componentType===5126?g.bin.readFloatLE(o):w===4?g.bin.readUInt32LE(o):g.bin.readUInt16LE(o);}));}
const report=[];
const manifest=JSON.parse(fs.readFileSync(path.join(root,'assets/world/manifest.json'))),baselineManifest=JSON.parse(fs.readFileSync(path.join(archive,'manifest.json')));
for(const kind of ['dawn','dawn-watch']){
 const g=glb(path.join(root,'assets/world',kind+'.glb')),old=glb(path.join(archive,kind+'.glb'));
 const pipeline=g.doc.asset.extras.aether_architecture_r54;
 assert.equal(manifest[kind].high.sha256,sha(g.bytes));assert.equal(manifest[kind].high.bytes,g.bytes.length);assert.deepEqual(manifest[kind].architecture_r54,pipeline);
 assert.equal(pipeline.original_glb_sha256,sha(old.bytes));
 for(const [key,file] of [['generator_sha256','build_facades_r54.py'],['patch_sha256','facades_r54.py'],['assembly_sha256','finish_facades_r54.mjs'],['base_sha256','build_world.py']])assert.equal(pipeline[key],sha(fs.readFileSync(path.join(root,'tools/art',file))),key);
 if(kind==='dawn-watch')assert.equal(pipeline.variant_sha256,sha(fs.readFileSync(path.join(root,'tools/art/build_world_variants.py'))));
 for(const image of g.doc.images){const bytes=fs.readFileSync(path.join(root,'assets/world',image.uri));assert.equal(sha(bytes),path.basename(image.uri).split('.')[0],'texture URI is content-addressed');}
 assert.equal(g.doc.meshes.length,old.doc.meshes.length);assert.equal(g.doc.nodes.length,old.doc.nodes.length);assert.equal(g.doc.materials.length,old.doc.materials.length);
 const poses=g=>g.doc.nodes.map(n=>({name:n.name,matrix:n.matrix,translation:n.translation,rotation:n.rotation,scale:n.scale,children:n.children}));assert.deepEqual(poses(g),poses(old),'same node identities and pivots');
 assert.deepEqual(g.doc.materials,old.doc.materials,'same material properties');assert.deepEqual(g.doc.images,old.doc.images,'same hashed texture dependencies');
 const bounds=v=>{const b=[[Infinity,-Infinity],[Infinity,-Infinity],[Infinity,-Infinity]];for(const m of v.doc.meshes)for(const p of m.primitives)for(const q of read(v,p.attributes.POSITION))for(let i=0;i<3;i++){b[i][0]=Math.min(b[i][0],q[i]);b[i][1]=Math.max(b[i][1],q[i]);}return b;};
 assert.deepEqual(bounds(g),bounds(old),'whole island extents exact');
 const modified=['03 |','04 |','05 |','07 |','10 |'];let worstNormal=0,worstTangent=0,degenerate=0,inverted=0,minDot=1,triangles=0;const unchanged=[];
 for(let m=0;m<g.doc.meshes.length;m++){
  const mesh=g.doc.meshes[m],p=mesh.primitives[0],op=old.doc.meshes[m].primitives[0],pos=read(g,p.attributes.POSITION),norm=read(g,p.attributes.NORMAL),tan=read(g,p.attributes.TANGENT),ind=read(g,p.indices).flat();triangles+=ind.length/3;
  for(let i=0;i<pos.length;i++){assert.ok(pos[i].every(Number.isFinite));worstNormal=Math.max(worstNormal,Math.abs(Math.hypot(...norm[i])-1));worstTangent=Math.max(worstTangent,Math.abs(Math.hypot(...tan[i].slice(0,3))-1));assert.ok(Math.abs(tan[i][3])===1);}
  for(let i=0;i<ind.length;i+=3){const ids=ind.slice(i,i+3),[a,b,c]=ids.map(j=>pos[j]),u=b.map((v,j)=>v-a[j]),v=c.map((v,j)=>v-a[j]),cross=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]],area=Math.hypot(...cross);if(area<1e-10){degenerate++;continue;}const n=[0,1,2].map(j=>ids.reduce((s,k)=>s+norm[k][j],0)),dot=cross.reduce((s,v,j)=>s+v*n[j],0)/(area*Math.hypot(...n));minDot=Math.min(minDot,dot);if(dot<-.001)inverted++;}
  if(!modified.some(prefix=>mesh.name.startsWith(prefix))){
   const fields=Object.keys(p.attributes);assert.deepEqual(fields,Object.keys(op.attributes));
   // R56 adds only new vertices/corners to these three botanical batches.
   // Every R54 prefix stays exact; its independent gate verifies the additions
   // and all R54 architecture bytes. No other batch gets this exception.
   const append=!!manifest[kind].terraces_r56 && ['06 |','08 |','09 |'].some(prefix=>mesh.name.startsWith(prefix));
   let maxError=0;for(const field of fields){const a=read(g,p.attributes[field]),b=read(old,op.attributes[field]);if(append)assert.ok(a.length>=b.length,mesh.name+field);else assert.equal(a.length,b.length,mesh.name+field);for(let i=0;i<b.length;i++)for(let j=0;j<b[i].length;j++)maxError=Math.max(maxError,Math.abs(a[i][j]-b[i][j]));}
   const oldIndices=read(old,op.indices).flat();assert.deepEqual(append?ind.slice(0,oldIndices.length):ind,oldIndices,mesh.name+' untouched indices');assert.equal(maxError,0,mesh.name);unchanged.push({mesh:mesh.name,max_error:maxError,later_append_only:append});
  }
 }
 assert.ok(worstNormal<1e-4 && worstTangent<1e-3);assert.equal(degenerate,0);assert.equal(inverted,0);
 assert.equal(triangles,manifest[kind].high.triangles);
 const later=manifest[kind].terraces_r56?.added_triangles??0;assert.ok(Number.isInteger(later)&&later>=0);
 assert.ok(triangles-later-baselineManifest[kind].high.triangles<=12000,'R54 per-island added triangle budget');
 const lod=fs.readFileSync(path.join(root,'assets/world',kind+'-lod.glb'));assert.ok(lod.equals(fs.readFileSync(path.join(archive,kind+'-lod.glb'))));
 report.push({kind,sha256:sha(g.bytes),triangles,unchanged_material_batches:unchanged,max_normal_length_error:worstNormal,max_tangent_length_error:worstTangent,degenerate_triangles:degenerate,inverted_triangles:inverted,min_normal_dot:minDot,lod_exact:true,nodes_and_pivots_exact:true,materials_and_images_exact:true,whole_island_extents_exact:true});
}
for(const name of ['collisions.json','landmarks.json'])assert.ok(fs.readFileSync(path.join(root,'assets/world',name)).equals(fs.readFileSync(path.join(archive,name))),name+' exact');
const evidence=path.join(root,'tools/art/architecture-source/evidence');fs.mkdirSync(evidence,{recursive:true});fs.writeFileSync(path.join(evidence,'facades-gate.json'),JSON.stringify({schema:1,islands:report},null,2)+'\n');console.log(JSON.stringify(report,null,2));
