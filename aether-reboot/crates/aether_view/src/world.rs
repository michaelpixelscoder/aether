//! Landmark kit instancing with independent high/low distance ranges.
use crate::environment::EnvironmentVisual;
use aether_core::{
    world::{self},
    world_geometry,
};
use bevy::{
    camera::primitives::{Aabb, MeshAabb},
    camera::visibility::VisibilityRange,
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    world_serialization::WorldInstanceReady,
};

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct FlowMaterial {
    #[uniform(0)]
    pub color: Vec4,
    /// x: current=0, cascade=1, pool=2; y: speed; z: opacity; w: simulation time.
    #[uniform(1)]
    pub settings: Vec4,
    #[texture(2)]
    #[sampler(3)]
    pub flow_texture: Option<Handle<Image>>,
}
impl Material for FlowMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/world-flow.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}
#[derive(Resource)]
pub struct WorldArt {
    pub vault: [Handle<bevy::world_serialization::WorldAsset>; 2],
    pub scenes: Vec<[Handle<bevy::world_serialization::WorldAsset>; 2]>,
    pub flow: Handle<FlowMaterial>,
    pub waterfall: Handle<FlowMaterial>,
    pub pool: Handle<FlowMaterial>,
    pub quad: Handle<Mesh>,
    quad_bounds: Aabb,
}
pub const FLOW_TEXTURE: &str = "textures/world-flow.ktx2";
pub const CURRENT_TEXTURE: &str = "textures/world-flow-current-r64.ktx2";
pub fn flow_texture(assets: &AssetServer) -> Handle<Image> {
    liquid_texture(assets, FLOW_TEXTURE)
}
pub fn current_texture(assets: &AssetServer) -> Handle<Image> {
    liquid_texture(assets, CURRENT_TEXTURE)
}
fn liquid_texture(assets: &AssetServer, path: &'static str) -> Handle<Image> {
    use bevy::image::*;
    assets
        .load_builder()
        .with_settings(|settings: &mut ImageLoaderSettings| {
            // Authored sRGB liquid with offline alpha-weighted linear mips.
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
        .load(path)
}
#[derive(Component)]
pub struct IslandLod {
    pub near: bool,
    pub distance: f32,
}
#[derive(Component)]
pub struct MillRotor {
    pub base: Quat,
    pub speed: f32,
}
pub fn bind(
    event: On<WorldInstanceReady>,
    mut commands: Commands,
    roots: Query<&IslandLod>,
    children: Query<&Children>,
    meshes: Query<(), With<Mesh3d>>,
    authored_bounds: Query<(), With<bevy::camera::primitives::Aabb>>,
    names: Query<(&Name, &Transform)>,
    mesh_materials: Query<&MeshMaterial3d<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(lod) = roots.get(event.entity) else {
        return;
    };
    for entity in children.iter_descendants(event.entity) {
        if let Ok((name, t)) = names.get(entity) {
            if name.contains("Aether mineral")
                && let Ok(handle) = mesh_materials.get(entity)
                && let Some(mut material) = materials.get_mut(&handle.0)
            {
                // A restrained internal light preserves the authored facet
                // colours. The mineral previously read as opaque blue plastic
                // in shadow, with an emission factor of only .04.
                let color = material.base_color.to_linear();
                material.emissive =
                    LinearRgba::new(color.red * 2.5, color.green * 2.5, color.blue * 2.5, 1.0);
            }
            let speed = match name.as_str() {
                "WindmillRotor" => Some(0.24),
                "UnderforgeGearWestForge" | "UnderforgeGearWestDrive" => Some(0.17),
                "UnderforgeGearEastForge"
                | "UnderforgeGearEastWorkshop"
                | "UnderforgeGearEastDrive" => Some(-0.21),
                _ => None,
            };
            if let Some(speed) = speed {
                commands.entity(entity).insert(MillRotor {
                    base: t.rotation,
                    speed,
                });
            }
        }
        if meshes.contains(entity) {
            if authored_bounds.contains(entity) {
                // GLTF supplies exact local bounds. Island geometry is immutable
                // after loading; mill rotations only change its world transform.
                // Exclude these instances from Bevy's per-frame asset-change scan.
                commands
                    .entity(entity)
                    .insert(bevy::camera::visibility::NoAutoAabb);
            }
            let range = if lod.near {
                VisibilityRange {
                    start_margin: 0.0..0.0,
                    end_margin: lod.distance..lod.distance + 120.0,
                    use_aabb: false,
                }
            } else {
                VisibilityRange {
                    start_margin: lod.distance..lod.distance + 120.0,
                    end_margin: 11000.0..12000.0,
                    use_aabb: false,
                }
            };
            commands.entity(entity).insert(range);
        }
    }
}
pub fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<FlowMaterial>>,
) {
    let quad = crate::voxel::to_mesh(aether_core::mesh::Surface {
        positions: vec![
            [-0.5, -0.5, 0.0],
            [0.5, -0.5, 0.0],
            [0.5, 0.5, 0.0],
            [-0.5, 0.5, 0.0],
        ],
        normals: vec![[0.0, 0.0, 1.0]; 4],
        uvs: vec![[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
        indices: vec![0, 1, 2, 0, 2, 3, 2, 1, 0, 3, 2, 0],
    });
    let quad_bounds = quad.compute_aabb().expect("water plane");
    let flow_texture = flow_texture(&assets);
    commands.insert_resource(WorldArt {
        vault: [
            assets.load(GltfAssetLabel::Scene(0).from_asset("world/arch-vault.glb")),
            assets.load(GltfAssetLabel::Scene(0).from_asset("world/arch-vault-lod.glb")),
        ],
        scenes: world::ASSETS
            .iter()
            .map(|b| {
                [
                    assets.load(GltfAssetLabel::Scene(0).from_asset(format!("world/{b}.glb"))),
                    assets.load(GltfAssetLabel::Scene(0).from_asset(format!("world/{b}-lod.glb"))),
                ]
            })
            .collect(),
        flow: materials.add(FlowMaterial {
            color: Vec4::new(0.025, 0.30, 1.45, 1.0),
            settings: Vec4::new(0.0, 0.8, 0.90, 0.0),
            flow_texture: Some(current_texture(&assets)),
        }),
        waterfall: materials.add(FlowMaterial {
            color: Vec4::new(0.12, 0.60, 0.90, 1.0),
            settings: Vec4::new(1.0, 1.2, 0.80, 0.0),
            flow_texture: Some(flow_texture.clone()),
        }),
        pool: materials.add(FlowMaterial {
            color: Vec4::new(0.018, 0.26, 0.40, 1.0),
            settings: Vec4::new(2.0, 0.18, 0.88, 0.0),
            flow_texture: Some(flow_texture),
        }),
        quad: meshes.add(quad),
        quad_bounds,
    });
}
pub fn spawn(world_ecs: &mut World) {
    world_ecs.resource_scope(|world_ecs, art: Mut<WorldArt>| {
        for vault in world::vaults() {
            for n in 0..2 {
                world_ecs
                    .spawn((
                        EnvironmentVisual,
                        WorldAssetRoot(art.vault[n].clone()),
                        IslandLod {
                            near: n == 0,
                            distance: 2600.0,
                        },
                        Transform::from_translation(vault.center)
                            .with_rotation(Quat::from_rotation_y(vault.yaw)),
                    ))
                    .observe(bind);
            }
        }
        for island in world::islands() {
            for index in 0..2 {
                world_ecs
                    .spawn((
                        EnvironmentVisual,
                        WorldAssetRoot(art.scenes[island.asset_index()][index].clone()),
                        IslandLod {
                            near: index == 0,
                            // Large distant capitals still occupy hundreds of
                            // pixels. Keep their masonry until their angular
                            // size is comparable to that of a small satellite.
                            distance: 300.0 + island.radius() * 8.0,
                        },
                        Transform::from_translation(island.center)
                            .with_rotation(Quat::from_rotation_y(island.yaw))
                            .with_scale(Vec3::splat(island.scale)),
                    ))
                    .observe(bind);
            }
            let marks = world_geometry::landmarks_for(island.asset_key());
            if island.capital {
                if matches!(
                    island.biome,
                    world::Biome::Hollow | world::Biome::Underforge
                ) {
                    // Lantern light reaches the paths and ship passage. The
                    // emissive windows alone do not illuminate nearby surfaces.
                    for z in [-40.0, 0.0, 40.0, 72.0] {
                        for x in [-18.0, 18.0] {
                            world_ecs.spawn((
                                EnvironmentVisual,
                                PointLight {
                                    color: if island.biome == world::Biome::Hollow {
                                        Color::srgb(0.48, 0.85, 0.60)
                                    } else {
                                        Color::srgb(1.0, 0.62, 0.30)
                                    },
                                    intensity: 1_200_000.0 * island.scale * island.scale,
                                    range: 42.0 * island.scale,
                                    radius: 2.0 * island.scale,
                                    ..default()
                                },
                                Transform::from_translation(
                                    island.transform_point(Vec3::new(x, 12.0, z)),
                                ),
                            ));
                        }
                    }
                }
                for (n, p) in marks.crystals.iter().take(4).enumerate() {
                    world_ecs.spawn((
                        EnvironmentVisual,
                        PointLight {
                            color: if n % 2 == 0 {
                                Color::srgb(0.18, 0.45, 1.0)
                            } else {
                                Color::srgb(0.70, 0.18, 1.0)
                            },
                            intensity: 220_000.0 * island.scale * island.scale,
                            range: 55.0 * island.scale,
                            radius: 1.5 * island.scale,
                            ..default()
                        },
                        Transform::from_translation(island.transform_point(*p + Vec3::Y * 3.0)),
                    ));
                }
                for p in [Vec3::new(-38.0, 12.0, -18.0), Vec3::new(39.0, 12.0, 26.0)] {
                    world_ecs.spawn((
                        EnvironmentVisual,
                        PointLight {
                            color: Color::srgb(1.0, 0.52, 0.18),
                            intensity: 380_000.0 * island.scale * island.scale,
                            range: 70.0 * island.scale,
                            radius: 2.0 * island.scale,
                            ..default()
                        },
                        Transform::from_translation(island.transform_point(p)),
                    ));
                }
            }
            for fall in &marks.waterfalls {
                let position = island.transform_point(fall.position - Vec3::Y * fall.height * 0.5);
                // Crossed sheets retain volume from orbit, with a fine animated
                // flow pattern; their ends fade into the atmospheric haze.
                for angle in [0.0, std::f32::consts::FRAC_PI_2] {
                    world_ecs.spawn((
                        EnvironmentVisual,
                        Mesh3d(art.quad.clone()),
                        art.quad_bounds,
                        bevy::camera::visibility::NoAutoAabb,
                        MeshMaterial3d(art.waterfall.clone()),
                        Transform::from_translation(position)
                            .with_rotation(Quat::from_rotation_y(island.yaw + angle))
                            .with_scale(Vec3::new(fall.width, fall.height, 1.0) * island.scale),
                        bevy::light::NotShadowCaster,
                        VisibilityRange {
                            start_margin: 0.0..0.0,
                            end_margin: 4500.0..5000.0,
                            use_aabb: true,
                        },
                    ));
                }
            }
            for pool in &marks.pools {
                world_ecs.spawn((
                    EnvironmentVisual,
                    Mesh3d(art.quad.clone()),
                    art.quad_bounds,
                    bevy::camera::visibility::NoAutoAabb,
                    MeshMaterial3d(art.pool.clone()),
                    Transform::from_translation(island.transform_point(pool.position))
                        .with_rotation(
                            Quat::from_rotation_y(island.yaw)
                                * Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                        )
                        .with_scale(Vec3::new(pool.size.x, pool.size.z, 1.0) * island.scale),
                    bevy::light::NotShadowCaster,
                ));
            }
        }
        for route in world::routes() {
            let mesh = flow_mesh(route);
            let bounds = crate::immutable_bounds(&mesh);
            let mesh = world_ecs.resource_mut::<Assets<Mesh>>().add(mesh);
            world_ecs.spawn((
                EnvironmentVisual,
                bounds,
                Mesh3d(mesh),
                MeshMaterial3d(art.flow.clone()),
                Transform::default(),
                bevy::light::NotShadowCaster,
                bevy::light::NotShadowReceiver,
            ));
        }
    });
}
pub fn animate(
    view: Res<crate::weather::WeatherView>,
    mut rotors: Query<(&MillRotor, &mut Transform)>,
    mut materials: ResMut<Assets<FlowMaterial>>,
) {
    if view.reduced_motion {
        return;
    }
    for (rotor, mut t) in &mut rotors {
        t.rotation = rotor.base * Quat::from_rotation_z(view.seconds * rotor.speed);
    }
    for (_, material) in materials.iter_mut() {
        material.settings.w = view.seconds;
    }
}
fn flow_mesh(current: &aether_core::fields::Current) -> Mesh {
    let mut surface = aether_core::mesh::Surface::default();
    let mut colors = Vec::new();
    let frames: Vec<_> = (0..current.points.len())
        .map(|n| {
            let previous = current.points[n.saturating_sub(1)];
            let next = current.points[(n + 1).min(current.points.len() - 1)];
            let tangent = (next - previous).normalize_or_zero();
            let horizontal = tangent.cross(Vec3::Y);
            let side = if horizontal.length_squared() > 0.001 {
                horizontal.normalize()
            } else {
                current
                    .points
                    .windows(2)
                    .map(|p| (p[1] - p[0]).cross(Vec3::Y))
                    .find(|v| v.length_squared() > 0.001)
                    .unwrap_or(Vec3::X)
                    .normalize()
            };
            (tangent, side)
        })
        .collect();
    let mut travelled = 0.0;
    for (segment_index, segment) in current.points.windows(2).enumerate() {
        let point = |t: f32| {
            let p0 = current.points[segment_index.saturating_sub(1)];
            let p1 = segment[0];
            let p2 = segment[1];
            let p3 = current.points[(segment_index + 2).min(current.points.len() - 1)];
            let line = p1.lerp(p2, t);
            let curve = (2.0 * p1
                + (-p0 + p2) * t
                + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t * t
                + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t * t * t)
                * 0.5;
            // The curve and its banks stay inside the sampled current tube.
            line + (curve - line).clamp_length_max(current.radius * 0.12)
        };
        for j in 0..16 {
            let a = point(j as f32 / 16.0);
            let b = point((j + 1) as f32 / 16.0);
            for strip in 0..8 {
                let left = strip as f32 / 4.0 - 1.0;
                let right = (strip + 1) as f32 / 4.0 - 1.0;
                let base = surface.positions.len() as u32;
                for (p, k, fraction) in [
                    (a, left, j as f32 / 16.0),
                    (b, left, (j + 1) as f32 / 16.0),
                    (b, right, (j + 1) as f32 / 16.0),
                    (a, right, j as f32 / 16.0),
                ] {
                    let (tangent_a, side_a) = frames[segment_index];
                    let (tangent_b, side_b) = frames[segment_index + 1];
                    let dir = tangent_a.lerp(tangent_b, fraction).normalize_or_zero();
                    let side = side_a.lerp(side_b, fraction).normalize_or_zero();
                    let phase = (travelled + segment[0].distance(segment[1]) * fraction) * 0.008;
                    // One curved cross-section, with irregular banks, avoids
                    // three parallel ribbons reading as a railway in the sky.
                    // Its complete envelope stays inside the physical current.
                    let width = 0.45 + 0.035 * (phase * 2.7).sin();
                    let height = phase.sin() * 0.045
                        + k * k * (0.08 + (phase * 1.9 + k * 2.0).sin() * 0.035)
                        + (phase * 3.1 + k * 4.0).sin() * 0.022;
                    let offset = (side * (k * width) + dir.cross(side) * height) * current.radius;
                    surface.positions.push((p + offset).to_array());
                    surface.normals.push(Vec3::Y.to_array());
                    surface.uvs.push([
                        (travelled + segment[0].distance(segment[1]) * fraction) * 0.04,
                        (k + 1.0) * 0.5,
                    ]);
                    let t = (segment_index as f32 + fraction) / (current.points.len() - 1) as f32;
                    // Compact thermals need their final turn to remain visible.
                    // The former 8.3% fade erased much of the spiral's eye.
                    let fade = if current.radius <= 20.0 { 40.0 } else { 12.0 };
                    colors.push([
                        1.0,
                        1.0,
                        1.0,
                        (t * fade).min((1.0 - t) * fade).clamp(0.0, 1.0)
                            * ((world::nearest_port(p + offset).2 - 65.0) / 75.0).clamp(0.0, 1.0),
                    ]);
                }
                surface.indices.extend([
                    base,
                    base + 1,
                    base + 2,
                    base,
                    base + 2,
                    base + 3,
                    base + 2,
                    base + 1,
                    base,
                    base + 3,
                    base + 2,
                    base,
                ]);
            }
        }
        travelled += segment[0].distance(segment[1]);
    }
    let mut mesh = crate::voxel::to_mesh(surface);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh
}
