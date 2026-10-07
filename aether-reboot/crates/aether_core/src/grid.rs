use crate::{CELL_SIZE, CHUNK_SIZE};
use glam::{IVec3, Vec3};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
pub struct Cell(pub i32, pub i32, pub i32);

impl Cell {
    pub const ZERO: Self = Self(0, 0, 0);
    pub const DIRECTIONS: [Self; 6] = [
        Self(1, 0, 0),
        Self(-1, 0, 0),
        Self(0, 1, 0),
        Self(0, -1, 0),
        Self(0, 0, 1),
        Self(0, 0, -1),
    ];
    pub fn ivec(self) -> IVec3 {
        IVec3::new(self.0, self.1, self.2)
    }
    pub fn center(self) -> Vec3 {
        self.ivec().as_vec3() * CELL_SIZE
    }
    pub fn offset(self, other: Self) -> Self {
        Self(self.0 + other.0, self.1 + other.1, self.2 + other.2)
    }
    pub fn chunk(self) -> Self {
        Self(
            self.0.div_euclid(CHUNK_SIZE),
            self.1.div_euclid(CHUNK_SIZE),
            self.2.div_euclid(CHUNK_SIZE),
        )
    }
    pub fn local_index(self) -> usize {
        let x = self.0.rem_euclid(CHUNK_SIZE) as usize;
        let y = self.1.rem_euclid(CHUNK_SIZE) as usize;
        let z = self.2.rem_euclid(CHUNK_SIZE) as usize;
        x + 16 * (y + 16 * z)
    }
    pub fn from_index(chunk: Self, index: usize) -> Self {
        Self(
            chunk.0 * 16 + (index % 16) as i32,
            chunk.1 * 16 + ((index / 16) % 16) as i32,
            chunk.2 * 16 + (index / 256) as i32,
        )
    }
}
impl From<IVec3> for Cell {
    fn from(v: IVec3) -> Self {
        Self(v.x, v.y, v.z)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[repr(u8)]
pub enum Block {
    #[default]
    Air,
    Wood,
    Metal,
    Glass,
}
impl Block {
    pub fn mass(self) -> f32 {
        match self {
            Self::Air => 0.0,
            Self::Wood => 18.0,
            Self::Metal => 90.0,
            Self::Glass => 30.0,
        }
    }
    pub fn opaque(self) -> bool {
        matches!(self, Self::Wood | Self::Metal)
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Air => "Vide",
            Self::Wood => "Bois",
            Self::Metal => "Métal",
            Self::Glass => "Verre",
        }
    }
}

#[derive(Clone, Debug)]
struct Chunk {
    cells: Box<[Block; 4096]>,
    occupied: usize,
}
impl Default for Chunk {
    fn default() -> Self {
        Self {
            cells: Box::new([Block::Air; 4096]),
            occupied: 0,
        }
    }
}

/// Sparse chunk allocation, dense 16³ interiors. Coordinates refer to cell centres.
#[derive(Clone, Debug, Default)]
pub struct Grid {
    chunks: BTreeMap<Cell, Chunk>,
    count: usize,
}
impl Grid {
    pub fn get(&self, cell: Cell) -> Block {
        self.chunks
            .get(&cell.chunk())
            .map_or(Block::Air, |c| c.cells[cell.local_index()])
    }
    pub fn len(&self) -> usize {
        self.count
    }
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
    pub fn chunk_coords(&self) -> impl Iterator<Item = Cell> + '_ {
        self.chunks.keys().copied()
    }
    pub fn iter(&self) -> impl Iterator<Item = (Cell, Block)> + '_ {
        self.chunks.iter().flat_map(|(&coord, chunk)| {
            chunk
                .cells
                .iter()
                .copied()
                .enumerate()
                .filter(|(_, b)| *b != Block::Air)
                .map(move |(i, b)| (Cell::from_index(coord, i), b))
        })
    }
    pub(crate) fn set(&mut self, cell: Cell, block: Block) {
        let old = self.get(cell);
        if old == block {
            return;
        }
        let coord = cell.chunk();
        let chunk = self.chunks.entry(coord).or_default();
        if old == Block::Air {
            chunk.occupied += 1;
            self.count += 1;
        }
        if block == Block::Air {
            chunk.occupied -= 1;
            self.count -= 1;
        }
        chunk.cells[cell.local_index()] = block;
        if chunk.occupied == 0 {
            self.chunks.remove(&coord);
        }
    }
    pub fn bounds(&self) -> Option<(IVec3, IVec3)> {
        self.iter().map(|(c, _)| c.ivec()).fold(None, |a, c| {
            Some(a.map_or((c, c), |(min, max): (IVec3, IVec3)| {
                (min.min(c), max.max(c))
            }))
        })
    }
    pub fn dirty_neighbours(cell: Cell) -> Vec<Cell> {
        let mut result = vec![cell.chunk()];
        for dir in Cell::DIRECTIONS {
            let neighbour = cell.offset(dir).chunk();
            if !result.contains(&neighbour) {
                result.push(neighbour);
            }
        }
        result
    }
}
