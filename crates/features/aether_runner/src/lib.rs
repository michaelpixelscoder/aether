use std::time::Duration;

use bevy::{
    animation::{AnimatedBy, AnimationTargetId},
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    input::touch::Touches,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

const CHARACTER: &str = "characters/kaykit/Knight.glb";
const LANE_WIDTH: f32 = 2.6;
const LANE_COUNT: i32 = 3;
const PATH_WIDTH: f32 = LANE_WIDTH * LANE_COUNT as f32 + 0.8;
const RUN_SPEED: f32 = 9.0;
const GROUND_Y: f32 = 0.72;
const ISLAND_GROUND_Y: f32 = 0.44;
const JUMP_SPEED: f32 = 10.0;
const GRAVITY: f32 = 20.0;
const GLIDE_SPEED: f32 = 12.0;
const GLIDE_GRAVITY: f32 = 4.5;
const GLIDE_MAX_FALL_SPEED: f32 = 3.25;
const GLIDE_DEPLOY_LIFT: f32 = 2.4;
const ROPE_RANGE: f32 = 15.0;
const ROPE_GRAVITY: f32 = 18.0;
const ROPE_DRAG: f32 = 0.18;
const ROUTE_LENGTH: f32 = 238.0;
const SEGMENT_LENGTH: f32 = 2.0;
const KNOT_DISTANCE: f32 = 24.0;

// Authored control points for a Catmull-Rom spline. Z progresses forward while X
// creates readable bends; gameplay, camera, islands and scenery all sample it.
const SPLINE_X: [f32; 13] = [
    0.0, 0.0, 4.0, 11.0, 8.0, -2.0, -11.0, -8.0, 2.0, 12.0, 7.0, -4.0, -4.0,
];

// Each tuple is one island's supported path interval. The spaces are real void.
const ISLAND_SPANS: [(f32, f32); 6] = [
    (0.0, 39.0),
    (47.0, 78.0),
    (86.0, 120.0),
    (129.0, 164.0),
    (173.0, 207.0),
    (216.0, ROUTE_LENGTH),
];

pub struct RunnerPlugin;

impl Plugin for RunnerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AnimationPlayer>()
            .register_type::<AnimationTargetId>()
            .register_type::<AnimatedBy>()
            .init_resource::<ActorIntent>()
            .init_resource::<SwipeGesture>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    read_local_input,
                    drive_actor,
                    sync_rope_visual,
                    sync_glider_visual,
                    follow_actor,
                    start_character_animation,
                    update_character_animation,
                )
                    .chain(),
            );
    }
}

#[derive(Resource, Default)]
pub struct ActorIntent {
    lane_delta: i32,
    jump_pressed: bool,
    rope_held: bool,
}

#[derive(Resource, Default)]
struct SwipeGesture {
    start: Option<(u64, Vec2)>,
}

#[derive(Component)]
pub struct RunnerActor {
    distance: f32,
    lane: i32,
    lane_offset: f32,
    vertical_speed: f32,
    airborne: bool,
    gliding: bool,
    forward_speed: f32,
    rope: Option<RopeSwing>,
}

#[derive(Clone, Copy)]
struct RopeSwing {
    anchor_distance: f32,
    anchor_height: f32,
    length: f32,
    angle: f32,
    angular_speed: f32,
}

#[derive(Component)]
struct RunnerCamera;

#[derive(Component)]
struct GliderVisual;

#[derive(Component)]
struct RopeAnchor {
    distance: f32,
    height: f32,
}

#[derive(Component)]
struct RopeVisual;

/// Collision surface for a playable island. Decorative background islands do
/// not receive this component and can never catch a failed jump.
#[derive(Component)]
struct IslandFloor {
    center: Vec2,
    radius: f32,
    height: f32,
}

#[derive(Resource)]
struct CharacterAnimations {
    graph: Handle<AnimationGraph>,
    run: AnimationNodeIndex,
    jump: AnimationNodeIndex,
}

