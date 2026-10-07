// Independent byte-level R66 whitelist. No author-generator import.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';
const author=path.dirname(fileURLToPath(import.meta.url));
const root=path.resolve(author,'../../..');
const sha=b=>crypto.createHash('sha256').update(b).digest('hex');
const must=(condition,message)=>{if(!condition)throw new Error(message);};
const history=fs.readFileSync(path.join(author,'history/world-flow-r64.wgsl'));
must(sha(history)==='0861bdc4197dab68fb10826a6165fa6ef5e325fa52ad6d42331a6b65e30c5cf5','R64 historical shader authority');
const original='let optical_profile=.12+1.88*middle;',replacement='let optical_profile=1.0;';
const text=history.toString('utf8');must(text.split(original).length===2,'exact one original profile');
const expected=Buffer.from(text.replace(original,replacement),'utf8');
must(sha(expected)==='b76e67027f1b34cf6f9d56192bcdf315ad43810ff32c4e64e075297ab9a16c31','independent reconstructed R66');
export function validateShader(raw){must(Buffer.isBuffer(raw)&&raw.equals(expected),'only exact R66 radiance-profile substitution permitted');return true;}
validateShader(fs.readFileSync(path.join(root,'assets/shaders/world-flow.wgsl')));
for(const [rel,digest] of [['assets/textures/world-flow-current-r64.ktx2','68755a59545f61bc047975e7af6c301bd98164468a6f293a1336e9bdbb5b1f05'],['assets/textures/world-flow.ktx2','8ecd62eb7df01a02d9864521e63b1c29a128ff18a267f369a1d7d96cd73b4973'],['tools/art/flow-r66-source/source/world-flow-current-native.png','eaf534efc2a2e30632fcfee7f3d5b731efe77dcd19f0849216e6c74c662497c3']])must(sha(fs.readFileSync(path.join(root,rel)))===digest,rel+' authority');
const mutations=[
 ['second radiance parameter','let gain=14.0*','let gain=14.1*'],
 ['coverage','let coverage=(body_alpha+.62*','let coverage=(body_alpha+.63*'],
 ['body alpha','let body_alpha=.018','let body_alpha=.019'],
 ['clock','let t=settings.w*settings.y;','let t=settings.w*settings.y+1.0;'],
 ['core','let core=thermal*smoothstep(16.5,18.25,uv.x)*middle;','let core=thermal*smoothstep(16.0,18.25,uv.x)*middle;'],
 ['core conversion','core*.85','core*.86'],
 ['middle preserved','smoothstep(.16,.43,abs(uv.y-.5))','smoothstep(.15,.43,abs(uv.y-.5))'],
 ['crop','let crop=.42;','let crop=.43;'],
 ['crop offset','let crop_start=.29;','let crop_start=.30;'],
 ['threshold','mix(.035,.020,','mix(.035,.019,'],
 ['sampler binding','@binding(3)','@binding(4)'],
 ['flow scroll','uv.x*.20-t*.10','uv.x*.20-t*.11'],
 ['drift','sin(uv.x*.37-t*.12)*.006','sin(uv.x*.37-t*.12)*.007'],
 ['rolloff','crest_peak*gain/6.0','crest_peak*gain/7.0'],
 ['edge alpha','smoothstep(.015,.095,uv.y)','smoothstep(.015,.090,uv.y)'],
 ['near fade','smoothstep(2.0,22.0,','smoothstep(2.0,20.0,'],
 ['cascade crop','uv.x*.24+.37+sway','uv.x*.25+.37+sway'],
 ['pool wave','sin(p.x*1.8+t*2.0','sin(p.x*1.9+t*2.0'],
 ['haze','haze*0.8','haze*0.7'],
 ['constant profile gain compensation',replacement,'let optical_profile=1.1;'],
];
for(const [name,before,after] of mutations){const src=expected.toString('utf8');must(src.includes(before),'negative fixture trigger '+name);const changed=Buffer.from(src.replace(before,after));let rejected=false;try{validateShader(changed);}catch{rejected=true;}must(rejected,'negative not rejected '+name);}
let originalRejected=false;try{validateShader(history);}catch{originalRejected=true;}must(originalRejected,'R64 does not masquerade as R66');
const proof=JSON.parse(fs.readFileSync(path.join(author,'proofs/radiance-delta.json'),'utf8'));
must(proof.passed&&proof.coverage_and_native_mass_identical&&proof.mips.length===9,'CPU proof structure');
must(proof.probe_sha256===sha(fs.readFileSync(path.join(author,'probe.py'))),'CPU probe authentic');
function finite(obj){if(typeof obj==='number')must(Number.isFinite(obj),'nonfinite proof');else if(obj&&typeof obj==='object')for(const value of Object.values(obj))finite(value);}
finite(proof);
for(const mip of proof.mips){must(mip.bands.length===3,'three native bands');must(mip.whole_crop.native_mass_delta_max_abs===0&&mip.whole_crop.coverage_delta_max_abs===0,'radiance only');must(mip.whole_crop.crest_radiance_luma_ratio<1,'total radiance decrease reported');must(mip.bands[0].crest_radiance_luma_ratio>1&&mip.bands[1].crest_radiance_luma_ratio<1,'native outer vs middle hierarchy');}
console.log('PASS R66: exact one shader expression, native and historical textures exact, 21 negative shader cases, 9 mips/3bands finite radiance-only CPU proofs; GPU pending.');
