// CPU-only snapshot/verification of the material-only R42 architectural pass.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
const snapshot=process.argv.includes('--snapshot');
const beforeArg=process.argv.indexOf('--before');
const baseline=beforeArg>=0?process.argv[beforeArg+1]:snapshot?'.dream-loop/masonry-material-before':null;
if(beforeArg>=0)assert(baseline,'--before requires a baseline directory');
const read=p=>JSON.parse(fs.readFileSync(p,'utf8').replace(/^\uFEFF/,''));
const hash=b=>crypto.createHash('sha256').update(b).digest('hex');
const isMasonry=name=>/^03 \| Dressed masonry(?:\.\d+)?$/.test(name);
const jsonData=value=>JSON.parse(JSON.stringify(value));
const geometry={},surfaces={},report={files:[],contracts:{},uv_roundoff:[]};
const surfaceDifferences=[];
const expected=snapshot||!baseline?null:read(baseline+'/surfaces.json');
// Blender appends numeric suffixes when multiple palettes are built in one process.
if(expected)for(const entry of Object.values(expected)){
 entry.materials=entry.materials.filter(m=>!isMasonry(m.name));
 entry.primitives=entry.primitives.filter(p=>!isMasonry(p.material));
}
const oldGeometry=snapshot||!baseline?null:read(baseline+'/geometry-contracts.json');
const published=snapshot?null:read('tools/art/masonry-r42-verification.json');
const gains=[1.2408519164957788,1.2179610548857251,1.169802004734367];
const files=fs.readdirSync('assets/world').filter(f=>f.endsWith('.glb')).sort();
assert.equal(files.length,26);
for(const file of files){
 const filename='assets/world/'+file,raw=fs.readFileSync(filename),size=raw.readUInt32LE(12),doc=JSON.parse(raw.subarray(20,20+size)),bin=raw.subarray(28+size);
 const data=index=>{
  const a=doc.accessors[index],v=doc.bufferViews[a.bufferView],n={SCALAR:1,VEC2:2,VEC3:3,VEC4:4}[a.type],bytes=a.componentType===5123?2:4;
  return Array.from({length:a.count},(_,i)=>Array.from({length:n},(_,j)=>{
   const offset=(v.byteOffset??0)+(a.byteOffset??0)+i*(v.byteStride??n*bytes)+j*bytes;
   return a.componentType===5126?bin.readFloatLE(offset):bytes===2?bin.readUInt16LE(offset):bin.readUInt32LE(offset);
  }));
 };
 const texture=index=>{
  const t=doc.textures[index],im=doc.images[t.source];
  assert(im.uri,'external image required');
  return {texture:{...t,source:undefined,sampler:undefined},sampler:doc.samplers?.[t.sampler],image:{...im,uri:undefined},sha256:hash(fs.readFileSync(path.join(path.dirname(filename),im.uri)))};
 };
 const normalize=(object,key='')=>{
  if(Array.isArray(object))return object.map(x=>normalize(x));
  if(object&&typeof object==='object')return Object.fromEntries(Object.entries(object).map(([k,v])=>[k,k==='index'&&key.endsWith('Texture')?texture(v):normalize(v,k)]));
  return object;
 };
 geometry[file]={nodes:doc.nodes,materials:doc.materials.map(m=>({name:m.name,factor:m.pbrMetallicRoughness?.baseColorFactor})),meshes:[]};
 surfaces[file]={materials:doc.materials.filter(m=>!isMasonry(m.name)).map(m=>normalize(m)),primitives:[]};
 let masonry=0,vertices=0,maxVError=0,emberCorniceUv;
 for(const mesh of doc.meshes){
  const triangles=[];
  for(const p of mesh.primitives){
   const m=doc.materials[p.material],indices=data(p.indices).flat(),position=data(p.attributes.POSITION);
   for(let i=0;i<indices.length;i+=3)triangles.push(indices.slice(i,i+3).map(j=>position[j]));
   if(isMasonry(m.name)){
    if(!snapshot){
     assert(m.normalTexture,file+' STONE normal missing');assert(Math.abs(m.normalTexture.scale-.4)<1e-6,file+' normal scale');
     assert(m.pbrMetallicRoughness.metallicRoughnessTexture,file+' roughness map missing');
     assert.equal(m.pbrMetallicRoughness.metallicFactor,0);
     assert('TANGENT' in p.attributes,file+' STONE tangent missing');
     const factor=oldGeometry?oldGeometry[file].materials.find(x=>x.name===m.name).factor.map((x,j)=>x*(gains[j]??1)):published.stone_material_factors[file];
     for(let j=0;j<3;j++)assert(Math.abs(m.pbrMetallicRoughness.baseColorFactor[j]-factor[j])<1e-12,file+' mean albedo compensation');
     for(const [binding,suffix] of [[m.normalTexture,'normal'],[m.pbrMetallicRoughness.baseColorTexture,'albedo'],[m.pbrMetallicRoughness.metallicRoughnessTexture,'roughness']]){
      assert.equal(doc.images[doc.textures[binding.index].source].name,'dressed-masonry-r42-'+suffix);
     }
     const n=data(p.attributes.NORMAL),uv=data(p.attributes.TEXCOORD_0);
     for(let i=0;i<position.length;i++){
      // Beveled bands interpolate UVs; only the axial un-beveled masonry planes are exact metric samples.
      if(Math.abs(n[i][1])<1e-6&&Math.max(Math.abs(n[i][0]),Math.abs(n[i][2]))>.99999){
       maxVError=Math.max(maxVError,Math.abs(uv[i][1]-(1-position[i][1]/9)));vertices++;
      }
     }
    }
    masonry++;
   }else{
    const attributes=Object.fromEntries(Object.entries(p.attributes).map(([key,index])=>{
     const values=data(index),ordered=indices.map(i=>values[i]);
     if(file==='ember.glb'&&mesh.name==='04 | Worn cornice'&&key==='TEXCOORD_0')emberCorniceUv=ordered;
     return [key,hash(JSON.stringify(ordered))];
    }));
    surfaces[file].primitives.push({mesh:mesh.name,material:m.name,attributes});
   }
  }
  geometry[file].meshes.push({name:mesh.name,triangles:triangles.length,positions_sha256:hash(JSON.stringify(triangles))});
 }
 if(!snapshot){
  const actual=jsonData(surfaces[file]);
  if(expected&&JSON.stringify(actual)!==JSON.stringify(expected[file])){
   if(JSON.stringify(actual.materials)!==JSON.stringify(expected[file].materials))surfaceDifferences.push({file,materials_changed:true});
   for(let i=0;i<actual.primitives.length;i++)for(const [attribute,digest] of Object.entries(actual.primitives[i].attributes)){
    const oldDigest=expected[file].primitives[i].attributes[attribute];
    if(digest!==oldDigest){
     if(file==='ember.glb'&&actual.primitives[i].mesh==='04 | Worn cornice'&&attribute==='TEXCOORD_0'){
      // A saved, hash-matched R41 bundle recovers the baseline's full UV values.
      // Blender bevel interpolation differs by one float32 ULP in 11 corners.
      const oldUv=read(baseline+'/ember-cornice-uv.json');assert.equal(hash(JSON.stringify(oldUv)),oldDigest);
      const changed=[];assert.equal(oldUv.length,emberCorniceUv.length);
      for(let j=0;j<oldUv.length;j++)for(let k=0;k<2;k++){
       const delta=Math.abs(oldUv[j][k]-emberCorniceUv[j][k]);
       if(delta){assert.equal(delta,2**-23,'TRIM UV difference exceeds documented one-ULP roundoff');changed.push({corner:j,component:k,delta});}
      }
      assert.equal(changed.length,11);report.uv_roundoff.push({file,mesh:actual.primitives[i].mesh,maximum_texel_error:2**-23*1254,changed});
     }else surfaceDifferences.push({file,mesh:actual.primitives[i].mesh,attribute,actual:digest,expected:oldDigest});
    }
   }
  }
  assert(maxVError<.00001,file+' masonry course UV discontinuity');
 }
 report.files.push({file,masonry_primitives:masonry,metric_facade_vertices:vertices,maximum_height_uv_error:maxVError,...(expected?{non_masonry_materials_normals_tangents_exact:true,non_masonry_uv_exact:!report.uv_roundoff.some(r=>r.file===file)}:{})});
}
if(snapshot){
 fs.mkdirSync(baseline,{recursive:true});
 for(const f of ['collisions.json','landmarks.json','vault-collisions.json','manifest.json','vault-manifest.json'])fs.copyFileSync('assets/world/'+f,baseline+'/'+f);
 fs.copyFileSync('tools/art/build_world.py',baseline+'/build_world.py');
 fs.writeFileSync(baseline+'/geometry-contracts.json',JSON.stringify(geometry,null,2));
 fs.writeFileSync(baseline+'/surfaces.json',JSON.stringify(surfaces,null,2));
 console.log('Saved baseline for 26 world GLB geometry, nodes, all non-STONE materials and vertex attributes.');
}else{
 if(surfaceDifferences.length){fs.writeFileSync('.dream-loop/masonry-surface-differences.json',JSON.stringify(surfaceDifferences,null,2));console.error(surfaceDifferences);}
 assert.equal(surfaceDifferences.length,0,'Non-STONE materials or vertex attributes changed; see .dream-loop/masonry-surface-differences.json');
 const old=oldGeometry;
 for(const f of old?files:[]){
  assert.deepEqual(geometry[f].nodes,old[f].nodes,f+' node or gear pivot changed');
  assert.deepEqual(geometry[f].meshes,old[f].meshes,f+' triangle geometry changed');
  assert.equal(geometry[f].materials.length,old[f].materials.length,f+' material count changed');
 }
 for(const name of old?['collisions.json','landmarks.json','vault-collisions.json']:[]){
  const bytes=fs.readFileSync('assets/world/'+name);assert(bytes.equals(fs.readFileSync(baseline+'/'+name)),name+' changed');
  report.contracts[name]={byte_identical:true,sha256:hash(bytes)};
 }
 if(old)report.geometry_exact=files.length;
 report.checked_at=new Date().toISOString();
 assert.equal(report.files.filter(f=>f.masonry_primitives>0).length,24);
 report.stone_material_factors=Object.fromEntries(files.filter(f=>geometry[f].materials.some(m=>isMasonry(m.name))).map(f=>[f,geometry[f].materials.find(m=>isMasonry(m.name)).factor]));
 fs.writeFileSync(old?'.dream-loop/masonry-material-verification.json':'docs/evidence/masonry-materials.json',JSON.stringify(report,null,2));
 console.log(old?'26 world GLB geometry and nodes exact; every non-STONE material/texture/normal/tangent exact; UV exact except 11 one-ULP bevel roundoffs (0.00015 texel) in Ember cornice; masonry PBR and 9m facade UV checked; collision/landmark JSON byte-identical.':'26 world GLB checked; registered masonry PBR bindings, calibrated factors and continuous 9m facade UV verified.');
}
