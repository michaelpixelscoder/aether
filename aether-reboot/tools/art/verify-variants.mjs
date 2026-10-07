import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';

const stage=process.argv.includes('--preview');
const dir=stage?'tools/art/world-preview/variants':'assets/world';
const digest=b=>crypto.createHash('sha256').update(b).digest('hex');
const generator=digest(fs.readFileSync('tools/art/build_world_variants.py'));
const base=digest(fs.readFileSync('tools/art/build_world.py'));
const collisions=JSON.parse(fs.readFileSync(path.join(dir,'collisions.json')));
const landmarks=JSON.parse(fs.readFileSync(path.join(dir,'landmarks.json')));
const manifest=JSON.parse(fs.readFileSync(path.join(dir,'manifest.json')));
const routes=JSON.parse(fs.readFileSync('tools/art/world-source/satellite-walk-routes.json'));
routes['dawn-ruin']=[
  {name:'terrace 1',a:[0,0,50],b:[0,3,36]},
  {name:'terrace 2',a:[0,3,30],b:[0,7,16]},
  {name:'terrace 3',a:[0,7,10],b:[0,12,-4]},
  {name:'terrace 4',a:[0,12,-10],b:[0,18,-24]},
  {name:'portal court',a:[0,7,14],b:[28,7.1,14]},
];
const report={glbs:[],stairs:[],original_assets_preserved:true};
for(const name of ['dawn','crystal','nomad','ember','frost','verdant','storm','hollow','underforge']){
  assert.equal(manifest[name].generator_sha256,base);
  for(const lod of ['high','lod'])assert.equal(digest(fs.readFileSync(path.join('assets/world',manifest[name][lod].file))),manifest[name][lod].sha256);
}
for(const name of ['dawn-watch','dawn-garden','dawn-ruin']){
  assert.equal(manifest[name].generator_sha256,generator);
  assert.equal(manifest[name].base_generator_sha256,base);
  assert.ok(collisions[name].every(c=>c.center.length===3&&c.size.length===3&&c.center.every(Number.isFinite)&&c.size.every(v=>Number.isFinite(v)&&v>0)));
  assert.ok(collisions[name].some(c=>JSON.stringify(c.center)==='[0,0,80]'&&JSON.stringify(c.size)==='[14,1,18]'));
  if(name==='dawn-watch'){
    assert.deepEqual(landmarks[name].portal,[0,12,54]);
    assert.ok(!collisions[name].some(c=>[0,12,54].every((p,j)=>Math.abs(c.center[j]-p)<(c.size[j]+[12,14,12][j])/2-.001)));
  }else if(name==='dawn-ruin'){
    const portal=[28,18,18];assert.deepEqual(landmarks[name].portal,portal);
    assert.ok(!collisions[name].some(c=>portal.every((p,j)=>Math.abs(c.center[j]-p)<(c.size[j]+[12,14,12][j])/2-.001)),'Ruin portal arrival volume must be empty');
    assert.ok(landmarks[name].crystals.length>=4,'Ruin crystal pillar and resource sites');
    assert.ok(landmarks[name].crystals.some(p=>p[0]===-28&&p[1]>28&&p[2]===9),'Ruin hero crystal resource anchor');
    assert.ok(collisions[name].some(c=>c.center[0]>46&&c.center[1]+c.size[1]/2>=60),'Ruin gate silhouette is a physical tall wall');
  }else assert.equal(landmarks[name].portal,null);
  for(const route of routes[name]){
    for(let i=1;i<300;i++){
      const p=route.a.map((v,j)=>v+(route.b[j]-v)*i/300);
      const hit=collisions[name].find(c=>Math.abs(c.center[0]-p[0])<c.size[0]/2-.002&&Math.abs(c.center[2]-p[2])<c.size[2]/2-.002&&c.center[1]-c.size[1]/2<p[1]+1.8&&c.center[1]+c.size[1]/2>p[1]+.2);
      assert.ok(!hit,`${name} ${route.name}: blocked at ${JSON.stringify(p)} by ${JSON.stringify(hit)}`);
    }
    report.stairs.push({name,route:route.name,clear:true});
  }
  for(const lod of [false,true]){
    const file=`${name}${lod?'-lod':''}.glb`,bytes=fs.readFileSync(path.join(dir,file));
    assert.equal(bytes.readUInt32LE(0),0x46546c67);assert.equal(bytes.readUInt32LE(8),bytes.length);
    const len=bytes.readUInt32LE(12),doc=JSON.parse(bytes.subarray(20,20+len)),binary=bytes.subarray(28+len);
    assert.equal(doc.asset.extras.aether_world_generator_sha256,generator);
    assert.equal(digest(bytes),manifest[name][lod?'lod':'high'].sha256);
    assert.ok(doc.materials.length<=12);
    for(const material of doc.materials.filter(m=>/^(01 |02 |03 |04 )/.test(m.name))){
      const factor=material.pbrMetallicRoughness.baseColorFactor;
      assert.ok(factor?.length===4&&factor.slice(0,3).every(c=>c>0&&c<.70)&&factor[3]===1,`${file}: authored linear stone colour survived export`);
    }
    for(const image of doc.images??[]){
      assert.ok(/^textures\/[a-f0-9]{64}\.png$/.test(image.uri));
      assert.equal(image.bufferView,undefined);
      assert.ok(image.uri.includes(digest(fs.readFileSync(path.join(dir,image.uri)))));
    }
    for(const v of doc.bufferViews)assert.ok(v.byteOffset>=0&&v.byteOffset+v.byteLength<=binary.length);
    let triangles=0,ruinRingUpwardVertices=0,ruinMaximumY=-Infinity;const terrain=[];
    for(const mesh of doc.meshes)for(const prim of mesh.primitives){
      const ia=doc.accessors[prim.indices],iv=doc.bufferViews[ia.bufferView];
      assert.equal(ia.count%3,0);triangles+=ia.count/3;
      for(const key of ['POSITION','NORMAL','TEXCOORD_0']){
        assert.ok(key in prim.attributes);
        const a=doc.accessors[prim.attributes[key]],v=doc.bufferViews[a.bufferView],count=key==='TEXCOORD_0'?2:3,stride=v.byteStride??count*4;
        for(let i=0;i<a.count;i++)for(let j=0;j<count;j++)assert.ok(Number.isFinite(binary.readFloatLE(v.byteOffset+(a.byteOffset??0)+i*stride+j*4)));
      }
      if(name==='dawn-ruin'){
        const pa=doc.accessors[prim.attributes.POSITION],pv=doc.bufferViews[pa.bufferView];
        const na=doc.accessors[prim.attributes.NORMAL],nv=doc.bufferViews[na.bufferView];
        ruinMaximumY=Math.max(ruinMaximumY,pa.max[1]);
        for(let i=0;i<pa.count;i++){
          const p=[0,1,2].map(j=>binary.readFloatLE(pv.byteOffset+(pa.byteOffset??0)+i*(pv.byteStride??12)+j*4));
          const r=Math.hypot(p[0]-28,p[2]-18);
          if(Math.abs(p[1]-7.31)<.002&&r>=10.7&&r<=14.1){
            const normalY=binary.readFloatLE(nv.byteOffset+(na.byteOffset??0)+i*(nv.byteStride??12)+4);
            if(normalY>.90)ruinRingUpwardVertices++;
          }
        }
      }
      if(lod&&/^(01 |02 |12 )/.test(mesh.name)){
        const a=doc.accessors[prim.attributes.POSITION],v=doc.bufferViews[a.bufferView],width=ia.componentType===5125?4:2;
        const vert=n=>[0,1,2].map(j=>binary.readFloatLE(v.byteOffset+(a.byteOffset??0)+n*(v.byteStride??12)+j*4));
        for(let i=0;i<ia.count;i+=3){
          const tri=[];
          for(let j=0;j<3;j++){const o=iv.byteOffset+(ia.byteOffset??0)+(i+j)*width;tri.push(vert(width===4?binary.readUInt32LE(o):binary.readUInt16LE(o)));}
          terrain.push(tri);
        }
      }
    }
    assert.equal(triangles,manifest[name][lod?'lod':'high'].triangles);
    assert.ok(triangles<=(lod?12000:200000));
    if(name==='dawn-ruin'){
      assert.ok(ruinMaximumY>=64,'Ruin broken gate silhouette must survive both LODs');
      assert.ok(ruinRingUpwardVertices>=100,'Ruin horizontal dressed ring must have outward upper faces');
    }
    let closed=0;
    if(lod)for(const col of collisions[name].filter(c=>c.size[0]===6&&c.size[2]===6)){
      const x=col.center[0]+.17,z=col.center[2]-.13,hits=[];
      for(const [a,b,c] of terrain){
        const den=(b[2]-c[2])*(a[0]-c[0])+(c[0]-b[0])*(a[2]-c[2]);if(Math.abs(den)<1e-8)continue;
        const u=((b[2]-c[2])*(x-c[0])+(c[0]-b[0])*(z-c[2]))/den;
        const v=((c[2]-a[2])*(x-c[0])+(a[0]-c[0])*(z-c[2]))/den;
        if(u>=-1e-5&&v>=-1e-5&&u+v<=1+1e-5)hits.push(u*a[1]+v*b[1]+(1-u-v)*c[1]);
      }
      for(const y of [col.center[1]-col.size[1]/2,col.center[1]+col.size[1]/2])assert.ok(hits.some(v=>Math.abs(v-y)<.03),`${file}: missing mass cap ${x},${y},${z}`);
      closed++;
    }
    let rotor=null;
    if(name==='dawn-garden'){
      const nodes=doc.nodes.filter(n=>n.name==='WindmillRotor');assert.equal(nodes.length,1);
      rotor=nodes[0];assert.ok(Number.isInteger(rotor.mesh));
      assert.equal(rotor.extras.aether_animation_axis,'local Z');
      assert.ok(rotor.translation.every((v,j)=>Math.abs(v-[24,49,-20.2][j])<.001));
    }
    report.glbs.push({file,triangles,materials:doc.materials.length,meshes:doc.meshes.length,closed_columns:closed,rotor,...(name==='dawn-ruin'?{ruin_gate_maximum_y:ruinMaximumY,ruin_ring_upward_vertices:ruinRingUpwardVertices}:{})});
  }
}
fs.writeFileSync('docs/evidence/world-art-variants-validation.json',JSON.stringify(report,null,2));
console.log(JSON.stringify(report,null,2));
