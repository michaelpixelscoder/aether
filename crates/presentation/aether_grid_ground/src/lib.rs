use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

pub struct GridGroundPlugin;

impl Plugin for GridGroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_grid_ground);
    }
}

fn spawn_grid_ground(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let texture = images.add(make_grid_texture());
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(texture),
        perceptual_roughness: 0.92,
        ..default()
    });

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(100.0, 100.0))),
        MeshMaterial3d(material),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 14_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, -0.6, 0.0)),
    ));

    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.62, 0.70, 0.86),
        brightness: 180.0,
        affects_lightmapped_meshes: true,
    });
}

fn make_grid_texture() -> Image {
    const SIZE: u32 = 1024;
    const CELL: u32 = 32;
    let mut pixels = Vec::with_capacity((SIZE * SIZE * 4) as usize);

    for y in 0..SIZE {
        for x in 0..SIZE {
            let major = x % (CELL * 5) < 3 || y % (CELL * 5) < 3;
            let minor = x % CELL < 2 || y % CELL < 2;
            let rgba = if major {
                [55, 112, 135, 255]
            } else if minor {
                [67, 81, 92, 255]
            } else {
                [37, 43, 49, 255]
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
