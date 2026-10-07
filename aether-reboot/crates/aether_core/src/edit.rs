use crate::{Block, Blueprint, Body, Cell, DomainError, Part, PartId};
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub enum Edit {
    Set(Cell, Block),
    Place(Part),
    RemovePart(PartId),
    Circuit(crate::aether_network::Circuit),
}
/// Same proposal and validation for the editor preview and the committed edit.
pub fn preview(body: &Body, edit: &Edit) -> Result<(), DomainError> {
    Body::from_blueprint(propose(body, edit)?.0).map(|_| ())
}
fn propose(body: &Body, edit: &Edit) -> Result<(Blueprint, Vec<Cell>), DomainError> {
    let mut after = body.blueprint();
    let mut dirty = Vec::new();
    match edit {
        Edit::Set(cell, block) => {
            if [cell.0, cell.1, cell.2]
                .iter()
                .any(|c| !(-16_384..=16_384).contains(c))
            {
                return Err(DomainError::Coordinates);
            }
            if *block == Block::Air && after.parts.iter().any(|p| p.cell == *cell) {
                return Err(DomainError::Occupied);
            }
            after.cells.retain(|(c, _)| c != cell);
            if *block != Block::Air {
                after.cells.push((*cell, *block));
            }
            dirty = crate::Grid::dirty_neighbours(*cell);
        }
        Edit::Place(part) => after.parts.push(part.clone()),
        Edit::RemovePart(id) => after.parts.retain(|p| p.id != *id),
        Edit::Circuit(circuit) => after.circuit_design = Some(circuit.clone()),
    }
    if !matches!(edit, Edit::Circuit(_)) {
        after.circuit_design = after.circuit_design.map(|d| d.for_parts(&after.parts));
    }
    Ok((after, dirty))
}
#[derive(Clone, Debug)]
struct Record {
    before: Blueprint,
    after: Blueprint,
    bytes: usize,
}
/// Bounded transactional undo. History never contains runtime fuel or physics state.
#[derive(Default)]
pub struct History {
    undo: VecDeque<Record>,
    redo: Vec<Record>,
    bytes: usize,
}
impl History {
    pub fn apply(&mut self, body: &mut Body, edit: Edit) -> Result<Vec<Cell>, DomainError> {
        let before = body.blueprint();
        let (after, dirty) = propose(body, &edit)?;
        body.replace(after)?;
        let after = body.blueprint();
        if before == after {
            return Ok(dirty);
        }
        let bytes = (before.cells.len() + after.cells.len()) * 16
            + (before.parts.len() + after.parts.len()) * 64
            + [&before, &after]
                .iter()
                .filter_map(|b| b.circuit_design.as_ref())
                .map(|d| d.ports.len() * 16 + d.links.len() * 24)
                .sum::<usize>();
        self.redo.clear();
        self.bytes += bytes;
        self.undo.push_back(Record {
            before,
            after,
            bytes,
        });
        while self.bytes > 16 * 1024 * 1024 || self.undo.len() > 128 {
            if let Some(old) = self.undo.pop_front() {
                self.bytes -= old.bytes;
            } else {
                break;
            }
        }
        Ok(dirty)
    }
    pub fn undo(&mut self, body: &mut Body) -> Result<bool, DomainError> {
        let Some(record) = self.undo.back() else {
            return Ok(false);
        };
        body.replace(record.before.clone())?;
        let record = self.undo.pop_back().expect("record checked");
        self.bytes -= record.bytes;
        self.redo.push(record);
        Ok(true)
    }
    pub fn redo(&mut self, body: &mut Body) -> Result<bool, DomainError> {
        let Some(record) = self.redo.last() else {
            return Ok(false);
        };
        body.replace(record.after.clone())?;
        let record = self.redo.pop().expect("record checked");
        self.bytes += record.bytes;
        self.undo.push_back(record);
        Ok(true)
    }
    pub fn clear(&mut self) {
        *self = Self::default();
    }
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
}
