// R62 isolated coherent current transport; authentic current alpha delta, unchanged native texture/geometry/clock.
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
// R62: true authored color samples are classified BEFORE either blend.
// All gradients arrive from the uniform caller; helpers contain no derivatives.
struct FlowOptics {
    body_mass:vec3<f32>,
    crest_mass:vec3<f32>,
    coverage:f32,
    source_alpha:f32,
}
fn blend_optics(a:FlowOptics,b:FlowOptics,weight:f32)->FlowOptics {
    return FlowOptics(mix(b.body_mass,a.body_mass,weight),
        mix(b.crest_mass,a.crest_mass,weight),
        mix(b.coverage,a.coverage,weight),mix(b.source_alpha,a.source_alpha,weight));
}
fn native_optics(tex:vec4<f32>,threshold:f32,grazing:f32)->FlowOptics {
    let energy=dot(tex.rgb,vec3<f32>(.12,.52,.36));
    let ridge=max(energy-.030,0.0)+energy*.10;
    let strong=smoothstep(threshold,threshold*3.5,ridge);
    let native_hue=tex.rgb/max(max(tex.r,tex.g),max(tex.b,.001));
    let cyan=mix(native_hue,vec3<f32>(.012,.48,1.0),.80);
    let violet_ink=smoothstep(.18,.32,tex.r/max(tex.g,.0005));
    let violet_weight=violet_ink*smoothstep(.035,.095,ridge)*.85;
    let hue=mix(cyan,vec3<f32>(.35,.025,1.0),violet_weight);
    // Weak authored water stays a deep blue with only1.8–3.3% coverage.
    // Strong native crests alone displace bright cloud backgrounds substantially.
    let body_alpha=.018+.010*grazing+.005*smoothstep(.025,.10,energy);
    // Coverage never inherits the relaxed distant-radiance threshold.
    // Otherwise low-pass mip averages turn most of the route into one sheet.
    let coverage_crest=smoothstep(.05,.18,ridge);
    let coverage=(body_alpha+.62*coverage_crest)*tex.a;
    let body_mass=(tex.rgb*.22+vec3<f32>(.002,.012,.04))*body_alpha*tex.a;
    let crest_mass=hue*ridge*(.006+.994*strong)*tex.a;
    return FlowOptics(body_mass,crest_mass,coverage,tex.a);
}
fn seam_optics(uv:vec2<f32>,grad_x:vec2<f32>,grad_y:vec2<f32>,threshold:f32,grazing:f32)->FlowOptics {
    let phase=fract(uv.x);
    // Only the native non-tileable boundary needs a distant seam substitute.
    // The middle87% of the tile is a single original network, not two overlays.
    let weight=smoothstep(.012,.065,phase)*smoothstep(.012,.065,1.0-phase);
    let a=textureSampleGrad(flow_texture,flow_sampler,vec2<f32>(phase,uv.y),grad_x,grad_y);
    let b=textureSampleGrad(flow_texture,flow_sampler,vec2<f32>(fract(phase+.5),uv.y),grad_x,grad_y);
    return blend_optics(native_optics(a,threshold,grazing),native_optics(b,threshold,grazing),weight);
}
fn advected_optics(uv:vec2<f32>,grad_x:vec2<f32>,grad_y:vec2<f32>,t:f32,threshold:f32,grazing:f32)->FlowOptics {
    // Existing native color supplies a low-frequency modulation carrier;
    // it is not claimed to be an authored flow vector or a normal/height map.
    let field_uv=vec2<f32>(.5+.35*sin(uv.x*.12+.21),.5+.35*sin(uv.y*.12+.17));
    let field=textureSampleLevel(flow_texture,flow_sampler,field_uv,6.0);
    let noise=clamp(field.b*4.0,0.0,1.0)*2.0-1.0;
    let phase_a=fract(t/6.0+noise*.19);
    let phase_b=fract(phase_a+.5);
    let triangle=1.0-abs(phase_a*2.0-1.0);
    let weight=triangle*triangle*(3.0-2.0*triangle);
    let flow=vec2<f32>(.035,clamp((field.r-field.g)*6.0,-1.0,1.0)*.003);
    // Color advection interval is centered on0, as in Valve's color variant.
    let a=seam_optics(uv-flow*(phase_a-.5),grad_x,grad_y,threshold,grazing);
    let b=seam_optics(uv-flow*(phase_b-.5),grad_x,grad_y,threshold,grazing);
    return blend_optics(a,b,weight);
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
        // Actual surface derivatives recover cross-width and a geometric normal.
        // Mesh normals are intentionally not assumed to encode the curved strip.
        let uv_dx=dpdx(uv);
        let uv_dy=dpdy(uv);
        let pos_dx=dpdx(in.world_position.xyz);
        let pos_dy=dpdy(in.world_position.xyz);
        let determinant=uv_dx.x*uv_dy.y-uv_dx.y*uv_dy.x;
        let denominator=select(-max(abs(determinant),1e-12),max(abs(determinant),1e-12),determinant>=0.0);
        let cross_metric=length((pos_dy*uv_dx.x-pos_dx*uv_dy.x)/denominator);
        let thermal=(1.0-smoothstep(18.0,26.0,cross_metric))*smoothstep(1e-12,4e-12,abs(determinant));
        let normal_cross=cross(pos_dx,pos_dy);
        let geometric_normal=normal_cross*inverseSqrt(max(dot(normal_cross,normal_cross),1e-14));
        let view_delta=view.world_position-in.world_position.xyz;
        let view_dir=view_delta*inverseSqrt(max(dot(view_delta,view_delta),1e-14));
        let facing=clamp(abs(dot(geometric_normal,view_dir)),0.0,1.0);
        let grazing=pow(1.0-facing,3.0);
        // Broad paths:~7 strong native crest groups per transverse section,
        // versus~20 in R61; the compact thermal keeps its R61 crop scale.
        let crop=mix(.16,.42,thermal);
        let crop_start=mix(.42,.29,thermal);
        let drift=sin(uv.x*.37-t*.12)*.006+sin(uv.x*.83+t*.09)*.003;
        let drift_du=cos(uv.x*.37-t*.12)*.00222+cos(uv.x*.83+t*.09)*.00249;
        let current_uv=vec2<f32>(uv.x*.20-t*.10,uv.y*crop+crop_start+drift);
        let grad_x=uv_dx*vec2<f32>(.20,crop)+vec2<f32>(0.0,drift_du*uv_dx.x);
        let grad_y=uv_dy*vec2<f32>(.20,crop)+vec2<f32>(0.0,drift_du*uv_dy.x);
        let tex_size=vec2<f32>(textureDimensions(flow_texture,0));
        let mip_footprint=log2(max(max(length(grad_x*tex_size),length(grad_y*tex_size)),1.0));
        let threshold=mix(.035,.020,smoothstep(2.0,6.0,mip_footprint));
        let optics=advected_optics(current_uv,grad_x,grad_y,t,threshold,grazing);
        let middle=1.0-smoothstep(.16,.43,abs(uv.y-.5));
        let optical_profile=.12+1.88*middle;
        let core=thermal*smoothstep(16.5,18.25,uv.x)*middle;
        let crest_peak=max(max(optics.crest_mass.r,optics.crest_mass.g),optics.crest_mass.b);
        let core_mass=mix(optics.crest_mass,vec3<f32>(.52,.92,1.0)*crest_peak,core*.85);
        let gain=14.0*optical_profile*(1.0+core*2.0);
        // Soft spectral-preserving rolloff bounds individual HDR peaks;
        // diffuse fields are not brightened to make the path readable.
        let bounded_gain=gain/(1.0+crest_peak*gain/6.0);
        let associated=optics.body_mass+core_mass*bounded_gain+vec3<f32>(.52,.92,1.0)*core*.14*optics.source_alpha;
        rgb=associated/max(optics.coverage,.00001);
        let edge=smoothstep(.015,.095,uv.y)*smoothstep(.015,.095,1.0-uv.y);
        alpha*=edge*optics.coverage;
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
        rgb=tex.rgb*.35+hue*ridge*12.0/opacity;
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
