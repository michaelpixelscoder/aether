// Strict actual-runtime gate; independent from the historical author generators.
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import assert from 'node:assert/strict';
import {createFlowValidator,AUTHORITIES} from './validate.mjs';
const author=path.dirname(fileURLToPath(import.meta.url));
const root=process.argv[2]==='--root'?path.resolve(process.argv[3]):path.resolve(author,'../../..');
const read=p=>fs.readFileSync(path.join(root,p));
const runtime=read('assets/shaders/world-flow.wgsl');const validate=createFlowValidator(root);
const accepted=validate(runtime);assert.equal(accepted.report.runtime_shader_sha256,AUTHORITIES.r66);
const mutations=[
 ['clock','settings.w*settings.y','settings.w+settings.y'],['vertex alpha','alpha*=in.color.a;','alpha=1.0;'],
 ['cascade alpha','let opacity=.012+.62*crest','let opacity=.5+.62*crest'],['pool wave','sin(p.x*1.8','sin(p.x*3.8'],['haze','haze*0.8','haze*0.2'],
 ['bindings','@binding(3)','@binding(4)'],['discard','var rgb=color.rgb;','discard; var rgb=color.rgb;'],
 ['asset crop','let crop=.42;','let crop=.16;'],['crop start','let crop_start=.29;','let crop_start=.42;'],
 ['coverage gate','smoothstep(.05,.18,ridge)','smoothstep(.005,.18,ridge)'],['reserved word','let wave=','let patch='],
 ['radiance gain','let gain=14.0*','let gain=14.1*'],['body alpha','let body_alpha=.018','let body_alpha=.019'],['seam','smoothstep(.012,.065,phase)','smoothstep(.010,.065,phase)'],
 ['temporal phases','fract(phase_a+.5)','fract(phase_a+.4)'],['advection','vec2<f32>(.035,','vec2<f32>(.045,'],['threshold','mix(.035,.020,','mix(.035,.019,'],
 ['rolloff','crest_peak*gain/6.0','crest_peak*gain/7.0'],['profile compensation','let optical_profile=1.0;','let optical_profile=1.1;'],
 ['core mask','smoothstep(16.5,18.25,uv.x)','smoothstep(16.0,18.25,uv.x)'],['core cyan','core*.14*','core*.15*'],['middle','smoothstep(.16,.43,abs(uv.y-.5))','smoothstep(.15,.43,abs(uv.y-.5))'],
 ['near fade','smoothstep(2.0,22.0,','smoothstep(2.0,20.0,']
];
for(const [name,before,after] of mutations){assert(runtime.toString().includes(before),'negative source trigger '+name);assert.throws(()=>validate(Buffer.from(runtime.toString().replace(before,after))),name);}
for(const rev of ['r53','r62','r64']){const old=rev==='r53'?read('tools/art/flow-current-source/history/world-flow-r53.wgsl'):read(`tools/art/flow-current-source/archive/${rev}/portable/assets/shaders/world-flow.wgsl`);assert.throws(()=>validate(old),rev+' cannot masquerade as final runtime');}
assert.throws(()=>validate(Buffer.concat([runtime,Buffer.from('\n')])), 'trailing source delta');
const report={...accepted.report,negative_shader_cases_rejected:mutations.map(m=>m[0]).concat(['r53 substitution','r62 substitution','r64 substitution','trailing source delta']),historical_archive_gates:'See proofs/portable-chain.json; archives kept byte exact, tests run on scratch copies.',production_binding_and_native_validation:'PENDING ROOT; diagnostic capture is not production binding proof',promotion_applied:false};
fs.writeFileSync(path.join(author,'proofs/runtime-contract.json'),JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify(report));
