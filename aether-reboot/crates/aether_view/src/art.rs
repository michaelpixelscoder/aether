//! Authored, shared art assets. Game geometry remains authoritative for interaction.
use aether_core::PartKind;
use bevy::{
    light::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
};
use bevy::{render::render_resource::AsBindGroup, shader::ShaderRef};
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct SkyMaterial {
    #[texture(0)]
    #[sampler(1)]
    texture: Handle<Image>,
    #[uniform(2)]
    settings: Vec4,
    #[uniform(3)]
    pub tint: Vec4,
}
impl Material for SkyMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/sky.wgsl".into()
    }
    // Draw the authored distant clouds behind the other transparent materials.
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
    fn depth_bias(&self) -> f32 {
        // The sphere follows the camera, so its AABB centre otherwise sorts
        // at zero and overwrites distant transparent rivers/clouds. Draw the
        // background first within the transparent phase; keep real depth
        // testing against opaque islands. The camera far plane is 18 km.
        -20_000.0
    }
}

pub const SCENES: [&str; 11] = [
    "helm",
    "sail",
    "tank",
    "lift",
    "harpoon",
    "island-0",
    "island-1",
    "island-2",
    "sanctuary",
    "anchor",
    "propeller",
];
#[derive(Resource)]
pub struct ArtAssets {
    pub scenes: Vec<Handle<bevy::world_serialization::WorldAsset>>,
    pub panorama: Handle<Image>,
    sky_mesh: Handle<Mesh>,
    sky_material: Handle<SkyMaterial>,
}
impl ArtAssets {
    pub fn part(&self, kind: PartKind) -> Handle<bevy::world_serialization::WorldAsset> {
        self.scenes[match kind {
            PartKind::Helm => 0,
            PartKind::Sail | PartKind::GrandSail => 1,
            PartKind::Tank => 2,
            PartKind::Lift => 3,
            PartKind::Harpoon => 4,
            PartKind::Propeller => 10,
        }]
        .clone()
    }
}
#[derive(Component)]
pub struct Panorama;
/// Broad illumination reflected by the upper cloud banks. This is an explicit
/// art approximation to volume bounce light, not a second stellar source.
#[derive(Component)]
pub struct CloudBounce(pub f32);
/// Weather controls the normalized strength; each radiance source declares its
/// unit conversion independently of Bevy's asynchronously created probe stages.
#[derive(Component, Clone, Copy)]
pub struct EnvironmentLighting {
    pub intensity: f32,
    pub radiance_scale: f32,
}
impl Default for EnvironmentLighting {
    fn default() -> Self {
        Self {
            intensity: 1.0,
            radiance_scale: 1.0,
        }
    }
}
pub const SKY_TEXTURE: &str = "textures/sky-world-linear.ktx2";
// Original scene-linear radiance, calibrated to the author's -0.9 EV. The
// KTX2 manifest records encoding_scale=0.25 to retain the sun in half-float;
// compensate that scale here. ACES is applied once by the camera.
const SKY_RADIANCE_GAIN: f32 = 2.143_546_9;
pub const IBL_TEXTURES: [&str; 2] = [
    "textures/ibl/sky-diffuse.ktx2",
    "textures/ibl/sky-specular.ktx2",
];
pub const IBL_RADIANCE_SCALE: f32 = 3000.0;
pub const CLOUD_BOUNCE_STRENGTH: f32 = 0.35;
pub const SUN_DIRECTION: Vec3 = Vec3::new(-0.853_562_65, -0.050_973_576, -0.518_490_6);

