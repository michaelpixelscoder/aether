//! Shared presentation for the game and the independent gallery.
pub mod art;
pub mod avatar;
pub mod clouds;
pub mod craft;
pub mod environment;
pub mod fauna;
pub mod materials;
pub mod mechanisms;
mod mipmaps;
pub mod sail;
pub mod ship;
pub mod traffic;
pub mod voxel;
pub mod weather;
pub mod widgets;
pub mod world;

use bevy::prelude::*;
pub use materials::Palette;
pub use voxel::{BodyVisual, ChunkVisual, MeshMetrics, PartVisual, PrepareVoxels};

/// Bounds for a newly built mesh whose vertices will never mutate in place.
/// Editable bodies replace both entity and mesh after an edit; their replacement
/// must call this again. Never apply this to skinned or GPU-morphed surfaces.
pub(crate) fn immutable_bounds(mesh: &Mesh) -> impl Bundle {
    use bevy::camera::{primitives::MeshAabb, visibility::NoAutoAabb};
    (
        mesh.compute_aabb().expect("nonempty immutable mesh"),
        NoAutoAabb,
    )
}

pub struct PresentationPlugin;
impl Plugin for PresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<art::SkyMaterial>::default())
            .add_plugins(MaterialPlugin::<clouds::CloudMaterial>::default())
            .add_systems(Update, clouds::animate)
            .add_plugins(MaterialPlugin::<world::FlowMaterial>::default())
            .add_systems(Startup, world::setup)
            .add_systems(Startup, fauna::setup)
            .add_systems(Startup, traffic::setup)
            .add_systems(Update, traffic::animate)
            .add_systems(Update, fauna::animate)
            .add_systems(Update, world::animate)
            .init_resource::<weather::WeatherView>()
            .add_systems(Startup, weather::setup)
            .add_systems(Update, weather::update)
            .add_systems(PostUpdate, art::sync_environment_lighting)
            .add_systems(Startup, materials::setup)
            .add_systems(Startup, art::setup)
            .add_systems(Startup, ship::setup)
            .init_resource::<ship::ShipMeshes>()
            .add_systems(Update, ship::prepare.after(PrepareVoxels))
            .add_systems(
                PostUpdate,
                art::follow_camera.before(TransformSystems::Propagate),
            )
            .add_systems(Startup, avatar::setup)
            .add_systems(Update, avatar::animate)
            .add_systems(PostUpdate, sail::animate)
            .add_systems(PostUpdate, mechanisms::animate)
            .add_systems(Startup, widgets::load_fonts)
            .add_systems(Update, widgets::apply_fonts)
            .add_systems(Update, mipmaps::prepare)
            .add_plugins(voxel::VoxelPlugin);
    }
}
pub fn asset_root() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        "assets".into()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Ok(path) = std::env::var("AETHER_ASSETS") {
            return path;
        }
        let packaged = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.join("assets")));
        if let Some(path) = packaged
            .as_ref()
            .filter(|p| p.join("textures/cedar.png").is_file())
        {
            return path.to_string_lossy().into_owned();
        }
        if std::path::Path::new("assets/textures/cedar.png").exists()
            && let Ok(directory) = std::env::current_dir()
        {
            return directory.join("assets").to_string_lossy().into_owned();
        }
        packaged
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| "assets".into())
    }
}
