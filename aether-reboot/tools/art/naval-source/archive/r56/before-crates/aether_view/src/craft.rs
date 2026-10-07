//! Small fittings derive from occupied cells, so edits cannot leave a ghost hull.
use aether_core::{Block, Body, Cell, PartKind, mesh::Surface};
use bevy::prelude::*;
#[derive(Component)]
pub struct CraftVisual(pub Entity);
#[derive(PartialEq, Eq)]
pub(crate) struct GeometryKey {
    cells: Vec<(Cell, Block)>,
    parts: Vec<(PartKind, Cell, u8)>,
}
impl GeometryKey {
    pub(crate) fn of(body: &Body) -> Self {
        Self {
            cells: body.grid().iter().collect(),
            // IDs, cargo, names and revision do not affect decoration geometry.
            // Preserve part order: rigging intentionally visits at most 8 sails.
            parts: body
                .parts()
                .iter()
                .map(|p| (p.kind, p.cell, p.quarter_turn))
                .collect(),
        }
    }
}
struct SharedGeometry {
    key: GeometryKey,
    batches: Vec<(usize, Handle<Mesh>)>,
}
/// Identical hulls share GPU geometry; edited hulls get independent batches.
/// Entries live only while at least one corresponding decoration is in the ECS.
#[derive(Resource, Default)]
pub struct CraftMeshes(Vec<SharedGeometry>);
impl CraftMeshes {
    pub(crate) fn retain_live(&mut self, live: &std::collections::HashSet<AssetId<Mesh>>) {
        self.0
            .retain(|entry| entry.batches.iter().any(|(_, h)| live.contains(&h.id())));
    }
    fn batches(&mut self, body: &Body, meshes: &mut Assets<Mesh>) -> &[(usize, Handle<Mesh>)] {
        let key = GeometryKey::of(body);
        let index = if let Some(index) = self.0.iter().position(|entry| entry.key == key) {
            index
        } else {
            let batches = surfaces(body)
                .into_iter()
                .enumerate()
                .filter(|(_, surface)| !surface.indices.is_empty())
                .map(|(slot, surface)| (slot, meshes.add(crate::voxel::to_flat_mesh(surface))))
                .collect();
            self.0.push(SharedGeometry { key, batches });
            self.0.len() - 1
        };
        &self.0[index].batches
    }
}
fn quad(mesh: &mut Surface, points: [Vec3; 4], normal: Vec3) {
    let base = mesh.positions.len() as u32;
    mesh.positions.extend(points.map(|p| p.to_array()));
    mesh.normals.extend([normal.to_array(); 4]);
    mesh.uvs.extend([[0., 0.], [1., 0.], [1., 1.], [0., 1.]]);
    mesh.indices
        .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
}
fn cuboid(mesh: &mut Surface, center: Vec3, size: Vec3) {
    for axis in 0..3 {
        for sign in [-1., 1.] {
            let u = (axis + 1) % 3;
            let v = (axis + 2) % 3;
            let mut normal = Vec3::ZERO;
            normal[axis] = sign;
            let mut points = [Vec3::ZERO; 4];
            for (i, (a, b)) in [(-1., -1.), (1., -1.), (1., 1.), (-1., 1.)]
                .into_iter()
                .enumerate()
            {
                let mut p = center;
                p[axis] += sign * size[axis] * 0.5;
                p[u] += a * size[u] * 0.5;
                p[v] += b * size[v] * 0.5;
                points[i] = p;
            }
            if sign < 0. {
                points.reverse();
            }
            quad(mesh, points, normal);
        }
    }
}
fn rope(mesh: &mut Surface, a: Vec3, b: Vec3) {
    rod(mesh, a, b, 0.018);
}
fn rod(mesh: &mut Surface, a: Vec3, b: Vec3, radius: f32) {
    let d = (b - a).normalize_or_zero();
    if d == Vec3::ZERO {
        return;
    }
    let reference = if d.y.abs() > 0.95 { Vec3::X } else { Vec3::Y };
    let u = d.cross(reference).normalize();
    let v = d.cross(u);
    for i in 0..6 {
        let radial = |j: f32| {
            u * (j * std::f32::consts::TAU / 6.).cos() + v * (j * std::f32::consts::TAU / 6.).sin()
        };
        let r = radial(i as f32) * radius;
        let s = radial(i as f32 + 1.) * radius;
        quad(
            mesh,
            [a + r, a + s, b + s, b + r],
            (r + s).normalize_or_zero(),
        );
    }
}
fn curved_rod(mesh: &mut Surface, points: &[Vec3], radius: f32) {
    for pair in points.windows(2) {
        rod(mesh, pair[0], pair[1], radius);
    }
}
fn rigging_clear(grid: &aether_core::Grid, a: Vec3, b: Vec3) -> bool {
    let steps = (a.distance(b) / 0.18).ceil() as usize;
    (1..steps).all(|step| {
        let p = a.lerp(b, step as f32 / steps as f32) / aether_core::CELL_SIZE;
        grid.get(Cell(
            p.x.round() as i32,
            p.y.round() as i32,
            p.z.round() as i32,
        )) == Block::Air
    })
}
/// A closed, carved cross-section swept only across its owning half-metre cell.
fn moulding(mesh: &mut Surface, center: Vec3, normal: Vec3, profile: &[(f32, f32)]) {
    let tangent = Vec3::Y.cross(normal);
    for index in 0..profile.len() {
        let (height_a, relief_a) = profile[index];
        let (height_b, relief_b) = profile[(index + 1) % profile.len()];
        let a = center + Vec3::Y * height_a + normal * relief_a;
        let b = center + Vec3::Y * height_b + normal * relief_b;
        let face_normal = tangent.cross(b - a).normalize();
        quad(
            mesh,
            [
                a - tangent * 0.249,
                a + tangent * 0.249,
                b + tangent * 0.249,
                b - tangent * 0.249,
            ],
            face_normal,
        );
    }
}
fn staysail(mesh: &mut Surface, a: Vec3, b: Vec3, c: Vec3) {
    let normal = (b - a).cross(c - a).normalize();
    let point =
        |u: f32, v: f32| a + (b - a) * u + (c - a) * v + normal * (u * v * (1.0 - u - v) * 5.4);
    let mut triangle = |uvs: [Vec2; 3]| {
        let [a, b, c] = uvs.map(|uv| point(uv.x, uv.y));
        let start = mesh.uvs.len();
        quad(mesh, [a, b, c, c], (b - a).cross(c - a).normalize());
        for (index, uv) in [uvs[0], uvs[1], uvs[2], uvs[2]].into_iter().enumerate() {
            mesh.uvs[start + index] = (uv * 2.0).to_array();
        }
    };
    for row in 0..10 {
        for column in 0..10 - row {
            let uv = Vec2::new(row as f32, column as f32) * 0.1;
            triangle([uv, uv + Vec2::X * 0.1, uv + Vec2::Y * 0.1]);
            if row + column < 9 {
                triangle([
                    uv + Vec2::X * 0.1,
                    uv + Vec2::splat(0.1),
                    uv + Vec2::Y * 0.1,
                ]);
            }
        }
    }
}
fn chamfered_plank(mesh: &mut Surface, center: Vec3, size: Vec3) {
    let h = size * 0.5;
    let inner = h - Vec3::splat(0.005);
    let mut face = |mut points: [Vec3; 4], normal: Vec3| {
        if (points[1] - points[0])
            .cross(points[2] - points[0])
            .dot(normal)
            < 0.0
        {
            points.reverse();
        }
        quad(mesh, points.map(|p| p + center), normal);
    };
    for axis in 0..3 {
        let u = (axis + 1) % 3;
        let v = (axis + 2) % 3;
        for sign in [-1.0, 1.0] {
            let mut n = Vec3::ZERO;
            n[axis] = sign;
            let points = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)].map(|(a, b)| {
                let mut p = Vec3::ZERO;
                p[axis] = sign * h[axis];
                p[u] = a * inner[u];
                p[v] = b * inner[v];
                p
            });
            face(points, n);
        }
        for a in [-1.0, 1.0] {
            for b in [-1.0, 1.0] {
                let mut n = Vec3::ZERO;
                n[u] = a;
                n[v] = b;
                let points = [(-1.0, false), (1.0, false), (1.0, true), (-1.0, true)].map(
                    |(along, other)| {
                        let mut p = Vec3::ZERO;
                        p[axis] = along * inner[axis];
                        p[u] = a * if other { inner[u] } else { h[u] };
                        p[v] = b * if other { h[v] } else { inner[v] };
                        p
                    },
                );
                face(points, n.normalize());
            }
        }
    }
    for x in [-1.0, 1.0] {
        for y in [-1.0, 1.0] {
            for z in [-1.0, 1.0] {
                let sign = Vec3::new(x, y, z);
                let mut points = [sign * inner; 4];
                for axis in 0..3 {
                    points[axis][axis] = sign[axis] * h[axis];
                }
                points[3] = points[2];
                face(points, sign.normalize());
            }
        }
    }
}
pub fn spawn(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    shared: &mut CraftMeshes,
    palette: &crate::Palette,
    canvas_material: &Handle<StandardMaterial>,
    owner: Entity,
    body: &Body,
) {
    if body.parts().is_empty() {
        return;
    }
    let materials = [
        &palette.gold,
        &palette.rope,
        &palette.lantern,
        &palette.wood,
        &palette.wood,
        &palette.metal,
        canvas_material,
    ];
    for (slot, mesh) in shared.batches(body, meshes) {
        commands.spawn((
            CraftVisual(owner),
            crate::immutable_bounds(meshes.get(mesh).expect("cached craft mesh")),
            Mesh3d(mesh.clone()),
            MeshMaterial3d(materials[*slot].clone()),
            Transform::default(),
            ChildOf(owner),
        ));
    }
}
fn surfaces(body: &Body) -> [Surface; 7] {
    let grid = body.grid();
    let mut brass = Surface::default();
    let mut cordage = Surface::default();
    let mut glow = Surface::default();
    let mut timber = Surface::default();
    let mut planking = Surface::default();
    let mut iron = Surface::default();
    let mut canvas = Surface::default();
    let mut decorated = 0;
    for (cell, block) in grid.iter() {
        // The authored cabin-detail fitting already supplies the complete
        // mullion, sill and cornice. A second procedural frame hid its relief.
        if block != Block::Wood {
            continue;
        }
        let c = cell.center();
        let top = grid.get(cell.offset(Cell(0, 1, 0))) == Block::Air;
        if top && cell.1 <= 1 {
            // Narrow deck boards follow the editable floor, including the
            // raised forecastle and the tapered bow. They never bridge air.
            for strip in 0..3 {
                let p = c + Vec3::new(-1.0 / 6.0 + strip as f32 / 6.0, 0.262, 0.0);
                chamfered_plank(&mut planking, p, Vec3::new(0.162, 0.024, 0.496));
            }
        }
        if top && cell.1 >= 3 {
            // Exposed cabin caps receive separate slats and a modest eave.
            // This follows every edited roof cell and is removed with it.
            for strip in 0..4 {
                let p = c + Vec3::new(0.0, 0.279, -0.1875 + strip as f32 * 0.125);
                chamfered_plank(&mut planking, p, Vec3::new(0.51, 0.055, 0.121));
            }
            for (axis, dir) in [
                (0, Cell(1, 0, 0)),
                (0, Cell(-1, 0, 0)),
                (2, Cell(0, 0, 1)),
                (2, Cell(0, 0, -1)),
            ] {
                if grid.get(cell.offset(dir)) == Block::Air {
                    let sign = if axis == 0 { dir.0 } else { dir.2 } as f32;
                    let mut p = c + Vec3::Y * 0.245;
                    p[axis] += sign * 0.26;
                    let mut size = Vec3::new(0.50, 0.075, 0.50);
                    size[axis] = 0.14;
                    chamfered_plank(&mut planking, p, size);
                }
            }
        }
        for (axis, dir) in [
            (0, Cell(1, 0, 0)),
            (0, Cell(-1, 0, 0)),
            (2, Cell(0, 0, 1)),
            (2, Cell(0, 0, -1)),
        ] {
            if grid.get(cell.offset(dir)) != Block::Air {
                continue;
            }
            let normal = Vec3::new(dir.0 as f32, 0.0, dir.2 as f32);
            let tangent = Vec3::Y.cross(normal);
            let sign = if axis == 0 {
                dir.0 as f32
            } else {
                dir.2 as f32
            };
            let mut p = c;
            p[axis] += sign * 0.257;
            for course in 0..4 {
                let mut plank = p;
                plank.y = c.y - 0.1875 + course as f32 * 0.125;
                let mut size = Vec3::new(0.496, 0.122, 0.496);
                size[axis] = 0.034;
                chamfered_plank(&mut planking, plank, size);
            }
            if axis == 0 && cell.2.rem_euclid(3) == 0 && cell.1 <= 1 {
                // Broad bent frames catch the sun at a changing angle. Each
                // frame terminates inside this cell, even after hull edits.
                let rib = [-0.235_f32, -0.15, 0.0, 0.15, 0.235].map(|height| {
                    c + Vec3::Y * height
                        + normal * (0.295 + 0.065 * (1.0 - (height / 0.235).powi(2)))
                });
                curved_rod(&mut iron, &rib, 0.052);
                curved_rod(&mut brass, &rib.map(|p| p + normal * 0.046), 0.013);
                for height in [-0.18, 0.18] {
                    cuboid(
                        &mut brass,
                        c + Vec3::Y * height + normal * 0.36,
                        Vec3::splat(0.055),
                    );
                }
            }
            if cell.1 <= 0 && cell.1 >= -2 {
                // A shaped wale and its gilded crown articulate the actual
                // tapered courses. No continuous replacement hull is added.
                moulding(
                    &mut iron,
                    c,
                    normal,
                    &[
                        (0.10, 0.275),
                        (0.11, 0.32),
                        (0.14, 0.365),
                        (0.195, 0.365),
                        (0.22, 0.315),
                        (0.22, 0.275),
                    ],
                );
                moulding(
                    &mut brass,
                    c,
                    normal,
                    &[
                        (0.185, 0.365),
                        (0.197, 0.384),
                        (0.221, 0.359),
                        (0.226, 0.313),
                    ],
                );
            }
            if cell.1 >= 2 && top {
                // Coved cornices separate the stepped cabin crown from its
                // vertical walls and make its carved edges readable at sea.
                moulding(
                    &mut timber,
                    c,
                    normal,
                    &[
                        (0.04, 0.253),
                        (0.07, 0.28),
                        (0.16, 0.295),
                        (0.21, 0.37),
                        (0.27, 0.385),
                        (0.30, 0.30),
                    ],
                );
                moulding(
                    &mut brass,
                    c,
                    normal,
                    &[(0.19, 0.353), (0.205, 0.39), (0.236, 0.39), (0.244, 0.369)],
                );
                moulding(
                    &mut iron,
                    c,
                    normal,
                    &[
                        (0.075, 0.278),
                        (0.088, 0.295),
                        (0.127, 0.302),
                        (0.143, 0.291),
                    ],
                );
            }
            if cell.1 >= 2 {
                for direction in [-1, 1] {
                    let neighbour = Cell(
                        (tangent.x as i32) * direction,
                        0,
                        (tangent.z as i32) * direction,
                    );
                    if grid.get(cell.offset(neighbour)) != Block::Air {
                        continue;
                    }
                    let post = c + normal * 0.287 + tangent * (direction as f32 * 0.205);
                    rod(
                        &mut iron,
                        post - Vec3::Y * 0.235,
                        post + Vec3::Y * 0.235,
                        0.045,
                    );
                    for height in [-0.20, 0.20] {
                        rod(
                            &mut brass,
                            post + Vec3::Y * (height - 0.025),
                            post + Vec3::Y * (height + 0.025),
                            0.059,
                        );
                    }
                }
            }
            if top && cell.1 == 1 {
                // Raised bulwarks and the stern gallery use the exact exposed
                // edge of the editable deck. The waist's boarding gaps remain.
                moulding(
                    &mut brass,
                    c,
                    normal,
                    &[(0.24, 0.22), (0.265, 0.31), (0.31, 0.32), (0.33, 0.23)],
                );
                let foot = c + normal * 0.27 + Vec3::Y * 0.29;
                let crown = foot + Vec3::Y * 0.38;
                rod(
                    &mut brass,
                    crown - tangent * 0.248,
                    crown + tangent * 0.248,
                    0.025,
                );
                rod(
                    &mut iron,
                    foot + Vec3::Y * 0.13 - tangent * 0.248,
                    foot + Vec3::Y * 0.13 + tangent * 0.248,
                    0.018,
                );
                for along in [-0.205, 0.205] {
                    let post = foot + tangent * along;
                    rod(&mut iron, post, post + Vec3::Y * 0.38, 0.026);
                    for height in [0.025, 0.34] {
                        rod(
                            &mut brass,
                            post + Vec3::Y * height,
                            post + Vec3::Y * (height + 0.035),
                            0.042,
                        );
                    }
                }
                if cell.2 > 0 {
                    let knee = [
                        (-0.20, 0.27),
                        (-0.12, 0.30),
                        (0.05, 0.36),
                        (0.19, 0.40),
                        (0.25, 0.32),
                    ]
                    .map(|(height, relief)| c + Vec3::Y * height + normal * relief);
                    curved_rod(&mut timber, &knee, 0.049);
                    curved_rod(&mut brass, &knee.map(|p| p + normal * 0.046), 0.014);
                }
            }
        }
        if top && cell.1 == 1 {
            // Finish the real raised bulwarks; the open working waist retains
            // the authored rail modules and existing boarding openings.
            if [Cell(1, 0, 0), Cell(-1, 0, 0), Cell(0, 0, 1), Cell(0, 0, -1)]
                .into_iter()
                .any(|dir| grid.get(cell.offset(dir)) == Block::Air)
            {
                chamfered_plank(
                    &mut timber,
                    c + Vec3::Y * 0.283,
                    Vec3::new(0.515, 0.045, 0.515),
                );
            }
            for x in [-0.19, 0.19] {
                for z in [-0.19, 0.19] {
                    cuboid(
                        &mut brass,
                        c + Vec3::new(x, 0.26, z),
                        Vec3::new(0.07, 0.025, 0.07),
                    );
                }
            }
        }
        decorated += 1;
        if decorated >= 1024 {
            break;
        }
    }
    let anchors: Vec<_> = grid
        .iter()
        .filter(|(cell, block)| {
            *block == Block::Wood && grid.get(cell.offset(Cell(0, 1, 0))) == Block::Air
        })
        .take(1024)
        .map(|(cell, _)| cell.center() + Vec3::Y * 0.29)
        .collect();
    let deck: Vec<_> = anchors.iter().copied().filter(|p| p.y <= 0.80).collect();
    for sail in body.parts().iter().filter(|p| p.kind.is_sail()).take(8) {
        let scale = if sail.kind == PartKind::GrandSail {
            2.5
        } else {
            1.0
        };
        let rotation =
            Quat::from_rotation_y(sail.quarter_turn as f32 * std::f32::consts::FRAC_PI_2);
        let origin = sail.cell.center();
        let forward = rotation * Vec3::NEG_Z;
        let right = rotation * Vec3::X;
        let mast = sail.center() + Vec3::Y * (1.65 * scale) + rotation * Vec3::Z * (0.13 * scale);
        // Fore/aft stays terminate at real exposed timber. A removed bow,
        // mast or sail cannot retain any of these derived visual attachments.
        for direction in [-1.0, 1.0] {
            if let Some(tie) = anchors
                .iter()
                .filter(|p| {
                    (**p - origin).dot(right).abs() < 0.55
                        && (**p - origin).dot(forward) * direction > 1.0
                        && rigging_clear(grid, mast, **p)
                })
                .max_by(|a, b| {
                    ((**a - origin).dot(forward) * direction)
                        .total_cmp(&((**b - origin).dot(forward) * direction))
                })
            {
                rope(&mut cordage, mast, *tie);
                if direction > 0.0
                    && sail.kind == PartKind::GrandSail
                    && (*tie - origin).dot(forward) > 3.0
                    && tie.y <= origin.y + 1.0
                    && let Some(clew_tie) = deck
                        .iter()
                        .filter(|p| {
                            (**p - origin).dot(forward) > 1.0
                                && (**p - origin).dot(forward) < 2.7
                                && (**p - origin).dot(right).abs() < 0.55
                        })
                        .min_by(|a, b| {
                            a.distance_squared(origin + forward * 1.65)
                                .total_cmp(&b.distance_squared(origin + forward * 1.65))
                        })
                {
                    // A taut indigo jib has a shallow sewn belly and remains
                    // static in reduced-motion mode. All three ties are real.
                    let a = mast - Vec3::Y * 1.3 + forward * 0.34;
                    let b = *tie + Vec3::Y * 0.60;
                    let c = *clew_tie + Vec3::Y * 1.05 + right * 0.18;
                    staysail(&mut canvas, a, b, c);
                    for (from, to) in [(a, b), (b, c), (c, a), (a, mast), (b, *tie), (c, *clew_tie)]
                    {
                        rope(&mut cordage, from, to);
                    }
                    // Narrow stitched seams share the canvas profile instead
                    // of adding a second broad untextured sail plane.
                    let normal = (b - a).cross(c - a).normalize();
                    for u in [0.25, 0.5, 0.75] {
                        let seam: Vec<_> = (0..=8)
                            .map(|step| {
                                let v = (1.0 - u) * step as f32 / 8.0;
                                a + (b - a) * u
                                    + (c - a) * v
                                    + normal * (u * v * (1.0 - u - v) * 5.4 + 0.012)
                            })
                            .collect();
                        curved_rod(&mut cordage, &seam, 0.010);
                    }
                }
            }
        }
        // Fan shrouds and ratlines sit outside the walkable waist. They rotate
        // with a quarter-turned sail and require both occupied deck anchors.
        for side in [-1.0, 1.0] {
            let mut ties = Vec::new();
            for aft in [0.6, 2.1] {
                let target = origin + right * (side * 2.0) - forward * aft;
                if let Some(tie) = deck
                    .iter()
                    .filter(|p| (**p - origin).dot(right) * side > 0.65)
                    .min_by(|a, b| {
                        a.distance_squared(target)
                            .total_cmp(&b.distance_squared(target))
                    })
                    .filter(|tie| tie.distance_squared(target) < 2.0)
                {
                    ties.push(*tie + right * (side * 0.15));
                }
            }
            if ties.len() == 2 && ties[0].distance(ties[1]) > 0.5 {
                let head = mast - Vec3::Y * (0.65 * scale);
                for tie in &ties {
                    rope(&mut cordage, *tie, head);
                    rod(
                        &mut iron,
                        *tie - Vec3::Y * 0.18,
                        *tie + Vec3::Y * 0.10,
                        0.05,
                    );
                    rod(
                        &mut brass,
                        *tie + Vec3::Y * 0.08,
                        *tie + Vec3::Y * 0.15,
                        0.063,
                    );
                }
                for rung in 1..=11 {
                    let fraction = rung as f32 / 15.0;
                    rod(
                        &mut cordage,
                        ties[0].lerp(head, fraction),
                        ties[1].lerp(head, fraction),
                        0.011,
                    );
                }
                rope(&mut cordage, ties[0].lerp(ties[1], 0.5), head);
            }
        }
    }
    if let Some((lo, _)) = grid.bounds() {
        for cell in grid
            .iter()
            .filter(|(c, b)| c.2 == lo.z && c.1 == 0 && *b == Block::Wood)
            .map(|(c, _)| c)
            .filter(|c| c.0 == 0)
            .take(1)
        {
            let p = cell.center() + Vec3::new(0., -0.2, -0.35);
            cuboid(&mut glow, p, Vec3::new(0.12, 0.17, 0.12));
            for y in [-0.13, 0.13] {
                cuboid(&mut brass, p + Vec3::Y * y, Vec3::new(0.23, 0.045, 0.23));
            }
            for x in [-0.085, 0.085] {
                for z in [-0.085, 0.085] {
                    cuboid(
                        &mut brass,
                        p + Vec3::new(x, 0., z),
                        Vec3::new(0.035, 0.25, 0.035),
                    );
                }
            }
            rope(
                &mut cordage,
                p + Vec3::Y * 0.16,
                cell.center() + Vec3::Y * 0.25,
            );
        }
    }
    // Metric grain follows every plank; the fine courses share one PBR material.
    for surface in [&mut planking, &mut timber] {
        for (index, position) in surface.positions.iter().enumerate() {
            let n = surface.normals[index];
            let p = Vec3::from_array(*position);
            surface.uvs[index] = if n[1].abs() > n[0].abs().max(n[2].abs()) {
                [p.x, p.z]
            } else {
                [if n[0].abs() > n[2].abs() { p.z } else { p.x }, p.y * 4.0]
            };
        }
    }
    [brass, cordage, glow, timber, planking, iron, canvas]
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::{ecs::world::CommandQueue, mesh::VertexAttributeValues};

    fn agrees_with_mikktspace(mesh: &Mesh) {
        let mut reference = mesh.clone();
        reference.generate_tangents().unwrap();
        let VertexAttributeValues::Float32x4(actual) =
            mesh.attribute(Mesh::ATTRIBUTE_TANGENT).unwrap()
        else {
            panic!("tangents")
        };
        let VertexAttributeValues::Float32x4(expected) =
            reference.attribute(Mesh::ATTRIBUTE_TANGENT).unwrap()
        else {
            panic!("reference tangents")
        };
        for (i, (a, b)) in actual.iter().zip(expected).enumerate() {
            assert!(
                Vec4::from_array(*a).abs_diff_eq(Vec4::from_array(*b), 2e-3),
                "vertex {i}: {a:?} != {b:?}"
            );
        }
    }

    #[test]
    fn editable_fittings_preserve_reference_normal_mapping() {
        for body in [
            aether_core::fixtures::starter(),
            aether_core::fixtures::explorer(),
        ] {
            let mut world = World::new();
            let owner = world.spawn_empty().id();
            let mut queue = CommandQueue::default();
            let mut meshes = Assets::<Mesh>::default();
            spawn(
                &mut Commands::new(&mut queue, &world),
                &mut meshes,
                &mut CraftMeshes::default(),
                &crate::Palette::default(),
                &Handle::default(),
                owner,
                &body,
            );
            assert_eq!(
                meshes.len(),
                if body.parts().iter().any(|p| p.kind == PartKind::GrandSail) {
                    7
                } else {
                    6
                }
            );
            for (_, mesh) in meshes.iter() {
                agrees_with_mikktspace(mesh);
            }
        }
    }

    #[test]
    fn identical_hulls_share_meshes_but_edits_and_sail_rotation_do_not() {
        let body = aether_core::fixtures::explorer();
        let mut meshes = Assets::<Mesh>::default();
        let mut shared = CraftMeshes::default();
        let original = shared.batches(&body, &mut meshes).to_vec();
        let ids = |batches: &[(usize, Handle<Mesh>)]| {
            batches
                .iter()
                .map(|(slot, handle)| (*slot, handle.id()))
                .collect::<Vec<_>>()
        };
        let mut renamed = body.blueprint();
        renamed.name = "Autre bateau".into();
        for part in &mut renamed.parts {
            part.id.0 += 10_000;
        }
        let renamed = Body::from_blueprint(renamed).unwrap();
        assert_eq!(ids(&original), ids(shared.batches(&renamed, &mut meshes)));
        assert_eq!(meshes.len(), 7);

        let mut edited = body.blueprint();
        let (_, block) = edited
            .cells
            .iter_mut()
            .find(|(_, b)| *b == Block::Wood)
            .unwrap();
        *block = Block::Metal;
        let edited = Body::from_blueprint(edited).unwrap();
        let edited_batches = shared.batches(&edited, &mut meshes).to_vec();
        assert_ne!(ids(&original), ids(&edited_batches));
        assert_eq!(ids(&original), ids(shared.batches(&body, &mut meshes)));

        let mut rotated = body.blueprint();
        let sail = rotated.parts.iter_mut().find(|p| p.kind.is_sail()).unwrap();
        sail.quarter_turn = (sail.quarter_turn + 2) % 4;
        let rotated = Body::from_blueprint(rotated).unwrap();
        assert_ne!(ids(&original), ids(shared.batches(&rotated, &mut meshes)));

        // Old edits release their retained GPU handles once no ECS decoration
        // uses them. An unchanged sibling keeps the original shared entry alive.
        shared.retain_live(&original.iter().map(|(_, h)| h.id()).collect());
        assert_eq!(shared.0.len(), 1);
        assert_eq!(ids(&original), ids(shared.batches(&renamed, &mut meshes)));
        shared.retain_live(&Default::default());
        assert!(shared.0.is_empty());
    }

    #[test]
    fn flat_tangents_preserve_rotated_and_mirrored_uvs() {
        for rotation in [
            Quat::IDENTITY,
            Quat::from_euler(EulerRot::YXZ, 0.71, -0.36, 0.29),
        ] {
            for mirrored in [-1.0, 1.0] {
                let mut surface = Surface::default();
                quad(
                    &mut surface,
                    [
                        Vec3::ZERO,
                        Vec3::X * 3.0,
                        Vec3::new(3.0, 2.0, 0.0),
                        Vec3::Y * 2.0,
                    ]
                    .map(|p| rotation * p),
                    rotation * Vec3::Z,
                );
                surface.uvs = vec![
                    [4.0, -2.0],
                    [4.0 + mirrored * 3.0, -1.0],
                    [5.0 + mirrored * 3.0, 1.0],
                    [5.0, 0.0],
                ];
                agrees_with_mikktspace(&crate::voxel::to_flat_mesh(surface));
            }
        }
    }
}