#[derive(Resource, Default)]
struct AnimationState {
    jumping: bool,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    commands.insert_resource(ClearColor(Color::srgb(0.18, 0.32, 0.48)));
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.72, 0.82, 1.0),
        brightness: 1_200.0,
        ..default()
    });

    let (graph, nodes) = AnimationGraph::from_clips([
        asset_server.load(GltfAssetLabel::Animation(48).from_asset(CHARACTER)),
        asset_server.load(GltfAssetLabel::Animation(38).from_asset(CHARACTER)),
    ]);
    commands.insert_resource(CharacterAnimations {
        graph: graphs.add(graph),
        run: nodes[0],
        jump: nodes[1],
    });
    commands.init_resource::<AnimationState>();

    let actor = commands
        .spawn((
            RunnerActor {
                distance: 3.0,
                lane: 0,
                lane_offset: 0.0,
                vertical_speed: 0.0,
                airborne: false,
                gliding: false,
                forward_speed: RUN_SPEED,
                rope: None,
            },
            Transform::from_xyz(0.0, GROUND_Y, -3.0),
            Visibility::default(),
        ))
        .id();
    commands.entity(actor).with_child((
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(CHARACTER))),
        Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
    ));
    let glider_mesh = meshes.add(Cuboid::new(2.45, 0.12, 0.72));
    let glider_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.92, 0.28, 0.16),
        metallic: 0.08,
        perceptual_roughness: 0.7,
        ..default()
    });
    commands.entity(actor).with_children(|parent| {
        for side in [-1.0_f32, 1.0] {
            parent.spawn((
                GliderVisual,
                Mesh3d(glider_mesh.clone()),
                MeshMaterial3d(glider_material.clone()),
                Transform::from_xyz(side * 1.12, 2.75, 0.12)
                    .with_rotation(Quat::from_rotation_z(side * -0.14)),
                Visibility::Hidden,
            ));
        }
    });
    commands.spawn((
        RopeVisual,
        Mesh3d(meshes.add(Cylinder::new(0.055, 1.0).mesh().resolution(8))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.96, 0.82, 0.48),
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::default(),
        Visibility::Hidden,
    ));

    let anchor_mesh = meshes.add(Sphere::new(0.48).mesh().ico(2).unwrap());
    let anchor_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.72, 0.12),
        emissive: LinearRgba::rgb(5.0, 2.0, 0.15),
        ..default()
    });
    for distance in [43.0, 82.0, 124.5, 168.5, 211.5] {
        let (position, _) = sample_route(distance);
        let height = 10.0;
        commands.spawn((
            RopeAnchor { distance, height },
            Mesh3d(anchor_mesh.clone()),
            MeshMaterial3d(anchor_material.clone()),
            Transform::from_translation(position + Vec3::Y * height),
        ));
    }

    let (start, _) = sample_route(3.0);
    commands.spawn((
        RunnerCamera,
        Camera3d::default(),
        Transform::from_translation(start + Vec3::new(0.0, 5.0, 9.0))
            .looking_at(start + Vec3::new(0.0, 1.0, -7.0), Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 18_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, -0.45, 0.0)),
    ));

    spawn_route(&mut commands, &mut meshes, &mut materials, &mut images);

    commands.spawn((
        Text::new(
            "ISLAND RUN  •  ← → steer  •  SPACE jump/glide  •  hold F or E to swing, release to launch",
        ),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: px(24),
            left: px(24),
            ..default()
        },
    ));
}

