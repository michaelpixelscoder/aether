use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::{fs, path::Path};

use aether_app::AetherAppPlugin;
use aether_voxels::{
    MacroAtlasRegion, VoxelWorld as AetherVoxelWorld, greedy_mesh, normalized_quad_uvs,
    partition_quads_by_normal_axis, tile_quads_with_macro_atlas,
};
use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::mesh::Indices;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::window::PrimaryWindow;

const NAVY: Color = Color::srgb(0.025, 0.055, 0.095);
const PANEL: Color = Color::srgba(0.025, 0.055, 0.095, 0.96);
const BRASS: Color = Color::srgb(0.78, 0.55, 0.23);
const PARCHMENT: Color = Color::srgb(0.96, 0.87, 0.68);
const VIOLET: Color = Color::srgb(0.55, 0.25, 0.94);
const BLUEPRINT_FILE: &str = "shipwright-ship.vox";

pub fn configure(app: &mut App) {
    app.add_plugins(AetherAppPlugin {
        title: "Aether Shipwright",
    })
    .insert_resource(ClearColor(Color::NONE))
    .insert_resource(EditorState::default())
    .insert_resource(BlueprintStatus::default())
    .insert_resource(DebugState::default())
    .insert_resource(new_ship_world())
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
    WoodenPlank,
    Stone,
    Grass,
    Iron,
    Glass,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum TechMapView {
    #[default]
    BaseColor,
    Height,
    Normal,
    AmbientOcclusion,
    Orm,
    EndGrain,
}

impl TechMapView {
    fn next(self) -> Self {
        match self {
            Self::BaseColor => Self::Height,
            Self::Height => Self::Normal,
            Self::Normal => Self::AmbientOcclusion,
            Self::AmbientOcclusion => Self::Orm,
            Self::Orm => Self::EndGrain,
            Self::EndGrain => Self::BaseColor,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::BaseColor => "Base color",
            Self::Height => "Height",
            Self::Normal => "Normal",
            Self::AmbientOcclusion => "Ambient occlusion",
            Self::Orm => "ORM",
            Self::EndGrain => "End grain",
        }
    }

    fn texture_path(self) -> &'static str {
        match self {
            Self::BaseColor => "voxel_materials/wooden_plank/textures/base_color.png",
            Self::Height => "voxel_materials/wooden_plank/textures/height.png",
            Self::Normal => "voxel_materials/wooden_plank/textures/normal.png",
            Self::AmbientOcclusion => "voxel_materials/wooden_plank/textures/ao.png",
            Self::Orm => "voxel_materials/wooden_plank/textures/orm.png",
            Self::EndGrain => "voxel_materials/wooden_plank/textures/end_grain.png",
        }
    }
}

type VoxelWorld = AetherVoxelWorld<BlockKind>;

// Packed complete-plank templates for the greedy-quad shape vocabulary.
const WOODEN_PLANK_5X5_ATLAS: [MacroAtlasRegion; 11] = [
    MacroAtlasRegion {
        origin: [0, 0],
        size: [1, 1],
    },
    MacroAtlasRegion {
        origin: [1, 0],
        size: [3, 1],
    },
    MacroAtlasRegion {
        origin: [4, 0],
        size: [1, 1],
    },
    MacroAtlasRegion {
        origin: [0, 1],
        size: [4, 1],
    },
    MacroAtlasRegion {
        origin: [4, 1],
        size: [1, 1],
    },
    MacroAtlasRegion {
        origin: [0, 2],
        size: [2, 1],
    },
    MacroAtlasRegion {
        origin: [2, 2],
        size: [2, 1],
    },
    MacroAtlasRegion {
        origin: [4, 2],
        size: [1, 1],
    },
    MacroAtlasRegion {
        origin: [0, 3],
        size: [5, 2],
    },
    MacroAtlasRegion {
        origin: [0, 3],
        size: [5, 1],
    },
    MacroAtlasRegion {
        origin: [0, 4],
        size: [5, 1],
    },
];

