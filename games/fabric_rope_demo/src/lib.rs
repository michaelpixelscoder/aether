use std::f32::consts::PI;

use aether_app::AetherAppPlugin;
use aether_flexible::{
    ConstraintKind, FabricSpec, FabricTopology, FlexibleGraph, FlexibleNode, RopeTopology,
    SolverConfig, build_fabric, build_rope,
};
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::time::Fixed;

const NAVY: Color = Color::srgb(0.018, 0.038, 0.075);
const PURPLE: Color = Color::srgb(0.34, 0.10, 0.66);
const PURPLE_LIGHT: Color = Color::srgb(0.62, 0.30, 0.96);
const GOLD: Color = Color::srgb(0.84, 0.57, 0.18);
const ROPE: Color = Color::srgb(0.38, 0.20, 0.08);
const PARCHMENT: Color = Color::srgb(0.96, 0.88, 0.69);
const FABRIC_WIDTH: usize = 16;
const FABRIC_HEIGHT: usize = 12;
const SPACING: f32 = 0.42;
const DEFAULT_TILE_SIZE: f32 = 0.36;
const DEFAULT_STITCH_GAP: f32 = 0.025;
const DEFAULT_STITCH_DAMPING: f32 = 0.025;
const DEFAULT_THICKNESS: f32 = 0.10;

pub fn configure(app: &mut App) {
    app.add_plugins(AetherAppPlugin {
        title: "Aether Isles — Fabric & Rope Demo",
    })
    .insert_resource(ClearColor(NAVY))
    .insert_resource(Time::<Fixed>::from_hz(60.0))
    .insert_resource(DemoState::canonical())
    .add_systems(Startup, setup)
    .add_systems(FixedUpdate, simulate)
    .add_systems(
        Update,
        (
            controls,
            orbit_camera,
            update_fabric_tiles,
            update_links,
            update_anchors,
            update_hud,
        )
            .chain(),
    );
}

pub fn run() {
    let mut app = App::new();
    configure(&mut app);
    app.run();
}

#[cfg(not(target_arch = "wasm32"))]
/// # Safety
///
/// `app` must be a valid, exclusive pointer to a Bevy [`App`] owned by the native game loader.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aether_register_game(app: *mut App) {
    let app = unsafe { app.as_mut() }.expect("game loader passed a null app pointer");
    configure(app);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum TileMode {
    #[default]
    Free,
    Quantized,
    AxisAligned,
}

impl TileMode {
    fn label(self) -> &'static str {
        match self {
            Self::Free => "free local orientation",
            Self::Quantized => "quantized orientation",
            Self::AxisAligned => "globally axis-aligned",
        }
    }
}

#[derive(Resource)]
struct DemoState {
    graph: FlexibleGraph,
    fabric: FabricTopology,
    ropes: Vec<RopeTopology>,
    rendered_links: Vec<usize>,
    anchor_nodes: Vec<usize>,
    elapsed: f32,
    anchor_offset: f32,
    wind: bool,
    paused: bool,
    single_step: bool,
    tile_mode: TileMode,
    cuts: u32,
    tile_size: f32,
    stitch_gap: f32,
    stitch_damping: f32,
    thickness: f32,
}