fn spawn_route(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
) {
    let (albedo, normal) = procedural_dirt_textures(images);
    let path_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(albedo),
        normal_map_texture: Some(normal),
        perceptual_roughness: 0.93,
        ..default()
    });
    let earth_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.23, 0.13, 0.075),
        perceptual_roughness: 1.0,
        ..default()
    });
    let grass_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.22, 0.43, 0.19),
        perceptual_roughness: 0.96,
        ..default()
    });
    let trunk_material = materials.add(Color::srgb(0.24, 0.12, 0.055));
    let leaf_material = materials.add(Color::srgb(0.08, 0.31, 0.15));

    let path_mesh = meshes.add(Cuboid::new(PATH_WIDTH, 0.3, SEGMENT_LENGTH + 0.18));
    let island_mesh = meshes.add(Cylinder::new(1.0, 1.0).mesh().resolution(12));
    let trunk_mesh = meshes.add(Cylinder::new(0.22, 1.8).mesh().resolution(8));
    let crown_mesh = meshes.add(Cone::new(1.25, 3.0).mesh().resolution(9));

    for &(start, end) in &ISLAND_SPANS {
        let mut distance = start + SEGMENT_LENGTH * 0.5;
        while distance < end - SEGMENT_LENGTH * 0.35 {
            let (position, tangent) = sample_route(distance);
            commands.spawn((
                Mesh3d(path_mesh.clone()),
                MeshMaterial3d(path_material.clone()),
                Transform::from_translation(position + Vec3::Y * 0.3)
                    .with_rotation(route_rotation(tangent)),
            ));
            distance += SEGMENT_LENGTH;
        }

        let mut island_distance = start + 4.0;
        while island_distance < end - 2.0 {
            let (position, _) = sample_route(island_distance);
            let wobble = hash01(island_distance as u32 * 17);
            let radius = 6.4 + wobble * 1.8;
            commands.spawn((
                Mesh3d(island_mesh.clone()),
                MeshMaterial3d(earth_material.clone()),
                Transform::from_translation(position - Vec3::Y * 0.55).with_scale(Vec3::new(
                    radius,
                    1.45 + wobble * 0.5,
                    radius,
                )),
            ));
            commands.spawn((
                Mesh3d(island_mesh.clone()),
                MeshMaterial3d(grass_material.clone()),
                Transform::from_translation(position + Vec3::Y * 0.06).with_scale(Vec3::new(
                    radius * 0.96,
                    0.22,
                    radius * 0.96,
                )),
                IslandFloor {
                    center: Vec2::new(position.x, position.z),
                    radius: radius * 0.96,
                    height: ISLAND_GROUND_Y,
                },
            ));
            island_distance += 7.0;
        }

        let mut tree_distance = start + 7.0;
        let mut tree_index = (start as u32) + 1;
        while tree_distance < end - 4.0 {
            let (center, tangent) = sample_route(tree_distance);
            let right = route_right(tangent);
            let side = if tree_index.is_multiple_of(2) {
                -1.0
            } else {
                1.0
            };
            let offset = side * (PATH_WIDTH * 0.5 + 2.0 + hash01(tree_index * 31) * 1.8);
            let tree_position = center + right * offset;
            commands.spawn((
                Mesh3d(trunk_mesh.clone()),
                MeshMaterial3d(trunk_material.clone()),
                Transform::from_translation(tree_position + Vec3::Y * 1.0),
            ));
            commands.spawn((
                Mesh3d(crown_mesh.clone()),
                MeshMaterial3d(leaf_material.clone()),
                Transform::from_translation(tree_position + Vec3::Y * 3.25)
                    .with_scale(Vec3::splat(0.85 + hash01(tree_index) * 0.35)),
            ));
            tree_index += 1;
            tree_distance += 8.5 + hash01(tree_index * 13) * 3.0;
        }
    }

    spawn_background_islands(
        commands,
        &island_mesh,
        &trunk_mesh,
        &crown_mesh,
        &earth_material,
        &grass_material,
        &trunk_material,
        &leaf_material,
    );
}