impl BlockKind {
    fn vox_color_index(self) -> u8 {
        match self {
            Self::WoodenPlank => 1,
            Self::Stone => 2,
            Self::Grass => 3,
            Self::Iron => 4,
            Self::Glass => 5,
        }
    }

    fn from_vox_color_index(value: u8) -> Self {
        match value {
            2 => Self::Stone,
            3 => Self::Grass,
            4 => Self::Iron,
            5 => Self::Glass,
            // Imported MagicaVoxel palettes are intentionally treated as
            // wooden planks so external shape edits preserve this material.
            _ => Self::WoodenPlank,
        }
    }

    const ALL: [Self; 5] = [
        Self::WoodenPlank,
        Self::Stone,
        Self::Grass,
        Self::Iron,
        Self::Glass,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::WoodenPlank => "Wooden plank",
            Self::Stone => "Stone",
            Self::Grass => "Grass",
            Self::Iron => "Iron",
            Self::Glass => "Glass",
        }
    }

    fn color(self) -> Color {
        match self {
            Self::WoodenPlank => Color::srgb(0.34, 0.17, 0.075),
            Self::Stone => Color::srgb(0.40, 0.43, 0.46),
            Self::Grass => Color::srgb(0.28, 0.48, 0.16),
            Self::Iron => Color::srgb(0.24, 0.28, 0.34),
            Self::Glass => Color::srgba(0.38, 0.72, 0.82, 0.46),
        }
    }
}

