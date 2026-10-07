//! Authored meshes and collision share this checked-in Blender export.
use crate::{
    terrain::{Surface, TerrainBox, segment_intersects},
    world::{self, Biome},
};
use glam::Vec3;
use serde::Deserialize;
use std::{collections::BTreeMap, sync::LazyLock};

#[derive(Deserialize)]
struct BoxData {
    center: Vec3,
    size: Vec3,
}
#[derive(Clone, Deserialize)]
pub struct Waterfall {
    pub position: Vec3,
    pub width: f32,
    pub height: f32,
}
#[derive(Clone, Deserialize)]
pub struct Pool {
    pub position: Vec3,
    pub size: Vec3,
}
#[derive(Clone, Deserialize)]
pub struct Landmarks {
    pub waterfalls: Vec<Waterfall>,
    pub pools: Vec<Pool>,
    pub crystals: Vec<Vec3>,
    pub portal: Option<Vec3>,
}
pub fn pieces(biome: Biome) -> &'static [TerrainBox] {
    pieces_for(biome.asset())
}
pub fn pieces_for(key: &str) -> &'static [TerrainBox] {
    static TEMPLATES: LazyLock<BTreeMap<String, Vec<TerrainBox>>> = LazyLock::new(|| {
        let raw: BTreeMap<String, Vec<BoxData>> =
            serde_json::from_str(include_str!("../../../assets/world/collisions.json"))
                .expect("authored collision manifest");
        raw.into_iter()
            .map(|(name, pieces)| {
                (
                    name,
                    pieces
                        .into_iter()
                        .map(|p| TerrainBox {
                            center: p.center,
                            size: p.size,
                            surface: Surface::Rock,
                        })
                        .collect(),
                )
            })
            .collect()
    });
    TEMPLATES.get(key).expect("every biome has collision")
}
pub fn landmarks(biome: Biome) -> &'static Landmarks {
    landmarks_for(biome.asset())
}
pub fn landmarks_for(key: &str) -> &'static Landmarks {
    static DATA: LazyLock<BTreeMap<String, Landmarks>> = LazyLock::new(|| {
        serde_json::from_str(include_str!("../../../assets/world/landmarks.json"))
            .expect("authored landmark manifest")
    });
    DATA.get(key).expect("every template has landmarks")
}
pub fn vault_pieces() -> &'static [TerrainBox] {
    static DATA: LazyLock<Vec<TerrainBox>> = LazyLock::new(|| {
        let raw: Vec<BoxData> =
            serde_json::from_str(include_str!("../../../assets/world/vault-collisions.json"))
                .expect("vault export");
        raw.into_iter()
            .map(|p| TerrainBox {
                center: p.center,
                size: p.size,
                surface: Surface::Rock,
            })
            .collect()
    });
    &DATA
}
pub fn line_clear(a: Vec3, b: Vec3) -> bool {
    corridor_clear(a, b, 0.0)
}
pub fn corridor_clear(a: Vec3, b: Vec3, radius: f32) -> bool {
    for vault in world::vaults() {
        let inverse = glam::Quat::from_rotation_y(-vault.yaw);
        let a = inverse * (a - vault.center);
        let b = inverse * (b - vault.center);
        let broad = TerrainBox {
            center: Vec3::new(0.0, -75.0, 0.0),
            size: Vec3::new(3500.0, 1150.0, 3500.0),
            surface: Surface::Rock,
        };
        if segment_intersects(a, b, &broad)
            && vault_pieces().iter().any(|p| {
                let mut p = *p;
                p.size += Vec3::splat(radius * 2.0);
                segment_intersects(a, b, &p)
            })
        {
            return false;
        }
    }
    for island in world::islands() {
        let inverse = glam::Quat::from_rotation_y(-island.yaw);
        let a = inverse * (a - island.center) / island.scale;
        let b = inverse * (b - island.center) / island.scale;
        let inflate = |mut piece: TerrainBox| {
            piece.size += Vec3::splat(2.0 * radius / island.scale);
            piece
        };
        if !segment_intersects(a, b, &inflate(bounds_for(island.asset_key()))) {
            continue;
        }
        if pieces_for(island.asset_key())
            .iter()
            .any(|p| segment_intersects(a, b, &inflate(*p)))
        {
            return false;
        }
    }
    true
}
pub fn bounds(biome: Biome) -> TerrainBox {
    bounds_for(biome.asset())
}
pub fn bounds_for(key: &str) -> TerrainBox {
    static BOUNDS: LazyLock<BTreeMap<&'static str, TerrainBox>> = LazyLock::new(|| {
        world::ASSETS
            .iter()
            .map(|b| {
                let mut min = Vec3::splat(f32::INFINITY);
                let mut max = Vec3::splat(f32::NEG_INFINITY);
                for p in pieces_for(b) {
                    min = min.min(p.center - p.size * 0.5);
                    max = max.max(p.center + p.size * 0.5);
                }
                (
                    *b,
                    TerrainBox {
                        center: (min + max) * 0.5,
                        size: max - min,
                        surface: Surface::Rock,
                    },
                )
            })
            .collect()
    });
    BOUNDS[key]
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_export_has_finite_solid_geometry_and_clear_dock_approach() {
        for biome in Biome::ALL {
            let boxes = pieces(biome);
            assert!(boxes.len() > 30, "missing environment {biome:?}");
            for p in boxes {
                assert!(p.center.is_finite() && p.size.is_finite() && p.size.min_element() > 0.0);
            }
            assert!(
                !boxes.iter().any(|p| segment_intersects(
                    Vec3::new(0.0, 4.0, 100.0),
                    Vec3::new(0.0, 4.0, 78.0),
                    p
                )),
                "blocked dock {biome:?}"
            );
            for w in &landmarks(biome).waterfalls {
                assert!(w.position.is_finite() && w.width > 0.0 && w.height > 0.0);
            }
        }
    }
}
