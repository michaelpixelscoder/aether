use std::collections::HashMap;

use aether_app::AetherAppPlugin;
use bevy::asset::RenderAssetUsages;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::window::PrimaryWindow;

const NAVY: Color = Color::srgb(0.025, 0.055, 0.095);
const PANEL: Color = Color::srgba(0.025, 0.055, 0.095, 0.96);
const BRASS: Color = Color::srgb(0.78, 0.55, 0.23);
const PARCHMENT: Color = Color::srgb(0.96, 0.87, 0.68);
const VIOLET: Color = Color::srgb(0.55, 0.25, 0.94);

pub fn configure(app: &mut App) {
    app.add_plugins(AetherAppPlugin {
        title: "Aether Shipwright",
    })
    .insert_resource(ClearColor(Color::NONE))
    .insert_resource(EditorState::default())
    .insert_resource(VoxelWorld::default())
    .add_systems(Startup, setup)
    .add_systems(
        Update,
        (
            material_buttons,
            action_buttons,
            keyboard_shortcuts,
            orbit_camera,
            update_hover,
            edit_voxels,
            sync_voxel_scene,
            sync_hud,
            style_buttons,
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aether_register_game(app: *mut App) {
    let app = unsafe { app.as_mut() }.expect("game loader passed a null app pointer");
    configure(app);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
enum BlockKind {
    #[default]
    Wood,
    Stone,
    Grass,
    Iron,
    Glass,
}

impl BlockKind {
    const ALL: [Self; 5] = [
        Self::Wood,
        Self::Stone,
        Self::Grass,
        Self::Iron,
        Self::Glass,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::Wood => "Wood",
            Self::Stone => "Stone",
            Self::Grass => "Grass",
            Self::Iron => "Iron",
            Self::Glass => "Glass",
        }
    }

    fn color(self) -> Color {
        match self {
            Self::Wood => Color::srgb(0.34, 0.17, 0.075),
            Self::Stone => Color::srgb(0.40, 0.43, 0.46),
            Self::Grass => Color::srgb(0.28, 0.48, 0.16),
            Self::Iron => Color::srgb(0.24, 0.28, 0.34),
            Self::Glass => Color::srgba(0.38, 0.72, 0.82, 0.46),
        }
    }
}

#[derive(Resource)]
struct VoxelWorld {
    blocks: HashMap<IVec3, BlockKind>,
    revision: u64,
}

impl Default for VoxelWorld {
    fn default() -> Self {
        Self {
            blocks: HashMap::from([(IVec3::ZERO, BlockKind::Wood)]),
            revision: 1,
        }
    }
}

#[derive(Clone)]
enum Edit {
    Place(IVec3, BlockKind),
    Remove(IVec3, BlockKind),
}

#[derive(Resource)]
struct EditorState {
    material: BlockKind,
    hover: Option<Hit>,
    undo: Vec<Edit>,
    redo: Vec<Edit>,
    drag_distance: f32,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            material: BlockKind::Wood,
            hover: None,
            undo: Vec::new(),
            redo: Vec::new(),
            drag_distance: 0.0,
        }
    }
}

#[derive(Clone, Copy)]
struct Hit {
    cell: IVec3,
    normal: IVec3,
}

#[derive(Component)]
struct VoxelEntity;

#[derive(Component)]
struct HoverGhost;

#[derive(Component)]
struct OrbitCamera {
    yaw: f32,
    pitch: f32,
    radius: f32,
}

#[derive(Component)]
struct MaterialButton(BlockKind);

#[derive(Component, Clone, Copy)]
enum ActionButton {
    Undo,
    Redo,
    New,
    Capture,
}

#[derive(Component)]
struct CountText;

#[derive(Component)]
struct MaterialText;

#[derive(Component)]
struct HelpText;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    commands
        .spawn((
            Camera3d::default(),
            Transform::from_xyz(8.5, 7.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            OrbitCamera {
                yaw: 0.70,
                pitch: -0.48,
                radius: 15.0,
            },
        ))
        .with_child((
            PointLight {
                color: Color::srgb(0.72, 0.84, 1.0),
                intensity: 185_000.0,
                range: 28.0,
                radius: 7.0,
                shadows_enabled: false,
                ..default()
            },
            // A broad camera-relative fill keeps the far side readable throughout orbiting.
            Transform::from_xyz(-3.5, 4.0, 2.0),
        ));

    commands.spawn((
        DirectionalLight {
            illuminance: 11_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, -0.65, 0.0)),
    ));
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.62, 0.71, 0.86),
        brightness: 420.0,
        affects_lightmapped_meshes: true,
    });

    let grid_texture = images.add(make_grid_texture());
    let grid_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.3, 0.58, 0.75, 0.34),
        base_color_texture: Some(grid_texture),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 1.0,
        unlit: true,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(42.0, 42.0))),
        MeshMaterial3d(grid_material),
        Transform::from_xyz(0.0, -0.505, 0.0),
    ));

    let ghost_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.55, 0.25, 0.94, 0.38),
        emissive: LinearRgba::new(0.4, 0.08, 1.0, 1.0),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_size(Vec3::splat(1.025)))),
        MeshMaterial3d(ghost_material),
        Transform::from_xyz(0.0, 1.0, 0.0),
        Visibility::Hidden,
        HoverGhost,
    ));

    spawn_ui(&mut commands);
}

