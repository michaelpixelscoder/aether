use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

use aether_voxels::{
    MacroAtlasRegion, greedy_mesh, normalized_quad_uvs, partition_quads_by_normal_axis,
    tile_quads_with_macro_atlas,
};
use bevy::{
    app::AppExit,
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor},
    mesh::Indices,
    prelude::*,
    render::{
        render_resource::PrimitiveTopology,
        view::screenshot::{Screenshot, ScreenshotCaptured},
    },
    window::{PresentMode, WindowResolution},
};
use image::{ImageBuffer, RgbImage};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;
const WARMUP_FRAMES: u32 = 30;

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

fn main() {
    let arguments = gallery_arguments();
    let output = arguments.output.clone();
    fs::create_dir_all(&output).unwrap_or_else(|error| {
        panic!("cannot create render gallery {}: {error}", output.display())
    });

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.035, 0.055, 0.085)))
        .insert_resource(GlobalAmbientLight {
            color: Color::srgb(0.62, 0.69, 0.80),
            brightness: 260.0,
            affects_lightmapped_meshes: true,
        })
        .insert_resource(GalleryState::new(arguments))
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: "../../../assets".into(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Aether voxel render gallery".into(),
                        resolution: WindowResolution::new(WIDTH, HEIGHT),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_systems(Startup, setup)
        .add_systems(Update, gallery_tick)
        .run();
}

struct GalleryArguments {
    output: PathBuf,
    texture: String,
    height: String,
    normal: String,
    orm: String,
}

fn gallery_arguments() -> GalleryArguments {
    let mut arguments = env::args().skip(1);
    let mut output = PathBuf::from("target/voxel-render-gallery");
    let mut texture = "voxel_materials/wooden_plank/textures/base_color.png".to_owned();
    let mut height = "voxel_materials/wooden_plank/textures/height.png".to_owned();
    let mut normal = "voxel_materials/wooden_plank/textures/normal.png".to_owned();
    let mut orm = "voxel_materials/wooden_plank/textures/orm.png".to_owned();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--output" => {
                output = arguments
                    .next()
                    .map(PathBuf::from)
                    .expect("--output requires a directory");
            }
            "--texture" => {
                texture = arguments.next().expect("--texture requires an asset path");
            }
            "--height" => height = arguments.next().expect("--height requires an asset path"),
            "--normal" => normal = arguments.next().expect("--normal requires an asset path"),
            "--orm" => orm = arguments.next().expect("--orm requires an asset path"),
            _ => panic!("unknown argument: {argument}"),
        }
    }
    GalleryArguments {
        output,
        texture,
        height,
        normal,
        orm,
    }
}

#[derive(Clone)]
struct Fixture {
    slug: &'static str,
    title: &'static str,
    blocks: HashMap<IVec3, ()>,
    body_transform: Transform,
}

impl Fixture {
    fn new(
        slug: &'static str,
        title: &'static str,
        cells: impl IntoIterator<Item = IVec3>,
    ) -> Self {
        Self {
            slug,
            title,
            blocks: cells.into_iter().map(|cell| (cell, ())).collect(),
            body_transform: Transform::IDENTITY,
        }
    }

    fn transformed(mut self, translation: Vec3, rotation: Quat) -> Self {
        self.body_transform = Transform::from_translation(translation).with_rotation(rotation);
        self
    }

    fn center_and_radius(&self) -> (Vec3, f32) {
        let mut minimum = IVec3::splat(i32::MAX);
        let mut maximum = IVec3::splat(i32::MIN);
        for cell in self.blocks.keys() {
            minimum = minimum.min(*cell);
            maximum = maximum.max(*cell);
        }
        let local_center = (minimum.as_vec3() + maximum.as_vec3()) * 0.5;
        let center = self.body_transform.transform_point(local_center);
        let extent = (maximum - minimum + IVec3::ONE).as_vec3();
        (center, extent.max_element().max(2.0))
    }
}

#[derive(Clone)]
struct CaptureRecord {
    slug: String,
    title: String,
    file_name: String,
    voxels: usize,
    vertices: usize,
    indices: usize,
}

