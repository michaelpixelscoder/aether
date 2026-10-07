use crate::{Block, CELL_SIZE, Cell, Grid};
use glam::{IVec3, Vec3};
#[derive(Clone, Copy, Debug)]
pub struct Hit {
    pub cell: Cell,
    pub normal: Cell,
    pub distance: f32,
}
/// Slab-clipped Amanatides–Woo traversal in body coordinates, measured in metres.
pub fn raycast(grid: &Grid, origin: Vec3, direction: Vec3, max_distance: f32) -> Option<Hit> {
    if !origin.is_finite()
        || !direction.is_finite()
        || !max_distance.is_finite()
        || max_distance <= 0.0
    {
        return None;
    }
    let dir = direction.try_normalize()?;
    let (min, max) = grid.bounds()?;
    let lo = min.as_vec3() * CELL_SIZE - Vec3::splat(CELL_SIZE * 0.5);
    let hi = max.as_vec3() * CELL_SIZE + Vec3::splat(CELL_SIZE * 0.5);
    let mut enter = 0.0_f32;
    let mut exit = max_distance;
    let mut face = IVec3::ZERO;
    for axis in 0..3 {
        if dir[axis].abs() < 1e-8 {
            if origin[axis] < lo[axis] || origin[axis] > hi[axis] {
                return None;
            }
            continue;
        }
        let a = (lo[axis] - origin[axis]) / dir[axis];
        let b = (hi[axis] - origin[axis]) / dir[axis];
        if a.min(b) > enter {
            enter = a.min(b);
            face = IVec3::ZERO;
            face[axis] = if dir[axis] > 0.0 { -1 } else { 1 };
        }
        exit = exit.min(a.max(b));
        if enter > exit {
            return None;
        }
    }
    let start = origin + dir * (enter + 1e-5);
    let mut cell = (start / CELL_SIZE + Vec3::splat(0.5)).floor().as_ivec3();
    let step = IVec3::new(
        if dir.x >= 0.0 { 1 } else { -1 },
        if dir.y >= 0.0 { 1 } else { -1 },
        if dir.z >= 0.0 { 1 } else { -1 },
    );
    let delta = Vec3::new(
        if dir.x.abs() < 1e-8 {
            f32::INFINITY
        } else {
            CELL_SIZE / dir.x.abs()
        },
        if dir.y.abs() < 1e-8 {
            f32::INFINITY
        } else {
            CELL_SIZE / dir.y.abs()
        },
        if dir.z.abs() < 1e-8 {
            f32::INFINITY
        } else {
            CELL_SIZE / dir.z.abs()
        },
    );
    let mut next = Vec3::splat(f32::INFINITY);
    for axis in 0..3 {
        if dir[axis].abs() > 1e-8 {
            next[axis] = ((cell[axis] as f32 + step[axis] as f32 * 0.5) * CELL_SIZE - origin[axis])
                / dir[axis];
        }
    }
    let mut distance = enter;
    for _ in 0..(crate::MAX_EXTENT * 3 + 6) {
        if distance > exit + 1e-5 {
            return None;
        }
        if grid.get(cell.into()) != Block::Air {
            return Some(Hit {
                cell: cell.into(),
                normal: face.into(),
                distance,
            });
        }
        let axis = if next.x <= next.y && next.x <= next.z {
            0
        } else if next.y <= next.z {
            1
        } else {
            2
        };
        distance = next[axis];
        next[axis] += delta[axis];
        cell[axis] += step[axis];
        face = IVec3::ZERO;
        face[axis] = -step[axis];
    }
    None
}
