use std::{collections::HashMap, hash::Hash};

use bevy::prelude::*;

/// Upstream voxel data primitives retained for future chunk and meshing adapters.
pub use voxelize_core as upstream;
pub use voxelize_mesher as mesher;

/// Sparse editable voxel storage shared by Aether voxel experiences.
#[derive(Resource)]
pub struct VoxelWorld<T> {
    pub blocks: HashMap<IVec3, T>,
    pub revision: u64,
}

impl<T> VoxelWorld<T> {
    pub fn single(cell: IVec3, block: T) -> Self {
        Self {
            blocks: HashMap::from([(cell, block)]),
            revision: 1,
        }
    }
}

/// Indexed geometry for one material layer of a voxel surface.
pub struct VoxelMesh<T> {
    pub material: T,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    /// Face-oriented coordinates in voxel-body-local units.
    ///
    /// Unlike [`Self::uvs`], these do not restart at the beginning of a greedy
    /// quad. Materials can scale them to project macro textures continuously
    /// across separately generated coplanar faces.
    pub surface_coordinates: Vec<[f32; 2]>,
    /// Quad-local UVs retained for materials that intentionally repeat once per voxel.
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

/// Converts face-oriented body-space coordinates into UVs that cover each
/// greedy quad exactly once.
///
/// This is for authored macro textures with borders. Unlike repeating UVs,
/// the four corners of every continuous face reach `0` and `1`, so a border is
/// visible only at the physical edge of that face and never at an arbitrary
/// world-coordinate wrap.
pub fn normalized_quad_uvs(surface_coordinates: &[[f32; 2]]) -> Vec<[f32; 2]> {
    assert!(surface_coordinates.len().is_multiple_of(4));
    let mut uvs = Vec::with_capacity(surface_coordinates.len());
    for quad in surface_coordinates.chunks_exact(4) {
        let u_min = quad
            .iter()
            .map(|coordinate| coordinate[0])
            .fold(f32::INFINITY, f32::min);
        let u_max = quad
            .iter()
            .map(|coordinate| coordinate[0])
            .fold(f32::NEG_INFINITY, f32::max);
        let v_min = quad
            .iter()
            .map(|coordinate| coordinate[1])
            .fold(f32::INFINITY, f32::min);
        let v_max = quad
            .iter()
            .map(|coordinate| coordinate[1])
            .fold(f32::NEG_INFINITY, f32::max);
        let u_span = (u_max - u_min).max(1.0);
        let v_span = (v_max - v_min).max(1.0);
        uvs.extend(
            quad.iter()
                .map(|[u, v]| [(u - u_min) / u_span, (v - v_min) / v_span]),
        );
    }
    uvs
}

/// A rectangular region of an authored macro-texture atlas, measured in
/// voxel-sized cells. Regions must cover a macro face without overlap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MacroAtlasRegion {
    pub origin: [i32; 2],
    pub size: [i32; 2],
}

