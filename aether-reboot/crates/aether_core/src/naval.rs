//! Fittings follow real hull cells. Their collision and presentation share poses.
use crate::{Block, Body, Cell, PartKind};
use glam::{Quat, Vec3};
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Rail,
    Trim,
    Window,
    Lantern,
}
impl Kind {
    pub fn asset(self) -> &'static str {
        match self {
            Self::Rail => "rail",
            Self::Trim => "trim",
            Self::Window => "cabin-detail",
            Self::Lantern => "lantern",
        }
    }
}
pub struct Fitting {
    pub kind: Kind,
    pub position: Vec3,
    pub rotation: Quat,
}
pub fn fittings(body: &Body) -> Vec<Fitting> {
    let Some(helm) = body.parts().iter().find(|p| p.kind == PartKind::Helm) else {
        return vec![];
    };
    let grid = body.grid();
    let reserved: std::collections::BTreeSet<_> =
        body.parts().iter().flat_map(|p| p.footprint()).collect();
    let mut cells: Vec<_> = grid.iter().collect();
    cells.sort_by_key(|(c, _)| *c);
    let mut result = Vec::new();
    let mut lamps = 0;
    for (cell, block) in cells {
        for (dir, yaw) in [
            (Cell(1, 0, 0), std::f32::consts::FRAC_PI_2),
            (Cell(-1, 0, 0), -std::f32::consts::FRAC_PI_2),
            (Cell(0, 0, 1), 0.0),
            (Cell(0, 0, -1), std::f32::consts::PI),
        ] {
            if grid.get(cell.offset(dir)) != Block::Air {
                continue;
            }
            let normal = Vec3::new(dir.0 as f32, 0.0, dir.2 as f32);
            let rotation = Quat::from_rotation_y(yaw);
            if block == Block::Glass {
                result.push(Fitting {
                    kind: Kind::Window,
                    position: cell.center() + normal * 0.254,
                    rotation,
                });
            } else if block == Block::Wood && cell.1 == helm.cell.1 {
                if (cell.0 + cell.2).rem_euclid(3) == 0 {
                    result.push(Fitting {
                        kind: Kind::Trim,
                        position: cell.center() + normal * 0.30,
                        rotation,
                    });
                }
                let above = cell.offset(Cell(0, 1, 0));
                // Leave boarding openings abreast of the helm and clear machinery.
                if grid.get(above) == Block::Air
                    && !reserved.contains(&above)
                    && (cell.2 - helm.cell.2).abs() > 1
                {
                    result.push(Fitting {
                        kind: Kind::Rail,
                        position: cell.center() + normal * 0.22 + Vec3::Y * 0.25,
                        rotation,
                    });
                    if lamps < 6 && cell.2.rem_euclid(8) == 0 {
                        result.push(Fitting {
                            kind: Kind::Lantern,
                            position: cell.center() + normal * 0.22 + Vec3::Y * 0.85,
                            rotation,
                        });
                        lamps += 1;
                    }
                }
            }
        }
        // Decoration is bounded even for the largest creative builds.
        if result.len() >= 192 {
            break;
        }
    }
    let remaining = 192usize.saturating_sub(result.len());
    result.extend(
        crate::naval_deck::fittings(body, remaining)
            .into_iter()
            .map(|(_, f)| f),
    );
    result
}
