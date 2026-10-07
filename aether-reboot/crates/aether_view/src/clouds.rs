//! True 3D cumulus: periodic Perlin-Worley shape and erosion, self-shadowed with
//! bounded ray marching. Shape/light separation follows Guerrilla's Nubis talks.
use bevy::{
    mesh::MeshVertexBufferLayoutRef,
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::render_resource::*,
    shader::ShaderRef,
};
/// The authoring cache and the renderer use the same finite bank definitions.
#[derive(serde::Deserialize)]
pub struct CloudBank {
    pub key: String,
    pub center: Vec3,
    pub extent: Vec3,
    pub shape: Vec4,
    pub resolution: UVec3,
    pub padding: Vec3,
}
pub fn banks() -> &'static [CloudBank] {
    static BANKS: std::sync::LazyLock<Vec<CloudBank>> = std::sync::LazyLock::new(|| {
        serde_json::from_str(include_str!("../../../assets/atmosphere/cloud-banks.json"))
            .expect("validated cloud bank definitions")
    });
    &BANKS
}
pub fn shape_texture(assets: &AssetServer, bank: &CloudBank) -> Handle<Image> {
    use bevy::image::*;
    assets
        .load_builder()
        .with_settings(|settings: &mut ImageLoaderSettings| {
            settings.is_srgb = false;
            settings.asset_usage = bevy::asset::RenderAssetUsages::RENDER_WORLD;
            settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                address_mode_u: ImageAddressMode::ClampToEdge,
                address_mode_v: ImageAddressMode::ClampToEdge,
                address_mode_w: ImageAddressMode::ClampToEdge,
                mag_filter: ImageFilterMode::Linear,
                min_filter: ImageFilterMode::Linear,
                ..default()
            });
        })
        .load(format!("atmosphere/{}-shape.ktx2", bank.key))
}
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct CloudMaterial {
    #[uniform(0)]
    pub center: Vec4,
    #[uniform(1)]
    pub extent: Vec4,
    #[texture(2, dimension = "3d")]
    #[sampler(3)]
    pub density: Handle<Image>,
    /// Coverage threshold, extinction / metre, noise phase, regional-bank flag.
    #[uniform(4)]
    pub shape: Vec4,
    #[uniform(5)]
    pub sun: Vec4,
    #[texture(6, dimension = "3d")]
    #[sampler(7)]
    pub macro_shape: Handle<Image>,
    #[uniform(8)]
    pub cache_extent: Vec4,
}
impl Material for CloudMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/clouds.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Premultiplied
    }
    fn specialize(
        _: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _: &MeshVertexBufferLayoutRef,
        _: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = Some(Face::Front);
        if let Some(depth) = &mut descriptor.depth_stencil {
            depth.depth_write_enabled = Some(false);
            depth.depth_compare = Some(CompareFunction::Always);
        }
        Ok(())
    }
}
pub fn animate(
    view: Res<crate::weather::WeatherView>,
    mut materials: ResMut<Assets<CloudMaterial>>,
    suns: Query<&GlobalTransform, (With<DirectionalLight>, Without<crate::art::CloudBounce>)>,
) {
    let direction = suns
        .single()
        .map_or(Vec3::new(-0.5, 0.8, 0.3).normalize(), |sun| -*sun.forward());
    for (_, m) in materials.iter_mut() {
        if !view.reduced_motion {
            m.center.w = view.seconds;
        }
        m.extent.w = view.sample.daylight;
        m.sun = direction.extend(1.0);
    }
}

