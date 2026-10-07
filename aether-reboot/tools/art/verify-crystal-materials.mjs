// Portable ongoing mineral-family audit. The migration proof separately checks
// every retained non-mineral surface; this does not freeze future rock/leaf art.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import {fileURLToPath} from 'node:url';
const arg=process.argv.indexOf('--root');
const root=arg<0?path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../..'):path.resolve(process.argv[arg+1]);
const read=p=>JSON.parse(fs.readFileSync(p,'utf8').replace(/^\uFEFF/,''));
const sha=b=>crypto.createHash('sha256').update(b).digest('hex');
const hash=p=>sha(fs.readFileSync(p));
const baseline=read(path.join(root,'tools/art/crystal-r45-baseline.json'));
const provenance=read(path.join(root,'tools/art/crystal-r45-provenance.json'));
const world=path.join(root,'assets/world'),manifest=read(path.join(world,'manifest.json'));
const sources=Object.fromEntries(['build_world.py','build_world_variants.py','build_vault.py'].map(f=>[f,hash(path.join(root,'tools/art',f))]));
for(const image of provenance.images){
  const bytes=fs.readFileSync(path.join(root,image.file));assert.equal(sha(bytes),image.sha256);
  assert.deepEqual([bytes.readUInt32BE(16),bytes.readUInt32BE(20)],image.native_dimensions);
}
let minerals=0,triangles=0;
for(const [name,entry] of Object.entries(manifest))for(const level of ['high','lod']){
  const file=entry[level].file,raw=fs.readFileSync(path.join(world,file)),length=raw.readUInt32LE(12);
  const doc=JSON.parse(raw.subarray(20,20+length)),bin=raw.subarray(28+length),expected=baseline.models[file];assert(expected);
  assert.equal(raw.readUInt32LE(0),0x46546c67);assert.equal(raw.readUInt32LE(8),raw.length);
  const generator=name.startsWith('dawn-')?'build_world_variants.py':'build_world.py';
  assert.equal(entry.generator_sha256,sources[generator]);assert.equal(doc.asset.extras.aether_world_generator_sha256,sources[generator]);
  if(generator!=='build_world.py')assert.equal(entry.base_generator_sha256,sources['build_world.py']);
  assert.equal(sha(raw),entry[level].sha256);assert(entry[level].triangles<=(level==='lod'?12000:200000));
  const data=i=>{
    const a=doc.accessors[i],v=doc.bufferViews[a.bufferView],n={SCALAR:1,VEC2:2,VEC3:3,VEC4:4}[a.type],bytes=a.componentType===5123?2:4;
    assert(!a.sparse);const out=[];
    for(let row=0;row<a.count;row++)for(let k=0;k<n;k++){
      const p=(v.byteOffset??0)+(a.byteOffset??0)+row*(v.byteStride??n*bytes)+k*bytes;
      const x=a.componentType===5126?bin.readFloatLE(p):bytes===2?bin.readUInt16LE(p):bin.readUInt32LE(p);assert(Number.isFinite(x));out.push(x);
    }
    return {values:out,n};
  };
  const texture=i=>{
    const t=doc.textures[i],im=doc.images[t.source];assert(/^textures\/[a-f0-9]{64}\.png$/.test(im.uri));
    const digest=hash(path.join(world,im.uri));assert(im.uri.includes(digest));
    return {texture:{...t,source:undefined,sampler:undefined},sampler:doc.samplers?.[t.sampler],image:{...im,uri:undefined},sha256:digest};
  };
  const normalize=(x,key='')=>Array.isArray(x)?x.map(y=>normalize(y)):x&&typeof x==='object'?Object.fromEntries(Object.entries(x).map(([k,v])=>[k,k==='index'&&key.endsWith('Texture')?texture(v):normalize(v,k)])):x;
  let count=0;
  for(const mesh of doc.meshes)for(const p of mesh.primitives){
    const material=doc.materials[p.material];if(!material.name.startsWith('11 | Aether mineral'))continue;
    const indices=data(p.indices).values;const signatures={};
    for(const key of ['POSITION','NORMAL','TEXCOORD_0','TANGENT']){
      const v=data(p.attributes[key]),ordered=Buffer.alloc(indices.length*v.n*4);
      for(let i=0;i<indices.length;i++)for(let k=0;k<v.n;k++)ordered.writeFloatLE(v.values[indices[i]*v.n+k],(i*v.n+k)*4);
      signatures[key]=sha(ordered);
    }
    assert(expected.mineral);assert.equal(indices.length/3,expected.mineral.triangles);
    assert.deepEqual(signatures,expected.mineral.attributes,file+': authored mineral geometry/UV/normal/tangent');
    assert.deepEqual(JSON.parse(JSON.stringify(normalize(material))),expected.mineral.material,file+': mineral PBR factor/map');
    const m=material,pbr=m.pbrMetallicRoughness;
    assert(pbr.baseColorTexture&&pbr.metallicRoughnessTexture&&m.normalTexture&&m.emissiveTexture);
    assert(Math.abs(m.normalTexture.scale-.1)<1e-6);assert.equal(m.alphaMode??'OPAQUE','OPAQUE');
    assert.equal(m.extensions.KHR_materials_emissive_strength.emissiveStrength,2.5);
    assert.deepEqual(m.emissiveFactor,pbr.baseColorFactor.slice(0,3));assert(!m.extensions.KHR_materials_transmission);
    count++;triangles+=indices.length/3;
  }
  assert.equal(count,expected.mineral?1:0);minerals+=count;
}
console.log(`${minerals} mineral batches: exact authored geometry and four PBR bindings, native texture hashes, honest generator hashes and budgets verified (${triangles} mineral triangles across HD/LOD).`);
