// R53 energy calibration strong; same native image, geometry and clock.
#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view
@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> color:vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<uniform> settings:vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var flow_texture:texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var flow_sampler:sampler;

// R53 native ImageGen: dark liquid, fine bifurcating cyan/violet crests.
// The two authored samples are associated only during the periodic blend;
// standard AlphaMode::Blend receives straight RGB at the fragment output.
fn liquid_sample(uv:vec2<f32>)->vec4<f32> {
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
        // 1 U = 25m. An authored tile spans125m, with layered fine ripples
        // across82% of its height. Only its own narrow crests receive HDR gain.
        let drift=sin(uv.x*.37-t*.12)*.006+sin(uv.x*.83+t*.09)*.003;
        let tex=liquid_sample(vec2<f32>(uv.x*.20-t*.10,uv.y*.82+.09+drift));
        let energy=dot(tex.rgb,vec3<f32>(.12,.52,.36));
        let ridge=max(energy-.030,0.0)+energy*.10;
        let crest=smoothstep(.025,.20,ridge);
        let opacity=.045+.40*crest;
        let hue=tex.rgb/max(max(tex.r,tex.g),max(tex.b,.001));
        // Positive RGB energy retains blue-dominant authored crests in mips.
        // A 10% energy foot survives when narrow crests average below .030.
        // Divide by our opacity only to return straight RGB to AlphaBlend.
        rgb=tex.rgb*.40+hue*ridge*24.0/opacity;
        let edge=smoothstep(.015,.095,uv.y)*smoothstep(.015,.095,1.0-uv.y);
        // Low background alpha avoids the rejected broad grey absorption sheet.
        // Only authored ripples carry substantial opacity and radiance.
        alpha*=edge*tex.a*opacity;
        alpha*=smoothstep(2.0,22.0,length(in.world_position.xyz-view.world_position));
    } else if settings.x < 1.5 {
        // A narrow transverse crop selects a few authored rivulets, avoiding
        // mip-averaging dozens of threads into one uniform blue paint strip.
        let sway=sin(uv.y*7.0-t*.30)*.004+sin(uv.y*15.0-t*.45)*.002;
        let tex=liquid_sample(vec2<f32>(uv.y*1.10-t*.20,uv.x*.24+.37+sway));
        let energy=dot(tex.rgb,vec3<f32>(.12,.52,.36));
        let ridge=max(energy-.030,0.0)+energy*.10;
        let crest=smoothstep(.025,.20,ridge);
        let opacity=.012+.62*crest;
        let hue=tex.rgb/max(max(tex.r,tex.g),max(tex.b,.001));
        rgb=tex.rgb*.35+hue*ridge*20.0/opacity;
        let edge=smoothstep(.015,.075,uv.x)*smoothstep(.015,.075,1.0-uv.x);
        let end=1.0-smoothstep(.68+tex.g*.12,.93+tex.g*.05,uv.y);
        // The cascade has no dark uniform fill: its native ripples split
        // alpha into moving rivulets while sparse dim water joins them.
        alpha*=edge*tex.a*smoothstep(0.0,.025,uv.y)*end
            *opacity;
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
