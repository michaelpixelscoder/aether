#import bevy_pbr::forward_io::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var sky_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var sky_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var<uniform> settings: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var<uniform> tint: vec4<f32>;

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    // The original Blender panorama covers the full sphere. Its Z-up camera
    // and Bevy's rotated UV sphere share this longitude convention; the baked
    // sun consequently agrees with the scene light. Repeat sampling closes
    // the true seam, without mirroring or cropping distant cloud silhouettes.
    let uv = vec2<f32>(settings.z-input.uv.x, input.uv.y*settings.y+settings.w);
    // Linear HDR source: no baked AgX curve or second sRGB decoding.
    // The shared camera applies ACES after this material joins the HDR scene.
    let sky = textureSample(sky_texture,sky_sampler,uv).rgb;
    return vec4<f32>(sky*settings.x*tint.rgb,1.0);
}