impl DemoState {
    fn canonical() -> Self {
        let mut graph = FlexibleGraph::default();
        let origin = Vec3::new(-2.31, 0.15, 0.0);
        let fabric = build_fabric(
            &mut graph,
            FabricSpec {
                bottom_left: origin,
                width: FABRIC_WIDTH,
                height: FABRIC_HEIGHT,
                spacing: SPACING,
                mass_per_node: 0.18,
                compliance: 1.5e-6,
                include_shear: true,
            },
        );

        let fabric_corners = [
            fabric.node(0, FABRIC_HEIGHT - 1),
            fabric.node(FABRIC_WIDTH - 1, FABRIC_HEIGHT - 1),
        ];
        let anchor_positions = [Vec3::new(-3.25, 4.85, 0.0), Vec3::new(3.25, 4.85, 0.0)];
        let mut ropes = Vec::new();
        let mut rendered_links = Vec::new();
        let mut anchor_nodes = Vec::new();

        for (side, (&corner, &anchor)) in fabric_corners.iter().zip(&anchor_positions).enumerate() {
            let corner_position = graph.nodes[corner].position;
            let direction = (corner_position - anchor).normalize();
            let rope_end = corner_position - direction * 0.22;
            let rope = build_rope(&mut graph, anchor, rope_end, 5, 0.12, 8.0e-7);
            let first = rope.nodes[0];
            graph.nodes[first] = FlexibleNode::fixed(anchor);
            anchor_nodes.push(first);
            rendered_links.extend(rope.constraints.iter().copied());
            let tether = graph.add_distance(
                *rope.nodes.last().expect("rope has nodes"),
                corner,
                0.22,
                5.0e-7,
                ConstraintKind::Tether,
            );
            graph.constraints[tether].distance.breaking_strain = 0.55;
            rendered_links.push(tether);
            ropes.push(rope);

            // The right anchor is the user-controlled one.
            if side == 1 {
                graph.set_fixed_position(first, anchor);
            }
        }

        Self {
            graph,
            fabric,
            ropes,
            rendered_links,
            anchor_nodes,
            elapsed: 0.0,
            anchor_offset: 0.0,
            wind: true,
            paused: false,
            single_step: false,
            tile_mode: TileMode::Free,
            cuts: 0,
            tile_size: DEFAULT_TILE_SIZE,
            stitch_gap: DEFAULT_STITCH_GAP,
            stitch_damping: DEFAULT_STITCH_DAMPING,
            thickness: DEFAULT_THICKNESS,
        }
    }

    fn reset(&mut self) {
        *self = Self::canonical();
    }

    fn tile_side(&self) -> f32 {
        (self.tile_size - self.stitch_gap).max(0.04)
    }
}

#[derive(Component)]
struct FabricTile {
    node: usize,
    x: usize,
    y: usize,
}

#[derive(Component)]
struct FlexibleLink {
    constraint: usize,
}

#[derive(Component)]
struct AnchorMarker {
    node: usize,
}

#[derive(Component)]
struct Hud;

#[derive(Component)]
struct OrbitCamera {
    yaw: f32,
    pitch: f32,
    radius: f32,
    target_radius: f32,
    center: Vec3,
}

fn setup(
    mut commands: Commands,
    state: Res<DemoState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 3.0, 10.5).looking_at(Vec3::new(0.0, 2.3, 0.0), Vec3::Y),
        OrbitCamera {
            yaw: 0.0,
            pitch: -0.07,
            radius: 10.5,
            target_radius: 10.5,
            center: Vec3::new(0.0, 2.3, 0.0),
        },
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 12_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.7, -0.5, 0.0)),
    ));
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.50, 0.58, 0.78),
        brightness: 360.0,
        affects_lightmapped_meshes: true,
    });

    let tile_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let tile_material = materials.add(StandardMaterial {
        base_color: PURPLE,
        perceptual_roughness: 0.72,
        metallic: 0.08,
        ..default()
    });
    for y in 0..state.fabric.height {
        for x in 0..state.fabric.width {
            let node = state.fabric.node(x, y);
            commands.spawn((
                Mesh3d(tile_mesh.clone()),
                MeshMaterial3d(tile_material.clone()),
                Transform::from_translation(state.graph.nodes[node].position).with_scale(
                    Vec3::new(state.tile_side(), state.tile_side(), state.thickness),
                ),
                FabricTile { node, x, y },
            ));
        }
    }

    let link_mesh = meshes.add(Cuboid::new(0.13, 0.13, 1.0));
    let rope_material = materials.add(StandardMaterial {
        base_color: ROPE,
        perceptual_roughness: 0.9,
        ..default()
    });
    for &constraint in &state.rendered_links {
        commands.spawn((
            Mesh3d(link_mesh.clone()),
            MeshMaterial3d(rope_material.clone()),
            Transform::default(),
            FlexibleLink { constraint },
        ));
    }

    let anchor_mesh = meshes.add(Cuboid::from_size(Vec3::splat(0.34)));
    let anchor_material = materials.add(StandardMaterial {
        base_color: GOLD,
        metallic: 0.65,
        perceptual_roughness: 0.28,
        ..default()
    });
    for &node in &state.anchor_nodes {
        commands.spawn((
            Mesh3d(anchor_mesh.clone()),
            MeshMaterial3d(anchor_material.clone()),
            Transform::from_translation(state.graph.nodes[node].position),
            AnchorMarker { node },
        ));
    }

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 20.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.025, 0.065, 0.10),
            perceptual_roughness: 1.0,
            ..default()
        })),
        Transform::from_xyz(0.0, -0.55, 0.0),
    ));

    spawn_ui(&mut commands);
}

