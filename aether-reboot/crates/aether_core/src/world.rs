//! Stable, authored geography and deterministic satellite placement, in metres.
//! Island IDs never depend on streaming order or Bevy entities.
use crate::{fields::Current, terrain::ISLANDS};
use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
pub const ASSETS: [&str; 12] = [
    "dawn",
    "crystal",
    "nomad",
    "ember",
    "frost",
    "verdant",
    "storm",
    "hollow",
    "underforge",
    "dawn-watch",
    "dawn-garden",
    "dawn-ruin",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Biome {
    Dawn,
    Crystal,
    Nomad,
    Ember,
    Frost,
    Verdant,
    Storm,
    Hollow,
    Underforge,
}
impl Biome {
    pub const ALL: [Self; 9] = [
        Self::Dawn,
        Self::Crystal,
        Self::Nomad,
        Self::Ember,
        Self::Frost,
        Self::Verdant,
        Self::Storm,
        Self::Hollow,
        Self::Underforge,
    ];
    pub fn asset(self) -> &'static str {
        [
            "dawn",
            "crystal",
            "nomad",
            "ember",
            "frost",
            "verdant",
            "storm",
            "hollow",
            "underforge",
        ][self as usize]
    }
    pub fn label(self) -> &'static str {
        [
            "Couronne de l'Aube",
            "Récifs de Cristal",
            "Routes du Soleil",
            "Forges des Braises",
            "Monastères du Givre",
            "Racines Verdoyantes",
            "Frontière des Tempêtes",
            "Profondeurs Oubliées",
            "Cité des Sous-forges",
        ][self as usize]
    }
}
#[derive(Clone, Debug)]
pub struct WorldIsland {
    pub id: u32,
    pub biome: Biome,
    pub center: Vec3,
    pub scale: f32,
    pub yaw: f32,
    pub capital: bool,
}
impl WorldIsland {
    pub fn asset_index(&self) -> usize {
        // These authored landmarks form a transition between inhabited Dawn
        // terraces, the portal ruins and the first mineral outcrop. Their
        // actual collision/interaction template follows the visible asset.
        match self.id {
            105 => return 1,
            104 | 107 => return 0,
            123 => return 11,
            _ => {}
        }
        if self.biome == Biome::Dawn && !self.capital {
            if self.id.is_multiple_of(2) { 9 } else { 10 }
        } else {
            self.biome as usize
        }
    }
    pub fn asset_key(&self) -> &'static str {
        ASSETS[self.asset_index()]
    }
    pub fn transform_point(&self, p: Vec3) -> Vec3 {
        self.center + Quat::from_rotation_y(self.yaw) * (p * self.scale)
    }
    pub fn radius(&self) -> f32 {
        100.0 * self.scale
    }
    pub fn dock(&self) -> Vec3 {
        // The authored deck top is y=.5. Preserve ship clearance in metres,
        // independently of island scaling, so a large port is still reachable.
        self.transform_point(Vec3::new(0.0, 0.5, 85.0)) + Vec3::Y * 1.25
    }
    pub fn label(&self) -> String {
        if self.capital {
            self.biome.label().into()
        } else {
            format!("{} · îlot {}", self.biome.label(), self.id % 100)
        }
    }
}
pub const REGION_CENTERS: [Vec3; 9] = [
    Vec3::new(-260.0, 45.0, -680.0),
    Vec3::new(1500.0, 480.0, -1750.0),
    Vec3::new(-1800.0, 160.0, -1400.0),
    Vec3::new(-2600.0, -120.0, -3400.0),
    Vec3::new(0.0, 1250.0, -3650.0),
    Vec3::new(2600.0, 100.0, -3400.0),
    Vec3::new(1300.0, 740.0, -5000.0),
    Vec3::new(-600.0, -1400.0, -1700.0),
    Vec3::new(1350.0, -2200.0, -3200.0),
];
#[derive(Clone, Copy)]
pub struct Vault {
    pub id: u32,
    pub center: Vec3,
    pub yaw: f32,
}
impl Vault {
    pub fn transform_point(self, p: Vec3) -> Vec3 {
        self.center + Quat::from_rotation_y(self.yaw) * p
    }
}
pub fn vaults() -> [Vault; 2] {
    let d = REGION_CENTERS[8] - REGION_CENTERS[7];
    let yaw = d.x.atan2(d.z);
    [
        Vault {
            id: 800,
            center: REGION_CENTERS[7],
            yaw,
        },
        Vault {
            id: 900,
            center: REGION_CENTERS[8],
            yaw,
        },
    ]
}
pub fn islands() -> &'static [WorldIsland] {
    static WORLD: LazyLock<Vec<WorldIsland>> = LazyLock::new(|| {
        let mut result = Vec::new();
        for (region, biome) in Biome::ALL.into_iter().enumerate() {
            result.push(WorldIsland {
                id: 100 + region as u32 * 100,
                biome,
                // The Nomad capital sits on the eastern side of its region.
                // Its formerly oversized silhouette was hidden by the old
                // cloud bank. Keep its real port, collision and art together.
                center: REGION_CENTERS[region]
                    + if region == 2 {
                        Vec3::new(100.0, -35.0, -70.0)
                    } else {
                        Vec3::ZERO
                    },
                scale: match region {
                    0 => 2.1,
                    2 => 1.4,
                    _ => 2.8,
                },
                yaw: 0.0,
                capital: true,
            });
        }
        // Authored landmarks compose the first archipelago. The remaining
        // satellites use the same reproducible placement rules further out.
        for (id, offset, scale, yaw) in [
            (102, Vec3::new(-75.0, -35.0, 420.0), 2.1, 0.4),
            (103, Vec3::new(390.0, 45.0, -80.0), 1.8, -0.5),
            (104, Vec3::new(-677.0, -37.0, -366.0), 1.5, 0.7),
            (105, Vec3::new(149.0, 129.0, -666.0), 1.5, -0.1),
            (107, Vec3::new(-512.0, 55.0, -1461.0), 2.1, 0.2),
            (123, Vec3::new(443.0, 10.0, 300.0), 1.4, 0.6),
        ] {
            result.push(WorldIsland {
                id,
                biome: Biome::Dawn,
                center: REGION_CENTERS[0] + offset,
                scale,
                yaw,
                capital: false,
            });
        }
        result.push(WorldIsland {
            id: 305,
            biome: Biome::Nomad,
            center: Vec3::new(-1170.0, 1.0, -883.0),
            scale: 1.25,
            yaw: 0.4,
            capital: false,
        });
        for (region, biome) in Biome::ALL.into_iter().enumerate() {
            for i in 1..=22 {
                if result
                    .iter()
                    .any(|p| p.id == 100 + region as u32 * 100 + i as u32)
                {
                    continue;
                }
                let angle = i as f32 * 2.399963 + region as f32 * 0.61;
                let radius = 390.0 + (i as f32).sqrt() * 170.0;
                let scale = 0.32 + ((i * 17 + region * 7) % 13) as f32 * 0.055;
                let mut center = REGION_CENTERS[region]
                    + Vec3::new(
                        angle.cos() * radius,
                        ((i * 47 + region * 29) % 540) as f32 - 270.0,
                        angle.sin() * radius,
                    );
                // Two ridge settlements sit above, rather than through, the
                // overlapping cavern shells; their IDs remain unchanged.
                if region == 3 && i == 10 {
                    center.y += 220.0;
                }
                if region == 7 && i == 11 {
                    center.y += 170.0;
                }
                // Keep the original tutorial area open, and never place two
                // physical island envelopes through one another.
                if center.distance(Vec3::ZERO) < 330.0
                    || result.iter().any(|p| {
                        let delta = center - p.center;
                        delta.y.abs() < 180.0 * p.scale.max(scale)
                            && delta.x.hypot(delta.z) < (p.scale + scale) * 110.0
                    })
                {
                    continue;
                }
                result.push(WorldIsland {
                    id: 100 + region as u32 * 100 + i as u32,
                    biome,
                    center,
                    scale,
                    yaw: angle,
                    capital: false,
                });
            }
        }
        result.sort_by_key(|i| i.id);
        result
    });
    &WORLD
}
pub fn island(id: u32) -> Option<&'static WorldIsland> {
    islands().iter().find(|i| i.id == id)
}
/// Stable mooring rings at every harbour; IDs do not depend on generation order.
pub fn anchors() -> &'static [(u32, Vec3)] {
    static ANCHORS: LazyLock<Vec<(u32, Vec3)>> = LazyLock::new(|| {
        let mut result: Vec<_> = crate::terrain::ANCHORS
            .iter()
            .enumerate()
            .map(|(id, p)| (id as u32, *p))
            .collect();
        for island in islands() {
            for side in 0..if island.capital { 2 } else { 1 } {
                let offset = Vec3::new(if side == 0 { -38.0 } else { 38.0 }, 22.0, 42.0);
                result.push((
                    10000 + island.id * 2 + side,
                    island.dock() + Quat::from_rotation_y(island.yaw) * offset,
                ));
            }
        }
        result
    });
    &ANCHORS
}
pub fn anchor(id: u32) -> Option<Vec3> {
    anchors().iter().find(|a| a.0 == id).map(|a| a.1)
}
pub fn dock(id: u32) -> Option<Vec3> {
    ISLANDS
        .get(id as usize)
        .map(|i| i.dock)
        .or_else(|| island(id).map(WorldIsland::dock))
}
pub fn dock_rotation(id: u32) -> Quat {
    island(id).map_or(Quat::IDENTITY, |i| Quat::from_rotation_y(i.yaw))
}
pub fn port_name(id: u32) -> String {
    ISLANDS.get(id as usize).map_or_else(
        || island(id).map_or_else(|| "Port inconnu".into(), WorldIsland::label),
        |i| i.name.into(),
    )
}
pub fn nearest_port(position: Vec3) -> (u32, Vec3, f32) {
    ISLANDS
        .iter()
        .enumerate()
        .map(|(i, p)| (i as u32, p.dock))
        .chain(islands().iter().map(|i| (i.id, i.dock())))
        .map(|(id, p)| (id, p, position.distance(p)))
        .min_by(|a, b| a.2.total_cmp(&b.2))
        .expect("world contains ports")
}
pub fn biome_at(position: Vec3) -> Biome {
    let i = REGION_CENTERS
        .iter()
        .enumerate()
        .min_by(|a, b| {
            position
                .distance_squared(*a.1)
                .total_cmp(&position.distance_squared(*b.1))
        })
        .expect("regions")
        .0;
    Biome::ALL[i]
}
pub fn routes() -> &'static [Current] {
    static ROUTES: LazyLock<Vec<Current>> = LazyLock::new(|| {
        let mut routes = vec![Current {
            points: vec![
                Vec3::new(65.0, 20.0, -190.0),
                Vec3::new(65.0, 55.0, -300.0),
                Vec3::new(-60.0, 75.0, -450.0),
                Vec3::new(-260.0, 60.0, -445.0),
            ],
            radius: 45.0,
            speed: 25.0,
        }];
        for (a, b) in [
            (0, 1),
            (0, 2),
            (0, 7),
            (1, 4),
            (1, 5),
            (2, 3),
            (3, 4),
            (4, 6),
            (5, 6),
            (7, 8),
            (8, 5),
        ] {
            // The eastern departure goes around the garden's lower cliffs.
            // A direct high chord obscured the capital's dome and crossed the
            // whole scene like a rigid bridge instead of an aerial river.
            let start = if (a, b) == (0, 1) {
                REGION_CENTERS[a] + Vec3::new(240.0, -65.0, 430.0)
            } else {
                REGION_CENTERS[a] + Vec3::new(0.0, 70.0, 320.0)
            };
            let end = REGION_CENTERS[b] + Vec3::new(0.0, 70.0, 320.0);
            let side = (end - start).cross(Vec3::Y).normalize_or_zero() * 140.0;
            let mut controls = vec![start];
            if (a, b) == (0, 1) {
                controls.extend([
                    Vec3::new(60.0, -20.0, -360.0),
                    Vec3::new(430.0, 45.0, -950.0),
                ]);
            }
            if a >= 7 {
                let v = vaults()[a - 7];
                controls.extend([
                    v.transform_point(Vec3::new(700.0, 200.0, 0.0)),
                    v.transform_point(Vec3::new(700.0, 550.0, 0.0)),
                ]);
            }
            if b >= 7 {
                let v = vaults()[b - 7];
                controls.extend([
                    v.transform_point(Vec3::new(700.0, 550.0, 0.0)),
                    v.transform_point(Vec3::new(700.0, 200.0, 0.0)),
                ]);
            }
            controls.push(end);
            let points = if controls.len() > 2 {
                let mut points = vec![];
                for span in controls.windows(2) {
                    let steps = (span[0].distance(span[1]) / 70.0).ceil().max(1.0) as usize;
                    for n in 0..steps {
                        points.push(span[0].lerp(span[1], n as f32 / steps as f32));
                    }
                }
                points.push(end);
                points
            } else {
                (0..=24)
                    .map(|i| {
                        let t = i as f32 / 24.0;
                        start.lerp(end, t)
                            + side * (t * std::f32::consts::PI).sin()
                            + Vec3::Y * (t * std::f32::consts::TAU).sin() * 45.0
                    })
                    .collect()
            };
            routes.push(Current {
                points,
                radius: 75.0,
                speed: 38.0,
            });
        }
        // A navigable ascending spiral in the storm region, with open ends.
        let c = REGION_CENTERS[6];
        routes.push(Current {
            points: (0..=96)
                .map(|i| {
                    let t = i as f32 / 96.0;
                    let a = t * std::f32::consts::TAU * 1.5;
                    c + Vec3::new(a.cos() * 450.0, -180.0 + t * 500.0, a.sin() * 450.0)
                })
                .collect(),
            radius: 65.0,
            speed: 42.0,
        });
        // Broad circular flow around the first capital; readable from the
        // departure harbour, and wide enough to leave under motor power.
        routes.push(Current {
            points: (0..=80)
                .map(|i| {
                    let t = i as f32 / 80.0;
                    let a = t * std::f32::consts::TAU * 1.15;
                    REGION_CENTERS[0]
                        + Vec3::new(180.0, -65.0, -40.0)
                        + Vec3::new(a.cos() * 600.0, (a * 0.5).sin() * 75.0, a.sin() * 350.0)
                })
                .collect(),
            radius: 65.0,
            speed: 30.0,
        });
        // A banked thermal curls in front of the garden, clear of its cliff.
        // This is the physical route, including its incline and open ends.
        // Its compact spiral stays readable instead of flattening to a band.
        routes.push(Current {
            points: (0..=132)
                .map(|i| {
                    // Preserve the established outer spiral, then continue
                    // into its eye instead of ending in a nine-metre hole.
                    // This remains a real current, escapable with propulsion.
                    let t = (i as f32 / 96.0).min(1.0);
                    let inner = ((i as f32 - 96.0) / 36.0).max(0.0);
                    let a = (t * 2.4 + inner * 0.9) * std::f32::consts::TAU;
                    let r = 48.0 - t * 39.0 - inner * 8.2;
                    let across = Vec3::new(0.824, 0.0, -0.566).normalize();
                    let bank = Vec3::new(-0.490, 0.5, -0.714).normalize();
                    Vec3::new(172.0, 102.0, -482.0)
                        + across * (a.cos() * r)
                        + bank * (a.sin() * r)
                        + Vec3::Y * (t * 18.0 + inner * 2.0)
                })
                .collect(),
            radius: 14.0,
            speed: 22.0,
        });
        for route in &mut routes {
            clear_route(&mut route.points);
            route.points = smooth_route(&route.points);
        }
        routes
    });
    &ROUTES
}
fn smooth_route(points: &[Vec3]) -> Vec<Vec3> {
    let mut result = vec![points[0]];
    for index in 0..points.len() - 1 {
        let p0 = points[index.saturating_sub(1)];
        let p1 = points[index];
        let p2 = points[index + 1];
        let p3 = points[(index + 2).min(points.len() - 1)];
        let steps = (p1.distance(p2) / 25.0).ceil().max(2.0) as usize;
        let curve: Vec<_> = (0..=steps)
            .map(|n| {
                let t = n as f32 / steps as f32;
                (2.0 * p1
                    + (-p0 + p2) * t
                    + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t * t
                    + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t * t * t)
                    * 0.5
            })
            .collect();
        if curve
            .windows(2)
            .all(|p| crate::world_geometry::corridor_clear(p[0], p[1], 12.0))
        {
            result.extend_from_slice(&curve[1..]);
        } else {
            // Keep the proven straight passage when a curved corner would
            // enter rock, notably beside a narrow skylight or port tower.
            result.extend((1..=steps).map(|n| p1.lerp(p2, n as f32 / steps as f32)));
        }
    }
    result
}
pub(crate) fn clear_route(points: &mut [Vec3]) {
    clear_route_envelope(points, 12.0);
}
pub(crate) fn clear_route_envelope(points: &mut [Vec3], radius: f32) {
    // Raise only obstructed spans, then spread the climb over their neighbours.
    // Geometric clearance follows the largest physical vessel on this route.
    for _ in 0..64 {
        let blocked: Vec<_> = points
            .windows(2)
            .enumerate()
            .filter_map(|(n, p)| {
                (!crate::world_geometry::corridor_clear(p[0], p[1], radius)).then_some(n)
            })
            .collect();
        if blocked.is_empty() {
            return;
        }
        for n in blocked {
            points[n].y += 15.0;
            points[n + 1].y += 15.0;
        }
        for n in 1..points.len() {
            let d = (points[n] - points[n - 1]).with_y(0.0).length();
            if d > 1.0 {
                points[n].y = points[n].y.max(points[n - 1].y - d * 0.55);
            }
        }
        for n in (0..points.len() - 1).rev() {
            let d = (points[n + 1] - points[n]).with_y(0.0).length();
            if d > 1.0 {
                points[n].y = points[n].y.max(points[n + 1].y - d * 0.55);
            }
        }
    }
}
pub fn sample_current(position: Vec3) -> (Vec3, f32) {
    // Routes are immutable. Compute their conservative influence boxes once,
    // instead of visiting every segment of every distant archipelago at 60 Hz.
    static BOUNDS: std::sync::LazyLock<Vec<(Vec3, Vec3)>> = std::sync::LazyLock::new(|| {
        routes()
            .iter()
            .map(|route| {
                let (min, max) = route.points.iter().fold(
                    (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)),
                    |(min, max), point| (min.min(*point), max.max(*point)),
                );
                let margin = Vec3::splat(route.radius.max(0.01) + 0.01);
                (min - margin, max + margin)
            })
            .collect()
    });
    let mut result = (Vec3::ZERO, 0.0);
    for (route, (min, max)) in routes().iter().zip(BOUNDS.iter()) {
        if position.cmplt(*min).any() || position.cmpgt(*max).any() {
            continue;
        }
        let sample = route.sample(position);
        if sample.1 > result.1 {
            result = sample;
        }
    }
    // Quiet approach water around ports. Fast travel must not prevent docking.
    if result.1 > 0.0 {
        let (_, _, distance) = nearest_port(position);
        result.1 *= ((distance - 45.0) / 90.0).clamp(0.0, 1.0);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_regions_have_ports_at_several_altitudes_with_stable_unique_ids() {
        let mut ids = std::collections::BTreeSet::new();
        assert!(islands().len() > 140, "{} islands", islands().len());
        for i in islands() {
            assert!(ids.insert(i.id));
            assert_eq!(dock(i.id), Some(i.dock()));
            assert!(i.center.abs().max_element() < 8000.0);
            assert!(i.dock().is_finite());
        }
        for b in Biome::ALL {
            assert!(islands().iter().any(|i| i.capital && i.biome == b));
        }
        assert!(
            islands()
                .iter()
                .map(|i| i.center.y)
                .fold(f32::NEG_INFINITY, f32::max)
                > 1400.0
        );
        assert!(islands().iter().any(|i| i.center.y < -1300.0));
        assert_eq!(dock(0), Some(ISLANDS[0].dock));
        assert!(dock(9999).is_none());
    }
    #[test]
    fn routes_boost_travel_but_leave_port_approaches_calm() {
        for i in islands().iter().filter(|i| i.capital) {
            assert_eq!(sample_current(i.dock()).1, 0.0);
        }
        let r = &routes()[2];
        let p = r.points[12];
        let (v, w) = sample_current(p);
        assert!(w > 0.9 && v.length() > 30.0);
        assert!(routes().iter().any(|r| {
            r.points
                .iter()
                .map(|p| p.y)
                .fold(f32::NEG_INFINITY, f32::max)
                - r.points.iter().map(|p| p.y).fold(f32::INFINITY, f32::min)
                > 500.0
        }));
    }
    #[test]
    fn rejected_current_segments_preserve_unbounded_reference_at_edges_and_ports() {
        // The original full scan is the reference. Exercise active segments,
        // ends, overlapping routes, radius boundaries, quiet ports and far space.
        fn reference(route: &Current, position: Vec3) -> (Vec3, f32) {
            let mut velocity = Vec3::ZERO;
            let mut sum = 0.0_f32;
            let mut influence = 0.0_f32;
            for segment in route.points.windows(2) {
                let d = segment[1] - segment[0];
                let length = d.length_squared();
                if length < 1e-6 {
                    continue;
                }
                let t = ((position - segment[0]).dot(d) / length).clamp(0.0, 1.0);
                let distance = position.distance(segment[0] + d * t);
                let x = (1.0 - distance / route.radius.max(0.01)).clamp(0.0, 1.0);
                let weight = x * x * (3.0 - 2.0 * x);
                velocity += d.normalize() * route.speed * weight;
                sum += weight;
                influence = influence.max(weight);
            }
            if sum > 0.0 {
                (velocity / sum, influence)
            } else {
                (Vec3::ZERO, 0.0)
            }
        }
        let exact = |actual: (Vec3, f32), expected: (Vec3, f32)| {
            assert_eq!(
                actual.0.to_array().map(f32::to_bits),
                expected.0.to_array().map(f32::to_bits)
            );
            assert_eq!(actual.1.to_bits(), expected.1.to_bits());
        };
        let mut probes: Vec<_> = islands().iter().map(|island| island.dock()).collect();
        let mut seed = 55_u64;
        for _ in 0..1024 {
            let mut component = || {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                ((seed >> 32) as u32 as f32 / u32::MAX as f32 - 0.5) * 16000.0
            };
            probes.push(Vec3::new(component(), component() * 0.5, component()));
        }
        for route in routes() {
            for point in route.points.iter().step_by(7) {
                for axis in [Vec3::X, Vec3::Y, Vec3::Z] {
                    for offset in [-1.000_01, -1.0, -0.999_99, 0.0, 0.999_99, 1.0, 1.000_01] {
                        let position = *point + axis * route.radius * offset;
                        exact(route.sample(position), reference(route, position));
                        probes.push(position);
                    }
                }
            }
        }
        for position in probes {
            let mut expected = (Vec3::ZERO, 0.0);
            for route in routes() {
                let sample = reference(route, position);
                if sample.1 > expected.1 {
                    expected = sample;
                }
            }
            if expected.1 > 0.0 {
                let (_, _, distance) = nearest_port(position);
                expected.1 *= ((distance - 45.0) / 90.0).clamp(0.0, 1.0);
            }
            exact(sample_current(position), expected);
        }
        for radius in [0.0, -1.0, 0.01, 13.0] {
            let route = Current {
                points: vec![Vec3::ZERO, Vec3::ZERO, Vec3::X * 20.0],
                radius,
                speed: 9.0,
            };
            for position in [
                Vec3::ZERO,
                Vec3::X * 10.0,
                Vec3::Y * 0.01,
                Vec3::splat(1000.0),
            ] {
                exact(route.sample(position), reference(&route, position));
            }
        }
    }
    #[test]
    fn current_centerlines_leave_room_for_a_vessel() {
        let bad: Vec<_> = routes()
            .iter()
            .enumerate()
            .flat_map(|(r, c)| {
                c.points.windows(2).enumerate().filter_map(move |(n, p)| {
                    (!crate::world_geometry::corridor_clear(p[0], p[1], 8.0)).then_some((r, n))
                })
            })
            .collect();
        assert!(bad.is_empty(), "obstructed routes: {bad:?}");
    }
    #[test]
    fn all_portal_arrival_volumes_are_free_of_solid_terrain() {
        let blocked: Vec<_> = islands()
            .iter()
            .filter_map(|i| {
                let p = i.dock() + Quat::from_rotation_y(i.yaw) * Vec3::new(0.0, 10.0, 45.0);
                (!crate::world_geometry::corridor_clear(p, p, 8.0)).then_some(i.id)
            })
            .collect();
        assert!(blocked.is_empty(), "blocked portal arrival: {blocked:?}");
    }
    #[test]
    fn underground_current_endpoints_remain_near_their_ports() {
        for id in [800, 900] {
            let port = island(id).unwrap();
            assert!(
                routes()
                    .iter()
                    .any(|r| r.points.iter().any(|p| p.distance(port.dock()) < 180.0)),
                "no approach current for {id}"
            );
        }
    }
}
