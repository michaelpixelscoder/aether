import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {fileURLToPath} from 'node:url';
import {readGlb,accessorBytes,sha,verifySailProvenance} from './sail_provenance.mjs';
const root=fileURLToPath(new URL('../../',import.meta.url));
const current=readGlb(path.join(root,'assets/art/sail.glb'));
const baseline=readGlb(path.join(root,'tools/art/sail-source/archive/r44-sail.glb'));
assert.equal(sha(baseline.bytes),'6fe29c97310f96a8c42e632da69684915571177fd341ea7734731f20567b4466','immutable R44 comparison');
const components={SCALAR:1,VEC2:2,VEC3:3,VEC4:4};
const floats=(g,index)=>{const a=g.doc.accessors[index],b=accessorBytes(g.doc,g.bin,index);assert.equal(a.componentType,5126);return Array.from({length:a.count},(_,i)=>Array.from({length:components[a.type]},(_,j)=>b.readFloatLE((i*components[a.type]+j)*4)));};
const ids=(g,index)=>{const a=g.doc.accessors[index],b=accessorBytes(g.doc,g.bin,index),w=a.componentType===5125?4:2;return Array.from({length:a.count},(_,i)=>w===4?b.readUInt32LE(i*w):b.readUInt16LE(i*w));};
const imageHashes=g=>g.doc.images.map(im=>{const v=g.doc.bufferViews[im.bufferView];return sha(g.bin.subarray(v.byteOffset,v.byteOffset+v.byteLength));}).sort();
assert.deepEqual(imageHashes(current),imageHashes(baseline),'exact same embedded cedar and navy source images');
assert.equal(current.doc.meshes.length,9);assert.equal(current.doc.materials.length,7);
assert.deepEqual(current.doc.materials,baseline.doc.materials,'no material/texture factor changes');
const cloth=current.doc.meshes.filter(m=>m.primitives.some(p=>p.targets));
const old=baseline.doc.meshes.filter(m=>m.primitives.some(p=>p.targets));
assert.equal(cloth.length,5);
let allBounds=[[Infinity,-Infinity],[Infinity,-Infinity],[Infinity,-Infinity]],maxNormalError=0,minDeformedNormal=Infinity,maxDeformedNormal=0,minDoubleArea=Infinity,minNormalDot=Infinity,negativePressureVertices=0,maxPressure=0;
const states=[];for(const p of [-1,0,1])for(const phase of [0,Math.PI/2,Math.PI,Math.PI*1.5])states.push([p,Math.sin(phase),Math.cos(phase)]);
const perSurface=[];
for(let m=0;m<cloth.length;m++){
 assert.deepEqual(cloth[m].extras.targetNames,['WindPressure','RippleSin','RippleCos']);
 const p=cloth[m].primitives[0],q=old[m].primitives[0];
 assert.equal(current.doc.accessors[p.indices].count,baseline.doc.accessors[q.indices].count,'same cloth triangles');
 assert.equal(current.doc.accessors[p.attributes.POSITION].count,baseline.doc.accessors[q.attributes.POSITION].count,'same exported cloth vertices');
 const pos=floats(current,p.attributes.POSITION),norm=floats(current,p.attributes.NORMAL),indices=ids(current,p.indices);
 const deltas=p.targets.map(t=>floats(current,t.POSITION)),normDeltas=p.targets.map(t=>floats(current,t.NORMAL));
 for(let i=0;i<pos.length;i++){
  const n=Math.hypot(...norm[i]);maxNormalError=Math.max(maxNormalError,Math.abs(n-1));
  assert.ok(pos[i].every(Number.isFinite));
  assert.ok(Math.abs(deltas[0][i][0])<1e-7 && Math.abs(deltas[0][i][1])<1e-7,'pressure only along actual +Z');
  if(deltas[0][i][2]<-1e-7)negativePressureVertices++;
  maxPressure=Math.max(maxPressure,deltas[0][i][2]);
 }
 for(const weights of states){
  const points=pos.map((v,i)=>v.map((x,j)=>x+weights.reduce((s,w,k)=>s+w*deltas[k][i][j],0)));
  const ns=norm.map((v,i)=>v.map((x,j)=>x+weights.reduce((s,w,k)=>s+w*normDeltas[k][i][j],0)));
  for(let i=0;i<points.length;i++){
   for(let j=0;j<3;j++){assert.ok(Number.isFinite(points[i][j]));allBounds[j][0]=Math.min(allBounds[j][0],points[i][j]);allBounds[j][1]=Math.max(allBounds[j][1],points[i][j]);}
   const length=Math.hypot(...ns[i]);assert.ok(Number.isFinite(length));minDeformedNormal=Math.min(minDeformedNormal,length);maxDeformedNormal=Math.max(maxDeformedNormal,length);
  }
  for(let i=0;i<indices.length;i+=3){
   const index=indices.slice(i,i+3),[a,b,c]=index.map(k=>points[k]),u=b.map((v,j)=>v-a[j]),v=c.map((v,j)=>v-a[j]);
   const cross=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]],area=Math.hypot(...cross);
   minDoubleArea=Math.min(minDoubleArea,area);assert.ok(area>1e-12,'no collapsed triangles at pressure/ripple extremes');
   const normal=[0,1,2].map(j=>index.reduce((s,k)=>s+ns[k][j],0)),dot=cross.reduce((s,x,j)=>s+x*normal[j],0)/(area*Math.hypot(...normal));
   minNormalDot=Math.min(minNormalDot,dot);assert.ok(dot>0,'no inverted morph triangles');
  }
 }
 perSurface.push({name:cloth[m].name,vertices:pos.length,triangles:indices.length/3,morphs:3});
}
assert.ok(maxNormalError<1e-4);assert.ok(minDeformedNormal>.5 && maxDeformedNormal<1.5);assert.equal(negativePressureVertices,0);assert.ok(maxPressure>.179 && maxPressure<.181);
assert.ok(allBounds[0][0]>=-1.251 && allBounds[0][1]<=1.251);
assert.ok(allBounds[1][0]>=-1.18 && allBounds[1][1]<=1.62);
assert.ok(allBounds[2][0]>-.75 && allBounds[2][1]<.22);
const grandMinimumY=4+2.5*allBounds[1][0];assert.ok(grandMinimumY>1.0,'GrandSail cloth stays above actual deck/cabin approach');
const pipeline=verifySailProvenance(current.doc,current.bin,root,current.bytes);
const report={schema:1,candidate_only:false,glb_sha256:sha(current.bytes),baseline_sha256:sha(baseline.bytes),per_surface:perSurface,wind_states:states,bounds_all_morph_extremes_xyz_m:allBounds,grand_sail_minimum_cloth_y_above_owner_m:grandMinimumY,normals:{rest_max_length_error:maxNormalError,deformed_min_length:minDeformedNormal,deformed_max_length:maxDeformedNormal,min_geometric_normal_dot:minNormalDot},minimum_triangle_double_area_m2:minDoubleArea,pressure:{negative_z_vertices:negativePressureVertices,maximum_z_displacement_m:maxPressure,sign_preserved:true},embedded_images_unchanged:true,materials_unchanged:true,source_pipeline:pipeline};
fs.writeFileSync(path.join(root,'tools/art/sail-source/evidence/r52/current-gate.json'),JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify(report,null,2));