/// Splits exact macro-sized greedy faces along an authored texture layout.
///
/// A 5×5 wooden panel, for example, can use several greedy quads internally
/// while still sampling one continuous image: every generated sub-quad gets
/// the matching atlas rectangle. Other face sizes remain one continuous,
/// border-to-border macro texture instead of wrapping at world coordinates.
pub fn split_quads_for_macro_atlas<T>(
    mesh: &VoxelMesh<T>,
    macro_size: [i32; 2],
    regions: &[MacroAtlasRegion],
) -> VoxelMesh<T>
where
    T: Copy,
{
    let mut split = VoxelMesh {
        material: mesh.material,
        positions: Vec::new(),
        normals: Vec::new(),
        surface_coordinates: Vec::new(),
        uvs: Vec::new(),
        indices: Vec::new(),
    };
    for quad_index in 0..(mesh.positions.len() / 4) {
        let vertex_offset = quad_index * 4;
        let index_offset = quad_index * 6;
        let positions = &mesh.positions[vertex_offset..vertex_offset + 4];
        let normals = &mesh.normals[vertex_offset..vertex_offset + 4];
        let coordinates = &mesh.surface_coordinates[vertex_offset..vertex_offset + 4];
        let u_min = coordinates
            .iter()
            .map(|coordinate| coordinate[0])
            .fold(f32::INFINITY, f32::min);
        let u_max = coordinates
            .iter()
            .map(|coordinate| coordinate[0])
            .fold(f32::NEG_INFINITY, f32::max);
        let v_min = coordinates
            .iter()
            .map(|coordinate| coordinate[1])
            .fold(f32::INFINITY, f32::min);
        let v_max = coordinates
            .iter()
            .map(|coordinate| coordinate[1])
            .fold(f32::NEG_INFINITY, f32::max);
        let width = (u_max - u_min).round() as i32;
        let height = (v_max - v_min).round() as i32;
        let winding_is_forward = mesh.indices[index_offset + 1] == (vertex_offset + 1) as u32;

        if [width, height] == macro_size {
            for region in regions {
                let u0 = region.origin[0] as f32 / width as f32;
                let v0 = region.origin[1] as f32 / height as f32;
                let u1 = (region.origin[0] + region.size[0]) as f32 / width as f32;
                let v1 = (region.origin[1] + region.size[1]) as f32 / height as f32;
                append_atlas_quad(
                    &mut split,
                    positions,
                    normals,
                    u_min,
                    v_min,
                    width as f32,
                    height as f32,
                    [u0, v0],
                    [u1, v1],
                    winding_is_forward,
                );
            }
        } else {
            append_atlas_quad(
                &mut split,
                positions,
                normals,
                u_min,
                v_min,
                (u_max - u_min).max(1.0),
                (v_max - v_min).max(1.0),
                [0.0, 0.0],
                [1.0, 1.0],
                winding_is_forward,
            );
        }
    }
    split
}

/// Chooses one complete packed atlas template for each greedy quad.
///
/// This is the counterpart to [`split_quads_for_macro_atlas`] for materials
/// whose atlas contains independent shape templates. A 1×3 quad selects the
/// 1×3 plank rectangle as a whole: it is never assembled from three 1×1 UV
/// crops, so no fiber break can appear at voxel boundaries.
pub fn map_quads_to_macro_atlas<T>(
    mesh: &VoxelMesh<T>,
    atlas_cells: [i32; 2],
    atlas_pixels: [u32; 2],
    regions: &[MacroAtlasRegion],
) -> VoxelMesh<T>
where
    T: Copy,
{
    let mut mapped = VoxelMesh {
        material: mesh.material,
        positions: mesh.positions.clone(),
        normals: mesh.normals.clone(),
        surface_coordinates: mesh.surface_coordinates.clone(),
        uvs: Vec::with_capacity(mesh.surface_coordinates.len()),
        indices: mesh.indices.clone(),
    };
    let inset = [0.5 / atlas_pixels[0] as f32, 0.5 / atlas_pixels[1] as f32];
    for quad in mesh.surface_coordinates.chunks_exact(4) {
        let u_min = quad
            .iter()
            .map(|coordinate| coordinate[0])
            .fold(f32::INFINITY, f32::min);
        let u_max = quad
            .iter()
            .map(|coordinate| coordinate[0])
            .fold(f32::NEG_INFINITY, f32::max);
        let v_min = quad
            .iter()
            .map(|coordinate| coordinate[1])
            .fold(f32::INFINITY, f32::min);
        let v_max = quad
            .iter()
            .map(|coordinate| coordinate[1])
            .fold(f32::NEG_INFINITY, f32::max);
        let size = [
            (u_max - u_min).round() as i32,
            (v_max - v_min).round() as i32,
        ];
        let candidates = regions
            .iter()
            .filter(|region| region.size == size)
            .collect::<Vec<_>>();
        let selected = if candidates.is_empty() {
            None
        } else {
            let seed = u_min.floor() as i32 + v_min.floor() as i32 * 31;
            Some(candidates[seed.rem_euclid(candidates.len() as i32) as usize])
        };
        let (min, max) = selected.map_or(([0.0, 0.0], [1.0, 1.0]), |region| {
            (
                [
                    region.origin[0] as f32 / atlas_cells[0] as f32 + inset[0],
                    region.origin[1] as f32 / atlas_cells[1] as f32 + inset[1],
                ],
                [
                    (region.origin[0] + region.size[0]) as f32 / atlas_cells[0] as f32 - inset[0],
                    (region.origin[1] + region.size[1]) as f32 / atlas_cells[1] as f32 - inset[1],
                ],
            )
        });
        mapped
            .uvs
            .extend([min, [max[0], min[1]], max, [min[0], max[1]]]);
    }
    mapped
}