fn make_grid_texture() -> Image {
    const SIZE: u32 = 512;
    const CELL: u32 = 32;
    let mut pixels = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let major = x % (CELL * 5) < 2 || y % (CELL * 5) < 2;
            let minor = x % CELL < 1 || y % CELL < 1;
            let rgba = if major {
                [157, 214, 240, 170]
            } else if minor {
                [116, 177, 205, 88]
            } else {
                [0, 0, 0, 0]
            };
            pixels.extend_from_slice(&rgba);
        }
    }
    Image::new(
        Extent3d {
            width: SIZE,
            height: SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

fn spawn_ui(commands: &mut Commands) {
    commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    height: px(76),
                    width: percent(100),
                    padding: UiRect::horizontal(px(24)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    border: UiRect::bottom(px(2)),
                    ..default()
                },
                BackgroundColor(NAVY),
                BorderColor::all(BRASS),
            ))
            .with_children(|bar| {
                bar.spawn((
                    Text::new("AETHER SHIPWRIGHT"),
                    TextFont::from_font_size(28.0),
                    TextColor(PARCHMENT),
                ));
                bar.spawn((
                    Text::new("UNTITLED SHIP"),
                    TextFont::from_font_size(14.0),
                    TextColor(Color::srgb(0.70, 0.76, 0.82)),
                ));
                bar.spawn(Node {
                    column_gap: px(10),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|actions| {
                    spawn_action(actions, "Undo", ActionButton::Undo, false);
                    spawn_action(actions, "Redo", ActionButton::Redo, false);
                    spawn_action(actions, "New", ActionButton::New, false);
                    spawn_action(actions, "Capture", ActionButton::Capture, true);
                });
            });

            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(18),
                    top: px(94),
                    width: px(276),
                    padding: UiRect::all(px(18)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(12),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(10)),
                    ..default()
                },
                BackgroundColor(PANEL),
                BorderColor::all(BRASS),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("BLOCKS"),
                    TextFont::from_font_size(13.0),
                    TextColor(BRASS),
                ));
                panel.spawn((
                    Text::new("Build voxel by voxel"),
                    TextFont::from_font_size(20.0),
                    TextColor(PARCHMENT),
                ));
                for kind in BlockKind::ALL {
                    panel
                        .spawn((
                            Button,
                            MaterialButton(kind),
                            Node {
                                height: px(56),
                                width: percent(100),
                                padding: UiRect::horizontal(px(12)),
                                align_items: AlignItems::Center,
                                column_gap: px(12),
                                border: UiRect::all(px(1)),
                                border_radius: BorderRadius::all(px(6)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.045, 0.085, 0.13)),
                            BorderColor::all(Color::srgb(0.18, 0.26, 0.34)),
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Node {
                                    width: px(30),
                                    height: px(30),
                                    border: UiRect::all(px(2)),
                                    border_radius: BorderRadius::all(px(4)),
                                    ..default()
                                },
                                BackgroundColor(kind.color()),
                                BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.22)),
                            ));
                            button.spawn((
                                Text::new(format!("{}    {}", kind.name(), kind as u8 + 1)),
                                TextFont::from_font_size(17.0),
                                TextColor(Color::WHITE),
                            ));
                        });
                }
            });

            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: px(18),
                    top: px(94),
                    width: px(250),
                    padding: UiRect::all(px(18)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(14),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(10)),
                    ..default()
                },
                BackgroundColor(PANEL),
                BorderColor::all(BRASS),
            ))
            .with_children(|panel| {
                panel.spawn((Text::new("SHIP STATUS"), TextFont::from_font_size(13.0), TextColor(BRASS)));
                panel.spawn((Text::new("1 voxel"), TextFont::from_font_size(25.0), TextColor(PARCHMENT), CountText));
                panel.spawn((Text::new("Selected: Wood"), TextFont::from_font_size(16.0), TextColor(Color::WHITE), MaterialText));
                panel.spawn((
                    Text::new("Click a block face to add\nShift + click to remove\nDrag to orbit · Wheel to zoom"),
                    TextFont::from_font_size(14.0),
                    TextColor(Color::srgb(0.65, 0.72, 0.79)),
                    HelpText,
                ));
            });

            root.spawn((
                Node {
                    align_self: AlignSelf::Center,
                    margin: UiRect::bottom(px(18)),
                    padding: UiRect::axes(px(20), px(10)),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(22)),
                    ..default()
                },
                BackgroundColor(PANEL),
                BorderColor::all(Color::srgba(0.78, 0.55, 0.23, 0.7)),
                Pickable::IGNORE,
            ))
            .with_child((
                Text::new("Add: click face    Remove: Shift + click    Materials: 1–5    Undo: Ctrl Z"),
                TextFont::from_font_size(13.0),
                TextColor(Color::srgb(0.75, 0.79, 0.83)),
            ));
        });
}

