//! Airship traffic follows continuous, solid-free circuits. Simulation time is
//! the only phase input, so a save does not respawn or reset the convoys.
use glam::{Quat, Vec3};
use std::sync::LazyLock;
#[derive(serde::Deserialize)]
pub struct CollisionBox {
    pub center: Vec3,
    pub size: Vec3,
}
/// Inscribed boxes exported and verified against both authored model LODs.
pub fn collisions() -> &'static [CollisionBox] {
    #[derive(serde::Deserialize)]
    struct Export {
        trader: Vec<CollisionBox>,
    }
    static BOXES: LazyLock<Vec<CollisionBox>> = LazyLock::new(|| {
        serde_json::from_str::<Export>(include_str!("../../../assets/fauna/trader-collisions.json"))
            .expect("verified courier collision export")
            .trader
    });
    &BOXES
}
#[derive(Clone, Copy)]
pub struct Courier {
    pub id: u32,
    pub circuit: usize,
    pub phase: f32,
}
pub const COURIERS: [Courier; 8] = [
    Courier {
        id: 1,
        circuit: 0,
        phase: 0.14,
    },
    Courier {
        id: 2,
        circuit: 0,
        phase: 0.63,
    },
    Courier {
        id: 3,
        circuit: 1,
        phase: 0.28,
    },
    Courier {
        id: 4,
        circuit: 1,
        phase: 0.78,
    },
    Courier {
        id: 5,
        circuit: 2,
        phase: 0.40,
    },
    Courier {
        id: 6,
        circuit: 5,
        phase: 0.376,
    },
    Courier {
        id: 7,
        circuit: 3,
        phase: 0.679,
    },
    Courier {
        id: 8,
        circuit: 4,
        phase: 0.185,
    },
];
struct Circuit {
    points: Vec<Vec3>,
    lengths: Vec<f32>,
    length: f32,
}
fn circuits() -> &'static [Circuit; 6] {
    static ROUTES: LazyLock<[Circuit; 6]> = LazyLock::new(|| {
        let regional = |region| crate::world::REGION_CENTERS[region] + Vec3::Y * 275.0;
        // Distinct harbour circuits keep near, middle and far traffic visible
        // while retaining continuous world-space motion in normal gameplay.
        [
            (regional(0), 420.0, 350.0, 45.0, 12.0),
            (regional(2), 420.0, 350.0, 45.0, 12.0),
            (regional(1), 420.0, 350.0, 45.0, 12.0),
            (
                crate::world::REGION_CENTERS[0] + Vec3::Y * 60.0,
                550.0,
                420.0,
                45.0,
                25.0,
            ),
            (Vec3::new(-260.0, 125.0, -550.0), 330.0, 150.0, 20.0, 35.0),
            (Vec3::new(-60.0, 60.0, -270.0), 130.0, 90.0, 15.0, 25.0),
        ]
        .map(|(center, radius_x, radius_z, rise, clearance)| {
            let mut points: Vec<_> = (0..=96)
                .map(|n| {
                    let a = n as f32 * std::f32::consts::TAU / 96.0;
                    center
                        + Vec3::new(
                            a.cos() * radius_x,
                            (a * 2.0).sin() * rise,
                            a.sin() * radius_z,
                        )
                })
                .collect();
            for _ in 0..8 {
                crate::world::clear_route_envelope(&mut points, clearance);
                let y = points[0].y.max(points[96].y);
                points[0].y = y;
                points[96] = points[0];
            }
            let lengths: Vec<_> = points.windows(2).map(|p| p[0].distance(p[1])).collect();
            let length = lengths.iter().sum();
            Circuit {
                points,
                lengths,
                length,
            }
        })
    });
    &ROUTES
}
impl Courier {
    pub fn scale(self) -> f32 {
        match self.id {
            6 | 7 => 2.0,
            8.. => 2.8,
            _ => 1.0,
        }
    }
    /// Sailing convoys share a domain blueprint with their rendered and solid
    /// hulls. They are time-driven traffic, not editable player-owned vessels.
    pub fn sailing_body(self) -> Option<&'static crate::Body> {
        static BODY: LazyLock<crate::Body> = LazyLock::new(crate::fixtures::explorer);
        (self.id >= 6).then(|| &*BODY)
    }
    pub fn pose(self, seconds: f32) -> (Vec3, Quat) {
        let circuit = &circuits()[self.circuit];
        let points = &circuit.points;
        let lengths = &circuit.lengths;
        let mut distance =
            (seconds * 14.0 + self.phase * circuit.length).rem_euclid(circuit.length);
        let mut index = 0;
        while index + 1 < lengths.len() && distance > lengths[index] {
            distance -= lengths[index];
            index += 1;
        }
        let t = distance / lengths[index];
        let count = points.len() - 1;
        let p0 = points[(index + count - 1) % count];
        let p1 = points[index];
        let p2 = points[index + 1];
        let p3 = points[(index + 2) % count];
        let a = 2.0 * p1;
        let b = -p0 + p2;
        let c = 2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3;
        let d = -p0 + 3.0 * p1 - 3.0 * p2 + p3;
        let p = (a + b * t + c * t * t + d * t * t * t) * 0.5;
        let tangent = (b + c * 2.0 * t + d * 3.0 * t * t).normalize_or_zero();
        let yaw = (-tangent.x).atan2(-tangent.z);
        let pitch = tangent.y.asin().clamp(-0.3, 0.3);
        (p, Quat::from_rotation_y(yaw) * Quat::from_rotation_x(pitch))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn moving_airships_never_teleport_or_cross_a_landmark() {
        for courier in COURIERS {
            let mut previous = courier.pose(0.0).0;
            for n in 1..=1400 {
                let (p, q) = courier.pose(n as f32 * 0.25);
                assert!(p.is_finite() && q.is_finite());
                assert!(p.distance(previous) < 5.0, "courier {} jumps", courier.id);
                assert!(
                    crate::world_geometry::corridor_clear(previous, p, 11.0 * courier.scale()),
                    "courier {} intersects scenery at {p:?}",
                    courier.id
                );
                previous = p;
            }
        }
    }
}
