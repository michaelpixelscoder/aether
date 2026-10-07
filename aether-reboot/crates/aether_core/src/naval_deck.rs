//! Walkable upper-deck fittings derive from the current hull and helm frame.
use crate::{
    Block, Body, Cell, PartKind,
    naval::{Fitting, Kind},
};
use glam::{Quat, Vec3};
use std::collections::BTreeSet;

fn fore(body: &Body) -> Option<Cell> {
    let helm = body.parts().iter().find(|p| p.kind == PartKind::Helm)?;
    Some(match helm.quarter_turn {
        0 => Cell(0, 0, -1),
        1 => Cell(-1, 0, 0),
        2 => Cell(0, 0, 1),
        _ => Cell(1, 0, 0),
    })
}

pub fn is_upper_deck(body: &Body, cell: Cell) -> bool {
    let Some(helm) = body.parts().iter().find(|p| p.kind == PartKind::Helm) else {
        return false;
    };
    cell.1 == helm.cell.1 + 2
        && body.grid().get(cell) == Block::Wood
        && body.grid().get(cell.offset(Cell(0, 1, 0))) == Block::Air
}

pub fn is_stair(body: &Body, cell: Cell) -> bool {
    let Some(helm) = body.parts().iter().find(|p| p.kind == PartKind::Helm) else {
        return false;
    };
    if cell.1 != helm.cell.1 + 1
        || body.grid().get(cell) != Block::Wood
        || body.grid().get(cell.offset(Cell(0, 1, 0))) != Block::Air
    {
        return false;
    }
    let Some(dir) = fore(body) else { return false };
    for sign in [-1, 1] {
        let low = cell.offset(Cell(dir.0 * sign, 0, dir.2 * sign));
        let high = cell.offset(Cell(-dir.0 * sign, 0, -dir.2 * sign));
        if body.grid().get(low) == Block::Air
            && body.grid().get(low.offset(Cell(0, -1, 0))) == Block::Wood
            && body.grid().get(high) == Block::Wood
            && body.grid().get(high.offset(Cell(0, 1, 0))) == Block::Wood
        {
            return true;
        }
    }
    false
}

/// Every returned item is attached to exactly one exposed, walkable wood cell.
/// Render through ShipKit; physics through the EXISTING Kind::Rail contract.
pub fn fittings(body: &Body, budget: usize) -> Vec<(Cell, Fitting)> {
    let Some(helm) = body.parts().iter().find(|p| p.kind == PartKind::Helm) else {
        return vec![];
    };
    let Some(forward) = fore(body) else {
        return vec![];
    };
    let reserved: BTreeSet<_> = body.parts().iter().flat_map(|p| p.footprint()).collect();
    let grid = body.grid();
    let mut out = vec![];
    for (cell, block) in grid.iter() {
        if block != Block::Wood || !(cell.1 == helm.cell.1 + 2 || is_stair(body, cell)) {
            continue;
        }
        if (1..=3).any(|dy| {
            let c = cell.offset(Cell(0, dy, 0));
            grid.get(c) != Block::Air || reserved.contains(&c)
        }) {
            continue;
        }
        for (dir, yaw) in [
            (Cell(1, 0, 0), std::f32::consts::FRAC_PI_2),
            (Cell(-1, 0, 0), -std::f32::consts::FRAC_PI_2),
            (Cell(0, 0, 1), 0.),
            (Cell(0, 0, -1), std::f32::consts::PI),
        ] {
            let adjacent = cell.offset(dir);
            if grid.get(adjacent) != Block::Air {
                continue;
            }
            let along = dir.0 * forward.0 + dir.2 * forward.2 != 0;
            let lower = adjacent.offset(Cell(0, -1, 0));
            if along
                && grid.get(lower) == Block::Wood
                && (0..=2).all(|dy| {
                    let c = adjacent.offset(Cell(0, dy, 0));
                    grid.get(c) == Block::Air && !reserved.contains(&c)
                })
            {
                continue;
            }
            let normal = Vec3::new(dir.0 as f32, 0., dir.2 as f32);
            if out.len() >= budget {
                return out;
            }
            out.push((
                cell,
                Fitting {
                    kind: Kind::Rail,
                    position: cell.center() + normal * 0.22 + Vec3::Y * 0.25,
                    rotation: Quat::from_rotation_y(yaw),
                },
            ));
        }
    }
    out
}