fn spawn_action(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    action: ActionButton,
    primary: bool,
) {
    parent
        .spawn((
            Button,
            action,
            Node {
                min_width: px(if primary { 112 } else { 72 }),
                height: px(40),
                padding: UiRect::horizontal(px(14)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(6)),
                ..default()
            },
            BackgroundColor(if primary {
                VIOLET
            } else {
                Color::srgb(0.06, 0.10, 0.15)
            }),
            BorderColor::all(if primary {
                PARCHMENT
            } else {
                Color::srgb(0.22, 0.29, 0.36)
            }),
        ))
        .with_child((
            Text::new(label),
            TextFont::from_font_size(15.0),
            TextColor(Color::WHITE),
        ));
}

fn material_buttons(
    interactions: Query<(&Interaction, &MaterialButton), Changed<Interaction>>,
    mut editor: ResMut<EditorState>,
) {
    for (interaction, button) in &interactions {
        if *interaction == Interaction::Pressed {
            editor.material = button.0;
        }
    }
}

fn action_buttons(
    interactions: Query<(&Interaction, &ActionButton), Changed<Interaction>>,
    mut world: ResMut<VoxelWorld>,
    mut editor: ResMut<EditorState>,
) {
    for (interaction, action) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match action {
            ActionButton::Undo => undo(&mut world, &mut editor),
            ActionButton::Redo => redo(&mut world, &mut editor),
            ActionButton::New => {
                world.blocks.clear();
                world.blocks.insert(IVec3::ZERO, BlockKind::Wood);
                world.revision += 1;
                editor.undo.clear();
                editor.redo.clear();
            }
            ActionButton::Capture => {}
        }
    }
}

