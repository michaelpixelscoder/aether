import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';

const root=process.cwd(),dir=path.join(root,'assets/world');
const names=['dawn','crystal','nomad','ember','frost','verdant','storm','hollow','underforge'];
const digest=bytes=>crypto.createHash('sha256').update(bytes).digest('hex');
const generator=digest(fs.readFileSync('tools/art/build_world.py'));
const collisions=JSON.parse(fs.readFileSync(path.join(dir,'collisions.json')));
const landmarks=JSON.parse(fs.readFileSync(path.join(dir,'landmarks.json')));
const manifest=JSON.parse(fs.readFileSync(path.join(dir,'manifest.json')));
const results=[],images=new Set();
const overlap=(c,p,s)=>p.every((v,i)=>Math.abs(c.center[i]-v)<(c.size[i]+s[i])/2-.001);
for(const name of names){
  assert.ok(collisions[name]?.length>100);
  assert.ok(collisions[name].every(c=>c.center.length===3&&c.size.length===3&&c.center.every(Number.isFinite)&&c.size.every(v=>Number.isFinite(v)&&v>0)));
  assert.ok(collisions[name].some(c=>JSON.stringify(c.center)==='[0,0,80]'&&JSON.stringify(c.size)==='[14,1,18]'),'pier exactly matches shared contract');
  assert.deepEqual(landmarks[name].portal,[0,12,54]);
  assert.ok(!collisions[name].some(c=>overlap(c,[0,12,54],[12,14,12])),'ship volume at portal remains clear');
  if(['hollow','underforge'].includes(name)){
    assert.ok(!collisions[name].some(c=>overlap(c,[0,15,0],[16,12,104])),'cavern ship corridor must be empty');
    assert.ok(collisions[name].some(c=>c.center[1]-c.size[1]/2>40&&c.center[1]+c.size[1]/2>55),'actual collision ceiling');
  }
  for(const lod of [false,true]){
    const file=`${name}${lod?'-lod':''}.glb`,bytes=fs.readFileSync(path.join(dir,file));
    assert.equal(bytes.readUInt32LE(0),0x46546c67);assert.equal(bytes.readUInt32LE(8),bytes.length);
    const jsonLength=bytes.readUInt32LE(12),doc=JSON.parse(bytes.subarray(20,20+jsonLength));
    const binary=bytes.subarray(28+jsonLength);
    assert.equal(doc.asset.extras.aether_world_generator_sha256,generator);
    assert.equal(doc.asset.extras.biome,name);assert.equal(doc.asset.extras.level_of_detail,lod?1:0);
    assert.equal(digest(bytes),manifest[name][lod?'lod':'high'].sha256);
    assert.ok(doc.materials.length<=12);
    for(const material of doc.materials.filter(m=>/^(01 |02 |03 |04 )/.test(m.name))){
      const factor=material.pbrMetallicRoughness.baseColorFactor;
      assert.ok(factor?.length===4&&factor.slice(0,3).every(c=>c>0&&c<.70)&&factor[3]===1,`${file}: authored linear stone colour survived export`);
    }
    for(const image of doc.images??[]){
      assert.ok(/^textures\/[a-f0-9]{64}\.(png|jpg)$/.test(image.uri));assert.equal(image.bufferView,undefined);
      const imageBytes=fs.readFileSync(path.join(dir,image.uri));assert.ok(image.uri.includes(digest(imageBytes)));images.add(image.uri);
    }
    for(const view of doc.bufferViews){assert.equal(view.buffer,0);assert.ok(view.byteOffset>=0&&view.byteOffset+view.byteLength<=binary.length);}
    let triangles=0;const rockTriangles=[];const bounds={min:[Infinity,Infinity,Infinity],max:[-Infinity,-Infinity,-Infinity]};
    for(const mesh of doc.meshes)for(const primitive of mesh.primitives){
      assert.equal(primitive.mode??4,4);const indices=doc.accessors[primitive.indices];assert.equal(indices.count%3,0);triangles+=indices.count/3;
      for(const key of ['POSITION','NORMAL','TEXCOORD_0']){
        assert.ok(key in primitive.attributes,`${file}: ${key}`);
        const accessor=doc.accessors[primitive.attributes[key]],view=doc.bufferViews[accessor.bufferView];
        assert.equal(accessor.componentType,5126);
        const components=key==='TEXCOORD_0'?2:3,stride=view.byteStride??components*4;
        assert.ok((accessor.byteOffset??0)+(accessor.count-1)*stride+components*4<=view.byteLength);
        for(let i=0;i<accessor.count;i++)for(let j=0;j<components;j++)assert.ok(Number.isFinite(binary.readFloatLE(view.byteOffset+(accessor.byteOffset??0)+i*stride+j*4)));
        if(key==='POSITION')for(let j=0;j<3;j++){bounds.min[j]=Math.min(bounds.min[j],accessor.min[j]);bounds.max[j]=Math.max(bounds.max[j],accessor.max[j]);}
      }
      if(lod&&/^(01 |02 |12 )/.test(mesh.name)){
        const pa=doc.accessors[primitive.attributes.POSITION],pv=doc.bufferViews[pa.bufferView];
        const ia=doc.accessors[primitive.indices],iv=doc.bufferViews[ia.bufferView],isize=ia.componentType===5125?4:2;
        const vertex=index=>[0,1,2].map(j=>binary.readFloatLE(pv.byteOffset+(pa.byteOffset??0)+index*(pv.byteStride??12)+j*4));
        for(let i=0;i<ia.count;i+=3){
          const tri=[];
          for(let j=0;j<3;j++){const offset=iv.byteOffset+(ia.byteOffset??0)+(i+j)*isize;tri.push(vertex(isize===4?binary.readUInt32LE(offset):binary.readUInt16LE(offset)));}
          rockTriangles.push(tri);
        }
      }
    }
    assert.equal(triangles,manifest[name][lod?'lod':'high'].triangles);
    assert.ok(triangles<=(lod?12000:200000));
    let closedTerrainColumns=0;
    if(lod){
      for(const c of collisions[name].filter(c=>c.size[0]===8.04&&c.size[2]===8.04)){
        const x=c.center[0]+.17,z=c.center[2]-.13,hits=[];
        for(const [a,b,c] of rockTriangles){
          const den=(b[2]-c[2])*(a[0]-c[0])+(c[0]-b[0])*(a[2]-c[2]);if(Math.abs(den)<1e-8)continue;
          const u=((b[2]-c[2])*(x-c[0])+(c[0]-b[0])*(z-c[2]))/den;
          const v=((c[2]-a[2])*(x-c[0])+(a[0]-c[0])*(z-c[2]))/den;
          if(u>=-1e-5&&v>=-1e-5&&u+v<=1+1e-5)hits.push(u*a[1]+v*b[1]+(1-u-v)*c[1]);
        }
        for(const y of [c.center[1]-c.size[1]/2,c.center[1]+c.size[1]/2])assert.ok(hits.some(v=>Math.abs(v-y)<.03),`${file}: LOD mass cap missing at ${x},${y},${z}`);
        closedTerrainColumns++;
      }
    }
    results.push({file,triangles,materials:doc.materials.length,bytes:bytes.length,bounds,lod_terrain_columns_with_both_caps:closedTerrainColumns});
  }
}
const report={generator_sha256:generator,glbs:results,shared_images:images.size,colliders:Object.fromEntries(names.map(n=>[n,collisions[n].length])),portal_openings_clear:true,cavern_channels_clear:true,external_textures_hash_verified:true};
fs.writeFileSync('docs/evidence/world-art-validation.json',JSON.stringify(report,null,2));
console.log(JSON.stringify(report,null,2));