fn new_ship_world() -> VoxelWorld {
    let mut blocks = HashMap::new();
    // Compact all-wood starter hull: floor, raised gunwales, tapered bow, and
    // a shallow stern. It intentionally exercises long side runs, 2×5 floor
    // sections, outside corners, and exposed end faces.
    for x in 1..=10 {
        for z in -2..=2 {
            blocks.insert(IVec3::new(x, 0, z), BlockKind::WoodenPlank);
        }
        blocks.insert(IVec3::new(x, 1, -3), BlockKind::WoodenPlank);
        blocks.insert(IVec3::new(x, 1, 3), BlockKind::WoodenPlank);
    }
    for z in -1..=1 {
        blocks.insert(IVec3::new(0, 1, z), BlockKind::WoodenPlank);
        blocks.insert(IVec3::new(11, 1, z), BlockKind::WoodenPlank);
        blocks.insert(IVec3::new(0, 2, z), BlockKind::WoodenPlank);
    }
    for x in [1, 2, 9, 10] {
        blocks.insert(IVec3::new(x, 2, -3), BlockKind::WoodenPlank);
        blocks.insert(IVec3::new(x, 2, 3), BlockKind::WoodenPlank);
    }
    let blocks = blocks
        .into_iter()
        .map(|(cell, kind)| (cell - IVec3::new(6, 0, 0), kind))
        .collect();
    VoxelWorld {
        blocks,
        revision: 1,
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

#[derive(Resource, Default)]
struct BlueprintStatus(String);

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
struct DebugState {
    uv_view: bool,
    wireframe: bool,
    tech_map: TechMapView,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            material: BlockKind::WoodenPlank,
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
struct WireframeEntity;

#[derive(Component)]
struct HoverGhost;

#[derive(Component)]
struct OrbitCamera {
    yaw: f32,
    pitch: f32,
    radius: f32,
    target_radius: f32,
    center: Vec3,
}

#[derive(Component)]
struct MaterialButton(BlockKind);

#[derive(Component, Clone, Copy)]
enum ActionButton {
    Undo,
    Redo,
    New,
    Import,
    Export,
    ToggleUvDebug,
    ToggleWireframe,
    CycleTechMap,
    Capture,
}

#[derive(Component)]
struct CountText;

#[derive(Component)]
struct MaterialText;

#[derive(Component)]
struct HelpText;

#[derive(Component)]
struct BlueprintStatusText;

#[derive(Component)]
struct DebugStatusText;

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
                target_radius: 15.0,
                center: Vec3::new(0.0, 1.5, 0.0),
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
                    spawn_action(actions, "Import", ActionButton::Import, false);
                    spawn_action(actions, "Export", ActionButton::Export, true);
                    spawn_action(actions, "Capture", ActionButton::Capture, true);
                });
            });

            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(18),
                    // The five 56px material controls extend past 500px from
                    // the viewport top; keep debug controls below that panel.
                    top: px(548),
                    width: px(276),
                    padding: UiRect::all(px(14)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(8),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(10)),
                    ..default()
                },
                BackgroundColor(PANEL),
                BorderColor::all(Color::srgb(0.25, 0.55, 0.72)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("DEBUG RENDER"),
                    TextFont::from_font_size(13.0),
                    TextColor(Color::srgb(0.45, 0.78, 1.0)),
                ));
                panel.spawn(Node { column_gap: px(8), ..default() }).with_children(|row| {
                    spawn_action(row, "UV zones", ActionButton::ToggleUvDebug, false);
                    spawn_action(row, "Wireframe", ActionButton::ToggleWireframe, false);
                });
                panel.spawn(Node { column_gap: px(8), ..default() }).with_children(|row| {
                    spawn_action(row, "Next map", ActionButton::CycleTechMap, false);
                });
                panel.spawn((
                    Text::new("UV: off · Wire: off · Map: Base color"),
                    TextFont::from_font_size(12.0),
                    TextColor(Color::srgb(0.68, 0.76, 0.84)),
                    DebugStatusText,
                ));
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
                panel.spawn((Text::new("Selected: Wooden plank"), TextFont::from_font_size(16.0), TextColor(Color::WHITE), MaterialText));
                panel.spawn((
                    Text::new("Blueprint ready"),
                    TextFont::from_font_size(13.0),
                    TextColor(Color::srgb(0.65, 0.72, 0.79)),
                    BlueprintStatusText,
                ));
                panel.spawn((
                    Text::new("MagicaVoxel .vox import/export\nUses shipwright-ship.vox in this folder\n\nClick a block face to add\nShift + click to remove\nDrag to orbit · Shift/middle drag to pan\nWheel to zoom"),
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
    mut blueprint_status: ResMut<BlueprintStatus>,
    mut debug: ResMut<DebugState>,
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
                world.blocks.insert(IVec3::ZERO, BlockKind::WoodenPlank);
                world.revision += 1;
                editor.undo.clear();
                editor.redo.clear();
                blueprint_status.0 = "New empty ship".into();
            }
            ActionButton::Export => export_blueprint(&world, &mut blueprint_status),
            ActionButton::Import => {
                import_blueprint(&mut world, &mut editor, &mut blueprint_status)
            }
            ActionButton::ToggleUvDebug => debug.uv_view = !debug.uv_view,
            ActionButton::ToggleWireframe => debug.wireframe = !debug.wireframe,
            ActionButton::CycleTechMap => debug.tech_map = debug.tech_map.next(),
            ActionButton::Capture => {}
        }
    }
}

fn write_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend(value.to_le_bytes());
}

fn read_u32(bytes: &[u8], offset: &mut usize) -> Result<u32, String> {
    let end = offset.checked_add(4).ok_or("VOX offset overflow")?;
    let chunk = bytes
        .get(*offset..end)
        .ok_or("unexpected end of VOX file")?;
    *offset = end;
    Ok(u32::from_le_bytes(chunk.try_into().expect("four bytes")))
}