fn spawn_ui(commands: &mut Commands) {
    commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                padding: UiRect::all(px(20)),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexStart,
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|root| {
            root.spawn(Node {
                position_type: PositionType::Absolute,
                left: px(20),
                top: px(20),
                flex_direction: FlexDirection::Column,
                row_gap: px(7),
                padding: UiRect::all(px(16)),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(8)),
                ..default()
            })
            .insert(BackgroundColor(Color::srgba(0.015, 0.035, 0.07, 0.92)))
            .insert(BorderColor::all(GOLD))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("FABRIC & ROPE LAB"),
                    TextFont::from_font_size(24.0),
                    TextColor(PARCHMENT),
                ));
                panel.spawn((
                    Text::new("Initializing"),
                    TextFont::from_font_size(14.0),
                    TextColor(PURPLE_LIGHT),
                    Hud,
                ));
                panel.spawn((
                    Text::new("Right-drag orbit · Shift-right/middle pan · Wheel zoom\nA/D move anchor · W wind · X cut rope · T tear fabric · 1/2/3 tiles · P pause · N step · R reset\n[/] tile size · ,/. stitch gap · -/= stitch damping · ;/' thickness"),
                    TextFont::from_font_size(13.0),
                    TextColor(Color::srgb(0.76, 0.80, 0.88)),
                ));
            });

            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(20),
                    right: px(20),
                    bottom: px(-100),
                    padding: UiRect::axes(px(16), px(10)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(18)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.015, 0.035, 0.07, 0.92)),
                BorderColor::all(Color::srgba(0.84, 0.57, 0.18, 0.65)),
            ))
            .with_child((
                Text::new("Right-drag orbit · Shift-right/middle pan · Wheel zoom\nA/D move anchor · W wind · X cut rope · T tear fabric · 1/2/3 tiles · P pause · N step · R reset\n[/] tile size · ,/. stitch gap · -/= stitch damping · ;/' thickness"),
                TextFont::from_font_size(13.0),
                TextColor(Color::srgb(0.76, 0.80, 0.88)),
            ));
        });
}

fn orbit_camera(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut motion: MessageReader<MouseMotion>,
    mut wheel: MessageReader<MouseWheel>,
    mut cameras: Query<(&mut OrbitCamera, &mut Transform)>,
    time: Res<Time>,
) {
    let delta = motion.read().map(|event| event.delta).sum::<Vec2>();
    let scroll = wheel.read().map(|event| event.y).sum::<f32>();
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    for (mut orbit, mut transform) in &mut cameras {
        let panning =
            buttons.pressed(MouseButton::Middle) || (buttons.pressed(MouseButton::Right) && shift);
        if panning && delta != Vec2::ZERO {
            let right = transform.rotation * Vec3::X;
            let up = transform.rotation * Vec3::Y;
            let pan_scale = orbit.radius * 0.001;
            orbit.center -= (right * delta.x - up * delta.y) * pan_scale;
        } else if buttons.pressed(MouseButton::Right) && delta != Vec2::ZERO {
            orbit.yaw -= delta.x * 0.007;
            orbit.pitch = (orbit.pitch - delta.y * 0.007).clamp(-1.45, 1.35);
        }
        if scroll != 0.0 {
            orbit.target_radius =
                (orbit.target_radius * 0.90_f32.powf(scroll.clamp(-3.0, 3.0))).clamp(3.5, 30.0);
        }
        let smoothing = 1.0 - (-12.0 * time.delta_secs()).exp();
        orbit.radius += (orbit.target_radius - orbit.radius) * smoothing;
        let offset = Quat::from_euler(EulerRot::YXZ, orbit.yaw, orbit.pitch, 0.0)
            * Vec3::new(0.0, 0.0, orbit.radius);
        *transform =
            Transform::from_translation(orbit.center + offset).looking_at(orbit.center, Vec3::Y);
    }
}

