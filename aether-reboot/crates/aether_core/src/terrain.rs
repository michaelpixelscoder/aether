//! Authored archipelago geometry, shared by rendering, collision and line of sight.
use glam::Vec3;
use std::sync::LazyLock;

#[derive(Clone, Copy, Debug)]
pub struct IslandSpec {
    pub center: Vec3,
    pub radius: f32,
    pub name: &'static str,
    pub dock: Vec3,
}
pub const ISLANDS: [IslandSpec; 3] = [
    IslandSpec {
        center: Vec3::new(-18.0, 6.0, 12.0),
        radius: 13.0,
        name: "LE CHANTIER",
        dock: Vec3::new(0.0, 12.0, 0.0),
    },
    IslandSpec {
        center: Vec3::new(84.0, 14.0, -166.0),
        radius: 18.0,
        name: "LES JARDINS SUSPENDUS",
        dock: Vec3::new(65.0, 20.0, -154.0),
    },
    IslandSpec {
        center: Vec3::new(-42.0, 2.0, -105.0),
        radius: 12.0,
        name: "LE REFUGE",
        dock: Vec3::new(-28.0, 8.0, -98.0),
    },
];
pub const ANCHORS: [Vec3; 3] = [
    Vec3::new(32.0, 28.0, -80.0),
    Vec3::new(58.0, 34.0, -121.0),
    Vec3::new(-18.0, 22.0, -57.0),
];
pub const SANCTUARY: Vec3 = Vec3::new(-72.0, 28.0, -65.0);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Surface {
    Rock,
    Grass,
    Trunk,
    Foliage,
    Crystal,
}
#[derive(Clone, Copy, Debug)]
pub struct TerrainBox {
    pub center: Vec3,
    pub size: Vec3,
    pub surface: Surface,
}
/// Join coplanar, touching boxes without changing their union. Large voxel hulls
/// otherwise repeat narrow-phase work against every 3 m turf tile.
pub fn collision_boxes(pieces: &[TerrainBox]) -> Vec<TerrainBox> {
    let mut out = pieces.to_vec();
    for axis in [0, 2, 1] {
        loop {
            let mut pair = None;
            'search: for i in 0..out.len() {
                for j in i + 1..out.len() {
                    let (a, b) = (out[i], out[j]);
                    if a.surface != b.surface {
                        continue;
                    }
                    if (0..3).filter(|&k| k != axis).any(|k| {
                        (a.center[k] - b.center[k]).abs() > 1e-5
                            || (a.size[k] - b.size[k]).abs() > 1e-5
                    }) {
                        continue;
                    }
                    if ((a.center[axis] - b.center[axis]).abs()
                        - (a.size[axis] + b.size[axis]) * 0.5)
                        .abs()
                        < 1e-5
                    {
                        pair = Some((i, j));
                        break 'search;
                    }
                }
            }
            let Some((i, j)) = pair else {
                break;
            };
            let b = out.remove(j);
            let a = &mut out[i];
            let low =
                (a.center[axis] - a.size[axis] * 0.5).min(b.center[axis] - b.size[axis] * 0.5);
            let high =
                (a.center[axis] + a.size[axis] * 0.5).max(b.center[axis] + b.size[axis] * 0.5);
            a.center[axis] = (low + high) * 0.5;
            a.size[axis] = high - low;
        }
    }
    out
}
/// The distant observatory is reachable and collidable, not a painted obstacle.
pub fn sanctuary_pieces() -> Vec<TerrainBox> {
    let mut pieces = Vec::new();
    let mut add = |center: Vec3, size: Vec3, surface| {
        pieces.push(TerrainBox {
            center: center + SANCTUARY,
            size,
            surface,
        })
    };
    for (y, r) in [
        (0.0, 8.0_f32),
        (-2.0, 7.0),
        (-4.0, 5.5),
        (-6.0, 4.0),
        (-8.0, 2.5),
        (-10.0, 1.0),
    ] {
        let bound = r.ceil() as i32;
        for x in (-bound..=bound).step_by(2) {
            for z in (-bound..=bound).step_by(2) {
                if (x as f32).hypot(z as f32) < r {
                    add(
                        Vec3::new(x as f32, y, z as f32),
                        Vec3::splat(2.0),
                        Surface::Rock,
                    );
                }
            }
        }
    }
    add(
        Vec3::new(0.0, 1.1, 0.0),
        Vec3::new(11.0, 0.3, 10.0),
        Surface::Grass,
    );
    for y in [2.0, 5.5, 9.0, 12.5] {
        add(
            Vec3::new(0.0, y, 0.0),
            Vec3::new(4.8, 0.55, 4.8),
            Surface::Rock,
        );
        for x in [-1.7, 1.7] {
            for z in [-1.7, 1.7] {
                add(
                    Vec3::new(x, y + 1.8, z),
                    Vec3::new(0.65, 3.0, 0.65),
                    Surface::Rock,
                );
            }
        }
        for z in [-2.0, 2.0] {
            add(
                Vec3::new(0.0, y + 0.8, z),
                Vec3::new(3.4, 0.3, 0.38),
                Surface::Rock,
            );
        }
    }
    for i in 0..5 {
        let i = i as f32;
        add(
            Vec3::new(0.0, 14.0 + i * 0.5, 0.0),
            Vec3::new(4.9 - i * 0.85, 0.5, 4.9 - i * 0.85),
            Surface::Rock,
        );
    }
    add(
        Vec3::new(0.0, 17.4, 0.0),
        Vec3::new(0.5, 1.6, 0.5),
        Surface::Crystal,
    );
    for (x, z) in [(-4.0, 3.0), (4.0, 2.0), (3.0, -3.0)] {
        add(
            Vec3::new(x, 3.1, z),
            Vec3::new(0.6, 3.6, 0.6),
            Surface::Trunk,
        );
        for n in 0..3 {
            let n = n as f32;
            let size = 3.0 - n * 0.7;
            add(
                Vec3::new(x, 4.3 + n, z),
                Vec3::new(size, 1.4, size),
                Surface::Foliage,
            );
        }
    }
    pieces
}
pub fn island_pieces(center: Vec3, radius: f32, seed: u32) -> Vec<TerrainBox> {
    let mut pieces = Vec::new();
    let mut add = |center, size, surface| {
        pieces.push(TerrainBox {
            center,
            size,
            surface,
        })
    };
    let steps = (radius / 3.0).ceil() as i32;
    for z in -steps..=steps {
        for x in -steps..=steps {
            let p = Vec3::new(x as f32 * 3.0, 0.0, z as f32 * 3.0);
            let d = p.length() / radius;
            if d > 1.0 {
                continue;
            }
            let noise = ((x * 73 + z * 37 + seed as i32 * 19).unsigned_abs() % 7) as f32 / 7.0;
            let depth = (1.0 - d) * 15.0 + 2.0 + noise * 2.0;
            add(
                center + p - Vec3::Y * (depth * 0.5 - 2.5),
                Vec3::new(3.0, depth, 3.0),
                Surface::Rock,
            );
            add(
                center + p + Vec3::Y * 2.75,
                Vec3::new(3.0, 0.5, 3.0),
                Surface::Grass,
            );
            if (x * 11 + z * 17 + seed as i32).rem_euclid(13) == 0 && d > 0.28 && d < 0.8 {
                let base = center + p + Vec3::Y * 3.0;
                add(
                    base + Vec3::Y * 1.8,
                    Vec3::new(0.6, 3.6, 0.6),
                    Surface::Trunk,
                );
                for n in 0..3 {
                    let size = 3.0 - n as f32 * 0.7;
                    add(
                        base + Vec3::Y * (3.0 + n as f32),
                        Vec3::new(size, 1.4, size),
                        Surface::Foliage,
                    );
                }
            }
        }
    }
    // Each island keeps a small, reachable remnant of the old observatories.
    // Architecture is part of the shared geometry, including cable occlusion.
    add(
        center + Vec3::Y * 3.2,
        Vec3::new(6.0, 0.4, 5.0),
        Surface::Rock,
    );
    for x in [-1.9, 1.9] {
        for z in [-1.4, 1.4] {
            let height = if x > 0.0 && z > 0.0 { 2.8 } else { 4.0 };
            add(
                center + Vec3::new(x, 3.6, z),
                Vec3::new(1.1, 0.4, 1.1),
                Surface::Rock,
            );
            add(
                center + Vec3::new(x, 3.8 + height * 0.5, z),
                Vec3::new(0.65, height, 0.65),
                Surface::Rock,
            );
            add(
                center + Vec3::new(x, 3.9 + height, z),
                Vec3::new(1.05, 0.3, 1.05),
                Surface::Rock,
            );
        }
    }
    add(
        center + Vec3::new(0.0, 8.1, -1.4),
        Vec3::new(5.0, 0.4, 1.1),
        Surface::Rock,
    );
    for x in [-1.8, -0.6, 0.6, 1.8] {
        add(
            center + Vec3::new(x, 8.5, -1.4),
            Vec3::new(0.65, 0.4, 0.8),
            Surface::Rock,
        );
    }
    for i in 0..4 {
        let angle = i as f32 * std::f32::consts::FRAC_PI_2 + seed as f32;
        add(
            center
                + Vec3::new(
                    angle.cos() * radius * 0.7,
                    -5.0 - i as f32 * 2.0,
                    angle.sin() * radius * 0.7,
                ),
            Vec3::new(0.9, 3.0, 0.9),
            Surface::Crystal,
        );
    }
    pieces
}
pub fn dock_boxes(position: Vec3) -> Vec<TerrainBox> {
    let mut boxes = vec![TerrainBox {
        center: position - Vec3::Y * 1.5,
        size: Vec3::new(7.0, 0.5, 10.0),
        surface: Surface::Trunk,
    }];
    for x in [-3.4, 3.4] {
        for z in [-4.7, 4.7] {
            boxes.push(TerrainBox {
                center: position + Vec3::new(x, 0.0, z),
                size: Vec3::new(0.25, 3.0, 0.25),
                surface: Surface::Trunk,
            });
        }
    }
    boxes
}
pub fn obstacles() -> &'static [TerrainBox] {
    static BOXES: LazyLock<Vec<TerrainBox>> = LazyLock::new(|| {
        ISLANDS
            .iter()
            .enumerate()
            .flat_map(|(i, s)| {
                let mut p = island_pieces(s.center, s.radius, i as u32);
                p.extend(dock_boxes(s.dock));
                p
            })
            .chain(sanctuary_pieces())
            .collect()
    });
    &BOXES
}
/// Exact finite-segment slab test; independent of sample spacing or frame rate.
pub fn segment_intersects(a: Vec3, b: Vec3, shape: &TerrainBox) -> bool {
    let lo = shape.center - shape.size * 0.5;
    let hi = shape.center + shape.size * 0.5;
    let d = b - a;
    let mut near = 0.001_f32;
    let mut far = 0.999_f32;
    for axis in 0..3 {
        if d[axis].abs() < 1e-7 {
            if a[axis] < lo[axis] || a[axis] > hi[axis] {
                return false;
            }
        } else {
            let x = (lo[axis] - a[axis]) / d[axis];
            let y = (hi[axis] - a[axis]) / d[axis];
            near = near.max(x.min(y));
            far = far.min(x.max(y));
            if near > far {
                return false;
            }
        }
    }
    near <= far
}
pub fn line_clear(a: Vec3, b: Vec3) -> bool {
    a.is_finite()
        && b.is_finite()
        && !obstacles().iter().any(|s| segment_intersects(a, b, s))
        && crate::world_geometry::line_clear(a, b)
}
