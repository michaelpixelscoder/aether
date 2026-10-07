use crate::{Block, Blueprint, Body, Cell, Part, PartId, PartKind};
pub fn starter() -> Body {
    let mut cells = Vec::new();
    for z in -5_i32..=5 {
        for x in -3_i32..=3 {
            if z.abs() < 4 || x.abs() < 2 {
                cells.push((Cell(x, 0, z), Block::Wood));
            }
        }
    }
    for z in -3..=3 {
        for x in [-3, 3] {
            if z != 0 {
                cells.push((Cell(x, 1, z), Block::Wood));
            }
        }
    }
    // Two tapered courses form the actual editable carène, including collision/mass.
    // Its lowest surface meets, rather than penetrates, the canonical dock.
    for z in -4_i32..=4 {
        for x in -2_i32..=2 {
            if z.abs() < 4 || x.abs() < 2 {
                cells.push((Cell(x, -1, z), Block::Wood));
            }
        }
    }
    for z in -3..=3 {
        for x in -1..=1 {
            cells.push((Cell(x, -2, z), Block::Wood));
        }
    }
    // Rear chart cabin; keep the helm approach at z=3 open.
    for z in 4..=5 {
        for x in [-1, 1] {
            for y in 1..=2 {
                cells.push((
                    Cell(x, y, z),
                    if y == 2 { Block::Glass } else { Block::Wood },
                ));
            }
        }
    }
    for x in -1..=1 {
        for z in 4..=5 {
            cells.push((Cell(x, 3, z), Block::Wood));
        }
    }
    cells.push((Cell(0, 1, 5), Block::Wood));
    cells.push((Cell(0, 2, 5), Block::Glass));
    let parts = vec![
        Part {
            id: PartId(1),
            kind: PartKind::Helm,
            cell: Cell(0, 0, 1),
            quarter_turn: 0,
        },
        Part {
            id: PartId(2),
            kind: PartKind::Sail,
            cell: Cell(0, 0, -1),
            quarter_turn: 0,
        },
        Part {
            id: PartId(3),
            kind: PartKind::Tank,
            cell: Cell(-2, 0, 1),
            quarter_turn: 0,
        },
        Part {
            id: PartId(4),
            kind: PartKind::Lift,
            cell: Cell(3, 0, 0),
            quarter_turn: 0,
        },
        Part {
            id: PartId(5),
            kind: PartKind::Lift,
            cell: Cell(-3, 0, 0),
            quarter_turn: 0,
        },
        Part {
            id: PartId(6),
            kind: PartKind::Harpoon,
            cell: Cell(0, 0, -5),
            quarter_turn: 0,
        },
    ];
    Body::from_blueprint(Blueprint {
        name: "L'Alcyon".into(),
        cells,
        parts,
    })
    .expect("validated starter fixture")
}
pub fn solid(dimensions: [i32; 3], block: Block) -> Body {
    let mut cells = Vec::new();
    for z in 0..dimensions[2] {
        for y in 0..dimensions[1] {
            for x in 0..dimensions[0] {
                cells.push((Cell(x, y, z), block));
            }
        }
    }
    Body::from_blueprint(Blueprint {
        name: "Fixture".into(),
        cells,
        parts: vec![],
    })
    .expect("fixture within domain limits")
}

/// The expedition vessel keeps every plank editable and physically meaningful.
/// The compact historical starter remains available to regression fixtures.
pub fn explorer() -> Body {
    use std::collections::BTreeMap;
    let mut cells = BTreeMap::new();
    // A long stem, flared shoulders and a narrowing transom replace the old
    // rectangular slab. All three hull courses are editable, load-bearing cells.
    // Keep the keel at y=-2 so the original dock clearance remains valid.
    for z in -18_i32..=14 {
        let width = match z {
            -18..=-16 => 0,
            -15..=-14 | 14 => 1,
            -13..=-12 | 12..=13 => 2,
            -11..=-8 | 8..=11 => 3,
            _ => 4,
        };
        for x in -width..=width {
            cells.insert(Cell(x, 0, z), Block::Wood);
        }
        if (-15..=13).contains(&z) {
            let bilge = (width - 1).max(0);
            for x in -bilge..=bilge {
                cells.insert(Cell(x, -1, z), Block::Wood);
            }
        }
        if (-11..=10).contains(&z) {
            let keel = if (-7..=7).contains(&z) { 1 } else { 0 };
            for x in -keel..=keel {
                cells.insert(Cell(x, -2, z), Block::Wood);
            }
        }
        // Raised ends produce a sheer line while the working waist and the
        // boarding opening beside the helm remain at the original deck height.
        if (-13..=-9).contains(&z) {
            for x in -width..=width {
                cells.insert(Cell(x, 1, z), Block::Wood);
            }
        } else if (8..=13).contains(&z) {
            for x in [-width, width] {
                cells.insert(Cell(x, 1, z), Block::Wood);
            }
        }
    }
    // A timber stern cabin with isolated lights and solid corner posts. The
    // former two continuous glass courses made every face a purple grid.
    for z in 8..=12 {
        let cabin_width = if z == 12 { 1_i32 } else { 2 };
        for x in -cabin_width..=cabin_width {
            for y in 1..=4 {
                if x.abs() == cabin_width || z == 12 || (z == 8 && x != 0) {
                    let window = y == 3
                        && ((x.abs() == 2 && matches!(z, 9 | 11))
                            || (z == 8 && x.abs() == 1)
                            || (z == 12 && x == 0));
                    cells.insert(
                        Cell(x, y, z),
                        if window { Block::Glass } else { Block::Wood },
                    );
                }
            }
            // A narrow arched crown follows the curved stern plan. Its three
            // strips are structural cells, with open cabin headroom below.
            cells.insert(Cell(x, 6 - x.abs(), z), Block::Wood);
            if matches!(z, 8 | 12) && x == 0 {
                cells.insert(Cell(x, 5, z), Block::Wood);
            }
        }
    }
    // A small mast step supports the existing mizzen instead of turning the
    // whole cabin roof into a heavy three-tier pyramid.
    for y in 6..=7 {
        cells.insert(Cell(0, y, 10), Block::Wood);
    }
    let mut parts = Vec::new();
    for (kind, cell) in [
        (PartKind::Helm, Cell(0, 0, 5)),
        (PartKind::GrandSail, Cell(0, 0, -3)),
        (PartKind::Sail, Cell(0, 7, 10)),
        (PartKind::Tank, Cell(-3, 0, 6)),
        (PartKind::Tank, Cell(3, 0, 6)),
        (PartKind::Harpoon, Cell(0, 0, -14)),
        (PartKind::Propeller, Cell(-4, 0, 7)),
        (PartKind::Propeller, Cell(4, 0, 7)),
    ] {
        parts.push(Part {
            id: PartId(parts.len() as u64 + 1),
            kind,
            cell,
            quarter_turn: 0,
        });
    }
    for x in [-4, 4] {
        for z in [-7, 0, 4] {
            parts.push(Part {
                id: PartId(parts.len() as u64 + 1),
                kind: PartKind::Lift,
                cell: Cell(x, 0, z),
                quarter_turn: 0,
            });
        }
    }
    Body::from_blueprint(Blueprint {
        name: "L'Alcyon — Expédition".into(),
        cells: cells.into_iter().collect(),
        parts,
    })
    .expect("valid expedition vessel")
}
