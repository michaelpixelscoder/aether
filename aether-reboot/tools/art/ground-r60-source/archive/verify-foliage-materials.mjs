// Portable CPU audit. Frozen evidence is data, never a dependency on old GLB files.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import {fileURLToPath} from 'node:url';
export const read = p => JSON.parse(fs.readFileSync(p, 'utf8').replace(/^\uFEFF/, ''));
export const hash = b => crypto.createHash('sha256').update(b).digest('hex');
export const foliage = n => /^(08 |09 )/.test(n);
export const foliageFamily = n => /^(06 |08 |09 )/.test(n);
const attributes = ['POSITION', 'NORMAL', 'TEXCOORD_0', 'TANGENT'];
const clean = o => JSON.parse(JSON.stringify(o));
export function normalizeTangentRounding(ordered, evidence, material) {
  assert(foliage(material.name) && !material.normalTexture && !material.extensions?.KHR_materials_anisotropy, 'Rounding exception requires foliage without a tangent-dependent normal/anisotropy map');
  assert.equal(ordered.length,evidence.corners);
  const currentSha=hash(JSON.stringify(ordered));
  if(currentSha===evidence.historical_sha256)return {changed_components:0,raw_sha256:currentSha,historical_sha256:currentSha};
  const restored=ordered.map(v=>v.slice()), seen=new Set(), changedCorners=new Set();
  let changed=0,maxAbs=0,maxAngle=0;
  for(const p of evidence.points) {
    assert(Number.isInteger(p.corner)&&p.corner>=0&&p.corner<ordered.length);
    assert(Number.isInteger(p.component)&&p.component>=0&&p.component<3,'Handedness cannot be waived');
    const key=p.corner+':'+p.component;assert(!seen.has(key));seen.add(key);
    assert.equal(Math.fround(Math.round(p.historical*10000)/10000),p.historical);
    assert.equal(Math.fround(Math.round(p.observed*10000)/10000),p.observed);
    assert.equal(Math.abs(Math.round(p.historical*10000)-Math.round(p.observed*10000)),1,'Only adjacent measured rounding bins');
    const actual=ordered[p.corner][p.component];
    assert(actual===p.historical||actual===p.observed,'Unmeasured tangent value');
    const delta=Math.abs(actual-p.historical);assert(delta<=0.00010002);
    if(delta){changed++;changedCorners.add(p.corner);maxAbs=Math.max(maxAbs,delta);}
    restored[p.corner][p.component]=p.historical;
  }
  // Every unlisted coordinate, including W, must still match the old digest.
  assert.equal(hash(JSON.stringify(restored)),evidence.historical_sha256,'Unlisted tangent change');
  for(const i of changedCorners) {
    const a=restored[i],b=ordered[i],cross=[a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]];
    const angle=Math.atan2(Math.hypot(...cross),a[0]*b[0]+a[1]*b[1]+a[2]*b[2])*180/Math.PI;
    assert(angle<=0.006,'Tangent angle exceeds measured rounding bound');maxAngle=Math.max(maxAngle,angle);
  }
  return {changed_components:changed,raw_sha256:currentSha,historical_sha256:evidence.historical_sha256,max_abs:maxAbs,max_angle_degrees:maxAngle};
}
export function inspect(file, roundingCases=[], originalFamilyCorners=null) {
  const raw = fs.readFileSync(file), jsonLength = raw.readUInt32LE(12);
  assert.equal(raw.readUInt32LE(0), 0x46546c67);
  assert.equal(raw.readUInt32LE(8), raw.length);
  const doc = JSON.parse(raw.subarray(20, 20 + jsonLength)), bin = raw.subarray(28 + jsonLength);
  const data = index => {
    const a = doc.accessors[index], v = doc.bufferViews[a.bufferView];
    const components = {SCALAR:1,VEC2:2,VEC3:3,VEC4:4}[a.type];
    const bytes = {5121:1,5123:2,5125:4,5126:4}[a.componentType];
    assert(components && bytes && !a.sparse);
    return Array.from({length:a.count}, (_, i) => Array.from({length:components}, (_, k) => {
      const p = (v.byteOffset??0) + (a.byteOffset??0) + i*(v.byteStride??components*bytes) + k*bytes;
      const value = a.componentType===5126 ? bin.readFloatLE(p) : bytes===4 ? bin.readUInt32LE(p) : bytes===2 ? bin.readUInt16LE(p) : bin.readUInt8(p);
      assert(Number.isFinite(value), `${file}: nonfinite accessor ${index}`);
      return value;
    }));
  };
  const texture = (i, leaf) => {
    const t=doc.textures[i], im=doc.images[t.source]; assert(im.uri);
    assert(!path.isAbsolute(im.uri) && !im.uri.split(/[\\/]/).includes('..'));
    return clean({texture:{...t,source:undefined,sampler:undefined},sampler:doc.samplers?.[t.sampler],image:{...im,uri:undefined,...(leaf?{name:undefined}:{})},sha256:hash(fs.readFileSync(path.join(path.dirname(file),im.uri)))});
  };
  const normalize = (x, leaf, key='') => Array.isArray(x) ? x.map(y=>normalize(y,leaf)) : x&&typeof x==='object' ? Object.fromEntries(Object.entries(x).map(([k,v])=>[k,k==='index'&&key.endsWith('Texture')?texture(v,leaf):normalize(v,leaf,k)])) : x;
  const materials=doc.materials.map(m=>clean(normalize(m,foliage(m.name))));
  const meshes=[], tangent_rounding=[]; let triangles=0, normals=0, tangents=0;
  for(const mesh of doc.meshes) for(const p of mesh.primitives) {
    assert.equal(p.mode??4,4);
    const indices=data(p.indices).flat(); assert.equal(indices.length%3,0);
    const originalCorners=originalFamilyCorners?.get(mesh.name);
    if(originalCorners!==undefined)assert(foliageFamily(doc.materials[p.material].name)&&indices.length>=originalCorners);
    const retainedIndices=originalCorners===undefined?indices:indices.slice(0,originalCorners);
    const signature={mesh:mesh.name,material:doc.materials[p.material].name,triangles:retainedIndices.length/3,attributes:{}};
    triangles+=indices.length/3;
    for(const key of attributes) {
      assert(key in p.attributes, `${file}: missing ${key}`);
      const values=data(p.attributes[key]);
      assert(indices.every(i=>i<values.length), `${file}: invalid index`);
      // Indexed corner order ignores exporter vertex deduplication, but preserves
      // every authored triangle and attribute, including all woody branches.
      const ordered=retainedIndices.map(i=>values[i]);
      signature.attributes[key]=hash(JSON.stringify(ordered));
      const evidence=key==='TANGENT'?roundingCases.find(c=>c.mesh===mesh.name&&c.material===signature.material):undefined;
      if(evidence&&signature.attributes[key]!==evidence.historical_sha256) {
        const audit=normalizeTangentRounding(ordered,evidence,materials[p.material]);
        tangent_rounding.push({mesh:mesh.name,material:signature.material,...audit});
        signature.attributes[key]=audit.historical_sha256;
      }
      if(key==='NORMAL')normals+=values.length;
      if(key==='TANGENT')tangents+=values.length;
    }
    meshes.push(signature);
  }
  return {sha256:hash(raw),bytes:raw.length,triangles,materials,meshes,nodes:clean(doc.nodes),source:doc.asset.extras?.aether_world_generator_sha256,normals,tangents,tangent_rounding};
}
export function familyNodes(snapshot) {
  // The frozen migration snapshot lists one record per primitive. Resolve the
  // real mesh table by unique mesh names before comparing node transforms;
  // unrelated empty batches may legitimately disappear after other art work.
  const names=[...new Set(snapshot.meshes.map(m=>m.mesh))];
  const family=new Set(snapshot.meshes.filter(m=>foliageFamily(m.material)).map(m=>m.mesh));
  const retained=new Set(snapshot.nodes.flatMap((n,i)=>n.mesh!==undefined&&family.has(names[n.mesh])?[i]:[]));
  let changed=true;
  while(changed){changed=false;for(let i=0;i<snapshot.nodes.length;i++)if(!retained.has(i)&&(snapshot.nodes[i].children??[]).some(c=>retained.has(c))){retained.add(i);changed=true;}}
  return [...retained].map(i=>{
    const n=snapshot.nodes[i];
    return clean({...n,...(n.mesh===undefined?{}:{mesh:names[n.mesh]}),...(n.children?{children:n.children.filter(c=>retained.has(c)).map(c=>snapshot.nodes[c].name??`node ${c}`)}:{})});
  }).sort((a,b)=>(a.name??'').localeCompare(b.name??''));
}
export function combineRoundingEvidence(historical,extensions) {
  const cases=clean(historical.cases);
  assert.equal(extensions.historical_evidence_sha256,hash(JSON.stringify(historical)));
  for(const extra of extensions.cases){
    const existing=cases.find(c=>c.file===extra.file&&c.mesh===extra.mesh&&c.material===extra.material);assert(existing);
    assert.equal(extra.historical_sha256,existing.historical_sha256);
    for(const p of extra.points){
      assert(!existing.points.some(old=>old.corner===p.corner&&old.component===p.component),'Extension cannot replace a previously audited value');
      existing.points.push(p);
    }
  }
  return cases;
}
export function verify(options={}) {
  const root=options.root??path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../..');
  const world=options.world??path.join(root,'assets/world');
  const sources=options.sources??path.join(root,'tools/art');
  const textures=options.textures??path.join(root,'assets/textures');
  const baseline=read(options.baseline??path.join(root,'tools/art/foliage-r44-baseline.json'));
  const provenance=read(options.provenance??path.join(root,'tools/art/foliage-r44-provenance.json'));
  const rounding=read(options.rounding??path.join(root,'tools/art/foliage-r44-tangent-rounding.json'));
  const archive=read(path.join(sources,'foliage-r44-promotion-archive.json'));
  for(const file of archive.files)assert.equal(hash(fs.readFileSync(path.join(sources,file.file))),file.sha256,`${file.file}: historical promotion evidence altered`);
  assert.equal(rounding.schema,1);assert.equal(rounding.rounding_digits,4);
  assert.equal(rounding.baseline_sha256,hash(fs.readFileSync(options.baseline??path.join(root,'tools/art/foliage-r44-baseline.json'))));
  assert.equal(rounding.cases.length,4);assert.equal(rounding.cases.reduce((n,c)=>n+c.points.length,0),20);
  const extensions=read(path.join(sources,'foliage-r45-tangent-rounding.json'));
  for(const proof of extensions.source_proofs)assert.equal(hash(fs.readFileSync(path.join(sources,proof.file))),proof.sha256,`${proof.file}: extended rounding proof altered`);
  const combinedRounding=combineRoundingEvidence(rounding,extensions);
  for(const c of rounding.cases) {
    const old=baseline.models[c.file]?.candidate.meshes.find(m=>m.mesh===c.mesh&&m.material===c.material);
    assert(old);assert.equal(c.historical_sha256,old.attributes.TANGENT);assert.equal(c.corners,old.triangles*3);
  }
  const manifest=read(path.join(world,'manifest.json')), vault=read(path.join(world,'vault-manifest.json'));
  assert.equal(Object.keys(manifest).length,12);
  const sourceHashes=Object.fromEntries(['build_world.py','build_world_variants.py','build_vault.py'].map(f=>[f,hash(fs.readFileSync(path.join(sources,f)))]));
  const report={checked_at:new Date().toISOString(),contracts:{},models:[],source_hashes:sourceHashes,baseline_sha256:hash(fs.readFileSync(options.baseline??path.join(root,'tools/art/foliage-r44-baseline.json')))};
  for(const [file,expected] of Object.entries(baseline.contracts)) {
    assert.equal(hash(fs.readFileSync(path.join(world,file))),expected,`${file}: physical contract changed`);
    report.contracts[file]={byte_identical:true,sha256:expected};
  }
  for(const asset of provenance.assets) assert.equal(hash(fs.readFileSync(path.join(textures,path.basename(asset.file)))),asset.sha256,`${asset.file}: native image changed`);
  for(const [name,entry] of [...Object.entries(manifest),['arch-vault',vault]]) {
    const source=name==='arch-vault'?'build_vault.py':name.startsWith('dawn-')?'build_world_variants.py':'build_world.py';
    assert.equal(entry.generator_sha256,sourceHashes[source],`${name}: manifest source hash`);
    if(source!=='build_world.py')assert.equal(entry.base_generator_sha256,sourceHashes['build_world.py']);
    for(const level of ['high','lod']) {
      const file=entry[level].file;
      let originalFamilyCorners=null;
      if(level==='high'&&entry.terraces_r56){
        assert(['dawn','dawn-watch'].includes(name),'R56 append scope');
        const retainedFile=path.join(sources,'terraces-source/baseline/assets/world',file);
        assert.equal(hash(fs.readFileSync(retainedFile)),entry.terraces_r56.baseline_glb_sha256,'exact R54 input');
        const retained=inspect(retainedFile,combinedRounding.filter(c=>c.file===file));
        originalFamilyCorners=new Map(retained.meshes.filter(m=>foliageFamily(m.material)).map(m=>[m.mesh,m.triangles*3]));
        assert.equal(originalFamilyCorners.size,3);
      }
      // R56 independently verifies every original accessor byte/index and the
      // appended geometry. R44 still audits the exact historical corner prefix,
      // including its explicit tangent rounding evidence; never new corners.
      const actual=inspect(path.join(world,file),combinedRounding.filter(c=>c.file===file),originalFamilyCorners), before=baseline.models[file]; assert(before);
      assert.equal(actual.sha256,entry[level].sha256); assert.equal(actual.bytes,entry[level].bytes);
      assert.equal(actual.source,sourceHashes[source],`${file}: GLB source hash`);
      assert.equal(actual.triangles,entry[level].triangles);
      assert(actual.triangles<=(level==='lod'?12000:200000));
      assert(actual.materials.length<=(name==='arch-vault'?5:12));
      assert.deepEqual(actual.meshes.filter(m=>foliageFamily(m.material)),before.candidate.meshes.filter(m=>foliageFamily(m.material)),`${file}: woody/foliar geometry/normal/UV/tangent changed`);
      assert.deepEqual(familyNodes(actual),familyNodes(before.candidate),`${file}: woody/foliar node, parent or pivot changed`);
      assert.deepEqual(actual.materials.filter(m=>foliageFamily(m.name)),before.candidate.materials.filter(m=>foliageFamily(m.name)),`${file}: woody/foliar material/factor/texture changed`);
      const leaf=actual.materials.filter(m=>foliage(m.name)); assert(leaf.length<=2);
      for(const m of leaf) {
        assert(m.pbrMetallicRoughness.baseColorTexture); assert.equal(m.doubleSided,true);
        assert.equal(m.alphaMode,m.name.startsWith('08 |')?'MASK':'OPAQUE');
        if(m.alphaMode==='MASK')assert.equal(m.alphaCutoff,.33);
      }
      report.models.push({file,triangles:actual.triangles,finite_normals:actual.normals,finite_tangents:actual.tangents,scope:'06 Cedar/roots + 08/09 foliage',family_geometry_exact:true,later_append_only:originalFamilyCorners!==null,family_material_factors_exact:true,family_nodes_parents_pivots_exact:true,tangents_exact:actual.tangent_rounding.length===0,tangent_rounding:actual.tangent_rounding});
    }
  }
  assert.equal(report.models.length,26);
  const destination=options.report??path.join(root,'docs/evidence/foliage-r44-validation.json');
  fs.mkdirSync(path.dirname(destination),{recursive:true});
  report.historical_promotion_archive=archive;
  fs.writeFileSync(destination,JSON.stringify(report,null,2)+'\n');
  console.log('26 world GLB: frozen woody/foliar family 06/08/09, explicit audited tangent coordinates, family material factors and transforms, native textures, physical contract bytes, finite attributes, budgets and honest source hashes verified. Historical full-surface promotion audit retained unchanged.');
  return report;
}
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url)) {
  const options={}; const keys={'--root':'root','--world':'world','--source-dir':'sources','--texture-dir':'textures','--baseline':'baseline','--provenance':'provenance','--report':'report','--rounding-evidence':'rounding'};
  for(let i=2;i<process.argv.length;i+=2){assert(keys[process.argv[i]]&&process.argv[i+1],`Unknown or incomplete argument ${process.argv[i]}`);options[keys[process.argv[i]]]=path.resolve(process.argv[i+1]);}
  verify(options);
}
