use aether_core::Block;
use bevy::image::{
    ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor,
};
use bevy::prelude::*;

/// All consumers use identical settings: AssetServer caches the first load.
pub fn wood_texture(assets: &AssetServer) -> Handle<Image> {
    surface_texture(assets, "textures/cedar.png", true)
}
pub fn surface_texture(assets: &AssetServer, path: &'static str, srgb: bool) -> Handle<Image> {
    assets
        .load_builder()
        .with_settings(move |s: &mut ImageLoaderSettings| {
            s.is_srgb = srgb;
            s.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                address_mode_u: ImageAddressMode::Repeat,
                address_mode_v: ImageAddressMode::Repeat,
                mag_filter: ImageFilterMode::Linear,
                min_filter: ImageFilterMode::Linear,
                mipmap_filter: ImageFilterMode::Linear,
                anisotropy_clamp: 8,
                ..default()
            });
        })
        .load(path)
}

#[derive(Resource, Default)]
pub struct Palette {
    pub wood: Handle<StandardMaterial>,
    pub metal: Handle<StandardMaterial>,
    pub glass: Handle<StandardMaterial>,
    pub sail: Handle<StandardMaterial>,
    pub violet: Handle<StandardMaterial>,
    pub gold: Handle<StandardMaterial>,
    pub rope: Handle<StandardMaterial>,
    pub lantern: Handle<StandardMaterial>,
    pub rock: Handle<StandardMaterial>,
    pub grass: Handle<StandardMaterial>,
    pub trunk: Handle<StandardMaterial>,
    pub foliage: Handle<StandardMaterial>,
    pub cloud: Handle<StandardMaterial>,
    pub current: Handle<StandardMaterial>,
    pub flow_ribbons: [Handle<Mesh>; 2],
    pub cube: Handle<Mesh>,
    pub sphere: Handle<Mesh>,
}
impl Palette {
    pub fn block(&self, block: Block) -> Handle<StandardMaterial> {
        match block {
            Block::Metal => self.metal.clone(),
            Block::Glass => self.glass.clone(),
            _ => self.wood.clone(),
        }
    }
}
pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Res<AssetServer>,
) {
    let wood = materials.add(StandardMaterial {
        base_color: Color::srgb(0.78, 0.72, 0.64),
        base_color_texture: Some(wood_texture(&assets)),
        normal_map_texture: Some(surface_texture(&assets, "textures/cedar-normal.png", false)),
        uv_transform: bevy::math::Affine2::from_scale(Vec2::new(0.4, 0.5)),
        perceptual_roughness: 0.82,
        ..default()
    });
    let metal = materials.add(StandardMaterial {
        base_color: Color::srgb(0.16, 0.22, 0.27),
        metallic: 0.65,
        perceptual_roughness: 0.45,
        ..default()
    });
    let glass = materials.add(StandardMaterial {
        base_color: Color::srgba(0.18, 0.26, 0.29, 0.48),
        emissive: LinearRgba::BLACK,
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.12,
        cull_mode: None,
        ..default()
    });
    let sail = materials.add(StandardMaterial {
        base_color: Color::srgb(0.94, 0.83, 0.62),
        perceptual_roughness: 0.96,
        cull_mode: None,
        ..default()
    });
    let violet = materials.add(StandardMaterial {
        base_color: Color::srgb(0.56, 0.27, 0.97),
        emissive: LinearRgba::new(1.5, 0.3, 4.0, 1.0),
        ..default()
    });
    let gold = materials.add(StandardMaterial {
        base_color: Color::srgb(0.70, 0.44, 0.12),
        metallic: 0.8,
        perceptual_roughness: 0.36,
        ..default()
    });
    let rock = materials.add(StandardMaterial {
        base_color: Color::srgb(0.22, 0.27, 0.34),
        perceptual_roughness: 0.98,
        ..default()
    });
    let grass = materials.add(StandardMaterial {
        base_color: Color::srgb(0.26, 0.48, 0.37),
        perceptual_roughness: 0.94,
        ..default()
    });
    let trunk = materials.add(Color::srgb(0.32, 0.23, 0.19));
    let foliage = materials.add(Color::srgb(0.16, 0.37, 0.31));
    let cloud = materials.add(StandardMaterial {
        base_color: Color::srgb(0.35, 0.43, 0.55),
        perceptual_roughness: 1.0,
        ..default()
    });
    let current = materials.add(StandardMaterial {
        base_color: Color::srgba(0.24, 0.50, 1.0, 0.65),
        emissive: LinearRgba::new(0.35, 1.6, 7.5, 1.0),
        alpha_mode: AlphaMode::Blend,
        emissive_exposure_weight: 0.0,
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        cull_mode: None,
        ..default()
    });
    commands.insert_resource(Palette {
        wood,
        metal,
        glass,
        sail,
        violet,
        gold,
        rope: materials.add(StandardMaterial {
            base_color: Color::srgb(0.38, 0.26, 0.13),
            perceptual_roughness: 0.95,
            ..default()
        }),
        lantern: materials.add(StandardMaterial {
            base_color: Color::srgb(1., 0.67, 0.2),
            emissive: LinearRgba::new(4., 1.8, 0.3, 1.),
            ..default()
        }),
        rock,
        grass,
        trunk,
        foliage,
        cloud,
        current,
        flow_ribbons: [
            meshes.add(crate::environment::ribbon(
                &aether_core::fields::archipelago_current(),
            )),
            meshes.add(crate::environment::ribbon(
                &aether_core::fields::refuge_current(),
            )),
        ],
        cube: meshes.add(Cuboid::default()),
        sphere: meshes.add(Sphere::new(1.0).mesh().ico(1).expect("icosphere")),
    });
}