fn serialize_blueprint(world: &VoxelWorld) -> Result<Vec<u8>, String> {
    let mut blocks = world.blocks.iter().collect::<Vec<_>>();
    blocks.sort_by_key(|(cell, _)| (cell.x, cell.y, cell.z));
    let min = blocks
        .iter()
        .map(|(cell, _)| **cell)
        .reduce(|a, b| a.min(b))
        .ok_or("cannot export an empty ship")?;
    let max = blocks
        .iter()
        .map(|(cell, _)| **cell)
        .reduce(|a, b| a.max(b))
        .expect("non-empty blocks");
    let size = max - min + IVec3::ONE;
    if size.x > 256 || size.y > 256 || size.z > 256 {
        return Err("MagicaVoxel .vox export supports a maximum 256×256×256 model".into());
    }
    let mut size_chunk = Vec::new();
    for value in [size.x, size.y, size.z] {
        write_u32(&mut size_chunk, value as u32);
    }
    let mut xyzi_chunk = Vec::with_capacity(4 + blocks.len() * 4);
    write_u32(&mut xyzi_chunk, blocks.len() as u32);
    for (cell, kind) in blocks {
        let local = *cell - min;
        xyzi_chunk.extend([
            local.x as u8,
            local.y as u8,
            local.z as u8,
            kind.vox_color_index(),
        ]);
    }
    let mut children = Vec::new();
    for (id, content) in [(b"SIZE", size_chunk), (b"XYZI", xyzi_chunk)] {
        children.extend(id);
        write_u32(&mut children, content.len() as u32);
        write_u32(&mut children, 0);
        children.extend(content);
    }
    let mut output = Vec::with_capacity(20 + children.len());
    output.extend(b"VOX ");
    write_u32(&mut output, 150);
    output.extend(b"MAIN");
    write_u32(&mut output, 0);
    write_u32(&mut output, children.len() as u32);
    output.extend(children);
    Ok(output)
}

fn parse_blueprint(bytes: &[u8]) -> Result<HashMap<IVec3, BlockKind>, String> {
    if bytes.get(0..4) != Some(b"VOX ") {
        return Err("expected MagicaVoxel `VOX ` header".into());
    }
    let mut offset = 4;
    let version = read_u32(bytes, &mut offset)?;
    if version < 150 {
        return Err(format!("unsupported VOX version {version}"));
    }
    if bytes.get(offset..offset + 4) != Some(b"MAIN") {
        return Err("missing MAIN VOX chunk".into());
    }
    offset += 4;
    let main_content = read_u32(bytes, &mut offset)? as usize;
    let main_children = read_u32(bytes, &mut offset)? as usize;
    offset = offset
        .checked_add(main_content)
        .ok_or("invalid MAIN chunk")?;
    let end = offset
        .checked_add(main_children)
        .ok_or("invalid MAIN child length")?
        .min(bytes.len());
    let mut size = None;
    let mut voxels = None;
    while offset < end {
        let id = bytes
            .get(offset..offset + 4)
            .ok_or("truncated VOX chunk id")?;
        offset += 4;
        let content_len = read_u32(bytes, &mut offset)? as usize;
        let child_len = read_u32(bytes, &mut offset)? as usize;
        let content_end = offset
            .checked_add(content_len)
            .ok_or("invalid VOX content length")?;
        let content = bytes
            .get(offset..content_end)
            .ok_or("truncated VOX content")?;
        if id == b"SIZE" && content.len() >= 12 {
            let mut cursor = 0;
            size = Some(IVec3::new(
                read_u32(content, &mut cursor)? as i32,
                read_u32(content, &mut cursor)? as i32,
                read_u32(content, &mut cursor)? as i32,
            ));
        } else if id == b"XYZI" && content.len() >= 4 {
            let mut cursor = 0;
            let count = read_u32(content, &mut cursor)? as usize;
            if content.len() < 4 + count * 4 {
                return Err("truncated XYZI voxel list".into());
            }
            voxels = Some(content[4..4 + count * 4].to_vec());
        }
        offset = content_end
            .checked_add(child_len)
            .ok_or("invalid VOX child length")?;
    }
    let size = size.ok_or("VOX file has no SIZE chunk")?;
    let voxels = voxels.ok_or("VOX file has no XYZI chunk")?;
    if size.x <= 0 || size.y <= 0 || size.z <= 0 || size.x > 256 || size.y > 256 || size.z > 256 {
        return Err("VOX dimensions must be in 1..=256".into());
    }
    let mut blocks = HashMap::new();
    for voxel in voxels.chunks_exact(4) {
        let cell = IVec3::new(
            voxel[0] as i32 - size.x / 2,
            voxel[1] as i32,
            voxel[2] as i32 - size.z / 2,
        );
        blocks.insert(cell, BlockKind::from_vox_color_index(voxel[3]));
    }
    if blocks.is_empty() {
        return Err("blueprint contains no voxels".into());
    }
    Ok(blocks)
}

