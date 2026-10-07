// CPU-only GLB normals/tangents audit, plus optional before/after contracts.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
const read=p=>JSON.parse(fs.readFileSync(p,'utf8').replace(/^\uFEFF/,''));
const digest=b=>crypto.createHash('sha256').update(b).digest('hex');
const beforeArg=process.argv.indexOf('--before');
const before=beforeArg>=0?process.argv[beforeArg+1]:null;
const previous=before?read(path.join(before,'geometry-contracts.json')):null;
const manifest=read('assets/world/manifest.json'),vault=read('assets/world/vault-manifest.json');
const files=[];
for(const entry of Object.values(manifest))for(const level of ['high','lod'])files.push('assets/world/'+entry[level].file);
for(const level of ['high','lod'])files.push('assets/world/'+vault[level].file);
for(const [file,key] of [['assets/ship/manifest.json','modules'],['assets/fauna/manifest.json','assets'],['assets/fauna/trader-manifest.json','assets']]){
 for(const entry of Object.values(read(file)[key]))files.push(entry.file);
}
const report={files:[],geometry_preserved:[],contracts:{},rock_materials:[]};
for(const file of files){
 const raw=fs.readFileSync(file),length=raw.readUInt32LE(12),doc=JSON.parse(raw.subarray(20,20+length)),bin=raw.subarray(28+length);
 const old=previous?.[path.basename(file)];
 const access=index=>{
  const a=doc.accessors[index],v=doc.bufferViews[a.bufferView],components={SCALAR:1,VEC2:2,VEC3:3,VEC4:4}[a.type];
  return {a,v,components,stride:v.byteStride??components*(a.componentType===5126||a.componentType===5125?4:2)};
 };
 const vector=(data,index)=>Array.from({length:data.components},(_,k)=>bin.readFloatLE((data.v.byteOffset??0)+(data.a.byteOffset??0)+index*data.stride+k*4));
 let normals=0,tangents=0,rockPrimitives=0,metricCliffVertices=0,maximumHeightUvError=0;
 if(old){assert.deepEqual(doc.nodes,old.nodes,file+' node IDs/transforms/gear pivots changed');assert.equal(doc.materials.length,old.materials.length,file+' material count changed');}
 for(const mesh of doc.meshes){
  const triangles=[];
  for(const primitive of mesh.primitives){
   const material=doc.materials[primitive.material];
   const rock=file.startsWith('assets/world/')&&/^(01 |02 )/.test(material.name);
   for(const [attribute,n] of [['NORMAL',3],['TANGENT',4]]){
    if(!(attribute in primitive.attributes)){assert(attribute!=='NORMAL'&&!rock,file+' rock tangent missing');continue;}
    const a=access(primitive.attributes[attribute]);assert.equal(a.a.componentType,5126);assert.equal(a.components,n);
    for(let i=0;i<a.a.count;i++)assert(vector(a,i).every(Number.isFinite),`${file}: non-finite ${attribute}`);
    if(attribute==='NORMAL')normals+=a.a.count;else tangents+=a.a.count;
   }
   if(rock){
    assert(material.normalTexture,file+' rock normal texture missing');
    assert(Math.abs(material.normalTexture.scale-.65)<.00001,file+' normal scale');
    assert(material.pbrMetallicRoughness.metallicRoughnessTexture,file+' roughness texture missing');
    assert.equal(material.pbrMetallicRoughness.metallicFactor,0);rockPrimitives++;
    const position=access(primitive.attributes.POSITION),normal=access(primitive.attributes.NORMAL),uv=access(primitive.attributes.TEXCOORD_0);
    for(let i=0;i<position.a.count;i++){
     const n=vector(normal,i);
     if(Math.abs(n[1])<.4&&Math.max(Math.abs(n[0]),Math.abs(n[2]))>.9){
      const p=vector(position,i),t=vector(uv,i);
      maximumHeightUvError=Math.max(maximumHeightUvError,Math.abs(t[1]-(1-p[1]/16)));metricCliffVertices++;
     }
    }
   }
   if(old){
    const pp=access(primitive.attributes.POSITION),ii=access(primitive.indices);
    for(let j=0;j<ii.a.count;j+=3){
     const tri=[];
     for(let k=0;k<3;k++){
      const offset=(ii.v.byteOffset??0)+(ii.a.byteOffset??0)+(j+k)*ii.stride;
      const index=ii.a.componentType===5125?bin.readUInt32LE(offset):bin.readUInt16LE(offset);
      tri.push(vector(pp,index));
     }
     triangles.push(tri);
    }
   }
  }
  if(old){
   const expected=old.meshes.find(m=>m.name===mesh.name);assert(expected,file+' mesh changed: '+mesh.name);
   assert.equal(triangles.length,expected.triangles,file+' triangle count changed');
   assert.equal(digest(JSON.stringify(triangles)),expected.positions_sha256,file+' triangle positions changed: '+mesh.name);
  }
 }
 if(old)report.geometry_preserved.push({file,exact_triangle_positions_and_counts:true,node_contracts_unchanged:true,material_count_unchanged:true});
 if(rockPrimitives)assert(maximumHeightUvError<.001,file+' cliff bedding UV height discontinuity');
 report.files.push({file,finite_normal_vectors:normals,finite_tangent_vectors:tangents,rock_primitives:rockPrimitives,metric_cliff_vertices:metricCliffVertices,maximum_height_uv_error:maximumHeightUvError});
 for(const m of doc.materials.filter(m=>file.startsWith('assets/world/')&&/^(01 |02 )/.test(m.name))){
  report.rock_materials.push({file,name:m.name,base_color_factor:m.pbrMetallicRoughness.baseColorFactor,roughness_factor:m.pbrMetallicRoughness.roughnessFactor??1,normal_scale:m.normalTexture.scale});
 }
}
if(before){
 for(const name of ['collisions.json','landmarks.json','vault-collisions.json']){
  const oldBytes=fs.readFileSync(path.join(before,name)),newBytes=fs.readFileSync('assets/world/'+name);
  assert(oldBytes.equals(newBytes),name+' is not byte-identical');
  report.contracts[name]={byte_identical:true,sha256:digest(newBytes)};
 }
}
report.checked_at=new Date().toISOString();
// Preserve the historical before/after proof when running ordinary QA again.
const output=before?'.dream-loop/rock-material-before-after.json':'docs/evidence/rock-materials.json';
fs.mkdirSync(path.dirname(output),{recursive:true});
fs.writeFileSync(output,JSON.stringify(report,null,2));
console.log(`${files.length} GLB: all normals/tangents finite; ${report.geometry_preserved.length} exact geometry contracts preserved; ${report.rock_materials.length} rock materials with registered PBR maps.`);
