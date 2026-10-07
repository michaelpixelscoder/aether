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
    let coverage=(body_alpha+.62*strong)*tex.a;
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