#[cfg(not(target_arch = "wasm32"))]
fn export_blueprint(world: &VoxelWorld, status: &mut BlueprintStatus) {
    match serialize_blueprint(world)
        .and_then(|bytes| fs::write(BLUEPRINT_FILE, bytes).map_err(|error| error.to_string()))
    {
        Ok(()) => status.0 = format!("Exported {} voxels to {BLUEPRINT_FILE}", world.blocks.len()),
        Err(error) => status.0 = format!("Export failed: {error}"),
    }
}

#[cfg(target_arch = "wasm32")]
fn export_blueprint(_world: &VoxelWorld, status: &mut BlueprintStatus) {
    status.0 = "Export is available in the native Shipwright build".into();
}

#[cfg(not(target_arch = "wasm32"))]
fn import_blueprint(
    world: &mut VoxelWorld,
    editor: &mut EditorState,
    status: &mut BlueprintStatus,
) {
    match fs::read(BLUEPRINT_FILE)
        .map_err(|error| error.to_string())
        .and_then(|bytes| parse_blueprint(&bytes))
    {
        Ok(blocks) => {
            let count = blocks.len();
            world.blocks = blocks;
            world.revision += 1;
            editor.undo.clear();
            editor.redo.clear();
            status.0 = format!("Imported {count} voxels from {BLUEPRINT_FILE}");
        }
        Err(error) => status.0 = format!("Import failed: {error}"),
    }
}

#[cfg(target_arch = "wasm32")]
fn import_blueprint(
    _world: &mut VoxelWorld,
    _editor: &mut EditorState,
    status: &mut BlueprintStatus,
) {
    status.0 = "Import is available in the native Shipwright build".into();
}

