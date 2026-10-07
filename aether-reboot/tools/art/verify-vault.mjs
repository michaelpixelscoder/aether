import fs from 'node:fs';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
const dir='assets/world/',manifest=JSON.parse(fs.readFileSync(dir+'vault-manifest.json'));
const cols=JSON.parse(fs.readFileSync(dir+'vault-collisions.json'));
const hash=b=>crypto.createHash('sha256').update(b).digest('hex');
const generator=hash(fs.readFileSync('tools/art/build_vault.py'));
assert.equal(generator,manifest.generator_sha256);
assert.equal(hash(fs.readFileSync('tools/art/build_world.py')),manifest.base_generator_sha256);
assert.deepEqual(manifest.roof_y,[350,450]);
assert.deepEqual(manifest.outer_massif_y,[418,650]);
const old=JSON.parse(fs.readFileSync(dir+'manifest.json'));
for(const val of Object.values(old))for(const level of ['high','lod'])assert.equal(hash(fs.readFileSync(dir+val[level].file)),val[level].sha256,'island asset changed');
for(const c of cols){
  assert.ok(c.center.every(Number.isFinite)&&c.size.every(v=>Number.isFinite(v)&&v>0));
  const [x,y,z]=c.center,[sx,sy,sz]=c.size;
  if(y-sy/2<300){
    assert.ok(Math.hypot(Math.max(0,Math.abs(x)-sx/2),Math.max(0,Math.abs(z)-sz/2))>1400,'central navigation must be empty');
    assert.ok(Math.abs(x)-sx/2>=250,'entry corridor must be empty');
  }
  if(y+sy/2>349)for(const cx of [-700,700])assert.ok(Math.hypot(Math.max(0,Math.abs(x-cx)-sx/2),Math.max(0,Math.abs(z)-sz/2))>=200,'skylight collision');
}
function ray(origin,dir,triangles){
  let closest=Infinity;
  for(const t of triangles){
    const [ax,ay,az,bx,by,bz,cx,cy,cz]=t;
    const e1x=bx-ax,e1y=by-ay,e1z=bz-az,e2x=cx-ax,e2y=cy-ay,e2z=cz-az;
    const px=dir[1]*e2z-dir[2]*e2y,py=dir[2]*e2x-dir[0]*e2z,pz=dir[0]*e2y-dir[1]*e2x;
    const det=e1x*px+e1y*py+e1z*pz;if(Math.abs(det)<1e-8)continue;
    const tx=origin[0]-ax,ty=origin[1]-ay,tz=origin[2]-az;
    const u=(tx*px+ty*py+tz*pz)/det;if(u<0||u>1)continue;
    const qx=ty*e1z-tz*e1y,qy=tz*e1x-tx*e1z,qz=tx*e1y-ty*e1x;
    const v=(dir[0]*qx+dir[1]*qy+dir[2]*qz)/det;if(v<0||u+v>1)continue;
    const distance=(e2x*qx+e2y*qy+e2z*qz)/det;if(distance>1e-4&&distance<closest)closest=distance;
  }
  return closest;
}
const report={glbs:[],colliders:cols.length,original_island_hashes_preserved:true};
let denseWallRays=0;
for(let y=-550;y<400;y+=100)for(let i=0;i<1440;i++){
  const a=i*Math.PI/720,dx=Math.cos(a),dz=Math.sin(a);
  if(Math.abs(dx)*1500<300)continue;
  const hit=cols.some(c=>{
    if(y<c.center[1]-c.size[1]/2||y>c.center[1]+c.size[1]/2)return false;
    const x0=(c.center[0]-c.size[0]/2)/dx,x1=(c.center[0]+c.size[0]/2)/dx;
    const z0=(c.center[2]-c.size[2]/2)/dz,z1=(c.center[2]+c.size[2]/2)/dz;
    return Math.max(Math.min(x0,x1),Math.min(z0,z1),0)<Math.min(Math.max(x0,x1),Math.max(z0,z1));
  });
  assert.ok(hit,`diagonal wall gap at height ${y}, angular sample ${i}`);denseWallRays++;
}
report.dense_wall_sightlines_closed=denseWallRays;
for(const lod of [false,true]){
  const file='arch-vault'+(lod?'-lod':'')+'.glb',bytes=fs.readFileSync(dir+file);
  assert.equal(bytes.readUInt32LE(0),0x46546c67);assert.equal(bytes.readUInt32LE(8),bytes.length);
  const len=bytes.readUInt32LE(12),doc=JSON.parse(bytes.subarray(20,20+len)),binary=bytes.subarray(28+len);
  assert.equal(doc.asset.extras.aether_world_generator_sha256,generator);
  assert.equal(hash(bytes),manifest[lod?'lod':'high'].sha256);
  assert.ok(doc.materials.length<=5);
  for(const material of doc.materials.filter(m=>/^(01 |02 |03 )/.test(m.name))){
    assert.ok(material.pbrMetallicRoughness.baseColorFactor?.slice(0,3).every(v=>v>0&&v<.4),'stone PBR factor survives glTF export');
  }
  for(const image of doc.images){assert.ok(/^textures\/[a-f0-9]{64}\.png$/.test(image.uri));assert.ok(image.uri.includes(hash(fs.readFileSync(dir+image.uri))));}
  const triangles=[];
  for(const mesh of doc.meshes)for(const primitive of mesh.primitives){
    for(const key of ['POSITION','NORMAL','TEXCOORD_0'])assert.ok(key in primitive.attributes);
    const a=doc.accessors[primitive.attributes.POSITION],v=doc.bufferViews[a.bufferView],stride=v.byteStride??12;
    const vertex=n=>[0,1,2].map(j=>binary.readFloatLE(v.byteOffset+(a.byteOffset??0)+n*stride+j*4));
    const ia=doc.accessors[primitive.indices],iv=doc.bufferViews[ia.bufferView],width=ia.componentType===5125?4:2;
    for(let i=0;i<ia.count;i+=3){
      const tri=[];
      for(let j=0;j<3;j++){const offset=iv.byteOffset+(ia.byteOffset??0)+(i+j)*width;tri.push(...vertex(width===4?binary.readUInt32LE(offset):binary.readUInt16LE(offset)));}
      assert.ok(tri.every(Number.isFinite));triangles.push(tri);
    }
  }
  assert.equal(triangles.length,manifest[lod?'lod':'high'].triangles);
  assert.ok(triangles.length<=(lod?12000:100000));
  const maximumY=Math.max(...doc.accessors.filter(a=>a.type==='VEC3'&&a.max).map(a=>a.max[1]));
  assert.ok(maximumY>=630&&maximumY<=650.01,`${file}: outer massif height ${maximumY}`);
  let skylights=0,entries=0,roof=0,walls=0,floor=0;
  for(const cx of [-700,700])for(const radius of [0,99,199])for(let i=0;i<16;i++){
    const a=i*Math.PI/8,p=[cx+Math.cos(a)*radius,-650,Math.sin(a)*radius];
    assert.equal(ray(p,[0,1,0],triangles),Infinity,`${file} skylight blocked ${p}`);skylights++;
  }
  for(const x of [-249,-125,0,125,249])for(const y of [-550,-200,0,200,299])for(const sign of [-1,1]){
    assert.equal(ray([x,y,sign*1800],[0,0,-sign],triangles),Infinity,`${file} axial entry blocked`);entries++;
  }
  for(let i=0;i<120;i++){
    const a=i*Math.PI/60,dir=[Math.cos(a),0,Math.sin(a)];
    if(Math.abs(dir[0])*1500<300)continue;
    for(const y of [-400,0,200]){
      const dist=ray([0,y,0],dir,triangles);
      assert.ok(Number.isFinite(dist)&&dist>=1400&&dist<1675,`${file}: wall sightline missing at ${i}, ${y}: ${dist}`);walls++;
    }
  }
  for(let x=-1200;x<=1200;x+=200)for(let z=-1200;z<=1200;z+=200){
    if(Math.hypot(x,z)>1300||[-700,700].some(cx=>Math.hypot(x-cx,z)<300))continue;
    const hit=ray([x,0,z],[0,1,0],triangles);assert.ok(hit>=349&&hit<=451,`${file}: missing roof ${x},${z}, ${hit}`);roof++;
    assert.equal(ray([x,0,z],[0,-1,0],triangles),Infinity,`${file}: unwanted solid floor`);floor++;
  }
  let upperCollisions=0;
  if(!lod){
    const upper=cols.filter(c=>c.center[1]-c.size[1]/2>=417.99&&c.center[1]+c.size[1]/2>450);
    for(let i=0;i<upper.length;i+=Math.max(1,Math.floor(upper.length/128))){
      const c=upper[i],x=c.center[0]+.173,z=c.center[2]-.137;
      const solidTop=Math.max(...cols.filter(k=>Math.abs(k.center[0]-x)<k.size[0]/2&&Math.abs(k.center[2]-z)<k.size[2]/2).map(k=>k.center[1]+k.size[1]/2));
      const visibleTop=800-ray([x,800,z],[0,-1,0],triangles);
      assert.ok(Math.abs(solidTop-visibleTop)<.15,`${file}: visible/collision upper surface mismatch at ${x},${z}: ${solidTop}/${visibleTop}`);
      upperCollisions++;
    }
  }
  report.glbs.push({file,triangles:triangles.length,materials:doc.materials.length,skylight_rays_clear:skylights,entry_rays_clear:entries,wall_sightlines_closed:walls,roof_hits:roof,floor_rays_clear:floor,outer_massif_max_y:maximumY,upper_collision_surfaces_matched:upperCollisions});
}
fs.writeFileSync('docs/evidence/world-art-vault-validation.json',JSON.stringify(report,null,2));
console.log(JSON.stringify(report,null,2));
