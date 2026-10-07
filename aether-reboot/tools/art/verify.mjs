import fs from 'node:fs';import crypto from 'node:crypto';import {spawnSync} from 'node:child_process';import assert from 'node:assert/strict';
import {verifySailProvenance} from './sail_provenance.mjs';
const digest=p=>crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const core=spawnSync('cargo',['run','-p','aether_core','--example','art_scene','--locked','--quiet'],{encoding:'utf8'});
if(core.status!==0)throw Error(core.stderr);
assert.deepEqual(JSON.parse(core.stdout),JSON.parse(fs.readFileSync('tools/art/scene.json','utf8').replace(/^\uFEFF/,'')));
const geometry=digest('tools/art/scene.json'),generator=digest('tools/art/build_art.py');
const sailGate=spawnSync(process.execPath,['tools/art/verify_sail_r52.mjs'],{encoding:'utf8'});
assert.equal(sailGate.status,0,`Sail pressure/ripple extremes: ${sailGate.stderr}`);
let sailMorphs=0;let sailPipeline;
const models=[];for(const file of fs.readdirSync('assets/art').filter(n=>n.endsWith('.glb'))){
 const bytes=fs.readFileSync('assets/art/'+file);assert.equal(bytes.readUInt32LE(0),0x46546c67);assert.equal(bytes.readUInt32LE(8),bytes.length);
 const doc=JSON.parse(bytes.subarray(20,20+bytes.readUInt32LE(12)));assert.equal(doc.asset.extras.aether_geometry_sha256,geometry);if(file!=='sail.glb')assert.equal(doc.asset.extras.aether_generator_sha256,generator);
 if(file==='sail.glb'){
  sailPipeline=verifySailProvenance(doc,bytes.subarray(28+bytes.readUInt32LE(12)),process.cwd(),bytes);
  for(const mesh of doc.meshes){
   const cloth=/^(Indigo canvas|Embroidered gold thread)/.test(mesh.name);
   for(const primitive of mesh.primitives){
    if(cloth){
     assert.equal(primitive.targets?.length,3);assert.deepEqual(mesh.extras.targetNames,['WindPressure','RippleSin','RippleCos']);sailMorphs++;
     for(const [i,target] of primitive.targets.entries()){
      const position=doc.accessors[target.POSITION];
      assert.equal(position.count,doc.accessors[primitive.attributes.POSITION].count);
      assert.ok(position.min[0]===0&&position.max[0]===0&&position.min[1]===0&&position.max[1]===0,'cloth stays pinned in its plane');
      assert.ok(Math.max(...position.min.map(Math.abs),...position.max.map(Math.abs))<=(i===0?.18001:.06501),'morph targets must each derive from the neutral basis, within authored amplitude');
     }
    }
    else assert.equal(primitive.targets,undefined,'mast, metal and rigging must remain rigid');
   }
  }
  assert.ok(sailMorphs>=3,'canvas and embroidery require authored morph targets');
 }
 models.push({file,sha256:digest('assets/art/'+file),meshes:doc.meshes.length,materials:doc.materials.length});
}
fs.writeFileSync('docs/evidence/art-assets.json',JSON.stringify({terrain_matches_domain:true,sail_morph_primitives:sailMorphs,geometry_sha256:geometry,base_generator_sha256:generator,sail_pipeline:sailPipeline,models},null,2));console.log(`${models.length} GLB vérifiés : domaine, générateur et tailles cohérents ; ${sailMorphs} surfaces de voile animables.`);
