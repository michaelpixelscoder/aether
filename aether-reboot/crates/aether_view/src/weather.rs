//! Presentation of weather; all samples and the clock arrive from the game.
use crate::environment::EnvironmentVisual;
use bevy::prelude::*;
#[derive(Resource)]
pub struct WeatherView {
    pub seconds: f32,
    pub sample: aether_core::weather::Weather,
    pub reduced_motion: bool,
}
impl Default for WeatherView {
    fn default() -> Self {
        Self {
            seconds: 0.0,
            sample: aether_core::weather::sample(Vec3::ZERO, 0.0),
            reduced_motion: false,
        }
    }
}
#[derive(Component)]
pub struct WeatherParticle {
    pub index: u32,
    pub kind: Precipitation,
}
#[derive(Clone, Copy)]
pub enum Precipitation {
    Rain,
    Snow,
    Ash,
}
#[derive(Component)]
pub struct LocalMist;
#[derive(Resource)]
pub struct WeatherAssets {
    rain: Handle<StandardMaterial>,
    snow: Handle<StandardMaterial>,
    ash: Handle<StandardMaterial>,
    cube: Handle<Mesh>,
    density: Handle<Image>,
    cloud_density: Handle<Image>,
    cloud_shapes: Vec<Handle<Image>>,
}
pub fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    assets: Res<AssetServer>,
) {
    use bevy::{
        asset::RenderAssetUsages,
        image::*,
        render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    };
    let mut data = Vec::with_capacity(32 * 32 * 32);
    for z in 0..32 {
        for y in 0..32 {
            for x in 0..32 {
                let p = Vec3::new(x as f32, y as f32, z as f32) * std::f32::consts::TAU / 32.0;
                let broad =
                    (p.x.sin() + (p.z + p.y.sin()).sin() + (p.y * 2.0 + p.x.cos()).sin()) / 3.0;
                let detail = (p.x * 3.0 + p.z * 2.0).sin() * (p.z * 3.0 - p.y * 2.0).cos() * 0.15;
                data.push(((broad + detail + 0.55).clamp(0.0, 1.0) * 255.0) as u8);
            }
        }
    }
    let mut density = Image::new(
        Extent3d {
            width: 32,
            height: 32,
            depth_or_array_layers: 32,
        },
        TextureDimension::D3,
        data,
        TextureFormat::R8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    density.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        address_mode_w: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        ..default()
    });
    let cloud = crate::clouds::density_texture();
    commands.insert_resource(WeatherAssets {
        cloud_shapes: crate::clouds::banks()
            .iter()
            .map(|bank| crate::clouds::shape_texture(&assets, bank))
            .collect(),
        cloud_density: images.add(cloud),
        rain: materials.add(StandardMaterial {
            base_color: Color::srgba(0.55, 0.76, 0.94, 0.42),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        }),
        snow: materials.add(StandardMaterial {
            base_color: Color::srgba(0.90, 0.95, 1.0, 0.85),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        }),
        ash: materials.add(StandardMaterial {
            base_color: Color::srgba(0.30, 0.19, 0.13, 0.72),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        }),
        cube: meshes.add(Cuboid::from_length(1.0)),
        density: images.add(density),
    });
}
pub fn spawn(world: &mut World) {
    world.resource_scope(|world, assets: Mut<WeatherAssets>| {
        for (index, bank) in crate::clouds::banks().iter().enumerate() {
            let (center, extent, shape) = (bank.center, bank.extent, bank.shape);
            let material = world
                .resource_mut::<Assets<crate::clouds::CloudMaterial>>()
                .add(crate::clouds::CloudMaterial {
                    center: center.extend(0.0),
                    extent: extent.extend(1.0),
                    density: assets.cloud_density.clone(),
                    shape,
                    sun: Vec3::new(-0.5, 0.8, 0.3).normalize().extend(1.0),
                    macro_shape: assets.cloud_shapes[index].clone(),
                    cache_extent: (extent + bank.padding * 2.0).extend(0.0),
                });
            world.spawn((
                EnvironmentVisual,
                Mesh3d(assets.cube.clone()),
                MeshMaterial3d(material),
                bevy::light::NotShadowCaster,
                bevy::light::NotShadowReceiver,
                Transform::from_translation(center).with_scale(extent),
            ));
        }
        for i in 0..176 {
            let (kind, index, material) = if i < 80 {
                (Precipitation::Rain, i, assets.rain.clone())
            } else if i < 128 {
                (Precipitation::Snow, i - 80, assets.snow.clone())
            } else {
                (Precipitation::Ash, i - 128, assets.ash.clone())
            };
            world.spawn((
                EnvironmentVisual,
                WeatherParticle { index, kind },
                Mesh3d(assets.cube.clone()),
                MeshMaterial3d(material),
                Transform::default(),
                Visibility::Hidden,
                bevy::light::NotShadowCaster,
            ));
        }
        world.spawn((
            EnvironmentVisual,
            LocalMist,
            bevy::light::FogVolume {
                density_factor: 0.0008,
                density_texture: Some(assets.density.clone()),
                fog_color: Color::srgb(0.75, 0.84, 0.94),
                scattering_asymmetry: 0.6,
                ..default()
            },
            Transform::from_scale(Vec3::new(500.0, 280.0, 500.0)),
        ));
    });
}
pub fn update(
    view: Res<WeatherView>,
    camera: Query<&Transform, (With<Camera3d>, Without<WeatherParticle>, Without<LocalMist>)>,
    mut particles: Query<(&WeatherParticle, &mut Transform, &mut Visibility), Without<LocalMist>>,
    mut mist: Query<
        (&mut Transform, &mut bevy::light::FogVolume),
        (With<LocalMist>, Without<WeatherParticle>),
    >,
    mut sun: Query<(&mut DirectionalLight, Option<&crate::art::CloudBounce>)>,
    mut ambient: ResMut<bevy::light::GlobalAmbientLight>,
    mut cameras: Query<
        (
            &mut bevy::camera::Exposure,
            &mut crate::art::EnvironmentLighting,
            &mut DistanceFog,
        ),
        With<Camera3d>,
    >,
    mut skies: ResMut<Assets<crate::art::SkyMaterial>>,
    mut volumetrics: Query<&mut bevy::light::VolumetricFog>,
) {
    let Ok(camera) = camera.single() else {
        return;
    };
    let s = view.sample;
    let cave = s.underground.powi(3);
    let cave_color = if aether_core::world::biome_at(camera.translation)
        == aether_core::world::Biome::Underforge
    {
        Vec3::new(0.18, 0.12, 0.10)
    } else {
        Vec3::new(0.12, 0.22, 0.18)
    };
    for mut fog in &mut volumetrics {
        fog.ambient_color = Color::srgb(0.75, 0.84, 1.0);
        fog.ambient_intensity = 0.1;
    }
    for (particle, mut t, mut visibility) in &mut particles {
        let (amount, count, speed, size) = match particle.kind {
            Precipitation::Rain => (s.storm, 80.0, 23.0, Vec3::new(0.012, 0.6, 0.012)),
            Precipitation::Snow => (s.snow, 48.0, 1.8, Vec3::splat(0.045)),
            Precipitation::Ash => (s.ash, 48.0, 0.65, Vec3::new(0.035, 0.01, 0.035)),
        };
        *visibility =
            if !view.reduced_motion && amount > 0.12 && particle.index as f32 / count < amount {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        let i = particle.index as f32;
        let x = (i * 13.71).rem_euclid(60.0) - 30.0;
        let z = (i * 23.17).rem_euclid(60.0) - 30.0;
        let y = (i * 3.71 - view.seconds * speed).rem_euclid(35.0) - 10.0;
        t.translation =
            camera.translation + Vec3::new(x + (view.seconds * 0.7 + i).sin() * 1.5, y, z);
        t.scale = size;
        t.rotation = Quat::from_rotation_z(if matches!(particle.kind, Precipitation::Rain) {
            -0.14
        } else {
            view.seconds * 0.5 + i
        });
    }
    for (mut t, mut fog) in &mut mist {
        t.translation = camera.translation;
        // Clear high-altitude air needs no local scattering veil. DistanceFog
        // already separates distant islands; volume is reserved for weather.
        fog.density_factor = s.storm * 0.001 + s.snow * 0.0004 + s.ash * 0.0003 + cave * 0.00002;
        fog.density_texture_offset = Vec3::new(view.seconds * 0.002, 0.0, view.seconds * 0.001);
        fog.fog_color = if s.underground > 0.35 {
            Color::srgb(0.40, 0.50, 0.58)
        } else {
            Color::srgb(0.75, 0.84, 0.94)
        };
    }
    for (mut light, bounce) in &mut sun {
        light.illuminance = (106_600.0 - 22_100.0 * cave)
            * (1.25 - cave * 0.25)
            * s.daylight
            * (1.0 - s.storm * 0.68)
            * (1.0 - cave * 0.998)
            * bounce.map_or(1.0, |bounce| bounce.0);
        // Same linear solar chromaticity as the authored sky. Treating these
        // radiance coefficients as display sRGB made the stone overly amber.
        light.color = Color::linear_rgb(1.0, 0.62 + s.snow * 0.36, 0.32 + s.snow * 0.66);
    }
    // Subterranean diffuse light stands in for the repeated lantern/crystal
    // bounces that a direct-light renderer cannot compute. Daylight exposure
    // made the previous cave paths almost black despite their point lights.
    ambient.color = Color::srgb(0.70 + cave * 0.30, 0.82 + cave * 0.18, 1.0);
    ambient.brightness = (128.0 - cave * 8.4) * (0.5 + s.daylight * 0.5);
    for (mut exposure, mut environment, mut fog) in &mut cameras {
        exposure.ev100 = 11.3 - cave * 4.05;
        // R57's incident sky carries the warm solar field into the IBL.
        // Balance its diffuse fill against the directional key; preserve the
        // underground endpoint instead of darkening cavern navigation.
        environment.intensity =
            (1.5 - cave * 1.14) * (0.16 + s.daylight * 0.84) * (0.5 + cave * 0.5);
        let daylight = 0.18 + s.daylight * 0.82;
        let color = (Vec3::new(0.42, 0.61, 0.80) * daylight).lerp(cave_color, cave);
        fog.color = Color::srgb(color.x, color.y, color.z);
        fog.falloff = FogFalloff::Linear {
            start: (550.0 - cave * 470.0) / 0.75,
            end: (7200.0 - cave * 5300.0) / 0.75,
        };
    }
    let tint = Vec4::new(
        (1.0 - cave * 0.975) * (0.10 + s.daylight * 0.90) * (1.0 - s.storm * 0.48),
        (1.0 - cave * 0.97) * (0.16 + s.daylight * 0.84) * (1.0 - s.storm * 0.45),
        (1.0 - cave * 0.945) * (0.28 + s.daylight * 0.72) * (1.0 - s.storm * 0.38),
        1.0,
    );
    for (_, sky) in skies.iter_mut() {
        if sky.tint.distance_squared(tint) > 0.000001 {
            sky.tint = tint;
        }
    }
}