#[derive(Resource)]
struct GalleryState {
    output: PathBuf,
    texture: String,
    height: String,
    normal: String,
    orm: String,
    fixtures: Vec<Fixture>,
    index: usize,
    warmup_frames: u32,
    capture_pending: bool,
    needs_build: bool,
    current_vertices: usize,
    current_indices: usize,
    captures: Vec<CaptureRecord>,
}

impl GalleryState {
    fn new(arguments: GalleryArguments) -> Self {
        Self {
            output: arguments.output,
            texture: arguments.texture,
            height: arguments.height,
            normal: arguments.normal,
            orm: arguments.orm,
            fixtures: fixtures(),
            index: 0,
            warmup_frames: 0,
            capture_pending: false,
            needs_build: true,
            current_vertices: 0,
            current_indices: 0,
            captures: Vec::new(),
        }
    }
}

#[derive(Resource)]
struct GalleryAssets {
    wooden_plank: Handle<StandardMaterial>,
    wooden_end_grain: Handle<StandardMaterial>,
    textures: Vec<Handle<Image>>,
}

#[derive(Component)]
struct FixtureScene;

#[derive(Component)]
struct GalleryCamera;

#[derive(Component)]
struct FixtureLabel;

fn setup(
    mut commands: Commands,
    gallery: Res<GalleryState>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Keep the shaded, outward-facing end grain readable in the gallery. The
    // directional key still defines form; this only prevents the opposite bow
    // from collapsing into an indistinguishable black silhouette.
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.32, 0.38, 0.46),
        brightness: 350.0,
        affects_lightmapped_meshes: true,
    });
    let wooden_plank_texture = asset_server.load_with_settings(
        gallery.texture.clone(),
        |settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                address_mode_u: ImageAddressMode::ClampToEdge,
                address_mode_v: ImageAddressMode::ClampToEdge,
                ..default()
            });
        },
    );
    let load_linear_repeat = |path: String| {
        asset_server.load_with_settings(path, |settings: &mut ImageLoaderSettings| {
            settings.is_srgb = false;
            settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                address_mode_u: ImageAddressMode::ClampToEdge,
                address_mode_v: ImageAddressMode::ClampToEdge,
                ..default()
            });
        })
    };
    let height_texture = load_linear_repeat(gallery.height.clone());
    let normal_texture = load_linear_repeat(gallery.normal.clone());
    let orm_texture = load_linear_repeat(gallery.orm.clone());
    let wooden_plank = materials.add(StandardMaterial {
        base_color_texture: Some(wooden_plank_texture.clone()),
        depth_map: Some(height_texture.clone()),
        parallax_depth_scale: 0.025,
        normal_map_texture: Some(normal_texture.clone()),
        metallic_roughness_texture: Some(orm_texture.clone()),
        occlusion_texture: Some(orm_texture.clone()),
        metallic: 1.0,
        perceptual_roughness: 1.0,
        reflectance: 0.32,
        ..default()
    });
    let end_grain_texture = asset_server.load_with_settings(
        "voxel_materials/wooden_plank/textures/end_grain.png",
        |settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                address_mode_u: ImageAddressMode::ClampToEdge,
                address_mode_v: ImageAddressMode::ClampToEdge,
                ..default()
            });
        },
    );
    let wooden_end_grain = materials.add(StandardMaterial {
        base_color_texture: Some(end_grain_texture.clone()),
        perceptual_roughness: 0.68,
        reflectance: 0.32,
        unlit: true,
        ..default()
    });
    commands.insert_resource(GalleryAssets {
        wooden_plank,
        wooden_end_grain,
        textures: vec![
            wooden_plank_texture,
            end_grain_texture,
            height_texture,
            normal_texture,
            orm_texture,
        ],
    });

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(8.0, 7.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        GalleryCamera,
    ));
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.91, 0.78),
            illuminance: 12_000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.85, -0.65, 0.0)),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(80.0, 80.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.12, 0.16, 0.20),
            perceptual_roughness: 1.0,
            ..default()
        })),
        Transform::from_xyz(0.0, -0.52, 0.0),
    ));
    commands.spawn((
        Text::new(""),
        TextFont::from_font_size(26.0),
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: px(20.0),
            left: px(24.0),
            padding: UiRect::all(px(10.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.82)),
        FixtureLabel,
    ));
}

