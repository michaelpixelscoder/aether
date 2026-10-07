//! Canonical visual fixtures. Uses the production renderer without game/simulation.
use aether_view::{BodyVisual, MeshMetrics, Palette, PresentationPlugin};
use bevy::{asset::AssetPlugin, light::GlobalAmbientLight, prelude::*};
#[derive(Resource)]
struct Capture {
    path: String,
    smoke: bool,
    elapsed: f32,
    captured: bool,
    wood: Handle<Image>,
}
fn main() -> AppExit {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.12, 0.17, 0.25)))
        .insert_resource(GlobalAmbientLight {
            brightness: 400.0,
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
                        title: "Aether Isles — Galerie de référence".into(),
                        resolution: bevy::window::WindowResolution::new(1280, 720)
                            .with_scale_factor_override(1.0),
                        ..default()
                    }),
                    ..default()
                }),
            PresentationPlugin,
        ))
        .add_systems(Startup, setup.after(aether_view::materials::setup))
        .add_systems(Update, capture)
        .run()
}
fn setup(mut commands: Commands, palette: Res<Palette>, assets: Res<AssetServer>) {
    let args: Vec<String> = std::env::args().collect();
    let path = args
        .windows(2)
        .find(|a| a[0] == "--capture")
        .map_or("docs/evidence/gallery.png".into(), |a| a[1].clone());
    commands.insert_resource(Capture {
        path,
        smoke: args.iter().any(|a| a == "--smoke"),
        elapsed: 0.0,
        captured: false,
        wood: aether_view::materials::wood_texture(&assets),
    });
    commands.spawn((
        Camera3d::default(),
        aether_view::art::camera_effects(&assets),
        Transform::from_xyz(20.0, 18.0, 26.0).looking_at(Vec3::new(0.0, 0.0, -1.0), Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 15000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(10.0, 20.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        BodyVisual::new(&aether_core::fixtures::starter()),
        Transform::from_xyz(-7.0, 0.0, 0.0),
        Visibility::default(),
    ));
    use aether_core::{Block, Blueprint, Body, Cell};
    let mut cells = Vec::new();
    for z in -4_i32..=4 {
        for x in -4_i32..=4 {
            cells.push((Cell(x, 0, z), Block::Wood));
            if x.abs() == 4 || z.abs() == 4 {
                for y in 1..=4 {
                    cells.push((
                        Cell(x, y, z),
                        if y == 4 { Block::Metal } else { Block::Glass },
                    ));
                }
            }
        }
    }
    let glass = Body::from_blueprint(Blueprint {
        circuit_design: None,
        name: "Vitrage".into(),
        cells,
        parts: vec![],
    })
    .unwrap();
    commands.spawn((
        BodyVisual::new(&glass),
        Transform::from_rotation(Quat::from_rotation_y(0.35)),
        Visibility::default(),
    ));
    let cells = (-17..=17)
        .flat_map(|x| (-2..=2).map(move |z| (Cell(x, 0, z), Block::Wood)))
        .collect();
    let bridge = Body::from_blueprint(Blueprint {
        circuit_design: None,
        name: "Raccords -16 / 0 / 16".into(),
        cells,
        parts: vec![],
    })
    .unwrap();
    commands.spawn((
        BodyVisual::new(&bridge),
        Transform::from_xyz(8.0, 0.0, -1.0).with_rotation(Quat::from_rotation_y(1.2)),
        Visibility::default(),
    ));
    commands.spawn((
        Mesh3d(palette.cube.clone()),
        MeshMaterial3d(palette.rock.clone()),
        Transform::from_xyz(0.0, -1.3, 0.0).with_scale(Vec3::new(33.0, 0.5, 18.0)),
    ));
    commands.spawn((Text::new("AETHER ISLES  /  GALERIE\nCoque fonctionnelle · vitrage tourné · raccords de chunks négatifs"),TextFont{font:bevy::text::FontSource::Handle(assets.load("fonts/notosans.ttf")),font_size:bevy::text::FontSize::Px(21.0),..default()},Node{position_type:PositionType::Absolute,left:px(28),top:px(24),..default()}));
}
fn capture(
    mut commands: Commands,
    time: Res<Time<Real>>,
    metrics: Res<MeshMetrics>,
    assets: Res<AssetServer>,
    art: Res<aether_view::art::ArtAssets>,
    mut capture: ResMut<Capture>,
    mut exit: MessageWriter<AppExit>,
) {
    capture.elapsed += time.delta_secs();
    if capture.elapsed > 8.0
        && metrics.pending == 0
        && assets.is_loaded_with_dependencies(capture.wood.id())
        && art
            .scenes
            .iter()
            .all(|scene| assets.is_loaded_with_dependencies(scene.id()))
        && !capture.captured
    {
        commands
            .spawn(bevy::render::view::screenshot::Screenshot::primary_window())
            .observe(bevy::render::view::screenshot::save_to_disk(
                capture.path.clone(),
            ));
        capture.captured = true;
    }
    if capture.smoke && capture.elapsed > 12.0 {
        exit.write(if capture.captured {
            AppExit::Success
        } else {
            AppExit::error()
        });
    }
}