fn keyboard_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    mut world: ResMut<VoxelWorld>,
    mut editor: ResMut<EditorState>,
) {
    for (key, kind) in [
        (KeyCode::Digit1, BlockKind::Wood),
        (KeyCode::Digit2, BlockKind::Stone),
        (KeyCode::Digit3, BlockKind::Grass),
        (KeyCode::Digit4, BlockKind::Iron),
        (KeyCode::Digit5, BlockKind::Glass),
    ] {
        if keys.just_pressed(key) {
            editor.material = kind;
        }
    }
    let control = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    if control && keys.just_pressed(KeyCode::KeyZ) {
        if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
            redo(&mut world, &mut editor);
        } else {
            undo(&mut world, &mut editor);
        }
    }
}

fn undo(world: &mut VoxelWorld, editor: &mut EditorState) {
    if let Some(edit) = editor.undo.pop() {
        match edit {
            Edit::Place(cell, kind) => {
                world.blocks.remove(&cell);
                editor.redo.push(Edit::Place(cell, kind));
            }
            Edit::Remove(cell, kind) => {
                world.blocks.insert(cell, kind);
                editor.redo.push(Edit::Remove(cell, kind));
            }
        }
        world.revision += 1;
    }
}

fn redo(world: &mut VoxelWorld, editor: &mut EditorState) {
    if let Some(edit) = editor.redo.pop() {
        match edit {
            Edit::Place(cell, kind) => {
                world.blocks.insert(cell, kind);
                editor.undo.push(Edit::Place(cell, kind));
            }
            Edit::Remove(cell, kind) => {
                world.blocks.remove(&cell);
                editor.undo.push(Edit::Remove(cell, kind));
            }
        }
        world.revision += 1;
    }
}

fn cursor_in_viewport(window: &Window) -> bool {
    window.cursor_position().is_some_and(|p| {
        p.x > 310.0 && p.x < window.width() - 285.0 && p.y > 78.0 && p.y < window.height() - 58.0
    })
}

fn orbit_camera(
    windows: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut motion: MessageReader<MouseMotion>,
    mut wheel: MessageReader<MouseWheel>,
    mut editor: ResMut<EditorState>,
    mut cameras: Query<(&mut OrbitCamera, &mut Transform)>,
) {
    let Ok(window) = windows.single() else { return };
    let delta = motion.read().map(|event| event.delta).sum::<Vec2>();
    if buttons.just_pressed(MouseButton::Left) {
        editor.drag_distance = 0.0;
    }
    if buttons.pressed(MouseButton::Left) {
        editor.drag_distance += delta.length();
    }
    let scroll = wheel.read().map(|event| event.y).sum::<f32>();
    for (mut orbit, mut transform) in &mut cameras {
        if buttons.pressed(MouseButton::Left) && cursor_in_viewport(window) && delta != Vec2::ZERO {
            orbit.yaw -= delta.x * 0.007;
            orbit.pitch = (orbit.pitch - delta.y * 0.007).clamp(-1.25, -0.08);
        }
        if cursor_in_viewport(window) && scroll != 0.0 {
            orbit.radius = (orbit.radius - scroll * 0.75).clamp(5.0, 32.0);
        }
        let target = Vec3::new(0.0, 1.5, 0.0);
        let offset = Quat::from_euler(EulerRot::YXZ, orbit.yaw, orbit.pitch, 0.0)
            * Vec3::new(0.0, 0.0, orbit.radius);
        *transform = Transform::from_translation(target + offset).looking_at(target, Vec3::Y);
    }
}

