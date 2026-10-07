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
