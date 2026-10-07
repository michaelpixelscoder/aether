//! Reproducible art fixture using the production renderer, explicitly no gameplay.
use bevy::{asset::AssetPlugin, light::GlobalAmbientLight, prelude::*};
#[derive(Resource)]
struct Capture {
    path: String,
    elapsed: f32,
    captured: bool,
}
#[derive(Component)]
struct FixtureVelocity(Vec3);
/// Isolated authored-kit inspection; this never installs a gameplay circuit.
#[derive(Resource)]
struct IndustrialReview {
    scenes: Vec<Handle<WorldAsset>>,
    lod: bool,
}

fn industrial_fixture(world: &mut World, origin: Vec3, lod: bool) -> (Vec3, Vec3) {
    let quality = if lod { "lod" } else { "hd" };
    let mut scenes = Vec::new();
    for (index, model) in [
        "straight",
        "coupling",
        "elbow",
        "tee",
        "valve-open",
        "valve-closed",
    ]
    .into_iter()
    .enumerate()
    {
        let scene = world.resource::<AssetServer>().load(
            GltfAssetLabel::Scene(0).from_asset(format!("aether-industrial/{model}-{quality}.glb")),
        );
        let position = Vec3::new(
            (index % 2) as f32 * 1.2 - 0.6,
            0.0,
            (index / 2) as f32 * 0.6 - 0.6,
        );
        world.spawn((
            Name::new(format!("Industrial review / {model}")),
            WorldAssetRoot(scene.clone()),
            Transform::from_translation(origin + position),
        ));
        scenes.push(scene);
    }
    world.insert_resource(IndustrialReview { scenes, lod });
    // All modules remain at their original metre scale. Production weather,
    // sunlight, HDR and IBL are shared with the archipelago fixture.
    let focus = origin + Vec3::Y * 0.08;
    (focus + Vec3::new(1.7, 1.8, 2.7), focus)
}
/// Inspection-only grading. Defaults retain production weather verbatim.
#[derive(Resource)]
struct ReviewLighting {
    sun: f32,
    ambient: f32,
    environment: f32,
    exposure: f32,
    solar_color: Option<Vec3>,
    distance_fog: f32,
}
fn review_lighting(
    review: Res<ReviewLighting>,
    mut suns: Query<&mut DirectionalLight>,
    mut ambient: ResMut<GlobalAmbientLight>,
    mut cameras: Query<(
        &mut bevy::camera::Exposure,
        &mut aether_view::art::EnvironmentLighting,
        &mut DistanceFog,
    )>,
) {
    // weather::update restores the production values in Update each frame.
    for mut sun in &mut suns {
        sun.illuminance *= review.sun;
        if let Some(color) = review.solar_color {
            // The Blender sky stores this as linear RGB, not display sRGB.
            sun.color = Color::linear_rgb(color.x, color.y, color.z);
        }
    }
    ambient.brightness *= review.ambient;
    for (mut exposure, mut environment, mut fog) in &mut cameras {
        exposure.ev100 += review.exposure;
        environment.intensity *= review.environment;
        if let FogFalloff::Linear { start, end } = fog.falloff {
            let scale = review.distance_fog.max(0.000_001);
            fog.falloff = FogFalloff::Linear {
                start: start / scale,
                end: end / scale,
            };
        }
    }
}
fn main() -> AppExit {
    App::new()
        .insert_resource(GlobalAmbientLight {
            brightness: 65.0,
            ..default()
        })
        .add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: aether_view::asset_root(),
                    meta_check: bevy::asset::AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Aether Isles — Revue des archipels".into(),
                        resolution: bevy::window::WindowResolution::new(1672, 941)
                            .with_scale_factor_override(1.0),
                        ..default()
                    }),
                    ..default()
                }),
            aether_view::PresentationPlugin,
        ))
        .add_systems(
            Startup,
            setup
                .after(aether_view::world::setup)
                .after(aether_view::art::setup)
                .after(aether_view::traffic::setup)
                .after(aether_view::fauna::setup)
                .after(aether_view::weather::setup),
        )
        .add_systems(Update, capture)
        .add_systems(PostUpdate, fixture_sails.before(aether_view::sail::animate))
        .add_systems(
            PostUpdate,
            review_lighting.before(aether_view::art::sync_environment_lighting),
        )
        .run()
}
fn setup(world: &mut World) {
    let args: Vec<_> = std::env::args().collect();
    let parameter = |name: &str, fallback: f32, min: f32, max: f32| {
        args.windows(2)
            .find(|a| a[0] == name)
            .map_or(fallback, |a| {
                let value = a[1]
                    .parse::<f32>()
                    .expect("finite lighting review parameter");
                assert!(value.is_finite() && (min..=max).contains(&value));
                value
            })
    };
    world.insert_resource(ReviewLighting {
        sun: parameter("--sun-scale", 1.0, 0.0, 10.0),
        ambient: parameter("--ambient-scale", 1.0, 0.0, 10.0),
        environment: parameter("--environment-scale", 1.0, 0.0, 10.0),
        exposure: parameter("--exposure-offset", 0.0, -6.0, 6.0),
        solar_color: args
            .iter()
            .any(|a| {
                matches!(
                    a.as_str(),
                    "--authored-solar-color" | "--solar-green" | "--solar-blue"
                )
            })
            .then(|| {
                Vec3::new(
                    1.0,
                    parameter("--solar-green", 0.62, 0.0, 1.0),
                    parameter("--solar-blue", 0.32, 0.0, 1.0),
                )
            }),
        distance_fog: parameter("--distance-fog-scale", 1.0, 0.0, 2.0),
    });
    let biome = args
        .windows(2)
        .find(|a| a[0] == "--world")
        .map_or("dawn", |a| a[1].as_str());
    assert!(
        !(args.iter().any(|a| a == "--flow-texture")
            && args.iter().any(|a| a == "--current-texture")),
        "choose one texture diagnostic"
    );
    if let Some(texture) = args
        .windows(2)
        .find(|a| a[0] == "--flow-texture" || a[0] == "--current-texture")
    {
        let current_only = texture[0] == "--current-texture";
        use bevy::image::*;
        let handle = world
            .resource::<AssetServer>()
            .load_builder()
            .with_settings(|settings: &mut ImageLoaderSettings| {
                settings.is_srgb = true;
                settings.asset_usage = bevy::asset::RenderAssetUsages::RENDER_WORLD;
                settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                    address_mode_u: ImageAddressMode::Repeat,
                    address_mode_v: ImageAddressMode::Repeat,
                    mag_filter: ImageFilterMode::Linear,
                    min_filter: ImageFilterMode::Linear,
                    mipmap_filter: ImageFilterMode::Linear,
                    ..default()
                });
            })
            .load(texture[1].clone());
        for (_, material) in world
            .resource_mut::<Assets<aether_view::world::FlowMaterial>>()
            .iter_mut()
        {
            if !current_only || material.settings.x < 0.5 {
                material.flow_texture = Some(handle.clone());
            }
        }
    }
    let selected = args
        .windows(2)
        .find(|a| a[0] == "--island")
        .map(|a| a[1].parse::<u32>().expect("island id"));
    let island = aether_core::world::islands()
        .iter()
        .find(|i| selected.map_or(i.capital && i.biome.asset() == biome, |id| i.id == id))
        .expect("known biome");
    let cave = matches!(
        island.biome,
        aether_core::world::Biome::Hollow | aether_core::world::Biome::Underforge
    );
    let interior = cave && args.iter().any(|a| a == "--interior");
    let eye = if interior {
        island.transform_point(Vec3::new(7.0, 16.0, 48.0))
    } else if cave {
        island.center + Vec3::new(320.0, 160.0, 440.0)
    } else {
        island.center + Vec3::new(460.0, 265.0, 670.0)
    };
    let focus = if interior {
        island.transform_point(Vec3::new(-5.0, 18.0, -36.0))
    } else if cave {
        island.center + Vec3::Y * 15.0
    } else {
        island.center - Vec3::Y * 70.0
    };
    let focus = args
        .windows(2)
        .find(|a| a[0] == "--focus-height")
        .map_or(focus, |a| {
            let height = a[1].parse::<f32>().expect("finite review focus height");
            assert!(height.is_finite());
            island.center + Vec3::Y * height
        });
    let eye = args
        .windows(2)
        .find(|a| a[0] == "--distance")
        .map_or(eye, |a| {
            let distance = a[1].parse::<f32>().expect("positive review distance");
            assert!(distance.is_finite() && distance > 1.0);
            focus + (eye - focus).normalize() * distance
        });
    let industrial_review = args.iter().any(|a| a == "--industrial-review");
    let (eye, focus) = if industrial_review {
        industrial_fixture(
            world,
            island.center + Vec3::Y * 160.0,
            args.iter().any(|a| a == "--industrial-lod"),
        )
    } else {
        (eye, focus)
    };
    let camera = Transform::from_translation(eye).looking_at(focus, Vec3::Y);
    let fog = bevy::light::VolumetricFog {
        step_count: 24,
        jitter: 0.35,
        ..default()
    };
    let camera_entity = world
        .spawn((
            Camera3d::default(),
            Projection::Perspective(PerspectiveProjection {
                far: 18000.0,
                ..default()
            }),
            aether_view::art::camera_effects(world.resource::<AssetServer>()),
            fog,
            DistanceFog {
                color: Color::srgb(0.42, 0.61, 0.80),
                falloff: FogFalloff::Linear {
                    start: 550.0,
                    end: 7200.0,
                },
                ..default()
            },
            camera,
        ))
        .id();
    if let Some(directory) = args.windows(2).find(|a| a[0] == "--authored-ibl") {
        use bevy::image::*;
        let assets = world.resource::<AssetServer>();
        let load = |name: &str| -> Handle<Image> {
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
                .load(format!("{}/{name}.ktx2", directory[1]))
        };
        let environment = bevy::light::EnvironmentMapLight {
            diffuse_map: load("sky-diffuse"),
            specular_map: load("sky-specular"),
            ..default()
        };
        world
            .entity_mut(camera_entity)
            .remove::<bevy::light::AtmosphereEnvironmentMapLight>()
            .insert(environment);
        world
            .get_mut::<aether_view::art::EnvironmentLighting>(camera_entity)
            .unwrap()
            .radiance_scale = parameter(
            "--ibl-scale",
            aether_view::art::IBL_RADIANCE_SCALE,
            0.0,
            10000.0,
        );
        // This diagnostic uses the authored, planetless sky and its own IBL.
        // Keeping the terrestrial atmosphere would reintroduce an unrelated
        // ground horizon and a second source of aerial scattering.
        let atmospheres: Vec<_> = world
            .query_filtered::<Entity, With<bevy::light::Atmosphere>>()
            .iter(world)
            .collect();
        for atmosphere in atmospheres {
            world.despawn(atmosphere);
        }
    }
    let sky_bounce = parameter(
        "--sky-bounce-scale",
        aether_view::art::CLOUD_BOUNCE_STRENGTH,
        0.0,
        2.0,
    );
    if sky_bounce > 0.0 {
        world.spawn((
            DirectionalLight {
                shadow_maps_enabled: false,
                ..default()
            },
            aether_view::art::CloudBounce(sky_bounce),
            aether_view::art::cloud_bounce_transform(),
        ));
    }
    let sun_transform = if args
        .iter()
        .any(|a| a == "--sun-elevation" || a == "--sun-azimuth")
    {
        let elevation = parameter(
            "--sun-elevation",
            aether_view::art::SUN_DIRECTION.y.asin().to_degrees(),
            -20.0,
            89.0,
        )
        .to_radians();
        let azimuth = parameter(
            "--sun-azimuth",
            aether_view::art::SUN_DIRECTION
                .x
                .atan2(aether_view::art::SUN_DIRECTION.z)
                .to_degrees(),
            -180.0,
            180.0,
        )
        .to_radians();
        let direction = Vec3::new(
            azimuth.sin() * elevation.cos(),
            elevation.sin(),
            azimuth.cos() * elevation.cos(),
        );
        Transform::from_translation(direction).looking_at(Vec3::ZERO, Vec3::Y)
    } else {
        aether_view::art::sun_transform()
    };
    world.spawn((
        DirectionalLight {
            illuminance: 50000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        bevy::light::VolumetricLight,
        sun_transform,
        bevy::light::CascadeShadowConfigBuilder {
            first_cascade_far_bound: 20.0,
            maximum_distance: parameter(
                "--shadow-distance",
                *aether_view::art::sunlight_shadows().bounds.last().unwrap(),
                100.0,
                5000.0,
            ),
            ..default()
        }
        .build(),
    ));
    world.resource_scope(|world, art: Mut<aether_view::art::ArtAssets>| {
        aether_view::art::sky(&mut world.commands(), &art)
    });
    aether_view::world::spawn(world);
    aether_view::weather::spawn(world);
    let hide_local_clouds = args.iter().any(|a| a == "--no-local-clouds");
    let hide_high_clouds = args.iter().any(|a| a == "--no-high-clouds");
    if hide_local_clouds || hide_high_clouds {
        // Diagnostic fixture only: isolate the high billows while retaining
        // regional banks, or isolate the panorama from every local volume.
        let mut clouds =
            world.query::<(Entity, &MeshMaterial3d<aether_view::clouds::CloudMaterial>)>();
        let materials = world.resource::<Assets<aether_view::clouds::CloudMaterial>>();
        let volumes: Vec<_> = clouds
            .iter(world)
            .filter_map(|(entity, material)| {
                (hide_local_clouds
                    || materials
                        .get(&material.0)
                        .is_some_and(|cloud| cloud.shape.w < 0.5))
                .then_some(entity)
            })
            .collect();
        for entity in volumes {
            world.entity_mut(entity).insert(Visibility::Hidden);
        }
    }
    world.resource_scope(|world, art: Mut<aether_view::traffic::TrafficArt>| {
        for courier in aether_core::traffic::COURIERS {
            let (position, rotation) = courier.pose(120.0);
            let owner = world
                .spawn((
                    FixtureVelocity((courier.pose(120.01).0 - courier.pose(119.99).0) / 0.02),
                    Transform::from_translation(position)
                        .with_rotation(rotation)
                        .with_scale(Vec3::splat(courier.scale())),
                    Visibility::Inherited,
                ))
                .id();
            aether_view::traffic::spawn(&mut world.commands(), &art, owner, courier);
        }
    });
    world.resource_scope(|world, art: Mut<aether_view::fauna::FaunaArt>| {
        for resident in aether_core::fauna::RESIDENTS {
            let (position, rotation) = resident.pose(120.0);
            world
                .spawn((
                    WorldAssetRoot(art.0[resident.species as usize].clone()),
                    aether_view::fauna::FaunaVisual(resident),
                    Transform::from_translation(position)
                        .with_rotation(rotation)
                        .with_scale(Vec3::splat(resident.scale())),
                ))
                .observe(aether_view::fauna::bind);
        }
    });
    world.insert_resource(aether_view::weather::WeatherView {
        seconds: 120.0,
        sample: aether_core::weather::sample(eye, 120.0),
        reduced_motion: false,
    });
    if !cave && !industrial_review && !args.iter().any(|a| a == "--no-player") {
        let position = eye + *camera.forward() * 36.0 + *camera.right() * 5.5 - Vec3::Y * 8.0;
        world.spawn((
            aether_view::BodyVisual::new(&aether_core::fixtures::explorer()),
            Transform::from_translation(position).with_rotation(Quat::from_rotation_y(-0.22)),
            Visibility::Inherited,
        ));
    }
    let path = args.windows(2).find(|a| a[0] == "--capture").map_or_else(
        || format!(".dream-loop/gallery-{biome}.png"),
        |a| a[1].clone(),
    );
    world.insert_resource(Capture {
        path,
        elapsed: 0.0,
        captured: false,
    });
}
fn fixture_sails(
    owners: Query<(
        &aether_view::BodyVisual,
        &Transform,
        Option<&FixtureVelocity>,
    )>,
    mut sails: Query<(
        &aether_view::PartVisual,
        &Transform,
        &mut aether_view::sail::SailWind,
    )>,
) {
    // The gallery freezes the same weather field as its world at 120 seconds.
    // Giving sails their production input avoids judging an unpressurized
    // default mesh while the surrounding couriers imply active navigation.
    for (part, transform, mut wind) in &mut sails {
        let Ok((body, owner, velocity)) = owners.get(part.owner) else {
            continue;
        };
        let Some(piece) = body.body.parts().iter().find(|p| p.id == part.id) else {
            continue;
        };
        let apparent = aether_core::fields::wind(owner.translation, 120.0)
            - velocity.map_or(Vec3::ZERO, |v| v.0);
        *wind = aether_view::sail::SailWind {
            seconds: 120.0,
            normal_speed: apparent.dot(owner.rotation * transform.rotation * Vec3::Z),
            speed: apparent.length(),
            phase: piece.center().dot(Vec3::new(0.73, 0.31, 0.57)),
            reduced_motion: false,
        };
    }
}
fn capture(
    mut commands: Commands,
    time: Res<Time<Real>>,
    mut c: ResMut<Capture>,
    metrics: Res<aether_view::MeshMetrics>,
    mut exit: MessageWriter<AppExit>,
    industrial: Option<Res<IndustrialReview>>,
    textures: (
        Res<AssetServer>,
        Res<Assets<Image>>,
        Res<Assets<StandardMaterial>>,
    ),
    bounds: Query<
        (
            Has<bevy::camera::primitives::Aabb>,
            Has<bevy::camera::visibility::NoAutoAabb>,
            Has<aether_view::craft::CraftVisual>,
            Has<aether_view::ChunkVisual>,
            &Mesh3d,
        ),
        With<Mesh3d>,
    >,
) {
    c.elapsed += time.delta_secs();
    let kit_ready = industrial.as_ref().is_none_or(|kit| {
        kit.scenes
            .iter()
            .all(|scene| textures.0.is_loaded_with_dependencies(scene.id()))
    });
    if c.elapsed > 12.0 && !c.captured && metrics.pending == 0 && kit_ready {
        commands
            .spawn(bevy::render::view::screenshot::Screenshot::primary_window())
            .observe(bevy::render::view::screenshot::save_to_disk(c.path.clone()));
        c.captured = true;
        let assets = &textures.0;
        if let Some(kit) = &industrial {
            let source_counts = bounds
                .iter()
                .filter_map(|(_, _, _, _, mesh)| assets.get_path(mesh.id()))
                .filter(|path| path.path().to_string_lossy().contains("aether-industrial/"))
                .count();
            assert!(source_counts > 0, "authored kit meshes actually spawned");
            std::fs::write(
                format!("{}.industrial.json", c.path),
                serde_json::to_vec_pretty(&serde_json::json!({
                    "fixture": true,
                    "gameplay_proof": false,
                    "quality": if kit.lod { "lod" } else { "hd" },
                    "original_metre_scale": true,
                    "ready_scenes": kit.scenes.len(),
                    "spawned_authored_meshes": source_counts,
                    "source_paths": kit.scenes.iter().map(|scene| assets.get_path(scene.id()).map(|p| p.to_string())).collect::<Vec<_>>(),
                }))
                .expect("industrial report"),
            )
            .expect("write industrial report");
        }
        let mut automatic = std::collections::BTreeMap::<String, usize>::new();
        for (_, frozen, _, _, mesh) in &bounds {
            if !frozen {
                let source = assets.get_path(mesh.id()).map_or_else(
                    || "procedural".to_owned(),
                    |p| p.path().to_string_lossy().into_owned(),
                );
                *automatic.entry(source).or_default() += 1;
            }
        }
        let counts = serde_json::json!({
            "mesh_entities": bounds.iter().count(),
            "bounded": bounds.iter().filter(|(a, _, _, _, _)| *a).count(),
            "immutable_bounded": bounds.iter().filter(|(a, frozen, _, _, _)| *a && *frozen).count(),
            "craft_automatic": bounds.iter().filter(|(_, frozen, craft, _, _)| !*frozen && *craft).count(),
            "voxel_automatic": bounds.iter().filter(|(_, frozen, _, voxel, _)| !*frozen && *voxel).count(),
            "automatic_by_source": automatic,
        });
        std::fs::write(
            format!("{}.bounds.json", c.path),
            serde_json::to_vec_pretty(&counts).expect("bounds report"),
        )
        .expect("write bounds report");
        let (assets, images, materials) = textures;
        // Optional inspection of an actual loaded surface, separate from the
        // foliage coverage report. Never modifies the source image or sampler.
        let args: Vec<_> = std::env::args().collect();
        if let Some(request) = args.windows(2).find(|a| a[0] == "--surface-texture") {
            let mut inspected = Vec::new();
            for (id, image) in images.iter() {
                let Some(path) = assets.get_path(id) else {
                    continue;
                };
                if path.path().to_string_lossy().replace('\\', "/") != request[1].replace('\\', "/")
                {
                    continue;
                }
                let rgba8 = matches!(
                    image.texture_descriptor.format,
                    bevy::render::render_resource::TextureFormat::Rgba8Unorm
                        | bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb
                );
                let level_zero = if rgba8 {
                    image.data.as_ref().and_then(|data| {
                        data.get(..image.width() as usize * image.height() as usize * 4)
                    })
                } else {
                    None
                };
                let dump = format!("{}.surface-{}.rgba", c.path, inspected.len());
                if let Some(pixels) = level_zero {
                    std::fs::write(&dump, pixels).expect("surface level zero dump");
                }
                inspected.push(serde_json::json!({
                    "path": path.to_string(), "dimensions": [image.width(), image.height()],
                    "format": format!("{:?}", image.texture_descriptor.format),
                    "mip_count": image.texture_descriptor.mip_level_count,
                    "data_bytes": image.data.as_ref().map(Vec::len),
                    "sampler": format!("{:?}", image.sampler),
                    "level_zero_dump": level_zero.map(|_| dump),
                }));
            }
            assert!(
                !inspected.is_empty(),
                "requested surface texture is not loaded"
            );
            std::fs::write(
                format!("{}.surface.json", c.path),
                serde_json::to_vec_pretty(&inspected).expect("surface report"),
            )
            .expect("write surface texture report");
        }
        let mut masks = Vec::new();
        for (_, material) in materials.iter() {
            let AlphaMode::Mask(cutoff) = material.alpha_mode else {
                continue;
            };
            let Some(texture) = &material.base_color_texture else {
                continue;
            };
            let Some(image) = images.get(texture) else {
                continue;
            };
            let Some(data) = &image.data else {
                continue;
            };
            if !matches!(
                image.texture_descriptor.format,
                bevy::render::render_resource::TextureFormat::Rgba8Unorm
                    | bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb
            ) {
                continue;
            }
            let (mut width, mut height) = (image.width() as usize, image.height() as usize);
            let mut offset = 0;
            let mut coverage = Vec::new();
            for _ in 0..image.texture_descriptor.mip_level_count {
                let length = width * height * 4;
                let pixels = &data[offset..offset + length];
                let visible = pixels
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .filter(|p| p[3] as f32 / 255.0 >= cutoff)
                    .count();
                coverage.push(serde_json::json!({"dimensions":[width,height],"coverage":visible as f32/(width*height) as f32}));
                offset += length;
                width = (width / 2).max(1);
                height = (height / 2).max(1);
            }
            masks.push(serde_json::json!({"path":assets.get_path(texture.id()).map(|p|p.to_string()),"cutoff":cutoff,"levels":coverage}));
        }
        std::fs::write(
            format!("{}.textures.json", c.path),
            serde_json::to_vec_pretty(&masks).expect("texture report"),
        )
        .expect("write texture report");
    }
    if c.elapsed > 15.0 {
        exit.write(if c.captured {
            AppExit::Success
        } else {
            AppExit::error()
        });
    }
}
