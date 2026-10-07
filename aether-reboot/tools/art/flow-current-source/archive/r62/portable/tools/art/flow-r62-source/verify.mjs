// CPU contract gate, not a WGSL compiler or GPU performance measurement.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import {fileURLToPath} from 'node:url';
const author=path.dirname(fileURLToPath(import.meta.url)),root=path.resolve(author,'../../..');
const sha=b=>crypto.createHash('sha256').update(b).digest('hex'),read=f=>fs.readFileSync(path.join(author,f));
const candidate=fs.readFileSync(path.join(root,'assets/shaders/world-flow.wgsl'));
const base=read('history/world-flow-r53.wgsl');assert.equal(sha(base),'f3f2efc413e71ae5e941325d59c2a457b6863cfa79dd1478a44af475f678fb41');
assert.equal(sha(read('history/world-flow-r58-prototype2.wgsl')),'0d289ec4dda88fbe72a0f7f8ca5a42a2e6e206b8a91427121cdc2b38b62a2951');
assert.equal(sha(read('history/world-flow-r61.wgsl')),'5e2e2c5b377ba30041a0493d0262d0dfc9c778d765548bdcd08c0990683d47d9');
assert.equal(sha(read('source/world-flow-native.png')),'b0925b91ad5c3a8e3f17099b8262963620e806476c493453c0a358722134fc3b');
assert.equal(sha(fs.readFileSync(path.join(root,'assets/textures/world-flow.ktx2'))),'8ecd62eb7df01a02d9864521e63b1c29a128ff18a267f369a1d7d96cd73b4973');
const old=base.toString().replaceAll('\r\n','\n'),block=read('source/current-block.wgsl').toString().replaceAll('\r\n','\n'),helpers=read('source/current-sampling.wgsl').toString().replaceAll('\r\n','\n');
const code=candidate.toString(),reserved=new Set(JSON.parse(read('source/naga-wgsl-reserved-29.0.4.json')).words);
const clean=s=>s.replace(/\/\/[^\n]*/g,'').replace(/^\s*#.*$/gm,'');
function validate(source){
 assert(source.includes(block)&&source.includes(helpers),'Exact audited current source fragments required');
 let restored=source.replace(helpers+'\n','');
 const begin=old.indexOf('    if settings.x < 0.5 {'),end=old.indexOf('    } else if settings.x < 1.5 {');
 restored=restored.replace(block,old.slice(begin,end));
 restored=restored.replace('// R62 isolated coherent current transport; authentic current alpha delta, unchanged native texture/geometry/clock.',old.split('\n')[0]);
 assert.equal(restored,old,'Every non-current source byte after newline normalization exact to R53');
 for(const word of clean(source).match(/[A-Za-z_][A-Za-z_0-9]*/g)??[])assert(!reserved.has(word),'Naga reserved identifier '+word);
 assert(!/\b(?:dpdx|dpdy|fwidth|discard)\s*\(/.test(clean(helpers)),'Sampling helpers cannot add derivative uniformity hazards');
 assert(!/\b(?:for|while|loop)\b/.test(clean(helpers)),'No helper loops');
 assert.equal((clean(helpers).match(/textureSampleGrad\(/g)??[]).length,2);assert.equal((clean(helpers).match(/textureSampleLevel\(/g)??[]).length,1);
 assert.equal((helpers.match(/let [ab]=seam_optics\(/g)??[]).length,2,'Two temporal phases execute four grad samples plus one field sample');
 assert(!/\btextureSample\s*\(/.test(clean(source)),'No implicit derivative texture sample');
 for(const n of [0,1,2,3])assert.equal(source.split('@binding('+n+')').length-1,1);
 assert(source.includes('let t=settings.w*settings.y;')&&source.includes('alpha*=in.color.a;'));
 return true;
}
validate(code);
const cases=[
 ['clock',s=>s.replace('settings.w*settings.y','settings.w+settings.y')],
 ['cascade alpha',s=>s.replace('let opacity=.012+.62*crest','let opacity=.5+.62*crest')],
 ['pool wave',s=>s.replace('sin(p.x*1.8','sin(p.x*3.8')],
 ['haze',s=>s.replace('haze*0.8','haze*0.2')],
 ['vertex alpha',s=>s.replace('alpha*=in.color.a;','alpha=1.0;')],
 ['bindings',s=>s.replace('@binding(3)','@binding(4)')],
 ['discard',s=>s.replace('var rgb=color.rgb;','discard; var rgb=color.rgb;')],
 ['crop',s=>s.replace('mix(.16,.42,thermal)','mix(.82,.42,thermal)')],
 ['coverage',s=>s.replace('smoothstep(.05,.18,ridge)','smoothstep(.005,.18,ridge)')],
 ['reserved identifier',s=>s.replace('let wave=','let patch=')]
];
for(const [label,mutate] of cases)assert.throws(()=>validate(mutate(code)),label);
const proof=JSON.parse(read('proofs/shader.json')),material=JSON.parse(read('proofs/material.json'));
assert.equal(proof.shader_sha256,sha(candidate));assert.equal(proof.generator_sha256,sha(read('build_shader.py')));
for(const [file,digest] of Object.entries(proof.source_fragments))assert.equal(sha(read('source/'+file)),digest);
assert.equal(material.shader_sha256,sha(candidate));assert.equal(material.probe_sha256,sha(read('probe.py')));assert(material.passed&&material.authentic_current_alpha_delta&&material.mips.length===9);
assert(material.mips.every(m=>m.finite_nonnegative&&m.alpha_max<=.9*.653&&m.mean_luma_ratio_to_R61>0&&m.mean_luma_ratio_to_R61<1.25));
assert(material.mips.filter(m=>m.mip>=6).every(m=>m.alpha_mean<.035),'Far native mip coverage cannot recreate a sheet');
assert(material.continuity_max_delta<1e-4&&material.jacobian_max_error<1e-10);
const report={passed:true,shader_sha256:sha(candidate),native_texture_exact:true,cascade_pool_haze_clock_bindings_vertex_alpha_exact:true,current_alpha_and_sampler_delta_explicit:true,current_fetches:5,cascade_fetches:2,helpers_derivative_free:true,negative_cases_rejected:cases.map(c=>c[0]),cpu_material_passed:true,gpu_compile:'PENDING ROOT NATIVE CAPTURE',performance:'UNMEASURED',promotion_authorized:false};
fs.writeFileSync(path.join(author,'proofs/contracts.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