fn lattice_hash(p: IVec3, period: i32) -> u32 {
    let p = p.rem_euclid(IVec3::splat(period));
    let mut h = (p.x as u32).wrapping_mul(0x8da6_b343)
        ^ (p.y as u32).wrapping_mul(0xd816_3841)
        ^ (p.z as u32).wrapping_mul(0xcb1a_b31f);
    h = (h ^ (h >> 13)).wrapping_mul(0x85eb_ca6b);
    h ^ (h >> 16)
}
fn unit_hash(h: u32) -> f32 {
    (h & 0x00ff_ffff) as f32 / 16_777_215.0
}
fn perlin(p: Vec3, period: i32) -> f32 {
    let cell = p.floor().as_ivec3();
    let f = p - p.floor();
    let u = f * f * f * (f * (f * 6.0 - Vec3::splat(15.0)) + Vec3::splat(10.0));
    let gradients = [
        Vec3::new(1.0, 1.0, 0.0),
        Vec3::new(-1.0, 1.0, 0.0),
        Vec3::new(1.0, -1.0, 0.0),
        Vec3::new(-1.0, -1.0, 0.0),
        Vec3::new(1.0, 0.0, 1.0),
        Vec3::new(-1.0, 0.0, 1.0),
        Vec3::new(1.0, 0.0, -1.0),
        Vec3::new(-1.0, 0.0, -1.0),
        Vec3::new(0.0, 1.0, 1.0),
        Vec3::new(0.0, -1.0, 1.0),
        Vec3::new(0.0, 1.0, -1.0),
        Vec3::new(0.0, -1.0, -1.0),
    ];
    let mut result = 0.0;
    for z in 0..=1 {
        for y in 0..=1 {
            for x in 0..=1 {
                let o = IVec3::new(x, y, z);
                let g = gradients[lattice_hash(cell + o, period) as usize % 12];
                let w = Vec3::new(
                    if x == 0 { 1.0 - u.x } else { u.x },
                    if y == 0 { 1.0 - u.y } else { u.y },
                    if z == 0 { 1.0 - u.z } else { u.z },
                );
                result += g.dot(f - o.as_vec3()) * w.x * w.y * w.z;
            }
        }
    }
    (result * 0.5 + 0.5).clamp(0.0, 1.0)
}
fn worley(p: Vec3, period: i32) -> f32 {
    let cell = p.floor().as_ivec3();
    let mut nearest = 3.0_f32;
    for z in -1..=1 {
        for y in -1..=1 {
            for x in -1..=1 {
                let c = cell + IVec3::new(x, y, z);
                let h = lattice_hash(c, period);
                let feature = c.as_vec3()
                    + Vec3::new(
                        unit_hash(h),
                        unit_hash(h.rotate_left(11)),
                        unit_hash(h.rotate_left(21)),
                    );
                nearest = nearest.min(p.distance_squared(feature));
            }
        }
    }
    (1.0 - nearest.sqrt() * 0.92).clamp(0.0, 1.0)
}
/// Tileable 64³ RGBA: Perlin-Worley base, two erosion bands, broad weather.
/// Built once and linearly filtered in 3D; no dense shader noise loops.
pub fn density_texture() -> Image {
    use bevy::{asset::RenderAssetUsages, image::*};
    const SIZE: u32 = 64;
    let mut data = Vec::with_capacity((SIZE * SIZE * SIZE * 4) as usize);
    for z in 0..SIZE {
        for y in 0..SIZE {
            for x in 0..SIZE {
                let p = (Vec3::new(x as f32, y as f32, z as f32) + Vec3::splat(0.5)) / SIZE as f32;
                let gradient = perlin(p * 4.0, 4) * 0.625
                    + perlin(p * 8.0, 8) * 0.25
                    + perlin(p * 16.0, 16) * 0.125;
                let cells = worley(p * 4.0, 4);
                let base = cells + gradient * (1.0 - cells);
                for value in [
                    base,
                    worley(p * 8.0, 8),
                    worley(p * 16.0, 16),
                    perlin(p * 2.0, 2),
                ] {
                    data.push((value.clamp(0.0, 1.0) * 255.0).round() as u8);
                }
            }
        }
    }
    let mut image = Image::new(
        Extent3d {
            width: SIZE,
            height: SIZE,
            depth_or_array_layers: SIZE,
        },
        TextureDimension::D3,
        data,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        address_mode_w: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        ..default()
    });
    image
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn density_bands_are_periodic_and_vary_in_all_three_dimensions() {
        let p = Vec3::new(-0.37, 1.19, 2.41);
        for axis in [Vec3::X, Vec3::Y, Vec3::Z] {
            assert!((perlin(p, 4) - perlin(p + axis * 4.0, 4)).abs() < 0.00001);
            assert!((worley(p, 4) - worley(p + axis * 4.0, 4)).abs() < 0.00001);
            assert!((worley(p, 4) - worley(p + axis * 0.43, 4)).abs() > 0.001);
        }
    }
}