fn gallery_tick(
    mut commands: Commands,
    mut gallery: ResMut<GalleryState>,
    gallery_assets: Option<Res<GalleryAssets>>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    scene_entities: Query<Entity, With<FixtureScene>>,
    mut camera: Single<&mut Transform, With<GalleryCamera>>,
    mut label: Single<&mut Text, With<FixtureLabel>>,
) {
    let Some(gallery_assets) = gallery_assets else {
        return;
    };
    if gallery.index >= gallery.fixtures.len() || gallery.capture_pending {
        return;
    }
    if gallery.needs_build {
        for entity in &scene_entities {
            commands.entity(entity).despawn();
        }
        let fixture = gallery.fixtures[gallery.index].clone();
        let voxel_meshes = greedy_mesh(&fixture.blocks, |_| true)
            .into_iter()
            .flat_map(|mesh| {
                let (mut end_grain, long_grain) = partition_quads_by_normal_axis(&mesh, 0);
                end_grain.uvs = normalized_quad_uvs(&end_grain.surface_coordinates);
                [
                    (end_grain, true),
                    (
                        tile_quads_with_macro_atlas(
                            &long_grain,
                            [5, 5],
                            [1280, 1280],
                            &WOODEN_PLANK_5X5_ATLAS,
                        ),
                        false,
                    ),
                ]
            })
            .filter(|(mesh, _)| !mesh.positions.is_empty())
            .collect::<Vec<_>>();
        gallery.current_vertices = voxel_meshes
            .iter()
            .map(|(mesh, _)| mesh.positions.len())
            .sum();
        gallery.current_indices = voxel_meshes
            .iter()
            .map(|(mesh, _)| mesh.indices.len())
            .sum();
        for (voxel_mesh, is_end_grain) in voxel_meshes {
            let mut mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::default(),
            );
            let uvs = voxel_mesh.uvs;
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, voxel_mesh.positions);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, voxel_mesh.normals);
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
            mesh.insert_indices(Indices::U32(voxel_mesh.indices));
            mesh.generate_tangents()
                .expect("voxel mesh must support tangent generation for normal mapping");
            commands.spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(if is_end_grain {
                    gallery_assets.wooden_end_grain.clone()
                } else {
                    gallery_assets.wooden_plank.clone()
                }),
                fixture.body_transform,
                FixtureScene,
            ));
        }
        let (center, radius) = fixture.center_and_radius();
        let view_direction = Vec3::new(1.25, 0.9, 1.45).normalize();
        let distance_scale = if fixture.slug == "starter-boat" {
            1.35
        } else {
            2.2
        };
        **camera =
            Transform::from_translation(center + view_direction * (radius * distance_scale + 2.0))
                .looking_at(center, Vec3::Y);
        ***label = format!(
            "Wood atlas | {}\n{} voxels | greedy face-size mapping",
            fixture.title,
            fixture.blocks.len()
        );
        gallery.needs_build = false;
        gallery.warmup_frames = 0;
        return;
    }

    if gallery_assets
        .textures
        .iter()
        .any(|texture| !asset_server.is_loaded_with_dependencies(texture))
    {
        return;
    }
    gallery.warmup_frames += 1;
    if gallery.warmup_frames < WARMUP_FRAMES {
        return;
    }

    gallery.capture_pending = true;
    let fixture = gallery.fixtures[gallery.index].clone();
    let output = gallery.output.clone();
    let vertices = gallery.current_vertices;
    let indices = gallery.current_indices;
    let file_name = format!("{:02}-{}.png", gallery.index + 1, fixture.slug);
    let path = output.join(&file_name);
    commands.spawn(Screenshot::primary_window()).observe(
        move |captured: On<ScreenshotCaptured>,
              mut gallery: ResMut<GalleryState>,
              mut exit: MessageWriter<AppExit>| {
            let save_result = captured
                .image
                .clone()
                .try_into_dynamic()
                .map(|image| image.to_rgb8())
                .map_err(|error| error.to_string())
                .and_then(|image| image.save(&path).map_err(|error| error.to_string()));
            if let Err(error) = save_result {
                eprintln!("failed to save {}: {error}", path.display());
                exit.write(AppExit::error());
                return;
            }
            gallery.captures.push(CaptureRecord {
                slug: fixture.slug.into(),
                title: fixture.title.into(),
                file_name: file_name.clone(),
                voxels: fixture.blocks.len(),
                vertices,
                indices,
            });
            gallery.index += 1;
            gallery.capture_pending = false;
            gallery.needs_build = true;
            if gallery.index == gallery.fixtures.len() {
                match finalize_gallery(&output, &gallery.captures) {
                    Ok(()) => {
                        println!(
                            "Voxel render gallery: {}",
                            output.join("index.html").display()
                        );
                        exit.write(AppExit::Success);
                    }
                    Err(error) => {
                        eprintln!("failed to finalize render gallery: {error}");
                        exit.write(AppExit::error());
                    }
                }
            }
        },
    );
}

