use std::time::Duration;

use bevy::{
    animation::{AnimatedBy, AnimationTargetId},
    input::touch::Touches,
    prelude::*,
};

const CHARACTER: &str = "characters/kaykit/Knight.glb";
const LANE_WIDTH: f32 = 2.6;
const LANE_COUNT: i32 = 3;
const RUN_SPEED: f32 = 9.0;
const GROUND_Y: f32 = 0.5;
const JUMP_SPEED: f32 = 8.5;
const GRAVITY: f32 = 22.0;
const TILE_LENGTH: f32 = 16.0;
const TILE_COUNT: usize = 10;

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
                    follow_actor,
                    recycle_track,
                    start_character_animation,
                    update_character_animation,
                )
                    .chain(),
            );
    }
}

/// Device-independent commands consumed by runner movement.
#[derive(Resource, Default)]
pub struct ActorIntent {
    lane_delta: i32,
    jump_pressed: bool,
}

#[derive(Resource, Default)]
struct SwipeGesture {
    start: Option<(u64, Vec2)>,
}

#[derive(Component)]
pub struct RunnerActor {
    lane: i32,
    vertical_speed: f32,
    airborne: bool,
}

#[derive(Component)]
struct RunnerCamera;

#[derive(Component)]
struct TrackTile;

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
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    commands.insert_resource(ClearColor(Color::srgb(0.025, 0.055, 0.09)));
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.6, 0.78, 1.0),
        brightness: 900.0,
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
                lane: 0,
                vertical_speed: 0.0,
                airborne: false,
            },
            Transform::from_xyz(0.0, GROUND_Y, 0.0),
            Visibility::default(),
        ))
        .id();
    commands.entity(actor).with_child((
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(CHARACTER))),
        Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
    ));

    commands.spawn((
        RunnerCamera,
        Camera3d::default(),
        Transform::from_xyz(0.0, 4.8, 8.5).looking_at(Vec3::new(0.0, 1.5, -7.0), Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 16_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, -0.5, 0.0)),
    ));

    let track_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.12, 0.2, 0.26),
        perceptual_roughness: 0.86,
        ..default()
    });
    let stripe_material = materials.add(Color::srgb(0.25, 0.8, 0.9));
    let rail_material = materials.add(Color::srgb(0.91, 0.48, 0.18));
    let track_mesh = meshes.add(Cuboid::new(
        LANE_WIDTH * LANE_COUNT as f32 + 1.2,
        0.45,
        TILE_LENGTH - 0.12,
    ));
    let stripe_mesh = meshes.add(Cuboid::new(0.055, 0.02, TILE_LENGTH - 0.5));
    let rail_mesh = meshes.add(Cuboid::new(0.28, 0.65, TILE_LENGTH - 0.12));
    let crate_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));

    for index in 0..TILE_COUNT {
        let z = -(index as f32) * TILE_LENGTH;
        commands
            .spawn((
                TrackTile,
                Transform::from_xyz(0.0, 0.0, z),
                Visibility::default(),
            ))
            .with_children(|tile| {
                tile.spawn((
                    Mesh3d(track_mesh.clone()),
                    MeshMaterial3d(track_material.clone()),
                ));
                for boundary in [-LANE_WIDTH / 2.0, LANE_WIDTH / 2.0] {
                    tile.spawn((
                        Mesh3d(stripe_mesh.clone()),
                        MeshMaterial3d(stripe_material.clone()),
                        Transform::from_xyz(boundary, 0.235, 0.0),
                    ));
                }
                for side in [-1.0, 1.0] {
                    tile.spawn((
                        Mesh3d(rail_mesh.clone()),
                        MeshMaterial3d(rail_material.clone()),
                        Transform::from_xyz(side * (LANE_WIDTH * 1.5 + 0.5), 0.35, 0.0),
                    ));
                }
                if index % 2 == 1 {
                    let side = if index % 4 == 1 { -1.0 } else { 1.0 };
                    tile.spawn((
                        Mesh3d(crate_mesh.clone()),
                        MeshMaterial3d(rail_material.clone()),
                        Transform::from_xyz(side * (LANE_WIDTH * 1.5 + 1.35), 0.65, -2.0),
                    ));
                }
            });
    }

    commands.spawn((
        Text::new("SKYWAY RUN  •  ← → / A D change lane  •  SPACE jump  •  swipe on mobile"),
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

fn read_local_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    touches: Res<Touches>,
    mut swipe: ResMut<SwipeGesture>,
    mut intent: ResMut<ActorIntent>,
) {
    intent.lane_delta = 0;
    intent.jump_pressed = keyboard.just_pressed(KeyCode::Space);
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
        if let Some((id, start)) = swipe.start.take().filter(|(id, _)| *id == touch.id()) {
            let _ = id;
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
    mut actors: Query<(&mut RunnerActor, &mut Transform)>,
) {
    for (mut actor, mut transform) in &mut actors {
        actor.lane = (actor.lane + intent.lane_delta).clamp(-(LANE_COUNT / 2), LANE_COUNT / 2);
        if intent.jump_pressed && !actor.airborne {
            actor.airborne = true;
            actor.vertical_speed = JUMP_SPEED;
        }

        transform.translation.z -= RUN_SPEED * time.delta_secs();
        let target_x = actor.lane as f32 * LANE_WIDTH;
        transform.translation.x +=
            (target_x - transform.translation.x) * (12.0 * time.delta_secs()).min(1.0);
        if actor.airborne {
            actor.vertical_speed -= GRAVITY * time.delta_secs();
            transform.translation.y += actor.vertical_speed * time.delta_secs();
            if transform.translation.y <= GROUND_Y {
                transform.translation.y = GROUND_Y;
                actor.vertical_speed = 0.0;
                actor.airborne = false;
            }
        }
    }
}

fn follow_actor(
    actors: Query<&Transform, (With<RunnerActor>, Without<RunnerCamera>)>,
    mut cameras: Query<&mut Transform, (With<RunnerCamera>, Without<RunnerActor>)>,
) {
    let Ok(actor) = actors.single() else { return };
    for mut camera in &mut cameras {
        let desired = actor.translation + Vec3::new(0.0, 4.3, 8.5);
        camera.translation = camera.translation.lerp(desired, 0.12);
        camera.look_at(actor.translation + Vec3::new(0.0, 1.1, -7.0), Vec3::Y);
    }
}

fn recycle_track(
    actors: Query<&Transform, With<RunnerActor>>,
    mut tiles: Query<&mut Transform, (With<TrackTile>, Without<RunnerActor>)>,
) {
    let Ok(actor) = actors.single() else { return };
    for mut tile in &mut tiles {
        if tile.translation.z > actor.translation.z + TILE_LENGTH {
            tile.translation.z -= TILE_LENGTH * TILE_COUNT as f32;
        }
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