#[allow(clippy::too_many_arguments)]
fn spawn_background_islands(
    commands: &mut Commands,
    island_mesh: &Handle<Mesh>,
    trunk_mesh: &Handle<Mesh>,
    crown_mesh: &Handle<Mesh>,
    earth_material: &Handle<StandardMaterial>,
    grass_material: &Handle<StandardMaterial>,
    trunk_material: &Handle<StandardMaterial>,
    leaf_material: &Handle<StandardMaterial>,
) {
    // distance along route, lateral displacement, vertical displacement, radius
    const FIELD: [(f32, f32, f32, f32); 14] = [
        (18.0, -31.0, -6.0, 7.0),
        (31.0, 38.0, 8.0, 10.0),
        (48.0, -47.0, 4.0, 12.0),
        (62.0, 29.0, -10.0, 7.5),
        (79.0, 52.0, 13.0, 9.0),
        (96.0, -35.0, -2.0, 8.5),
        (112.0, 43.0, -7.0, 13.0),
        (128.0, -55.0, 11.0, 8.0),
        (145.0, 32.0, 5.0, 6.5),
        (162.0, -39.0, -11.0, 11.0),
        (179.0, 53.0, 1.0, 9.0),
        (195.0, -30.0, 14.0, 7.0),
        (213.0, 40.0, -5.0, 12.0),
        (229.0, -48.0, 7.0, 9.5),
    ];

    for (index, &(distance, lateral, height, radius)) in FIELD.iter().enumerate() {
        let (route_center, tangent) = sample_route(distance);
        let center = route_center + route_right(tangent) * lateral + Vec3::Y * height;
        commands.spawn((
            Mesh3d(island_mesh.clone()),
            MeshMaterial3d(earth_material.clone()),
            Transform::from_translation(center - Vec3::Y * 0.7).with_scale(Vec3::new(
                radius,
                2.2 + radius * 0.08,
                radius * 0.82,
            )),
        ));
        commands.spawn((
            Mesh3d(island_mesh.clone()),
            MeshMaterial3d(grass_material.clone()),
            Transform::from_translation(center + Vec3::Y * 0.05).with_scale(Vec3::new(
                radius * 0.96,
                0.25,
                radius * 0.78,
            )),
        ));

        // A little distant silhouette detail without turning these into routes.
        let tree_count = 1 + index % 3;
        for tree in 0..tree_count {
            let angle = (index as f32 * 2.17 + tree as f32 * 2.4).sin_cos();
            let tree_position =
                center + Vec3::new(angle.1, 0.0, angle.0) * (radius * (0.25 + tree as f32 * 0.18));
            commands.spawn((
                Mesh3d(trunk_mesh.clone()),
                MeshMaterial3d(trunk_material.clone()),
                Transform::from_translation(tree_position + Vec3::Y * 1.0),
            ));
            commands.spawn((
                Mesh3d(crown_mesh.clone()),
                MeshMaterial3d(leaf_material.clone()),
                Transform::from_translation(tree_position + Vec3::Y * 3.2)
                    .with_scale(Vec3::splat(0.8 + hash01((index * 7 + tree) as u32) * 0.3)),
            ));
        }
    }
}

