// Independent CPU gate for the saved, shared R57 incident World field.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
export function validateSkyField(scene,manifest){
  const field=manifest.atmosphere_authoring.circumsolar_field;
  assert(field&&field.model==='shared incident boundary replacement');
  const nodes=scene.world.nodes,links=scene.world.links;
  const node=prefix=>{const a=nodes.filter(n=>n.name.startsWith(prefix));assert.equal(a.length,1,`Unique source node ${prefix}`);return a[0];};
  const input=(n,name)=>{const a=Object.entries(n.inputs).filter(([k,v])=>k.replace(/^\d+:/,'')===name&&typeof v==='number');assert.equal(a.length,1,`Unique scalar input ${name}`);return a[0][1];};
  const number=(n,index)=>{const a=Object.entries(n.inputs).find(([k])=>k.startsWith(index+':'));assert(a);return a[1];};
  const connection=(a,ai,b,bi)=>assert(links.some(l=>l[0]===a.name&&l[1]===ai&&l[2]===b.name&&l[3]===bi),'Source graph connection differs');
  const dot=node('Dot with actual stellar direction'),blue=node('Explicit broad blue sky poles');
  const disk=node('Uniform radiance stellar surface'),star=node('Calibrated finite angular radiance');
  const dist=node('R57 C | solar angular distance'),divider=node('R57 C | angular support divisor');
  const weight=node('R57 C | shared smooth incident weight'),mix=node('R57 C | replace blue incident field');
  const background=node('Blue boundary radiance | all ray types');
  assert.equal(dot.properties.operation,'DOT_PRODUCT');assert.equal(disk.properties.operation,'GREATER_THAN');
  const radius=manifest.atmosphere_authoring.disk_angular_diameter_degrees*Math.PI/360;
  assert.equal(number(disk,1),Math.fround(Math.cos(radius)),'Actual angular mask differs from declared1.20degree source');
  assert.equal(dist.properties.operation,'SUBTRACT');assert.equal(number(dist,0),1);
  assert.equal(divider.properties.operation,'DIVIDE');
  const support=field.support_radius_degrees*Math.PI/180,d=number(divider,1);
  assert.equal(d,Math.fround(1-Math.cos(support)),'Actual angular field support differs');
  assert.equal(weight.properties.interpolation_type,'SMOOTHSTEP');assert.equal(weight.properties.clamp,true);
  assert.equal(weight.properties.data_type,'FLOAT');
  assert.equal(input(weight,'From Min'),0);assert.equal(input(weight,'From Max'),1);
  assert.equal(input(weight,'To Min'),1);assert.equal(input(weight,'To Max'),0);
  assert.equal(mix.properties.blend_type,'MIX');assert.equal(mix.properties.use_clamp,false);assert.equal(mix.properties.use_alpha,false);
  assert.deepEqual(number(mix,2).slice(0,3),field.warm_boundary_rgb.map(Math.fround));
  assert.deepEqual(number(blue,1).slice(0,3),manifest.atmosphere_authoring.sky_pole_low.slice(0,3).map(Math.fround));
  assert.deepEqual(number(blue,2).slice(0,3),manifest.atmosphere_authoring.sky_pole_high.slice(0,3).map(Math.fround));
  connection(dot,1,dist,1);connection(dist,0,divider,0);connection(divider,0,weight,0);
  connection(weight,0,mix,0);connection(blue,0,mix,1);connection(mix,0,background,0);
  connection(dot,1,disk,0);
  const mask=node('IBL omit direct camera disk'),lightPath=node('IBL separation keeps star illumination');
  const retain=node('Retain star for all non-camera paths'),surface=node('Same emitter surface for camera and lighting');
  const stellarBackground=node('Single warm stellar emitter'),sumNode=node('Physical radiance sum'),output=node('Planetless world radiance');
  assert.equal(background.type,'ShaderNodeBackground');assert.equal(number(background,1),1);
  assert(!links.some(l=>l[2]===background.name&&l[3]===1),'Boundary strength must not be overridden');
  assert.equal(mask.properties.operation,'MULTIPLY');assert.equal(number(mask,1),0);
  assert.equal(lightPath.type,'ShaderNodeLightPath');assert.equal(retain.properties.operation,'SUBTRACT');assert.equal(number(retain,0),1);
  assert.equal(surface.properties.operation,'MULTIPLY');assert.equal(star.properties.operation,'MULTIPLY');
  assert.equal(output.properties.is_active_output,true);
  assert(!links.some(l=>l[2]===output.name&&l[3]===1),'Unrecorded World volume is forbidden');
  connection(lightPath,0,mask,0);connection(mask,0,retain,1);connection(retain,0,surface,1);
  connection(disk,0,surface,0);connection(surface,0,star,0);connection(star,0,stellarBackground,1);
  connection(background,0,sumNode,0);connection(stellarBackground,0,sumNode,1);connection(sumNode,0,output,0);
  const ancestors=new Set();const visit=n=>{if(ancestors.has(n.name))return;ancestors.add(n.name);for(const l of links.filter(l=>l[2]===n.name))visit(nodes.find(x=>x.name===l[0]));};visit(mix);
  assert(!nodes.some(n=>ancestors.has(n.name)&&n.type==='ShaderNodeLightPath'),'Incident field must not have a camera/light-path mask');
  const axis=number(dot,1),length=Math.hypot(...axis),unitZ=axis[2]/length;
  const low=number(blue,1),high=number(blue,2),warm=number(mix,2);
  const minimum=(1-d)/length,count=65536,step=(1-minimum)/count;
  const sum=[0,0,0];
  for(let i=0;i<=count;i++){
    const mu=minimum+i*step,t=Math.max(0,Math.min(1,(1-length*mu)/d));
    const w=1-t*t*(3-2*t),quadrature=i===0||i===count?1:i%2?4:2;
    for(let k=0;k<3;k++){const b=(low[k]+high[k])*.5+(high[k]-low[k])*.5*unitZ*mu;sum[k]+=quadrature*w*(warm[k]-b)*mu;}
  }
  const extra=sum.map(x=>2*Math.PI*step*x/3);
  const core=number(star,1)*Math.PI*(1-(number(disk,1)/length)**2);
  const total=core+extra[0];assert(Math.abs(total-manifest.sun_energy)<2e-6,'Actual core + circumsolar irradiance differs from22');
  assert(Math.abs(total-field.total_red_irradiance)<2e-6);
  assert(Math.abs(extra[0]-field.additional_red_irradiance)<2e-10);
  assert(Math.abs(core/manifest.sun_energy-field.core_fraction)<2e-8);
  for(const angle of [4.501,5,10,20,90,180]){const t=Math.max(0,Math.min(1,(1-length*Math.cos(angle*Math.PI/180))/d));assert.equal(1-t*t*(3-2*t),0,'Outside cone must retain B exactly');}
  return {angular_mask_verified:true,all_ray_field_verified:true,core_red_irradiance:core,
    additional_normal_irradiance_rgb:extra,total_red_irradiance:total,core_fraction:core/manifest.sun_energy,
    actual_disk_diameter_degrees:2*Math.acos(number(disk,1)/length)*180/Math.PI,quadrature_intervals:count};
}
function validateCloudShader(shader,field){
  assert(shader.includes('let direct=vec3<f32>(3.0,1.86,.96)*scattering'),'Cloud direct chromaticity differs from source');
  assert(shader.includes('let bounce=vec3<f32>(3.0,1.86,.96)*.35'),'Cloud bounce chromaticity differs from source');
  assert(shader.includes('h)*.65*vec3<f32>(4.0,4.5,5.0)'),'Cloud incident blue field calibration differs');
  assert.deepEqual(field.cloud_blue_fill_ratio,[4,4.5,5]);
  const original=shader.replace('h)*.65*vec3<f32>(4.0,4.5,5.0)','h)*.65').replace('let direct=vec3<f32>(3.0,1.86,.96)*scattering','let direct=vec3<f32>(3.0,2.55,1.95)*scattering');
  return original;
}
const direct=process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url);
if(direct){
  const argument=flag=>{const i=process.argv.indexOf(flag);return i<0?undefined:process.argv[i+1];};
  const root=path.resolve(argument('--root')??path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../..'));
  const read=file=>fs.readFileSync(path.resolve(root,file));const hash=b=>createHash('sha256').update(b).digest('hex');
  const manifest=JSON.parse(read('tools/art/sky-source/manifest.json'));
  assert.equal(hash(read('tools/art/sky-source/'+manifest.source)),manifest.source_sha256,'Source SHA differs');
  const scene=JSON.parse(read('tools/art/sky-source/render-scene-descriptor.json'));
  const proof=validateSkyField(scene,manifest),shader=read('assets/shaders/clouds.wgsl').toString();
  const old=validateCloudShader(shader,manifest.atmosphere_authoring.circumsolar_field);
  assert.equal(hash(Buffer.from(old)),hash(read('tools/art/sky-source/history/r55-cloud-before-calibration.wgsl')),'Cloud density/integration differs from R55');
  let rejected=0;
  if(process.argv.includes('--self-test')){
    const test=mutate=>{const s=structuredClone(scene),m=structuredClone(manifest);mutate(s,m);assert.throws(()=>validateSkyField(s,m));rejected++;};
    const n=(s,p)=>s.world.nodes.find(n=>n.name.startsWith(p));
    test(s=>{n(s,'Uniform radiance stellar surface').inputs['1:Value']=Math.fround(Math.cos(.375*Math.PI/180));});
    test(s=>{n(s,'Calibrated finite angular radiance').inputs['1:Value']*=1.01;});
    test(s=>{n(s,'R57 C | angular support divisor').inputs['1:Value']*=2;});
    test(s=>{n(s,'R57 C | shared smooth incident weight').properties.clamp=false;});
    test(s=>{n(s,'R57 C | shared smooth incident weight').properties.data_type='VECTOR';});
    test(s=>{s.world.links=s.world.links.filter(l=>!l[0].startsWith('Explicit broad blue sky poles'));});
    test(s=>{n(s,'World ray direction').type='ShaderNodeLightPath';});
    test(s=>{n(s,'IBL omit direct camera disk').inputs['1:Value']=1;});
    test(s=>{n(s,'Blue boundary radiance').inputs['1:Strength']=2;});
    test(s=>{s.world.links=s.world.links.filter(l=>!l[0].startsWith('Physical radiance sum'));});
    test(s=>{n(s,'Explicit broad blue sky poles').inputs['1:Color1'][2]*=1.1;});
    test((s,m)=>{m.atmosphere_authoring.circumsolar_field.warm_boundary_rgb[0]*=1.1;});
    assert.throws(()=>validateCloudShader(shader.replace('3.0,1.86,.96','3.0,2.55,1.95'),manifest.atmosphere_authoring.circumsolar_field));rejected++;
  }
  const report={status:'PASS',source_sha256:manifest.source_sha256,source_field:proof,
    shader_sha256:hash(Buffer.from(shader)),cloud_density_and_integration_exact_r55:true,self_test_rejections:rejected};
  const output=path.resolve(root,'docs/evidence/sky-r57-field.json');fs.mkdirSync(path.dirname(output),{recursive:true});fs.writeFileSync(output,JSON.stringify(report,null,2)+'\n');
  console.log(JSON.stringify(report,null,2));
}
