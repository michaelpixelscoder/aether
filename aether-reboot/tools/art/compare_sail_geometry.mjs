// Optional comparison of two independently rebuilt Blender exports. Assembly
// itself uses stricter byte-exact checks; this tool diagnoses export ordering
// and floating-point normal variation across fresh Blender processes.
import fs from 'node:fs';
import assert from 'node:assert/strict';
import {readGlb,accessorBytes,sha} from './sail_provenance.mjs';
const [beforeFile,afterFile,outputFile]=process.argv.slice(2);
assert.ok(beforeFile&&afterFile,'usage: node compare_sail_geometry.mjs before.glb after.glb [report.json]');
function resolve(file){
  const {doc,bin,bytes}=readGlb(file);
  const meshes=doc.meshes.filter(m=>m.primitives.some(p=>p.targets)).map(mesh=>{
    assert.equal(mesh.primitives.length,1);const p=mesh.primitives[0];
    const geometry=[p.attributes.POSITION,p.attributes.TEXCOORD_0,...p.targets.map(t=>t.POSITION)].map(i=>accessorBytes(doc,bin,i));
    const normals=[p.attributes.NORMAL,...p.targets.map(t=>t.NORMAL)].map(i=>accessorBytes(doc,bin,i));
    const count=doc.accessors[p.attributes.POSITION].count,groups=new Map(),keys=[],vertices=[];
    for(let n=0;n<count;n++){
      const values=buffers=>buffers.flatMap(b=>{const size=b.length/count;return Array.from({length:size/4},(_,i)=>b.readFloatLE(n*size+i*4));});
      const position=values(geometry),key=position.map(v=>Math.round(v*1e5)).join(',');keys.push(key);
      const cell=position.slice(0,3).map(v=>Math.floor(v*1e4)),bucket=cell.join(',');
      const vertex={position,normal:values(normals),cell};vertices.push(vertex);
      if(!groups.has(bucket))groups.set(bucket,[]);groups.get(bucket).push(vertex);
    }
    const a=doc.accessors[p.indices],indexBytes=accessorBytes(doc,bin,p.indices),width=indexBytes.length/a.count;
    const index=i=>width===4?indexBytes.readUInt32LE(i*4):indexBytes.readUInt16LE(i*2),triangles=[];
    for(let i=0;i<a.count;i+=3)triangles.push([keys[index(i)],keys[index(i+1)],keys[index(i+2)]].sort().join('|'));
    return {name:mesh.name,groups,vertices,triangle_count:triangles.length,topology_sha256:sha(Buffer.from(triangles.sort().join('\n')))};
  });
  return {sha256:sha(bytes),meshes};
}
const before=resolve(beforeFile),after=resolve(afterFile),results=[];
for(const a of before.meshes){
  const b=after.meshes.find(m=>m.name===a.name);assert.ok(b,a.name);assert.equal(a.triangle_count,b.triangle_count);
  let maxPositionDelta=0,maxNormalDelta=0,maxUvDelta=0,uvMismatchVertices=0;
  for(const [from,to] of [[a,b],[b,a]])for(const vertex of from.vertices){
    let match,error=Infinity;
    for(let x=-1;x<=1;x++)for(let y=-1;y<=1;y++)for(let z=-1;z<=1;z++){
      const bucket=[vertex.cell[0]+x,vertex.cell[1]+y,vertex.cell[2]+z].join(',');
      for(const other of to.groups.get(bucket)??[]){
        if(vertex.position.some((v,j)=>j!==3&&j!==4&&Math.abs(v-other.position[j])>.00001))continue;
        const uvError=Math.max(...[3,4].map(j=>Math.abs(vertex.position[j]-other.position[j])));
        const delta=Math.max(...vertex.normal.map((v,j)=>Math.abs(v-other.normal[j])))+uvError*.0001;
        if(delta<error){error=delta;match=other;}
      }
    }
    assert.ok(match,`${a.name}: unmatched position/morph vertex`);
    maxNormalDelta=Math.max(maxNormalDelta,...vertex.normal.map((v,j)=>Math.abs(v-match.normal[j])));
    const uvError=Math.max(...[3,4].map(j=>Math.abs(vertex.position[j]-match.position[j])));maxUvDelta=Math.max(maxUvDelta,uvError);if(uvError>.00001)uvMismatchVertices++;
    maxPositionDelta=Math.max(maxPositionDelta,...vertex.position.filter((_,j)=>j!==3&&j!==4).map((v,j)=>Math.abs(v-match.position[j<3?j:j+2])));
  }
  assert.ok(maxPositionDelta<=.000011);
  results.push({mesh:a.name,triangles:a.triangle_count,triangulation_identical:a.topology_sha256===b.topology_sha256,before_triangle_sha256:a.topology_sha256,after_triangle_sha256:b.topology_sha256,max_position_and_morph_delta:maxPositionDelta,max_uv_delta:maxUvDelta,uv_mismatch_vertices:uvMismatchVertices,max_normal_and_morph_normal_delta:maxNormalDelta});
}
const report={before_sha256:before.sha256,after_sha256:after.sha256,comparison:'fresh exports: positions/morph positions checked at 10 micrometres; UV, normals and triangulation differences explicitly reported, not claimed identical',meshes:results};
if(outputFile)fs.writeFileSync(outputFile,JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify(report,null,2));