fn keyboard_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    mut world: ResMut<VoxelWorld>,
    mut editor: ResMut<EditorState>,
) {
    for (key, kind) in [
        (KeyCode::Digit1, BlockKind::WoodenPlank),
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
    keys: Res<ButtonInput<KeyCode>>,
    mut motion: MessageReader<MouseMotion>,
    mut wheel: MessageReader<MouseWheel>,
    mut editor: ResMut<EditorState>,
    mut cameras: Query<(&mut OrbitCamera, &mut Transform)>,
    time: Res<Time>,
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
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    for (mut orbit, mut transform) in &mut cameras {
        let panning =
            buttons.pressed(MouseButton::Middle) || (buttons.pressed(MouseButton::Left) && shift);
        if panning && cursor_in_viewport(window) && delta != Vec2::ZERO {
            const PAN_SPEED: f32 = 0.01;
            let right = transform.rotation * Vec3::X;
            let up = transform.rotation * Vec3::Y;
            orbit.center -= (right * delta.x - up * delta.y) * PAN_SPEED;
        } else if buttons.pressed(MouseButton::Left)
            && !shift
            && cursor_in_viewport(window)
            && delta != Vec2::ZERO
        {
            orbit.yaw -= delta.x * 0.007;
            orbit.pitch = (orbit.pitch - delta.y * 0.007).clamp(-1.25, -0.08);
        }
        if cursor_in_viewport(window) && scroll != 0.0 {
            let scroll = scroll.clamp(-3.0, 3.0);
            orbit.target_radius = (orbit.target_radius * 0.92_f32.powf(scroll)).clamp(5.0, 128.0);
        }
        let smoothing = 1.0 - (-12.0 * time.delta_secs()).exp();
        orbit.radius += (orbit.target_radius - orbit.radius) * smoothing;
        let offset = Quat::from_euler(EulerRot::YXZ, orbit.yaw, orbit.pitch, 0.0)
            * Vec3::new(0.0, 0.0, orbit.radius);
        *transform =
            Transform::from_translation(orbit.center + offset).looking_at(orbit.center, Vec3::Y);
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
    debug: Res<DebugState>,
    mut last_render: Local<Option<(u64, DebugState)>>,
) {
    if *last_render == Some((world.revision, *debug)) {
        return;
    }
    for entity in &entities {
        commands.entity(entity).despawn();
    }
    let mut material_handles = HashMap::new();
    for kind in BlockKind::ALL {
        let glass = kind == BlockKind::Glass;
        let texture_path = match kind {
            BlockKind::WoodenPlank if debug.uv_view => {
                "voxel_materials/wooden_plank/textures/debug_atlas.png"
            }
            BlockKind::WoodenPlank => debug.tech_map.texture_path(),
            BlockKind::Stone => "textures/shipwright/stone.png",
            BlockKind::Grass => "textures/shipwright/grass.png",
            BlockKind::Iron => "textures/shipwright/iron.png",
            BlockKind::Glass => "textures/shipwright/glass.png",
        };
        let show_technical_map = kind == BlockKind::WoodenPlank
            && (debug.uv_view || debug.tech_map != TechMapView::BaseColor);
        let technical_maps = (!show_technical_map)
            .then(|| technical_map_paths(texture_path))
            .flatten();
        let normal_map_texture = technical_maps
            .as_ref()
            .map(|maps| load_clamped_texture(&asset_server, maps.normal.clone(), false));
        let depth_map = technical_maps
            .as_ref()
            .map(|maps| load_clamped_texture(&asset_server, maps.height.clone(), false));
        let orm_texture = technical_maps
            .as_ref()
            .map(|maps| load_clamped_texture(&asset_server, maps.orm.clone(), false));
        let uses_orm = orm_texture.is_some();
        let handle = materials.add(StandardMaterial {
            base_color: if glass {
                Color::srgba(0.58, 0.88, 0.96, 0.48)
            } else {
                Color::WHITE
            },
            base_color_texture: Some(load_clamped_texture(
                &asset_server,
                texture_path.to_owned(),
                true,
            )),
            normal_map_texture,
            depth_map,
            // Keep the authored bevel visible at grazing angles without making
            // a single voxel appear deeply displaced.
            parallax_depth_scale: if show_technical_map { 0.0 } else { 0.025 },
            metallic_roughness_texture: orm_texture.clone(),
            occlusion_texture: orm_texture,
            metallic: if uses_orm {
                1.0
            } else if kind == BlockKind::Iron {
                0.72
            } else {
                0.0
            },
            perceptual_roughness: if uses_orm {
                1.0
            } else {
                match kind {
                    BlockKind::Iron => 0.38,
                    BlockKind::Glass => 0.10,
                    BlockKind::WoodenPlank => 0.72,
                    _ => 0.82,
                }
            },
            alpha_mode: if glass {
                AlphaMode::Blend
            } else {
                AlphaMode::Opaque
            },
            reflectance: if glass { 0.7 } else { 0.35 },
            unlit: show_technical_map,
            ..default()
        });
        material_handles.insert(kind, handle);
    }
    // Cut faces run perpendicular to the hull's long X axis.  A separate
    // end-grain texture avoids the almost-black, vertically stretched plank
    // appearance those faces get when they reuse the long-grain atlas.
    let wooden_end_grain = materials.add(StandardMaterial {
        base_color_texture: Some(load_clamped_texture(
            &asset_server,
            "voxel_materials/wooden_plank/textures/end_grain.png".to_owned(),
            true,
        )),
        perceptual_roughness: 0.68,
        reflectance: 0.32,
        // End grain is an authored diagnostic layer: keeping it unlit ensures
        // the cut-ring read remains visible even on inward-facing hull caps.
        unlit: true,
        ..default()
    });
    let wire_mesh = debug
        .wireframe
        .then(|| meshes.add(Cuboid::new(1.0, 1.0, 1.0)));
    let wire_material = debug.wireframe.then(|| {
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.20, 0.92, 1.0),
            emissive: Color::srgb(0.08, 0.45, 0.55).into(),
            unlit: true,
            ..default()
        })
    });
    for voxel_mesh in greedy_mesh(&world.blocks, |_| true) {
        let mesh_layers = if voxel_mesh.material == BlockKind::WoodenPlank {
            let (mut end_grain, long_grain) = partition_quads_by_normal_axis(&voxel_mesh, 0);
            end_grain.uvs = normalized_quad_uvs(&end_grain.surface_coordinates);
            vec![(end_grain, true), (long_grain, false)]
        } else {
            vec![(voxel_mesh, false)]
        };
        for (voxel_mesh, is_end_grain) in mesh_layers {
            if voxel_mesh.positions.is_empty() {
                continue;
            }
            let voxel_mesh = if voxel_mesh.material == BlockKind::WoodenPlank && !is_end_grain {
                tile_quads_with_macro_atlas(
                    &voxel_mesh,
                    [5, 5],
                    [1280, 1280],
                    &WOODEN_PLANK_5X5_ATLAS,
                )
            } else {
                voxel_mesh
            };
            let mut mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::default(),
            );
            let wire_edges = debug.wireframe.then(|| quad_edges(&voxel_mesh.positions));
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, voxel_mesh.positions);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, voxel_mesh.normals);
            let uvs = if voxel_mesh.material == BlockKind::WoodenPlank {
                voxel_mesh.uvs
            } else {
                voxel_mesh.uvs
            };
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
            mesh.insert_indices(Indices::U32(voxel_mesh.indices));
            mesh.generate_tangents()
                .expect("voxel mesh must support tangent generation for optional normal maps");
            let material = if is_end_grain {
                wooden_end_grain.clone()
            } else {
                material_handles[&voxel_mesh.material].clone()
            };
            commands.spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(material),
                VoxelEntity,
            ));
            if let (Some(wire_mesh), Some(wire_material), Some(wire_edges)) =
                (&wire_mesh, &wire_material, wire_edges)
            {
                for (start, end) in wire_edges {
                    let delta = end - start;
                    let length = delta.length();
                    if length <= f32::EPSILON {
                        continue;
                    }
                    commands.spawn((
                        Mesh3d(wire_mesh.clone()),
                        MeshMaterial3d(wire_material.clone()),
                        Transform {
                            translation: (start + end) * 0.5,
                            rotation: Quat::from_rotation_arc(Vec3::Y, delta / length),
                            scale: Vec3::new(0.022, length + 0.035, 0.022),
                        },
                        VoxelEntity,
                        WireframeEntity,
                    ));
                }
            }
        }
    }
    *last_render = Some((world.revision, *debug));
}