fn procedural_dirt_textures(images: &mut Assets<Image>) -> (Handle<Image>, Handle<Image>) {
    const SIZE: u32 = 256;
    let mut color = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    let mut normal = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let noise = layered_noise(x, y);
            let grain = ((noise - 0.5) * 42.0) as i16;
            let mut rgb = [142_i16 + grain, 91 + grain / 2, 48 + grain / 3];
            let u = x as f32 / SIZE as f32;
            let lane_line = (u - 1.0 / 3.0).abs() < 0.012 || (u - 2.0 / 3.0).abs() < 0.012;
            if lane_line {
                rgb = [226, 204, 143];
            }
            color.extend(rgb.map(|v| v.clamp(0, 255) as u8));
            color.push(255);

            let left = layered_noise(x.saturating_sub(1), y);
            let right = layered_noise((x + 1).min(SIZE - 1), y);
            let down = layered_noise(x, y.saturating_sub(1));
            let up = layered_noise(x, (y + 1).min(SIZE - 1));
            let nx = ((left - right) * 95.0 + 128.0).clamp(0.0, 255.0) as u8;
            let ny = ((down - up) * 95.0 + 128.0).clamp(0.0, 255.0) as u8;
            normal.extend([nx, ny, 250, 255]);
        }
    }
    let descriptor = Extent3d {
        width: SIZE,
        height: SIZE,
        depth_or_array_layers: 1,
    };
    let sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        ..default()
    });
    let mut albedo = Image::new(
        descriptor,
        TextureDimension::D2,
        color,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    albedo.sampler = sampler.clone();
    let mut bump = Image::new(
        descriptor,
        TextureDimension::D2,
        normal,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    bump.sampler = sampler;
    (images.add(albedo), images.add(bump))
}

fn layered_noise(x: u32, y: u32) -> f32 {
    let fine = hash01(x.wrapping_mul(1_597_334_677) ^ y.wrapping_mul(3_812_015_801));
    let medium = hash01((x / 5).wrapping_mul(747_796_405) ^ (y / 5).wrapping_mul(2_891_336_453));
    let broad = hash01((x / 19).wrapping_mul(277_803_737) ^ (y / 19).wrapping_mul(1_443_059_021));
    fine * 0.28 + medium * 0.42 + broad * 0.30
}

fn hash01(mut value: u32) -> f32 {
    value ^= value >> 16;
    value = value.wrapping_mul(0x7feb_352d);
    value ^= value >> 15;
    value = value.wrapping_mul(0x846c_a68b);
    value ^= value >> 16;
    value as f32 / u32::MAX as f32
}

fn sample_route(distance: f32) -> (Vec3, Vec3) {
    let max_segment = SPLINE_X.len() as i32 - 3;
    let scaled = (distance.max(0.0) / KNOT_DISTANCE).min(max_segment as f32 - 0.001);
    let segment = scaled.floor() as usize;
    let t = scaled.fract();
    let point =
        |index: usize| Vec3::new(SPLINE_X[index], 0.0, -(index as f32 - 1.0) * KNOT_DISTANCE);
    let p0 = point(segment);
    let p1 = point(segment + 1);
    let p2 = point(segment + 2);
    let p3 = point(segment + 3);
    let t2 = t * t;
    let t3 = t2 * t;
    let position = 0.5
        * ((2.0 * p1)
            + (-p0 + p2) * t
            + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
            + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3);
    let tangent = (0.5
        * ((-p0 + p2)
            + 2.0 * (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t
            + 3.0 * (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t2))
        .normalize_or_zero();
    (position, tangent)
}

fn route_right(tangent: Vec3) -> Vec3 {
    Vec3::new(-tangent.z, 0.0, tangent.x).normalize_or_zero()
}

fn route_rotation(tangent: Vec3) -> Quat {
    // The imported character's corrected local forward is -Z. Rotate that axis
    // onto the spline tangent so it keeps facing into every bend.
    Quat::from_rotation_y((-tangent.x).atan2(-tangent.z))
}

fn floor_height(distance: f32, position: Vec3, floors: &Query<&IslandFloor>) -> Option<f32> {
    // The raised dirt path is its own support surface.
    if ISLAND_SPANS
        .iter()
        .any(|&(start, end)| distance >= start && distance <= end)
    {
        return Some(GROUND_Y);
    }

    // Outside the path, collide against the actual circular island pieces.
    // These centers/radii are copied directly from the rendered grass discs.
    let horizontal = Vec2::new(position.x, position.z);
    floors
        .iter()
        .filter(|floor| horizontal.distance_squared(floor.center) <= floor.radius * floor.radius)
        .map(|floor| floor.height)
        .max_by(f32::total_cmp)
}

fn read_local_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    touches: Res<Touches>,
    mut swipe: ResMut<SwipeGesture>,
    mut intent: ResMut<ActorIntent>,
) {
    intent.lane_delta = 0;
    intent.jump_pressed = keyboard.just_pressed(KeyCode::Space);
    intent.rope_held = keyboard.pressed(KeyCode::KeyF)
        || keyboard.pressed(KeyCode::KeyE)
        || touches.iter().next().is_some();
    if keyboard.just_pressed(KeyCode::ArrowLeft) || keyboard.just_pressed(KeyCode::KeyA) {
        intent.lane_delta = -1;
    }
    if keyboard.just_pressed(KeyCode::ArrowRight) || keyboard.just_pressed(KeyCode::KeyD) {
        intent.lane_delta = 1;
    }
    for touch in touches.iter_just_pressed() {
        swipe.start = Some((touch.id(), touch.position()));
    }
    for touch in touches.iter_just_released() {
        if let Some((_, start)) = swipe.start.take().filter(|(id, _)| *id == touch.id()) {
            let delta = touch.position() - start;
            if delta.length() > 45.0 {
                if delta.x.abs() > delta.y.abs() {
                    intent.lane_delta = if delta.x < 0.0 { -1 } else { 1 };
                } else if delta.y < 0.0 {
                    intent.jump_pressed = true;
                }
            }
        }
    }
}

fn drive_actor(
    time: Res<Time>,
    intent: Res<ActorIntent>,
    floors: Query<&IslandFloor>,
    anchors: Query<&RopeAnchor>,
    mut actors: Query<(&mut RunnerActor, &mut Transform)>,
) {
    for (mut actor, mut transform) in &mut actors {
        actor.lane = (actor.lane + intent.lane_delta).clamp(-(LANE_COUNT / 2), LANE_COUNT / 2);
        if !intent.rope_held && actor.rope.is_some() {
            release_rope(&mut actor);
        }
        if intent.rope_held && actor.airborne && actor.rope.is_none() {
            let nearest = anchors
                .iter()
                .filter(|anchor| anchor.distance >= actor.distance - 2.0)
                .min_by(|a, b| {
                    (a.distance - actor.distance)
                        .abs()
                        .total_cmp(&(b.distance - actor.distance).abs())
                });
            if let Some(anchor) = nearest
                && (anchor.distance - actor.distance).abs() <= ROPE_RANGE
            {
                let dx = actor.distance - anchor.distance;
                let dy = transform.translation.y - anchor.height;
                let length = (dx * dx + dy * dy).sqrt().clamp(5.5, ROPE_RANGE);
                let angle = dx.atan2(-dy);
                let tangent_factor = (length * angle.cos()).abs().max(1.0);
                actor.rope = Some(RopeSwing {
                    anchor_distance: anchor.distance,
                    anchor_height: anchor.height,
                    length,
                    angle,
                    angular_speed: actor.forward_speed / tangent_factor,
                });
                actor.gliding = false;
            }
        }
        if intent.jump_pressed
            && !actor.airborne
            && floor_height(actor.distance, transform.translation, &floors).is_some()
        {
            actor.airborne = true;
            actor.vertical_speed = JUMP_SPEED;
        } else if intent.jump_pressed && actor.airborne {
            actor.gliding = !actor.gliding;
            if actor.gliding {
                actor.vertical_speed = actor.vertical_speed.min(GLIDE_DEPLOY_LIFT);
            }
        }
        let forward_speed = if actor.gliding {
            GLIDE_SPEED
        } else {
            actor.forward_speed
        };
        if let Some(mut rope) = actor.rope {
            let dt = time.delta_secs();
            let acceleration = -(ROPE_GRAVITY / rope.length) * rope.angle.sin();
            rope.angular_speed += acceleration * dt;
            rope.angular_speed *= (-ROPE_DRAG * dt).exp();
            rope.angle += rope.angular_speed * dt;
            actor.distance = rope.anchor_distance + rope.length * rope.angle.sin();
            transform.translation.y = rope.anchor_height - rope.length * rope.angle.cos();
            actor.rope = Some(rope);
        } else {
            actor.distance += forward_speed * time.delta_secs();
            actor.forward_speed +=
                (RUN_SPEED - actor.forward_speed) * (2.2 * time.delta_secs()).min(1.0);
        }
        actor.lane_offset += (actor.lane as f32 * LANE_WIDTH - actor.lane_offset)
            * (12.0 * time.delta_secs()).min(1.0);

        let (center, tangent) = sample_route(actor.distance);
        let horizontal = center + route_right(tangent) * actor.lane_offset;
        transform.translation.x = horizontal.x;
        transform.translation.z = horizontal.z;
        transform.rotation = route_rotation(tangent);

        let landing_height = floor_height(actor.distance, transform.translation, &floors);
        if let Some(height) = landing_height
            && let Some(rope) = actor.rope
            && transform.translation.y <= height
            && rope.angle.sin() * rope.angular_speed <= 0.0
        {
            actor.rope = None;
            actor.airborne = false;
            actor.gliding = false;
            actor.vertical_speed = 0.0;
            actor.forward_speed = RUN_SPEED;
            transform.translation.y = height;
        }
        if landing_height.is_none() && !actor.airborne {
            actor.airborne = true;
            actor.vertical_speed = 0.0;
        } else if let Some(height) = landing_height
            && !actor.airborne
        {
            // Follow the small step between the island turf and raised path.
            transform.translation.y = height;
        }
        if actor.airborne && actor.rope.is_none() {
            if actor.gliding {
                actor.vertical_speed = (actor.vertical_speed - GLIDE_GRAVITY * time.delta_secs())
                    .max(-GLIDE_MAX_FALL_SPEED);
            } else {
                actor.vertical_speed -= GRAVITY * time.delta_secs();
            }
            transform.translation.y += actor.vertical_speed * time.delta_secs();
            if let Some(height) = landing_height
                && transform.translation.y <= height
                && actor.vertical_speed <= 0.0
            {
                transform.translation.y = height;
                actor.vertical_speed = 0.0;
                actor.airborne = false;
                actor.gliding = false;
                actor.forward_speed = actor.forward_speed.max(RUN_SPEED);
            }
        }
        if transform.translation.y < -12.0 || actor.distance >= ROUTE_LENGTH - 1.0 {
            actor.distance = 3.0;
            actor.lane = 0;
            actor.lane_offset = 0.0;
            actor.vertical_speed = 0.0;
            actor.airborne = false;
            actor.gliding = false;
            actor.forward_speed = RUN_SPEED;
            actor.rope = None;
            let (reset, reset_tangent) = sample_route(actor.distance);
            transform.translation = reset + Vec3::Y * GROUND_Y;
            transform.rotation = route_rotation(reset_tangent);
        }
    }
}

fn release_rope(actor: &mut RunnerActor) {
    if let Some(rope) = actor.rope.take() {
        actor.forward_speed = (rope.length * rope.angle.cos() * rope.angular_speed)
            .max(RUN_SPEED * 0.65)
            .min(18.0);
        actor.vertical_speed = rope.length * rope.angle.sin() * rope.angular_speed;
    }
}

fn sync_rope_visual(
    actors: Query<(&RunnerActor, &Transform)>,
    mut visuals: Query<(&mut Transform, &mut Visibility), (With<RopeVisual>, Without<RunnerActor>)>,
) {
    let Ok((actor, transform)) = actors.single() else {
        return;
    };
    let Ok((mut rope_transform, mut visibility)) = visuals.single_mut() else {
        return;
    };
    if let Some(rope) = actor.rope {
        let (anchor, _) = sample_route(rope.anchor_distance);
        let start = transform.translation + Vec3::Y * 1.35;
        let end = anchor + Vec3::Y * rope.anchor_height;
        let delta = end - start;
        rope_transform.translation = (start + end) * 0.5;
        rope_transform.rotation = Quat::from_rotation_arc(Vec3::Y, delta.normalize_or_zero());
        rope_transform.scale = Vec3::new(1.0, delta.length(), 1.0);
        *visibility = Visibility::Visible;
    } else {
        *visibility = Visibility::Hidden;
    }
}

fn sync_glider_visual(
    actors: Query<&RunnerActor>,
    mut visuals: Query<&mut Visibility, With<GliderVisual>>,
) {
    let Ok(actor) = actors.single() else {
        return;
    };
    let visibility = if actor.gliding {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    for mut visual in &mut visuals {
        *visual = visibility;
    }
}

fn follow_actor(
    actors: Query<(&RunnerActor, &Transform), (With<RunnerActor>, Without<RunnerCamera>)>,
    mut cameras: Query<&mut Transform, (With<RunnerCamera>, Without<RunnerActor>)>,
) {
    let Ok((actor, transform)) = actors.single() else {
        return;
    };
    let (_, tangent) = sample_route(actor.distance);
    let desired = transform.translation - tangent * 9.0 + Vec3::Y * 4.7;
    let focus = transform.translation + tangent * 8.0 + Vec3::Y * 1.1;
    for mut camera in &mut cameras {
        camera.translation = camera.translation.lerp(desired, 0.1);
        camera.look_at(focus, Vec3::Y);
    }
}

fn start_character_animation(
    mut commands: Commands,
    animations: Res<CharacterAnimations>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut player) in &mut players {
        let mut transitions = AnimationTransitions::new();
        transitions
            .play(&mut player, animations.run, Duration::ZERO)
            .repeat();
        commands
            .entity(entity)
            .insert((AnimationGraphHandle(animations.graph.clone()), transitions));
    }
}

fn update_character_animation(
    actors: Query<&RunnerActor>,
    animations: Res<CharacterAnimations>,
    mut state: ResMut<AnimationState>,
    mut players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
) {
    let Ok(actor) = actors.single() else { return };
    if actor.airborne == state.jumping {
        return;
    }
    state.jumping = actor.airborne;
    for (mut player, mut transitions) in &mut players {
        let clip = if actor.airborne {
            animations.jump
        } else {
            animations.run
        };
        transitions
            .play(&mut player, clip, Duration::from_millis(90))
            .repeat();
    }
}