pub fn sun_transform() -> Transform {
    Transform::from_translation(SUN_DIRECTION).looking_at(Vec3::ZERO, Vec3::Y)
}
pub fn cloud_bounce_transform() -> Transform {
    let azimuth = SUN_DIRECTION.x.atan2(SUN_DIRECTION.z);
    let elevation = 42.0_f32.to_radians();
    Transform::from_translation(Vec3::new(
        azimuth.sin() * elevation.cos(),
        elevation.sin(),
        azimuth.cos() * elevation.cos(),
    ))
    .looking_at(Vec3::ZERO, Vec3::Y)
}
pub fn spawn_sun(commands: &mut Commands) {
    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: true,
            ..default()
        },
        bevy::light::VolumetricLight,
        sun_transform(),
        sunlight_shadows(),
    ));
    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: false,
            ..default()
        },
        CloudBounce(CLOUD_BOUNCE_STRENGTH),
        cloud_bounce_transform(),
    ));
}

pub fn environment_texture(assets: &AssetServer, path: &'static str) -> Handle<Image> {
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
                mipmap_filter: ImageFilterMode::Linear,
                ..default()
            });
        })
        .load(path)
}

/// Shared world-scale shadows retain the nearby deck cascade while covering
/// the main islands. Beyond this range, distance haze softens distant detail.
pub fn sunlight_shadows() -> bevy::light::CascadeShadowConfig {
    bevy::light::CascadeShadowConfigBuilder {
        first_cascade_far_bound: 20.0,
        maximum_distance: 2400.0,
        ..default()
    }
    .build()
}
pub fn panorama_texture(assets: &AssetServer) -> Handle<Image> {
    use bevy::image::*;
    assets
        .load_builder()
        .with_settings(|settings: &mut ImageLoaderSettings| {
            // Immutable HDR samples need no CPU copy after GPU upload.
            settings.is_srgb = false;
            settings.asset_usage = bevy::asset::RenderAssetUsages::RENDER_WORLD;
            settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                address_mode_u: ImageAddressMode::Repeat,
                address_mode_v: ImageAddressMode::ClampToEdge,
                mag_filter: ImageFilterMode::Linear,
                min_filter: ImageFilterMode::Linear,
                ..default()
            });
        })
        .load(SKY_TEXTURE)
}
pub fn camera_effects(assets: &AssetServer) -> impl Bundle + use<> {
    (
        bevy::camera::Hdr,
        bevy::core_pipeline::prepass::DepthPrepass,
        bevy::camera::Exposure { ev100: 11.6 },
        bevy::core_pipeline::tonemapping::Tonemapping::AcesFitted,
        bevy::light::EnvironmentMapLight {
            diffuse_map: environment_texture(assets, IBL_TEXTURES[0]),
            specular_map: environment_texture(assets, IBL_TEXTURES[1]),
            ..default()
        },
        EnvironmentLighting {
            radiance_scale: IBL_RADIANCE_SCALE,
            ..default()
        },
        bevy::post_process::bloom::Bloom::NATURAL,
        Msaa::Off,
        bevy::pbr::ScreenSpaceAmbientOcclusion {
            quality_level: bevy::pbr::ScreenSpaceAmbientOcclusionQualityLevel::Medium,
            ..default()
        },
        bevy::anti_alias::taa::TemporalAntiAliasing::default(),
    )
}
pub fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SkyMaterial>>,
) {
    // The planetless panorama and offline lighting maps share one authored
    // source. DistanceFog and local volumes provide the near aerial perspective.
    let panorama = panorama_texture(&assets);
    let sky_material = materials.add(SkyMaterial {
        texture: panorama.clone(),
        settings: Vec4::new(SKY_RADIANCE_GAIN, 1.0, 0.75, 0.0),
        tint: Vec4::ONE,
    });
    commands.insert_resource(ArtAssets {
        scenes: SCENES
            .iter()
            .map(|name| assets.load(GltfAssetLabel::Scene(0).from_asset(format!("art/{name}.glb"))))
            .collect(),
        panorama,
        sky_material,
        sky_mesh: meshes.add(
            Sphere::new(14000.0)
                .mesh()
                .uv(96, 48)
                .with_inverted_winding()
                .expect("sphere triangles"),
        ),
    });
}
pub fn sky(commands: &mut Commands, art: &ArtAssets) {
    commands.spawn((
        super::environment::EnvironmentVisual,
        Panorama,
        Mesh3d(art.sky_mesh.clone()),
        MeshMaterial3d(art.sky_material.clone()),
        NotShadowCaster,
        NotShadowReceiver,
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
}
pub fn follow_camera(
    camera: Query<&Transform, (With<Camera3d>, Without<Panorama>)>,
    mut sky: Query<&mut Transform, With<Panorama>>,
) {
    if let Ok(camera) = camera.single() {
        for mut t in &mut sky {
            t.translation = camera.translation;
        }
    }
}

/// Bevy 0.19.1 copies atmosphere intensity only when creating each probe stage.
/// Weather must reach both the generated source and its filtered lighting maps,
/// including probes whose textures become ready several frames after the camera.
pub fn sync_environment_lighting(
    mut probes: Query<(
        &EnvironmentLighting,
        Option<&mut bevy::light::AtmosphereEnvironmentMapLight>,
        Option<&mut bevy::light::GeneratedEnvironmentMapLight>,
        Option<&mut bevy::light::EnvironmentMapLight>,
    )>,
) {
    for (source, atmosphere, generated, filtered) in &mut probes {
        let intensity = source.intensity * source.radiance_scale;
        if let Some(mut atmosphere) = atmosphere {
            atmosphere.intensity = intensity;
        }
        if let Some(mut generated) = generated {
            generated.intensity = intensity;
        }
        if let Some(mut filtered) = filtered {
            filtered.intensity = intensity;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::light::{
        AtmosphereEnvironmentMapLight, EnvironmentMapLight, GeneratedEnvironmentMapLight,
    };

    #[test]
    fn environment_lighting_follows_weather_after_delayed_probe_creation() {
        let mut app = App::new();
        app.add_systems(Update, sync_environment_lighting);
        let entity = app
            .world_mut()
            .spawn((
                EnvironmentLighting {
                    intensity: 1.5,
                    ..default()
                },
                AtmosphereEnvironmentMapLight::default(),
            ))
            .id();
        app.update(); // No GPU-generated components yet.
        app.world_mut()
            .entity_mut(entity)
            .insert(GeneratedEnvironmentMapLight::default());
        app.update();
        assert_eq!(
            app.world()
                .get::<GeneratedEnvironmentMapLight>(entity)
                .unwrap()
                .intensity,
            1.5
        );
        app.world_mut()
            .entity_mut(entity)
            .insert(EnvironmentMapLight::default());
        for intensity in [0.36, 0.0, 1.5] {
            // Cave, darkness, return to daylight.
            app.world_mut()
                .get_mut::<EnvironmentLighting>(entity)
                .unwrap()
                .intensity = intensity;
            app.update();
            assert_eq!(
                app.world()
                    .get::<GeneratedEnvironmentMapLight>(entity)
                    .unwrap()
                    .intensity,
                intensity
            );
            assert_eq!(
                app.world()
                    .get::<EnvironmentMapLight>(entity)
                    .unwrap()
                    .intensity,
                intensity
            );
        }
    }

    #[test]
    fn authored_lighting_retains_maps_and_scales_weather_without_atmosphere() {
        let mut app = App::new();
        app.add_systems(Update, sync_environment_lighting);
        let mut images = Assets::<Image>::default();
        let diffuse = images.add(Image::default());
        let specular = images.add(Image::default());
        let entity = app
            .world_mut()
            .spawn((
                EnvironmentLighting {
                    intensity: 0.36,
                    radiance_scale: 1000.0,
                },
                EnvironmentMapLight {
                    diffuse_map: diffuse.clone(),
                    specular_map: specular.clone(),
                    affects_lightmapped_mesh_diffuse: false,
                    ..default()
                },
            ))
            .id();
        app.update();
        let result = app.world().get::<EnvironmentMapLight>(entity).unwrap();
        assert_eq!(result.intensity, 360.0);
        assert_eq!(result.diffuse_map, diffuse);
        assert_eq!(result.specular_map, specular);
        assert!(!result.affects_lightmapped_mesh_diffuse);
    }
}