fn quad_edges(positions: &[[f32; 3]]) -> Vec<(Vec3, Vec3)> {
    positions
        .chunks_exact(4)
        .flat_map(|quad| {
            let vertices = [
                Vec3::from_array(quad[0]),
                Vec3::from_array(quad[1]),
                Vec3::from_array(quad[2]),
                Vec3::from_array(quad[3]),
            ];
            [
                (vertices[0], vertices[1]),
                (vertices[1], vertices[2]),
                (vertices[2], vertices[3]),
                (vertices[3], vertices[0]),
            ]
        })
        .collect()
}

struct TechnicalMapPaths {
    height: String,
    normal: String,
    orm: String,
}

fn technical_map_paths(base_color: &str) -> Option<TechnicalMapPaths> {
    let directory = base_color.strip_suffix("/base_color.png")?;
    let maps = TechnicalMapPaths {
        height: format!("{directory}/height.png"),
        normal: format!("{directory}/normal.png"),
        orm: format!("{directory}/orm.png"),
    };
    technical_maps_exist(&maps).then_some(maps)
}

#[cfg(not(target_arch = "wasm32"))]
fn technical_maps_exist(maps: &TechnicalMapPaths) -> bool {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
    asset_root.join(&maps.height).is_file()
        && asset_root.join(&maps.normal).is_file()
        && asset_root.join(&maps.orm).is_file()
}

#[cfg(target_arch = "wasm32")]
fn technical_maps_exist(_maps: &TechnicalMapPaths) -> bool {
    // Web assets are declared in web/games.json and preloaded before the game starts.
    true
}

fn load_clamped_texture(asset_server: &AssetServer, path: String, is_srgb: bool) -> Handle<Image> {
    asset_server.load_with_settings(path, move |settings: &mut ImageLoaderSettings| {
        settings.is_srgb = is_srgb;
        settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::ClampToEdge,
            address_mode_v: ImageAddressMode::ClampToEdge,
            ..default()
        });
    })
}

