// Reproduce the reviewed R55 shader from the frozen R54 source; no staging inputs.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import {fileURLToPath} from 'node:url';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../..');
export function generate(repository=root) {
 const input=fs.readFileSync(path.join(repository,'tools/art/cloud-r55/archive/captured-clouds-r54.wgsl'));
 assert.equal(crypto.createHash('sha256').update(input).digest('hex'),'ebd34cf654f6b9fa4a37640a7d37b15405cbd317a00a0ec31204b4ab5b823d48');
 let shader=input.toString('utf8');
 const start=shader.indexOf('fn sunlight('),end=shader.indexOf('@fragment',start);
 assert(start>=0&&end>start);
const replacement=`// R55 isolated lighting prototype. Density and integration are unchanged.
// The upper warm lobe is the same documented cloud-bounce approximation
// used by the objects: elevation42 degrees, solar azimuth, relative strength.35.
// It is incoming reflected illumination, not another stellar emitter/disk.
fn light_transmission(p:vec3<f32>,light_dir:vec3<f32>)->f32 {
    var optical=0.0;var distance=18.0;var stride=30.0;
    let side=normalize(cross(light_dir,vec3<f32>(0.0,0.0,1.0)));
    for(var j=0u;j<4u;j+=1u) {
        let offset=side*select(-1.0,1.0,(j&1u)==0u)*distance*.10;
        optical+=density(p+light_dir*distance+offset)*stride;
        distance+=stride;stride*=1.9;
    }
    let extinction=optical*shape.y;
    // Preserve the existing two-scale attenuation approximation. This is
    // not represented as an exact multiple-scattering solution.
    return exp(-extinction)*.80+exp(-extinction*.27)*.20;
}
fn cloud_phase(cos_angle:f32)->f32 {
    let g=.45;
    let phase=(1.0-g*g)/pow(max(.1,1.0+g*g-2.0*g*cos_angle),1.5);
    return clamp(phase,.65,1.7);
}
fn sunlight(p:vec3<f32>,d:f32,normal:vec3<f32>,cos_angle:f32)->vec3<f32> {
    let light_dir=normalize(sun.xyz);
    let scattering=light_transmission(p,light_dir);
    let powder=.65+.35*(1.0-exp(-d*4.0));
    let h=clamp((p.y-center.y)/extent.y+.5,0.0,1.0);
    let exposure=.20+.80*max(0.0,dot(normal,light_dir));
    let ambient=mix(vec3<f32>(.045,.10,.22),vec3<f32>(.18,.30,.53),h)*.65;
    let direct=vec3<f32>(3.0,2.55,1.95)*scattering*powder*cloud_phase(cos_angle)*exposure;
    let horizontal=normalize(vec3<f32>(light_dir.x,0.0,light_dir.z));
    let bounce_dir=horizontal*.7431448255+vec3<f32>(0.0,.6691306064,0.0);
    let view_dir=normalize(p-view.world_position);
    // The same .35 ratio and linear (1,.62,.32) chromaticity as the object
    // bounce. The legacy cloud red-channel radiance scale3 is retained.
    // Participating particles use a phase function, not a Lambert surface
    // gate; actual volume shadowing determines which lobes receive this light.
    let bounce=vec3<f32>(3.0,1.86,.96)*.35
        *light_transmission(p,bounce_dir)*powder*cloud_phase(dot(view_dir,bounce_dir));
    return (ambient+direct+bounce)*mix(.20,1.0,extent.w);
}
`;

 return shader.slice(0,start)+replacement+shader.slice(end);
}
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url)) {
 const output=path.join(root,'assets/shaders/clouds.wgsl');
 fs.mkdirSync(path.dirname(output),{recursive:true});fs.writeFileSync(output,generate());
 console.log('R55 directional cloud-bounce shader regenerated from immutable source.');
}
