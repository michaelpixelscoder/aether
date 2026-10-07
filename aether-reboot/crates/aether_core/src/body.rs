use crate::{Block, CELL_SIZE, Cell, Grid, MAX_CELLS, MAX_EXTENT};
use glam::{Mat3, Vec3};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, VecDeque};
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BodyId(pub u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PartId(pub u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PartKind {
    Helm,
    Sail,
    Tank,
    Lift,
    Harpoon,
    Propeller,
    GrandSail,
}
impl PartKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Helm => "Poste de pilotage",
            Self::Sail => "Voile",
            Self::Tank => "Réservoir",
            Self::Lift => "Sustentateur",
            Self::Harpoon => "Harpon",
            Self::Propeller => "Moteur à hélice",
            Self::GrandSail => "Grand-voile",
        }
    }
    pub fn mass(self) -> f32 {
        match self {
            Self::Helm => 25.0,
            Self::Sail => 40.0,
            Self::Tank => 80.0,
            Self::Lift => 55.0,
            Self::Harpoon => 35.0,
            Self::Propeller => 95.0,
            Self::GrandSail => 180.0,
        }
    }
    pub fn height(self) -> i32 {
        match self {
            Self::Sail => 6,
            Self::GrandSail => 15,
            Self::Tank => 2,
            _ => 1,
        }
    }
    pub fn size(self) -> Vec3 {
        match self {
            Self::Sail => Vec3::new(2.5, 2.35, 0.06),
            Self::Tank => Vec3::new(0.43, 0.9, 0.43),
            Self::Lift => Vec3::splat(0.40),
            Self::Helm => Vec3::new(0.4, 0.35, 0.18),
            Self::Harpoon => Vec3::new(0.18, 0.20, 0.7),
            Self::Propeller => Vec3::new(0.46, 0.46, 0.46),
            Self::GrandSail => Vec3::new(6.25, 5.875, 0.15),
        }
    }
    pub fn is_sail(self) -> bool {
        matches!(self, Self::Sail | Self::GrandSail)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Part {
    pub id: PartId,
    pub kind: PartKind,
    pub cell: Cell,
    pub quarter_turn: u8,
}
impl Part {
    pub fn center(&self) -> Vec3 {
        self.cell.center() + Vec3::Y * CELL_SIZE * (self.kind.height() as f32 + 1.0) * 0.5
    }
    pub fn footprint(&self) -> Vec<Cell> {
        let width = match self.kind {
            PartKind::Sail => 2,
            PartKind::GrandSail => 6,
            _ => 0,
        };
        let mut cells = Vec::new();
        for y in 1..=self.kind.height() {
            for x in -width..=width {
                let offset = match self.quarter_turn {
                    0 => Cell(x, y, 0),
                    1 => Cell(0, y, -x),
                    2 => Cell(-x, y, 0),
                    _ => Cell(0, y, x),
                };
                cells.push(self.cell.offset(offset));
            }
        }
        cells
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Blueprint {
    pub name: String,
    pub cells: Vec<(Cell, Block)>,
    pub parts: Vec<Part>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub circuit_design: Option<crate::aether_network::Circuit>,
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum DomainError {
    #[error("La construction doit contenir entre 1 et {MAX_CELLS} cellules.")]
    CellLimit,
    #[error("La construction dépasse {MAX_EXTENT} cellules par axe.")]
    Extent,
    #[error("Coordonnées invalides ou cellule répétée.")]
    Coordinates,
    #[error("Composant invalide, sans support ou en collision.")]
    Part,
    #[error("Identifiant dupliqué ou invalide.")]
    Id,
    #[error("Cette cellule porte un composant : retirez-le d'abord.")]
    Occupied,
    #[error("Nom invalide (1–64 caractères). ")]
    Name,
    #[error("Le circuit d'Aether ne correspond pas aux composants de la construction.")]
    Circuit,
}

#[derive(Clone, Debug)]
pub struct Body {
    grid: Grid,
    parts: Vec<Part>,
    name: String,
    revision: u64,
    circuit_design: Option<crate::aether_network::Circuit>,
}
impl Body {
    pub fn from_blueprint(mut blueprint: Blueprint) -> Result<Self, DomainError> {
        if blueprint.name.trim().is_empty() || blueprint.name.chars().count() > 64 {
            return Err(DomainError::Name);
        }
        if blueprint.cells.is_empty() || blueprint.cells.len() > MAX_CELLS {
            return Err(DomainError::CellLimit);
        }
        // Reject dispersed and invalid coordinates before allocating dense chunks.
        let mut low = glam::IVec3::splat(i32::MAX);
        let mut high = glam::IVec3::splat(i32::MIN);
        for (cell, _) in &blueprint.cells {
            if [cell.0, cell.1, cell.2]
                .iter()
                .any(|v| !(-16_384..=16_384).contains(v))
            {
                return Err(DomainError::Coordinates);
            }
            low = low.min(cell.ivec());
            high = high.max(cell.ivec());
        }
        if (high - low).max_element() >= MAX_EXTENT {
            return Err(DomainError::Extent);
        }
        let mut grid = Grid::default();
        for (cell, block) in blueprint.cells {
            if [cell.0, cell.1, cell.2]
                .iter()
                .any(|v| !(-16_384..=16_384).contains(v))
                || block == Block::Air
                || grid.get(cell) != Block::Air
            {
                return Err(DomainError::Coordinates);
            }
            grid.set(cell, block);
        }
        let (min, max) = grid.bounds().ok_or(DomainError::CellLimit)?;
        if (max - min).max_element() >= MAX_EXTENT {
            return Err(DomainError::Extent);
        }
        blueprint.parts.sort_by_key(|p| p.id);
        let mut ids = BTreeSet::new();
        let mut occupied = BTreeSet::new();
        for part in &blueprint.parts {
            if part.id.0 == 0 || part.id.0 > u64::MAX - 1024 || !ids.insert(part.id) {
                return Err(DomainError::Id);
            }
            if part.quarter_turn > 3 || grid.get(part.cell) == Block::Air {
                return Err(DomainError::Part);
            }
            for cell in part.footprint() {
                if grid.get(cell) != Block::Air || !occupied.insert(cell) {
                    return Err(DomainError::Part);
                }
            }
        }
        let circuit_design = blueprint
            .circuit_design
            .map(|design| {
                design
                    .validate_for_parts(&blueprint.parts)
                    .map_err(|_| DomainError::Circuit)
            })
            .transpose()?;
        Ok(Self {
            grid,
            parts: blueprint.parts,
            name: blueprint.name,
            revision: 0,
            circuit_design,
        })
    }
    pub fn grid(&self) -> &Grid {
        &self.grid
    }
    pub fn parts(&self) -> &[Part] {
        &self.parts
    }
    pub fn circuit_design(&self) -> Option<&crate::aether_network::Circuit> {
        self.circuit_design.as_ref()
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Free headroom near the helm, so leaving the post never spawns over empty air.
    pub fn boarding_cell(&self) -> Option<Cell> {
        let target = self
            .parts
            .iter()
            .find(|p| p.kind == PartKind::Helm)
            .map_or(Vec3::ZERO, |p| p.cell.center() + Vec3::Z);
        let reserved: BTreeSet<_> = self.parts.iter().flat_map(Part::footprint).collect();
        self.grid
            .iter()
            .map(|(cell, _)| cell)
            .filter(|cell| {
                (1..=3).all(|y| {
                    let above = cell.offset(Cell(0, y, 0));
                    self.grid.get(above) == Block::Air && !reserved.contains(&above)
                })
            })
            .min_by(|a, b| {
                a.center()
                    .distance_squared(target)
                    .total_cmp(&b.center().distance_squared(target))
            })
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn blueprint(&self) -> Blueprint {
        let mut cells: Vec<_> = self.grid.iter().collect();
        cells.sort_by_key(|(c, _)| *c);
        Blueprint {
            name: self.name.clone(),
            cells,
            parts: self.parts.clone(),
            circuit_design: self.circuit_design.clone(),
        }
    }
    pub fn replace(&mut self, blueprint: Blueprint) -> Result<(), DomainError> {
        let mut next = Self::from_blueprint(blueprint)?;
        next.revision = self.revision.saturating_add(1);
        *self = next;
        Ok(())
    }
    pub fn count_parts(&self, kind: PartKind) -> usize {
        self.parts.iter().filter(|p| p.kind == kind).count()
    }
    pub fn fuel_capacity(&self) -> f32 {
        self.count_parts(PartKind::Tank) as f32 * 1800.0
    }
    pub fn mass_properties(&self) -> MassProperties {
        let elements: Vec<_> = self
            .grid
            .iter()
            .map(|(c, b)| (c.center(), b.mass(), Vec3::splat(CELL_SIZE)))
            .chain(self.parts.iter().map(|p| {
                let size = p.kind.size();
                (
                    p.center(),
                    p.kind.mass(),
                    if p.quarter_turn % 2 == 0 {
                        size
                    } else {
                        Vec3::new(size.z, size.y, size.x)
                    },
                )
            }))
            .collect();
        let mass: f32 = elements.iter().map(|(_, m, _)| *m).sum();
        let center = elements.iter().fold(Vec3::ZERO, |s, (p, m, _)| s + *p * *m) / mass;
        let inertia = elements.iter().fold(Mat3::ZERO, |s, (p, m, size)| {
            let r = *p - center;
            let outer = Mat3::from_cols(r * r.x, r * r.y, r * r.z);
            let squared = *size * *size;
            let local = Mat3::from_diagonal(
                Vec3::new(
                    squared.y + squared.z,
                    squared.x + squared.z,
                    squared.x + squared.y,
                ) / 12.0,
            );
            s + (local + Mat3::IDENTITY * r.length_squared() - outer) * *m
        });
        MassProperties {
            mass,
            center,
            inertia,
        }
    }
    /// Exact face connectivity. Stable ordering keeps the helm's component first.
    pub fn split(&self) -> Vec<Blueprint> {
        let mut remaining: BTreeSet<_> = self.grid.iter().map(|(c, _)| c).collect();
        let mut result = Vec::new();
        while let Some(&first) = remaining.first() {
            remaining.remove(&first);
            let mut queue = VecDeque::from([first]);
            let mut component = BTreeSet::from([first]);
            while let Some(cell) = queue.pop_front() {
                for dir in Cell::DIRECTIONS {
                    let n = cell.offset(dir);
                    if remaining.remove(&n) {
                        component.insert(n);
                        queue.push_back(n);
                    }
                }
            }
            let parts: Vec<_> = self
                .parts
                .iter()
                .filter(|p| component.contains(&p.cell))
                .cloned()
                .collect();
            result.push(Blueprint {
                name: self.name.clone(),
                cells: component.iter().map(|c| (*c, self.grid.get(*c))).collect(),
                circuit_design: self.circuit_design.as_ref().map(|d| d.for_parts(&parts)),
                parts,
            });
        }
        result.sort_by_key(|b| {
            (
                !b.parts.iter().any(|p| p.kind == PartKind::Helm),
                std::cmp::Reverse(b.cells.len()),
                b.cells[0].0,
            )
        });
        result
    }
}
#[derive(Clone, Copy, Debug)]
pub struct MassProperties {
    pub mass: f32,
    pub center: Vec3,
    pub inertia: Mat3,
}
impl MassProperties {
    /// Stowed payload is secured around the hull's centre, as a 2 × 1 × 4 m crate.
    pub fn with_payload(self, kilograms: f32) -> Self {
        let extra = kilograms.max(0.0);
        Self {
            mass: self.mass + extra,
            center: self.center,
            inertia: self.inertia
                + Mat3::from_diagonal(Vec3::new(17.0, 20.0, 5.0) * (extra / 12.0)),
        }
    }
}