#[allow(clippy::too_many_arguments)]
fn append_atlas_quad<T>(
    mesh: &mut VoxelMesh<T>,
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    coordinate_u: f32,
    coordinate_v: f32,
    coordinate_width: f32,
    coordinate_height: f32,
    uv_min: [f32; 2],
    uv_max: [f32; 2],
    winding_is_forward: bool,
) {
    let first = mesh.positions.len() as u32;
    let point = |u: f32, v: f32| {
        let top = lerp3(positions[0], positions[1], u);
        let bottom = lerp3(positions[3], positions[2], u);
        lerp3(top, bottom, v)
    };
    mesh.positions.extend([
        point(uv_min[0], uv_min[1]),
        point(uv_max[0], uv_min[1]),
        point(uv_max[0], uv_max[1]),
        point(uv_min[0], uv_max[1]),
    ]);
    mesh.normals.extend([normals[0]; 4]);
    mesh.surface_coordinates.extend([
        [
            coordinate_u + coordinate_width * uv_min[0],
            coordinate_v + coordinate_height * uv_min[1],
        ],
        [
            coordinate_u + coordinate_width * uv_max[0],
            coordinate_v + coordinate_height * uv_min[1],
        ],
        [
            coordinate_u + coordinate_width * uv_max[0],
            coordinate_v + coordinate_height * uv_max[1],
        ],
        [
            coordinate_u + coordinate_width * uv_min[0],
            coordinate_v + coordinate_height * uv_max[1],
        ],
    ]);
    mesh.uvs.extend([
        uv_min,
        [uv_max[0], uv_min[1]],
        uv_max,
        [uv_min[0], uv_max[1]],
    ]);
    if winding_is_forward {
        mesh.indices
            .extend([first, first + 1, first + 2, first, first + 2, first + 3]);
    } else {
        mesh.indices
            .extend([first, first + 2, first + 1, first, first + 3, first + 2]);
    }
}

fn lerp3(start: [f32; 3], end: [f32; 3], amount: f32) -> [f32; 3] {
    [
        start[0] + (end[0] - start[0]) * amount,
        start[1] + (end[1] - start[1]) * amount,
        start[2] + (end[2] - start[2]) * amount,
    ]
}

/// Builds greedy-merged surfaces for opaque voxels, grouped by material.
pub fn greedy_mesh<T>(
    blocks: &HashMap<IVec3, T>,
    is_opaque: impl Fn(T) -> bool,
) -> Vec<VoxelMesh<T>>
where
    T: Copy + Eq + Hash,
{
    let mut faces = HashMap::new();
    for (&cell, &material) in blocks {
        if !is_opaque(material) {
            continue;
        }
        for axis in 0..3 {
            for direction in [-1, 1] {
                let normal = IVec3::AXES[axis] * direction;
                if blocks
                    .get(&(cell + normal))
                    .is_some_and(|neighbor| is_opaque(*neighbor))
                {
                    continue;
                }
                let plane = cell[axis] + i32::from(direction > 0);
                let u_axis = (axis + 1) % 3;
                let v_axis = (axis + 2) % 3;
                faces
                    .entry((axis, direction, plane))
                    .or_insert_with(HashMap::new)
                    .insert((cell[u_axis], cell[v_axis]), material);
            }
        }
    }

    let mut meshes = HashMap::<T, VoxelMesh<T>>::new();
    for ((axis, direction, plane), mut plane_faces) in faces {
        while let Some((&(u, v), &material)) = plane_faces.iter().min_by_key(|(cell, _)| *cell) {
            let mut width = 1;
            while plane_faces.get(&(u + width, v)) == Some(&material) {
                width += 1;
            }
            let mut height = 1;
            'height: loop {
                for offset in 0..width {
                    if plane_faces.get(&(u + offset, v + height)) != Some(&material) {
                        break 'height;
                    }
                }
                height += 1;
            }
            for u_offset in 0..width {
                for v_offset in 0..height {
                    plane_faces.remove(&(u + u_offset, v + v_offset));
                }
            }
            let mesh = meshes.entry(material).or_insert_with(|| VoxelMesh {
                material,
                positions: Vec::new(),
                normals: Vec::new(),
                surface_coordinates: Vec::new(),
                uvs: Vec::new(),
                indices: Vec::new(),
            });
            append_quad(mesh, axis, direction, plane, u, v, width, height);
        }
    }
    meshes.into_values().collect()
}