fn update_hover(
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<OrbitCamera>>,
    world: Res<VoxelWorld>,
    mut editor: ResMut<EditorState>,
    mut ghost: Query<(&mut Transform, &mut Visibility), With<HoverGhost>>,
) {
    let Ok(window) = windows.single() else { return };
    let Ok((camera, camera_transform)) = cameras.single() else {
        return;
    };
    let Ok((mut transform, mut visibility)) = ghost.single_mut() else {
        return;
    };
    if !cursor_in_viewport(window) {
        editor.hover = None;
        *visibility = Visibility::Hidden;
        return;
    }
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor) else {
        return;
    };
    editor.hover = raycast_voxels(ray.origin, ray.direction.as_vec3(), &world.blocks);
    if let Some(hit) = editor.hover {
        let target = hit.cell + hit.normal;
        transform.translation = target.as_vec3();
        *visibility = if world.blocks.contains_key(&target) {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    } else {
        *visibility = Visibility::Hidden;
    }
}

fn edit_voxels(
    windows: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut world: ResMut<VoxelWorld>,
    mut editor: ResMut<EditorState>,
) {
    let Ok(window) = windows.single() else { return };
    if !buttons.just_released(MouseButton::Left)
        || !cursor_in_viewport(window)
        || editor.drag_distance > 4.0
    {
        return;
    }
    let Some(hit) = editor.hover else { return };
    let removing = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    if removing {
        if world.blocks.len() > 1 {
            if let Some(kind) = world.blocks.remove(&hit.cell) {
                editor.undo.push(Edit::Remove(hit.cell, kind));
                editor.redo.clear();
                world.revision += 1;
            }
        }
    } else {
        let target = hit.cell + hit.normal;
        if !world.blocks.contains_key(&target) {
            let kind = editor.material;
            world.blocks.insert(target, kind);
            editor.undo.push(Edit::Place(target, kind));
            editor.redo.clear();
            world.revision += 1;
        }
    }
}

fn raycast_voxels(
    origin: Vec3,
    direction: Vec3,
    blocks: &HashMap<IVec3, BlockKind>,
) -> Option<Hit> {
    let mut best: Option<(f32, Hit)> = None;
    for &cell in blocks.keys() {
        let center = cell.as_vec3();
        let min = center - Vec3::splat(0.5);
        let max = center + Vec3::splat(0.5);
        if let Some((distance, normal)) = ray_box(origin, direction, min, max) {
            if distance >= 0.0 && best.is_none_or(|(current, _)| distance < current) {
                best = Some((distance, Hit { cell, normal }));
            }
        }
    }
    best.map(|(_, hit)| hit)
}

fn ray_box(origin: Vec3, direction: Vec3, min: Vec3, max: Vec3) -> Option<(f32, IVec3)> {
    let mut near = f32::NEG_INFINITY;
    let mut far = f32::INFINITY;
    let mut normal = IVec3::ZERO;
    for axis in 0..3 {
        let d = direction[axis];
        if d.abs() < 1e-7 {
            if origin[axis] < min[axis] || origin[axis] > max[axis] {
                return None;
            }
            continue;
        }
        let mut t1 = (min[axis] - origin[axis]) / d;
        let mut t2 = (max[axis] - origin[axis]) / d;
        let sign = if d > 0.0 { -1 } else { 1 };
        if t1 > t2 {
            std::mem::swap(&mut t1, &mut t2);
        }
        if t1 > near {
            near = t1;
            normal = match axis {
                0 => IVec3::new(sign, 0, 0),
                1 => IVec3::new(0, sign, 0),
                _ => IVec3::new(0, 0, sign),
            };
        }
        far = far.min(t2);
        if near > far {
            return None;
        }
    }
    (far >= 0.0).then_some((near.max(0.0), normal))
}

