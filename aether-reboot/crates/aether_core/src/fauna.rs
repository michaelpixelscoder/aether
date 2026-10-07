//! Deterministic residents of the cavern capitals, reconstructed from the clock.
use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Species {
    Guardian,
    Cavewing,
}
impl Species {
    pub const ALL: [Self; 2] = [Self::Guardian, Self::Cavewing];
    pub fn asset(self) -> &'static str {
        match self {
            Self::Guardian => "guardian",
            Self::Cavewing => "cavewing",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Guardian => "Gardien de pierre",
            Self::Cavewing => "Planeur des cavernes",
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Resident {
    pub id: u32,
    pub island: u32,
    pub species: Species,
    pub phase: f32,
}
pub const RESIDENTS: [Resident; 6] = [
    Resident {
        id: 1,
        island: 800,
        species: Species::Guardian,
        phase: 0.0,
    },
    Resident {
        id: 2,
        island: 900,
        species: Species::Guardian,
        phase: 2.0,
    },
    Resident {
        id: 3,
        island: 800,
        species: Species::Cavewing,
        phase: 0.0,
    },
    Resident {
        id: 4,
        island: 800,
        species: Species::Cavewing,
        phase: 3.1,
    },
    Resident {
        id: 5,
        island: 900,
        species: Species::Cavewing,
        phase: 1.0,
    },
    Resident {
        id: 6,
        island: 900,
        species: Species::Cavewing,
        phase: 4.1,
    },
];
impl Resident {
    pub fn scale(self) -> f32 {
        if self.species == Species::Guardian {
            2.0
        } else {
            1.5
        }
    }
    /// Feet for the guardian; flight origin for the cavewing. No random respawn.
    pub fn pose(self, seconds: f32) -> (Vec3, Quat) {
        let i = crate::world::island(self.island).expect("resident capital");
        let a = seconds
            * if self.species == Species::Guardian {
                0.16
            } else {
                0.24
            }
            + self.phase;
        let (local, tangent) = if self.species == Species::Guardian {
            (
                Vec3::new(-25.0 + a.cos() * 1.0, 0.035, 40.0 + a.sin() * 0.8),
                Vec3::new(-a.sin(), 0.0, a.cos() * 0.8),
            )
        } else {
            (
                Vec3::new(a.cos() * 4.0, 15.0 + (a * 2.0).sin() * 2.0, a.sin() * 35.0),
                Vec3::new(-a.sin() * 4.0, 0.0, a.cos() * 35.0),
            )
        };
        (
            i.transform_point(local),
            Quat::from_rotation_y(i.yaw + (-tangent.x).atan2(-tangent.z)),
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn patrols_have_floor_or_clear_flight_volume_and_no_solid_overlap() {
        for r in RESIDENTS {
            let island = crate::world::island(r.island).unwrap();
            for step in 0..400 {
                let (p, q) = r.pose(step as f32 * 0.5);
                assert!(p.is_finite() && q.is_finite());
                let local = (p - island.center) / island.scale;
                let half = if r.species == Species::Guardian {
                    Vec3::new(0.6, 0.9, 0.6)
                } else {
                    Vec3::new(2.0, 0.35, 0.8)
                } * r.scale()
                    / island.scale;
                let center = local
                    + if r.species == Species::Guardian {
                        Vec3::Y * half.y
                    } else {
                        Vec3::ZERO
                    };
                for b in crate::world_geometry::pieces_for(island.asset_key()) {
                    let overlap = (b.center - center)
                        .abs()
                        .cmplt(b.size * 0.5 + half - Vec3::splat(0.01));
                    assert!(
                        !overlap.all(),
                        "resident {} intersects {:?} at {local:?}",
                        r.id,
                        b.center
                    );
                }
                if r.species == Species::Guardian {
                    assert!(
                        crate::world_geometry::pieces_for(island.asset_key())
                            .iter()
                            .any(|b| (local.x - b.center.x).abs() < b.size.x * 0.5
                                && (local.z - b.center.z).abs() < b.size.z * 0.5
                                && (local.y - (b.center.y + b.size.y * 0.5)).abs() < 0.08)
                    );
                }
            }
        }
    }
}
