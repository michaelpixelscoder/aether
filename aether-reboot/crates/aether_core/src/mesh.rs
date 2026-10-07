use crate::{Block, CELL_SIZE, Cell, Grid};
use glam::Vec3;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Surface {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}
pub fn visible(source: Block, neighbour: Block) -> bool {
    source != Block::Air
        && (neighbour == Block::Air || (source.opaque() && neighbour == Block::Glass))
}
/// Deterministic greedy rectangles. Neighbour reads include adjacent chunks.
/// Mesh vertices stay in body-local space; chunk boundaries never restart UVs.
pub fn chunk_mesh(grid: &Grid, chunk: Cell) -> BTreeMap<Block, Surface> {
    let mut mesher = ChunkMesher::new(chunk);
    while !mesher.step(grid) {}
    mesher.finish()
}
/// One step visits at most one 16×16 face mask. The browser can yield between
/// steps while the native worker uses the identical algorithm to completion.
pub struct ChunkMesher {
    chunk: Cell,
    next: usize,
    output: BTreeMap<Block, Surface>,
}
impl ChunkMesher {
    pub fn new(chunk: Cell) -> Self {
        Self {
            chunk,
            next: 0,
            output: BTreeMap::new(),
        }
    }
    pub fn step(&mut self, grid: &Grid) -> bool {
        if self.next >= 96 {
            return true;
        }
        let base = self.chunk.ivec() * 16;
        let axis = self.next / 32;
        let sign = if self.next % 32 < 16 { -1 } else { 1 };
        let slice = (self.next % 16) as i32;
        let output = &mut self.output;
        let u = (axis + 1) % 3;
        let v = (axis + 2) % 3;
        let mut mask = [Block::Air; 256];
        for j in 0..16 {
            for i in 0..16 {
                let mut p = base;
                p[axis] += slice;
                p[u] += i;
                p[v] += j;
                let mut q = p;
                q[axis] += sign;
                let material = grid.get(p.into());
                if visible(material, grid.get(q.into())) {
                    mask[(i + j * 16) as usize] = material;
                }
            }
        }
        for j in 0..16_usize {
            let mut i = 0;
            while i < 16 {
                let material = mask[i + j * 16];
                if material == Block::Air {
                    i += 1;
                    continue;
                }
                let mut width = 1;
                while i + width < 16 && mask[i + width + j * 16] == material {
                    width += 1;
                }
                let mut height = 1;
                'height: while j + height < 16 {
                    for x in i..i + width {
                        if mask[x + (j + height) * 16] != material {
                            break 'height;
                        }
                    }
                    height += 1;
                }
                for y in j..j + height {
                    for x in i..i + width {
                        mask[x + y * 16] = Block::Air;
                    }
                }
                let mut p = base.as_vec3() - Vec3::splat(0.5);
                p[axis] += slice as f32 + if sign > 0 { 1.0 } else { 0.0 };
                p[u] += i as f32;
                p[v] += j as f32;
                let mut du = Vec3::ZERO;
                du[u] = width as f32;
                let mut dv = Vec3::ZERO;
                dv[v] = height as f32;
                let vertices = if sign > 0 {
                    [p, p + du, p + du + dv, p + dv]
                } else {
                    [p, p + dv, p + du + dv, p + du]
                };
                let mesh: &mut Surface = output.entry(material).or_default();
                let start = mesh.positions.len() as u32;
                let mut normal = Vec3::ZERO;
                normal[axis] = sign as f32;
                for point in vertices {
                    mesh.positions.push((point * CELL_SIZE).to_array());
                    mesh.normals.push(normal.to_array());
                    // Vertical walls keep the plank courses horizontal on both axes.
                    let (texture_u, texture_v) = if axis == 0 { (2, 1) } else { (u, v) };
                    mesh.uvs
                        .push([point[texture_u] * CELL_SIZE, point[texture_v] * CELL_SIZE]);
                }
                mesh.indices
                    .extend([start, start + 1, start + 2, start, start + 2, start + 3]);
                i += width;
            }
        }
        self.next += 1;
        self.next == 96
    }
    pub fn finish(self) -> BTreeMap<Block, Surface> {
        assert_eq!(self.next, 96, "meshing must be complete before publication");
        self.output
    }
}