fn sync_voxel_scene(
    mut commands: Commands,
    world: Res<VoxelWorld>,
    entities: Query<Entity, With<VoxelEntity>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut last_revision: Local<u64>,
) {
    if *last_revision == world.revision {
        return;
    }
    for entity in &entities {
        commands.entity(entity).despawn();
    }
    let cube = meshes.add(Cuboid::from_size(Vec3::splat(0.96)));
    let mut material_handles = HashMap::new();
    for kind in BlockKind::ALL {
        let glass = kind == BlockKind::Glass;
        let texture_path = match kind {
            BlockKind::Wood => "textures/shipwright/wood.png",
            BlockKind::Stone => "textures/shipwright/stone.png",
            BlockKind::Grass => "textures/shipwright/grass.png",
            BlockKind::Iron => "textures/shipwright/iron.png",
            BlockKind::Glass => "textures/shipwright/glass.png",
        };
        let handle = materials.add(StandardMaterial {
            base_color: if glass {
                Color::srgba(0.58, 0.88, 0.96, 0.48)
            } else {
                Color::WHITE
            },
            base_color_texture: Some(asset_server.load(texture_path)),
            metallic: if kind == BlockKind::Iron { 0.72 } else { 0.0 },
            perceptual_roughness: match kind {
                BlockKind::Iron => 0.38,
                BlockKind::Glass => 0.10,
                BlockKind::Wood => 0.72,
                _ => 0.82,
            },
            alpha_mode: if glass {
                AlphaMode::Blend
            } else {
                AlphaMode::Opaque
            },
            reflectance: if glass { 0.7 } else { 0.35 },
            ..default()
        });
        material_handles.insert(kind, handle);
    }
    for (&cell, &kind) in &world.blocks {
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(material_handles[&kind].clone()),
            Transform::from_translation(cell.as_vec3()),
            VoxelEntity,
        ));
    }
    *last_revision = world.revision;
}

fn sync_hud(
    world: Res<VoxelWorld>,
    editor: Res<EditorState>,
    mut count: Query<&mut Text, (With<CountText>, Without<MaterialText>)>,
    mut selected: Query<&mut Text, (With<MaterialText>, Without<CountText>)>,
) {
    if world.is_changed() {
        if let Ok(mut text) = count.single_mut() {
            **text = format!(
                "{} voxel{}",
                world.blocks.len(),
                if world.blocks.len() == 1 { "" } else { "s" }
            );
        }
    }
    if editor.is_changed() {
        if let Ok(mut text) = selected.single_mut() {
            **text = format!("Selected: {}", editor.material.name());
        }
    }
}

fn style_buttons(
    editor: Res<EditorState>,
    mut materials: Query<
        (
            &Interaction,
            &MaterialButton,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<Button>,
    >,
) {
    for (interaction, material, mut background, mut border) in &mut materials {
        let selected = material.0 == editor.material;
        match *interaction {
            Interaction::Pressed => background.0 = Color::srgb(0.20, 0.12, 0.34),
            Interaction::Hovered => background.0 = Color::srgb(0.10, 0.14, 0.21),
            Interaction::None => {
                background.0 = if selected {
                    Color::srgb(0.13, 0.08, 0.22)
                } else {
                    Color::srgb(0.045, 0.085, 0.13)
                }
            }
        }
        *border = BorderColor::all(if selected {
            VIOLET
        } else {
            Color::srgb(0.18, 0.26, 0.34)
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_world_starts_with_one_centered_wood_voxel() {
        let world = VoxelWorld::default();
        assert_eq!(world.blocks.len(), 1);
        assert_eq!(world.blocks[&IVec3::ZERO], BlockKind::Wood);
    }

    #[test]
    fn ray_hits_front_face_of_center_voxel() {
        let hit = raycast_voxels(
            Vec3::new(0.0, 0.0, 5.0),
            Vec3::NEG_Z,
            &VoxelWorld::default().blocks,
        )
        .expect("center voxel should be hit");
        assert_eq!(hit.cell, IVec3::ZERO);
        assert_eq!(hit.normal, IVec3::Z);
    }
}
