#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view
@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> color:vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<uniform> settings:vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var flow_texture:texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var flow_sampler:sampler;
fn patch_hash(p:vec2<f32>)->f32 {
    return fract(sin(dot(p,vec2<f32>(127.1,311.7)))*43758.5453);
}
fn chromatic_patch(p:vec2<f32>)->f32 {
    let cell=floor(p);let f=fract(p);let w=f*f*(3.0-2.0*f);
    return mix(mix(patch_hash(cell),patch_hash(cell+vec2<f32>(1.0,0.0)),w.x),
        mix(patch_hash(cell+vec2<f32>(0.0,1.0)),patch_hash(cell+vec2<f32>(1.0,1.0)),w.x),w.y);
}
// Native ImageGen RGBA, sampled as sRGB color + linear alpha. Neither the
// texture nor a generated normal field is rebuilt from procedural lines.
fn liquid_sample(uv:vec2<f32>)->vec4<f32> {
    // Each image copy has zero weight at its own wrap boundary. Crossfading
    // premultiplied samples gives a genuinely continuous periodic signal even
    // though the native authored edge pixels are not guaranteed to match.
    let phase=fract(uv.x);
    let triangle=1.0-abs(phase*2.0-1.0);
    let weight=triangle*triangle*(3.0-2.0*triangle);
    let grad_x=dpdx(uv);
    let grad_y=dpdy(uv);
    let a=textureSampleGrad(flow_texture,flow_sampler,vec2<f32>(phase,uv.y),grad_x,grad_y);
    let b=textureSampleGrad(flow_texture,flow_sampler,vec2<f32>(fract(phase+.5),uv.y),grad_x,grad_y);
    let opacity=a.a*weight+b.a*(1.0-weight);
    let premultiplied=a.rgb*a.a*weight+b.rgb*b.a*(1.0-weight);
    return vec4<f32>(premultiplied/max(opacity,.00001),opacity);
}
@fragment
fn fragment(in:VertexOutput)->@location(0) vec4<f32> {
    let t=settings.w*settings.y;
    let uv=in.uv;
    var rgb=color.rgb;
    var alpha=settings.z;
#ifdef VERTEX_COLORS
    alpha*=in.color.a;
#endif
    if settings.x < 0.5 {
        // One native tile spans192m in U. Modest transverse deformation keeps
        // the authored lamellae intact; the geometry supplies the large curve.
        let drift=sin(uv.x*.37-t*.12)*.012+sin(uv.x*.83+t*.09)*.006;
        let tex=liquid_sample(vec2<f32>(uv.x*.13-t*.12,uv.y*.72+.14+drift));
        let energy=dot(tex.rgb,vec3<f32>(.08,.62,.30));
        let crest=smoothstep(.38,.90,energy);
        // Color only: sparse irregular violet pools run through the medium
        // layers of the authored liquid. Alpha and cyan-white crests stay put.
        let chromatic_region=chromatic_patch(vec2<f32>(uv.x*.10-t*.016,uv.y*3.6+sin(uv.x*.045)*.4));
        let violet_weight=smoothstep(.66,.82,chromatic_region)*(1.0-smoothstep(.30,.76,energy))*.90;
        let violet=vec3<f32>(max(tex.r,tex.b*.78),tex.g*.22,tex.b*1.08);
        rgb=mix(tex.rgb,violet,violet_weight)*(3.8+crest*1.4);
        // The native alpha and dark body both leave local open space. There
        // is no uniform opacity sheet, no fixed lanes and no white wire term.
        let edge=smoothstep(.015,.105,uv.y)*smoothstep(.015,.105,1.0-uv.y);
        alpha*=edge*tex.a*(.16+.65*smoothstep(.025,.55,energy));
        alpha*=smoothstep(2.0,22.0,length(in.world_position.xyz-view.world_position));
    } else if settings.x < 1.5 {
        // Rotate the same authored flowing surface into the vertical axis.
        // Native holes and folds split the water instead of numbered stripes.
        let sway=sin(uv.y*8.0-t*.40)*.010+sin(uv.y*17.0-t*.65)*.004;
        let tex=liquid_sample(vec2<f32>(uv.y*2.2-t*.26,uv.x*.76+.12+sway));
        let energy=dot(tex.rgb,vec3<f32>(.08,.62,.30));
        let crest=smoothstep(.26,.80,energy);
        rgb=tex.rgb*(1.10+crest*1.75);
        let edge=smoothstep(.015,.085,uv.x)*smoothstep(.015,.085,1.0-uv.x);
        let end=1.0-smoothstep(.66+tex.g*.18,.91+tex.g*.08,uv.y);
        alpha*=edge*tex.a*smoothstep(0.0,.025,uv.y)*end
            *smoothstep(.025,.25,energy)*(.35+crest*.42);
    } else {
        let p=in.world_position.xz;
        let wave=sin(p.x*1.8+t*2.0+sin(p.y*.9))*sin(p.y*2.3-t*1.5);
        let tile=step(0.055,fract(p.x*2.0))*step(0.055,fract(p.y*2.0));
        rgb=color.rgb*(0.9+wave*.12)+vec3<f32>(0.2,0.65,0.75)*pow(max(0.0,wave),12.0);
        rgb*=0.78+0.22*tile;
    }
    let distance=length(in.world_position.xyz-view.world_position);
    let haze=smoothstep(900.0,6500.0,distance);
    rgb=mix(rgb,vec3<f32>(0.38,0.55,0.7),haze*0.6);
    return vec4<f32>(rgb,alpha*(1.0-haze*0.8));
}