fn finalize_gallery(output: &Path, captures: &[CaptureRecord]) -> Result<(), String> {
    let columns = 3u32;
    let rows = (captures.len() as u32).div_ceil(columns);
    let mut contact_sheet: RgbImage = ImageBuffer::new(WIDTH * columns, HEIGHT * rows);
    for (index, capture) in captures.iter().enumerate() {
        let image = image::open(output.join(&capture.file_name))
            .map_err(|error| error.to_string())?
            .to_rgb8();
        image::imageops::replace(
            &mut contact_sheet,
            &image,
            i64::from((index as u32 % columns) * WIDTH),
            i64::from((index as u32 / columns) * HEIGHT),
        );
    }
    contact_sheet
        .save(output.join("contact-sheet.png"))
        .map_err(|error| error.to_string())?;

    let cards = captures
        .iter()
        .map(|capture| {
            format!(
                "<figure><a href=\"{file}\"><img src=\"{file}\" alt=\"{title}\"></a><figcaption><strong>{title}</strong><br>{voxels} voxels · {vertices} vertices · {indices} indices</figcaption></figure>",
                file = capture.file_name,
                title = capture.title,
                voxels = capture.voxels,
                vertices = capture.vertices,
                indices = capture.indices,
            )
        })
        .collect::<String>();
    let html = format!(
        "<!doctype html><meta charset=\"utf-8\"><title>Aether Cycle 1 voxel renders</title><style>body{{margin:0;padding:32px;background:#09111c;color:#edf3fa;font:16px system-ui}}h1{{margin-top:0}}p{{color:#adbac8}}main{{display:grid;grid-template-columns:repeat(auto-fit,minmax(360px,1fr));gap:24px}}figure{{margin:0;background:#111e2d;padding:12px;border-radius:8px}}img{{display:block;width:100%;height:auto}}figcaption{{padding:10px 4px 2px;line-height:1.5}}</style><h1>Voxel shading · Cycle 1</h1><p>Wooden-plank surface mapping · fixed camera, lighting, 800×600 output, 5×5-voxel macro atlas.</p><p><a href=\"contact-sheet.png\">Open contact sheet</a></p><main>{cards}</main>"
    );
    fs::write(output.join("index.html"), html).map_err(|error| error.to_string())?;

    let records = captures
        .iter()
        .map(|capture| {
            format!(
                "  {{\"slug\":\"{}\",\"file\":\"{}\",\"voxels\":{},\"vertices\":{},\"indices\":{}}}",
                capture.slug, capture.file_name, capture.voxels, capture.vertices, capture.indices
            )
        })
        .collect::<Vec<_>>()
        .join(",\n");
    fs::write(
        output.join("manifest.json"),
        format!(
            "{{\n\"cycle\":1,\n\"texture_macro_voxels\":5,\n\"captures\":[\n{records}\n]\n}}\n"
        ),
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

fn fixtures() -> Vec<Fixture> {
    let single = Fixture::new("single", "Single voxel", [IVec3::ZERO]);
    let row = Fixture::new(
        "row",
        "Eight-voxel row",
        (0..8).map(|x| IVec3::new(x, 0, 0)),
    );
    let wall = Fixture::new(
        "wall",
        "Broad coplanar wall",
        (0..8).flat_map(|x| (0..5).map(move |y| IVec3::new(x, y, 0))),
    );
    let corner = Fixture::new(
        "outside-corner",
        "Connected outside corner",
        (0..6)
            .flat_map(|x| (0..4).map(move |y| IVec3::new(x, y, 0)))
            .chain((1..5).flat_map(|z| (0..4).map(move |y| IVec3::new(0, y, z)))),
    );
    let window = Fixture::new(
        "window-and-steps",
        "Window, end caps, and steps",
        (0..8)
            .flat_map(|x| (0..5).map(move |y| IVec3::new(x, y, 0)))
            .filter(|cell| !(cell.x >= 3 && cell.x <= 4 && cell.y >= 1 && cell.y <= 3))
            .chain((0..4).flat_map(|x| (0..=x).map(move |z| IVec3::new(x + 2, 0, z + 1)))),
    );
    let chunk_seam = Fixture::new(
        "chunk-seam",
        "Construction crossing x=16",
        (13..21).flat_map(|x| (0..4).map(move |y| IVec3::new(x, y, 0))),
    );
    let hull_cells = (-5i32..=5).flat_map(|x| {
        (-3i32..=3).filter_map(move |z| {
            let half_width = 3 - (x.abs() / 2);
            (z.abs() <= half_width).then_some(IVec3::new(x, 0, z))
        })
    });
    let hull = Fixture::new("hull", "Small hull-like construction", hull_cells);
    let transformed = Fixture::new(
        "transformed-body",
        "Moved and rotated voxel body",
        (0..6).flat_map(|x| (0..3).map(move |y| IVec3::new(x, y, 0))),
    )
    .transformed(Vec3::new(-4.0, 0.0, 3.0), Quat::from_rotation_y(0.65));
    let mut boat_cells = Vec::new();
    for x in 1..=10 {
        for z in -2..=2 {
            boat_cells.push(IVec3::new(x, 0, z));
        }
        boat_cells.extend([IVec3::new(x, 1, -3), IVec3::new(x, 1, 3)]);
    }
    for z in -1..=1 {
        boat_cells.extend([
            IVec3::new(0, 1, z),
            IVec3::new(11, 1, z),
            IVec3::new(0, 2, z),
        ]);
    }
    for x in [1, 2, 9, 10] {
        boat_cells.extend([IVec3::new(x, 2, -3), IVec3::new(x, 2, 3)]);
    }
    let boat = Fixture::new("starter-boat", "Simple all-wood starter boat", boat_cells);

    vec![
        single,
        row,
        wall,
        corner,
        window,
        chunk_seam,
        hull,
        transformed,
        boat,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_names_and_slugs_are_unique() {
        let fixtures = fixtures();
        let mut slugs = fixtures
            .iter()
            .map(|fixture| fixture.slug)
            .collect::<Vec<_>>();
        let mut titles = fixtures
            .iter()
            .map(|fixture| fixture.title)
            .collect::<Vec<_>>();
        slugs.sort_unstable();
        titles.sort_unstable();
        slugs.dedup();
        titles.dedup();
        assert_eq!(slugs.len(), fixtures.len());
        assert_eq!(titles.len(), fixtures.len());
    }

    #[test]
    fn cycle_one_fixture_matrix_covers_required_constructions() {
        let slugs = fixtures()
            .into_iter()
            .map(|fixture| fixture.slug)
            .collect::<Vec<_>>();
        for required in [
            "single",
            "row",
            "wall",
            "outside-corner",
            "window-and-steps",
            "chunk-seam",
            "hull",
            "transformed-body",
        ] {
            assert!(slugs.contains(&required), "missing fixture {required}");
        }
    }
}