/// Only a real, narrow support below a Sail. Removing/turning parts or editing
/// cells recomputes this classification through the existing GeometryKey.
pub fn mast_support(body: &Body, cell: Cell) -> bool {
    let Some(helm) = body.parts().iter().find(|p| p.kind == PartKind::Helm) else {
        return false;
    };
    if body.grid().get(cell) != Block::Wood || cell.1 < helm.cell.1 + 3 {
        return false;
    }
    if [Cell(1, 0, 0), Cell(-1, 0, 0), Cell(0, 0, 1), Cell(0, 0, -1)]
        .iter()
        .any(|d| body.grid().get(cell.offset(*d)) != Block::Air)
    {
        return false;
    }
    body.parts()
        .iter()
        .filter(|p| p.kind == PartKind::Sail)
        .any(|p| {
            p.cell.0 == cell.0
                && p.cell.2 == cell.2
                && p.cell.1 >= cell.1
                && (cell.1..=p.cell.1)
                    .all(|y| body.grid().get(Cell(cell.0, y, cell.2)) == Block::Wood)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edited_or_transformed_upper_decks_keep_cell_owned_railings() {
        let original = crate::fixtures::explorer();
        let baseline = fittings(&original, 192);
        assert_eq!(baseline.len(), 16);
        let owners: BTreeSet<_> = baseline.iter().map(|(owner, _)| *owner).collect();
        assert_eq!(owners.len(), 12);
        for owner in owners {
            let mut edited = original.clone();
            crate::edit::History::default()
                .apply(&mut edited, crate::edit::Edit::Set(owner, Block::Air))
                .unwrap();
            assert!(
                fittings(&edited, 192)
                    .iter()
                    .all(|(cell, _)| *cell != owner)
            );
        }
        for turn in 0..4_u8 {
            let rotation = Quat::from_rotation_y(turn as f32 * std::f32::consts::FRAC_PI_2);
            let offset = Cell(-6, -8, 11);
            let transform = |c: Cell| {
                let rotated = rotation * c.ivec().as_vec3();
                Cell(
                    rotated.x.round() as i32,
                    rotated.y.round() as i32,
                    rotated.z.round() as i32,
                )
                .offset(offset)
            };
            let mut blueprint = original.blueprint();
            blueprint.cells = blueprint
                .cells
                .into_iter()
                .map(|(c, b)| (transform(c), b))
                .collect();
            for part in &mut blueprint.parts {
                part.cell = transform(part.cell);
                part.quarter_turn = (part.quarter_turn + turn) % 4;
            }
            let moved = Body::from_blueprint(blueprint).unwrap();
            let next = fittings(&moved, 192);
            assert_eq!(next.len(), baseline.len());
            for (owner, rail) in &baseline {
                let position =
                    rotation * rail.position + offset.ivec().as_vec3() * crate::CELL_SIZE;
                assert!(next.iter().any(|(c, r)| *c == transform(*owner)
                    && r.position.distance(position) < 1e-5
                    && (r.rotation * Vec3::Z).distance(rotation * rail.rotation * Vec3::Z) < 1e-5));
            }
            for (cell, _) in original.grid().iter() {
                assert_eq!(
                    is_upper_deck(&original, cell),
                    is_upper_deck(&moved, transform(cell))
                );
                assert_eq!(is_stair(&original, cell), is_stair(&moved, transform(cell)));
                assert_eq!(
                    mast_support(&original, cell),
                    mast_support(&moved, transform(cell))
                );
            }
        }
        assert!(fittings(&original, 0).is_empty());
        assert_eq!(fittings(&original, 3).len(), 3);
        assert_eq!(crate::naval::fittings(&original).len(), 84);
    }
}