fn sync_hud(
    world: Res<VoxelWorld>,
    editor: Res<EditorState>,
    blueprint_status: Res<BlueprintStatus>,
    debug: Res<DebugState>,
    mut count: Query<
        &mut Text,
        (
            With<CountText>,
            Without<MaterialText>,
            Without<BlueprintStatusText>,
        ),
    >,
    mut selected: Query<
        &mut Text,
        (
            With<MaterialText>,
            Without<CountText>,
            Without<BlueprintStatusText>,
        ),
    >,
    mut status: Query<
        &mut Text,
        (
            With<BlueprintStatusText>,
            Without<CountText>,
            Without<MaterialText>,
        ),
    >,
    mut debug_status: Query<
        &mut Text,
        (
            With<DebugStatusText>,
            Without<CountText>,
            Without<MaterialText>,
            Without<BlueprintStatusText>,
        ),
    >,
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
    if blueprint_status.is_changed() {
        if let Ok(mut text) = status.single_mut() {
            **text = blueprint_status.0.clone();
        }
    }
    if debug.is_changed() {
        if let Ok(mut text) = debug_status.single_mut() {
            **text = format!(
                "UV: {} · Wire: {} · Map: {}",
                if debug.uv_view { "on" } else { "off" },
                if debug.wireframe { "on" } else { "off" },
                debug.tech_map.label(),
            );
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
    fn new_world_starts_with_a_centered_wooden_boat() {
        let world = new_ship_world();
        assert_eq!(world.blocks.len(), 87);
        assert!(
            world
                .blocks
                .values()
                .all(|kind| *kind == BlockKind::WoodenPlank)
        );
    }

    #[test]
    fn ray_hits_front_face_of_starter_boat() {
        let hit = raycast_voxels(
            Vec3::new(0.0, 0.0, 5.0),
            Vec3::NEG_Z,
            &new_ship_world().blocks,
        )
        .expect("starter boat should be hit");
        assert_eq!(hit.normal, IVec3::Z);
    }

    #[test]
    fn blueprint_round_trip_preserves_cells_and_materials() {
        // The portable MagicaVoxel subset stores model-local coordinates; the
        // importer centers X/Z in the editor, matching a centered export.
        let mut world = VoxelWorld::single(IVec3::new(-3, 1, 2), BlockKind::WoodenPlank);
        world.blocks.insert(IVec3::new(2, 0, -3), BlockKind::Iron);
        let restored = parse_blueprint(&serialize_blueprint(&world).expect("blueprint writes"))
            .expect("blueprint parses");
        assert_eq!(restored, world.blocks);
    }

    #[test]
    fn blueprint_rejects_non_vox_data() {
        assert!(
            parse_blueprint(b"not a voxel file")
                .unwrap_err()
                .contains("VOX")
        );
    }

    #[test]
    fn tech_map_cycle_returns_to_base_color() {
        let mut map = TechMapView::BaseColor;
        for _ in 0..6 {
            map = map.next();
        }
        assert_eq!(map, TechMapView::BaseColor);
    }

    #[test]
    fn quad_edge_builder_emits_four_edges_per_quad() {
        let edges = quad_edges(&[
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ]);
        assert_eq!(edges.len(), 4);
        assert_eq!(edges[0], (Vec3::ZERO, Vec3::X));
    }

    #[test]
    fn generated_technical_maps_are_discovered_beside_base_color() {
        let maps = technical_map_paths("voxel_materials/wooden_plank/textures/base_color.png")
            .expect("checked-in wooden plank maps should be discovered");
        assert_eq!(
            maps.height,
            "voxel_materials/wooden_plank/textures/height.png"
        );
        assert_eq!(
            maps.normal,
            "voxel_materials/wooden_plank/textures/normal.png"
        );
        assert_eq!(maps.orm, "voxel_materials/wooden_plank/textures/orm.png");
        assert!(technical_map_paths("textures/shipwright/stone.png").is_none());
    }
}
