#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view
#import bevy_pbr::mesh_view_bindings::globals
#import bevy_pbr::prepass_utils::prepass_depth
#import bevy_pbr::view_transformations::position_ndc_to_world
@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> center:vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<uniform> extent:vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var field:texture_3d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var field_sampler:sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var<uniform> shape:vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(5) var<uniform> sun:vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(6) var macro_field:texture_3d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(7) var macro_sampler:sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(8) var<uniform> cache_extent:vec4<f32>;
fn billows(p:vec3<f32>)->vec4<f32> {
    let cached=textureSampleLevel(macro_field,macro_sampler,(p-center.xyz)/cache_extent.xyz+.5,0.0);
    let normal=cached.yzw*2.0-1.0;
    return vec4<f32>(cached.x,normal/max(length(normal),.0001));
}
// Perlin-Worley volumes and erosion, never density below a 2D height field.
fn density_and_normal(p:vec3<f32>)->vec4<f32> {
    let local=p-center.xyz;
    let h=local.y/extent.y+0.5;
    if h<=.01 || h>=.99 { return vec4<f32>(0.0); }
    let radial=length(local.xz/(extent.xz*.5));
    // The boundary noise can shift the radial cutoff by at most .06.
    // Reject guaranteed-empty samples before evaluating any cloud lobes.
    if radial>=1.05 { return vec4<f32>(0.0); }
    if shape.w<.5 && length(local/(extent.xyz*.5))>=.96 {
        return vec4<f32>(0.0);
    }
    let drift=vec3<f32>(center.w*.72,center.w*.04,center.w*.31);
    // Bounded continuous advection stays within the cache's 224 m padding.
    // Unlike a modulo wrap, it cannot pop after a long session. Fine erosion
    // still drifts independently through the live periodic noise field.
    let macro_drift=vec3<f32>(sin(center.w*.00375)*192.0,0.0,sin(center.w*.0016145833)*192.0);
    let lobes=billows(p+macro_drift);
    let organic=lobes.x;
    // A continuous stratocumulus base joins the lower regional billows. The
    // old isolated puffs left straight cavern-roof slabs exposed from above.
    let floor_profile=shape.w*.22*smoothstep(.025,.09,h)
        *(1.0-smoothstep(.26,.43,h));
    if organic<.05 && floor_profile<.001 {return vec4<f32>(0.0);}
    let phase=vec3<f32>(shape.z,shape.z*.731,shape.z*.417);
    let q=(p+drift)*vec3<f32>(.00105,.00130,.00105)+phase;
    let base=textureSampleLevel(field,field_sampler,q,0.0);
    let boundary=1.0-smoothstep(.80,.99,radial+(base.a-.5)*.12);
    if boundary<=0.0 {return vec4<f32>(0.0);}
    let vertical=smoothstep(.01,.08,h)*(1.0-smoothstep(.90,.99,h));
    let body=organic-(1.0-base.r)*.80;
    let strata=floor_profile*(.75+base.r*.25);
    var shaped=0.0;
    // Erosion only removes density. A body already below the lower threshold
    // cannot contribute, even at the maximum value of the erosion texture.
    if body>shape.x-.60 {
        let erosion=textureSampleLevel(field,field_sampler,q*3.73+vec3<f32>(.17,.37,.61),0.0);
        let detail=(1.0-(erosion.g*.72+erosion.b*.28))*.28;
        shaped=smoothstep(shape.x-.60,.35,body-detail);
    }
    var d=max(shaped,strata)*boundary*vertical;
    if shape.w<.5 {d*=1.0-smoothstep(.50,.96,length(local/(extent.xyz*.5)));}
    let departure=(1.0-smoothstep(320.0,550.0,length(p.xz)))*smoothstep(-90.0,-20.0,p.y);
    let harbor=(1.0-smoothstep(350.0,570.0,length(p.xz-vec2<f32>(-260.0,-680.0))))*smoothstep(-130.0,-60.0,p.y);
    return vec4<f32>(d*(1.0-max(departure,harbor)),lobes.yzw);
}
fn density(p:vec3<f32>)->f32 { return density_and_normal(p).x; }
// R55 isolated lighting prototype. Density and integration are unchanged.
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
@fragment fn fragment(input:VertexOutput)->@location(0) vec4<f32> {
    let origin=view.world_position;
    let dir=normalize(input.world_position.xyz-origin);
    let safe=select(vec3<f32>(-1.0),vec3<f32>(1.0),dir>=vec3<f32>(0.0))*max(abs(dir),vec3<f32>(0.000001));
    let lo=(center.xyz-extent.xyz*.5-origin)/safe;
    let hi=(center.xyz+extent.xyz*.5-origin)/safe;
    let near=min(lo,hi);let far=max(lo,hi);
    let t0=max(0.0,max(near.x,max(near.y,near.z)));
    var t1=min(far.x,min(far.y,far.z));
    let depth=prepass_depth(input.position,0u);
    if depth>0.0 {
        let uv=(input.position.xy-view.viewport.xy)/view.viewport.zw;
        let opaque=position_ndc_to_world(vec3<f32>(uv*vec2<f32>(2.0,-2.0)+vec2<f32>(-1.0,1.0),depth));
        t1=min(t1,dot(opaque-origin,dir));
    }
    if t1<=t0 {discard;}
    let step=(t1-t0)/128.0;
    // Camera effects include TAA; a low-discrepancy sequence removes marching
    // contours while the temporal history reconstructs stable soft edges.
    let jitter=fract(52.9829189*fract(dot(input.position.xy,vec2<f32>(.06711056,.00583715)))+f32(globals.frame_count%16u)*.61803398875);
    let cos_angle=dot(dir,normalize(sun.xyz));
    var transmission=1.0;var light=vec3<f32>(0.0);var shade=vec3<f32>(0.5);
    for(var i=0u;i<128u;i+=1u) {
        let p=origin+dir*(t0+(f32(i)+jitter)*step);
        let cloud=density_and_normal(p);
        let d=cloud.x;
        if d>.001 {
            let alpha=1.0-exp(-d*step*shape.y);
            if (i&1u)==0u || transmission>.99 {shade=sunlight(p,d,cloud.yzw,cos_angle);}
            light+=shade*alpha*transmission;
            transmission*=1.0-alpha;
            if transmission<.012 {break;}
        }
    }
    return vec4<f32>(light,1.0-transmission);
}