fn simulate(time: Res<Time<Fixed>>, mut state: ResMut<DemoState>) {
    if state.paused && !state.single_step {
        return;
    }
    state.single_step = false;
    let dt = time.delta_secs();
    state.elapsed += dt;

    let right_anchor = Vec3::new(3.25 + state.anchor_offset, 4.85, 0.0);
    let right_node = state.anchor_nodes[1];
    state.graph.set_fixed_position(right_node, right_anchor);

    let mut accelerations = vec![Vec3::new(0.0, -9.81, 0.0); state.graph.nodes.len()];
    if state.wind {
        let gust = 5.2 + (state.elapsed * 1.7).sin() * 1.2;
        for &node in &state.fabric.nodes {
            accelerations[node].z += gust;
        }
    }
    let stitch_damping = state.stitch_damping;
    state.graph.step(
        dt,
        &accelerations,
        SolverConfig {
            substeps: 2,
            iterations: 10,
            damping: stitch_damping,
        },
    );
}

fn controls(keys: Res<ButtonInput<KeyCode>>, time: Res<Time>, mut state: ResMut<DemoState>) {
    let direction = f32::from(keys.pressed(KeyCode::KeyD)) - f32::from(keys.pressed(KeyCode::KeyA));
    state.anchor_offset =
        (state.anchor_offset + direction * time.delta_secs() * 1.2).clamp(-1.2, 1.2);
    if keys.just_pressed(KeyCode::KeyW) {
        state.wind = !state.wind;
    }
    if keys.just_pressed(KeyCode::KeyP) {
        state.paused = !state.paused;
    }
    if keys.just_pressed(KeyCode::KeyN) {
        state.single_step = true;
    }
    if keys.just_pressed(KeyCode::Digit1) {
        state.tile_mode = TileMode::Free;
    }
    if keys.just_pressed(KeyCode::Digit2) {
        state.tile_mode = TileMode::Quantized;
    }
    if keys.just_pressed(KeyCode::Digit3) {
        state.tile_mode = TileMode::AxisAligned;
    }
    if keys.just_pressed(KeyCode::KeyR) {
        state.reset();
    }
    if keys.just_pressed(KeyCode::BracketLeft) {
        state.tile_size = (state.tile_size - 0.02).max(0.08);
    }
    if keys.just_pressed(KeyCode::BracketRight) {
        state.tile_size = (state.tile_size + 0.02).min(SPACING);
    }
    if keys.just_pressed(KeyCode::Comma) {
        state.stitch_gap = (state.stitch_gap - 0.01).max(0.0);
    }
    if keys.just_pressed(KeyCode::Period) {
        state.stitch_gap = (state.stitch_gap + 0.01).min(state.tile_size - 0.04);
    }
    if keys.just_pressed(KeyCode::Minus) {
        state.stitch_damping = (state.stitch_damping - 0.01).max(0.0);
    }
    if keys.just_pressed(KeyCode::Equal) {
        state.stitch_damping = (state.stitch_damping + 0.01).min(0.25);
    }
    if keys.just_pressed(KeyCode::Semicolon) {
        state.thickness = (state.thickness - 0.02).max(0.02);
    }
    if keys.just_pressed(KeyCode::Quote) {
        state.thickness = (state.thickness + 0.02).min(0.35);
    }
    if keys.just_pressed(KeyCode::KeyX) {
        let constraint = state.ropes[1].constraints[state.ropes[1].constraints.len() / 2];
        if !state.graph.constraints[constraint].distance.broken {
            state.graph.cut_constraint(constraint);
            state.cuts += 1;
        }
    }
    if keys.just_pressed(KeyCode::KeyT) {
        let x = FABRIC_WIDTH / 2;
        let y = FABRIC_HEIGHT / 2;
        let a = state.fabric.node(x, y);
        let b = state.fabric.node(x + 1, y);
        if let Some(index) = state.graph.constraints.iter().position(|constraint| {
            !constraint.distance.broken
                && ((constraint.distance.a == a && constraint.distance.b == b)
                    || (constraint.distance.a == b && constraint.distance.b == a))
        }) {
            state.graph.cut_constraint(index);
            state.cuts += 1;
        }
    }
}