fn append_quad<T>(
    mesh: &mut VoxelMesh<T>,
    axis: usize,
    direction: i32,
    plane: i32,
    u: i32,
    v: i32,
    width: i32,
    height: i32,
) {
    let u_axis = (axis + 1) % 3;
    let v_axis = (axis + 2) % 3;
    let mut corners = [[0.0; 3]; 4];
    for (corner, (u_value, v_value)) in [
        (u, v),
        (u + width, v),
        (u + width, v + height),
        (u, v + height),
    ]
    .into_iter()
    .enumerate()
    {
        corners[corner][axis] = plane as f32 - 0.5;
        corners[corner][u_axis] = u_value as f32 - 0.5;
        corners[corner][v_axis] = v_value as f32 - 0.5;
    }
    let first = mesh.positions.len() as u32;
    mesh.positions.extend(corners);
    let mut normal = [0.0; 3];
    normal[axis] = direction as f32;
    mesh.normals.extend([normal; 4]);
    mesh.surface_coordinates.extend([
        [u as f32, v as f32],
        [(u + width) as f32, v as f32],
        [(u + width) as f32, (v + height) as f32],
        [u as f32, (v + height) as f32],
    ]);
    mesh.uvs.extend([
        [0.0, 0.0],
        [width as f32, 0.0],
        [width as f32, height as f32],
        [0.0, height as f32],
    ]);
    if direction > 0 {
        mesh.indices
            .extend([first, first + 1, first + 2, first, first + 2, first + 3]);
    } else {
        mesh.indices
            .extend([first, first + 2, first + 1, first, first + 3, first + 2]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_world_contains_its_initial_voxel() {
        let world = VoxelWorld::single(IVec3::new(2, -1, 4), "hull");

        assert_eq!(world.blocks.len(), 1);
        assert_eq!(world.blocks[&IVec3::new(2, -1, 4)], "hull");
        assert_eq!(world.revision, 1);
    }

    #[test]
    fn greedy_mesher_merges_adjacent_voxels_and_culls_internal_faces() {
        let blocks = HashMap::from([(IVec3::ZERO, "hull"), (IVec3::X, "hull")]);

        let meshes = greedy_mesh(&blocks, |_| true);

        assert_eq!(meshes.len(), 1);
        assert_eq!(meshes[0].indices.len(), 36);
    }

    #[test]
    fn surface_coordinates_use_voxel_body_space_instead_of_quad_local_space() {
        let blocks = HashMap::from([(IVec3::new(4, 7, -3), "hull")]);

        let meshes = greedy_mesh(&blocks, |_| true);
        let coordinates = &meshes[0].surface_coordinates;

        assert!(coordinates.contains(&[7.0, -3.0]));
        assert!(coordinates.contains(&[8.0, -2.0]));
        assert!(!coordinates.iter().all(|coordinate| {
            (0.0..=1.0).contains(&coordinate[0]) && (0.0..=1.0).contains(&coordinate[1])
        }));
    }

    #[test]
    fn surface_coordinates_span_a_greedy_quad_in_voxel_units() {
        let blocks = HashMap::from([
            (IVec3::new(2, 0, 0), "hull"),
            (IVec3::new(3, 0, 0), "hull"),
            (IVec3::new(4, 0, 0), "hull"),
        ]);

        let meshes = greedy_mesh(&blocks, |_| true);
        let has_three_voxel_span = meshes[0].surface_coordinates.chunks_exact(4).any(|quad| {
            let u_min = quad.iter().map(|uv| uv[0]).fold(f32::INFINITY, f32::min);
            let u_max = quad
                .iter()
                .map(|uv| uv[0])
                .fold(f32::NEG_INFINITY, f32::max);
            let v_min = quad.iter().map(|uv| uv[1]).fold(f32::INFINITY, f32::min);
            let v_max = quad
                .iter()
                .map(|uv| uv[1])
                .fold(f32::NEG_INFINITY, f32::max);
            (u_max - u_min == 3.0) || (v_max - v_min == 3.0)
        });

        assert!(has_three_voxel_span);
    }

    #[test]
    fn normalized_quad_uvs_put_borders_only_at_greedy_quad_edges() {
        let blocks = HashMap::from([
            (IVec3::ZERO, "hull"),
            (IVec3::X, "hull"),
            (IVec3::new(2, 0, 0), "hull"),
        ]);
        let mesh = &greedy_mesh(&blocks, |_| true)[0];
        let uvs = normalized_quad_uvs(&mesh.surface_coordinates);

        assert!(
            uvs.iter()
                .all(|[u, v]| (0.0..=1.0).contains(u) && (0.0..=1.0).contains(v))
        );
        assert!(uvs.chunks_exact(4).all(|quad| {
            let u_min = quad.iter().map(|uv| uv[0]).fold(f32::INFINITY, f32::min);
            let u_max = quad
                .iter()
                .map(|uv| uv[0])
                .fold(f32::NEG_INFINITY, f32::max);
            let v_min = quad.iter().map(|uv| uv[1]).fold(f32::INFINITY, f32::min);
            let v_max = quad
                .iter()
                .map(|uv| uv[1])
                .fold(f32::NEG_INFINITY, f32::max);
            u_min == 0.0 && u_max == 1.0 && v_min == 0.0 && v_max == 1.0
        }));
    }

    #[test]
    fn macro_atlas_splits_a_five_by_five_face_into_authored_regions() {
        let mut blocks = HashMap::new();
        for x in 0..5 {
            for y in 0..5 {
                blocks.insert(IVec3::new(x, y, 0), "panel");
            }
        }
        let mesh = &greedy_mesh(&blocks, |_| true)[0];
        let atlas = [
            MacroAtlasRegion {
                origin: [0, 0],
                size: [1, 1],
            },
            MacroAtlasRegion {
                origin: [1, 0],
                size: [3, 1],
            },
            MacroAtlasRegion {
                origin: [4, 0],
                size: [1, 1],
            },
            MacroAtlasRegion {
                origin: [0, 1],
                size: [4, 1],
            },
            MacroAtlasRegion {
                origin: [4, 1],
                size: [1, 1],
            },
            MacroAtlasRegion {
                origin: [0, 2],
                size: [2, 1],
            },
            MacroAtlasRegion {
                origin: [2, 2],
                size: [2, 1],
            },
            MacroAtlasRegion {
                origin: [4, 2],
                size: [1, 1],
            },
            MacroAtlasRegion {
                origin: [0, 3],
                size: [5, 2],
            },
        ];
        let split = split_quads_for_macro_atlas(mesh, [5, 5], &atlas);

        // Its two 5×5 faces become nine atlas regions; the four thin side
        // faces stay single full-macro quads.
        assert_eq!(split.positions.len(), (2 * atlas.len() + 4) * 4);
        assert_eq!(split.indices.len(), (2 * atlas.len() + 4) * 6);
        assert!(split.uvs.contains(&[0.2, 0.0]));
        assert!(split.uvs.contains(&[0.8, 0.6]));
    }

    #[test]
    fn packed_atlas_maps_one_three_voxel_quad_to_one_template() {
        let blocks = HashMap::from([
            (IVec3::ZERO, "plank"),
            (IVec3::X, "plank"),
            (IVec3::new(2, 0, 0), "plank"),
        ]);
        let mesh = &greedy_mesh(&blocks, |_| true)[0];
        let atlas = [MacroAtlasRegion {
            origin: [1, 0],
            size: [3, 1],
        }];
        let mapped = map_quads_to_macro_atlas(mesh, [5, 5], [1280, 1280], &atlas);

        // The 3×1 outer face remains one quad and samples the complete 1×3
        // template, rather than repeating/cropping a 1×1 tile per voxel.
        assert_eq!(mapped.positions.len(), mesh.positions.len());
        assert_eq!(mapped.indices.len(), mesh.indices.len());
        assert!(
            mapped
                .uvs
                .contains(&[1.0 / 5.0 + 0.5 / 1280.0, 0.5 / 1280.0])
        );
        assert!(
            mapped
                .uvs
                .contains(&[4.0 / 5.0 - 0.5 / 1280.0, 1.0 / 5.0 - 0.5 / 1280.0])
        );
    }
}