fn update_fabric_tiles(state: Res<DemoState>, mut tiles: Query<(&FabricTile, &mut Transform)>) {
    for (tile, mut transform) in &mut tiles {
        transform.translation = state.graph.nodes[tile.node].position;
        transform.scale = Vec3::new(state.tile_side(), state.tile_side(), state.thickness);
        transform.rotation = match state.tile_mode {
            TileMode::AxisAligned => Quat::IDENTITY,
            TileMode::Free => tile_rotation(&state, tile.x, tile.y),
            TileMode::Quantized => {
                let rotation = tile_rotation(&state, tile.x, tile.y);
                let (x, y, z) = rotation.to_euler(EulerRot::XYZ);
                let snap = PI / 8.0;
                Quat::from_euler(
                    EulerRot::XYZ,
                    (x / snap).round() * snap,
                    (y / snap).round() * snap,
                    (z / snap).round() * snap,
                )
            }
        };
    }
}

fn tile_rotation(state: &DemoState, x: usize, y: usize) -> Quat {
    let left = state.fabric.node(x.saturating_sub(1), y);
    let right = state.fabric.node((x + 1).min(state.fabric.width - 1), y);
    let down = state.fabric.node(x, y.saturating_sub(1));
    let up = state.fabric.node(x, (y + 1).min(state.fabric.height - 1));
    let tangent_x = (state.graph.nodes[right].position - state.graph.nodes[left].position)
        .normalize_or(Vec3::X);
    let approximate_y =
        (state.graph.nodes[up].position - state.graph.nodes[down].position).normalize_or(Vec3::Y);
    let normal = tangent_x.cross(approximate_y).normalize_or(Vec3::Z);
    let tangent_y = normal.cross(tangent_x).normalize_or(Vec3::Y);
    Quat::from_mat3(&Mat3::from_cols(tangent_x, tangent_y, normal))
}

fn update_links(
    state: Res<DemoState>,
    mut links: Query<(&FlexibleLink, &mut Transform, &mut Visibility)>,
) {
    for (link, mut transform, mut visibility) in &mut links {
        let constraint = &state.graph.constraints[link.constraint].distance;
        if constraint.broken {
            *visibility = Visibility::Hidden;
            continue;
        }
        *visibility = Visibility::Visible;
        let a = state.graph.nodes[constraint.a].position;
        let b = state.graph.nodes[constraint.b].position;
        let delta = b - a;
        let length = delta.length();
        transform.translation = (a + b) * 0.5;
        transform.rotation = Quat::from_rotation_arc(Vec3::Z, delta.normalize_or(Vec3::Z));
        transform.scale = Vec3::new(1.0, 1.0, length.max(0.001));
    }
}

fn update_anchors(state: Res<DemoState>, mut anchors: Query<(&AnchorMarker, &mut Transform)>) {
    for (anchor, mut transform) in &mut anchors {
        transform.translation = state.graph.nodes[anchor.node].position;
    }
}

fn update_hud(state: Res<DemoState>, mut hud: Query<&mut Text, With<Hud>>) {
    let broken = state
        .graph
        .constraints
        .iter()
        .filter(|constraint| constraint.distance.broken)
        .count();
    let components = state
        .graph
        .connected_components()
        .into_iter()
        .max()
        .map_or(0, |value| value + 1);
    let max_strain = state
        .graph
        .constraints
        .iter()
        .filter(|constraint| !constraint.distance.broken)
        .map(|constraint| constraint.distance.strain(&state.graph.nodes).abs())
        .fold(0.0, f32::max);
    let status = if state.paused { "PAUSED" } else { "RUNNING" };
    for mut text in &mut hud {
        **text = format!(
            "{status} · wind {} · tiles {}\ntile {:.3} · gap {:.3} · stitch damping {:.3} · thickness {:.3}\n{} nodes · {} constraints · {} broken · {} components\nmax strain {:.1}% · anchor offset {:+.2}",
            if state.wind { "ON" } else { "OFF" },
            state.tile_mode.label(),
            state.tile_size,
            state.stitch_gap,
            state.stitch_damping,
            state.thickness,
            state.graph.nodes.len(),
            state.graph.constraints.len(),
            broken,
            components,
            max_strain * 100.0,
            state.anchor_offset,
        );
    }
}
