use aether_core::{
    edit::{Edit, History},
    fields, mesh, picking,
    save::*,
    *,
};
use glam::{Quat, Vec3};

fn session() -> Session {
    let body = fixtures::starter();
    Session {
        version: VERSION,
        seed: 7391,
        tick: 0,
        vessels: vec![VesselSave {
            circuit: None,
            id: BodyId(1),
            fuel: body.fuel_capacity(),
            blueprint: body.blueprint(),
            position: Vec3::new(0.0, 12.0, 0.0),
            rotation: Quat::IDENTITY,
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            trim: 0.0,
            target_altitude: Some(12.0),
            docked: true,
        }],
        active: BodyId(1),
        checkpoint: 0,
        progress: 0,
        tether: None,
        walker: None,
        expedition: Default::default(),
    }
}

#[test]
fn negative_chunks_round_trip() {
    for z in [-17, -16, -1, 0, 15, 16, 17] {
        for y in [-17, -1, 0, 16] {
            for x in [-17, -16, -1, 0, 15, 16, 17] {
                let c = Cell(x, y, z);
                assert_eq!(Cell::from_index(c.chunk(), c.local_index()), c);
            }
        }
    }
}
#[test]
fn starter_has_every_function_and_valid_mass() {
    let b = fixtures::starter();
    for kind in [
        PartKind::Helm,
        PartKind::Sail,
        PartKind::Tank,
        PartKind::Lift,
        PartKind::Harpoon,
    ] {
        assert!(b.count_parts(kind) > 0);
    }
    assert!(b.mass_properties().inertia.determinant() > 0.0);
    assert_eq!(b.split().len(), 1);
}
#[test]
fn expedition_hull_keeps_a_walkable_boarding_space_and_valid_buoyancy_budget() {
    let body = fixtures::explorer();
    let boarding = body
        .boarding_cell()
        .expect("the helm must remain accessible");
    assert_ne!(body.grid().get(boarding), Block::Air);
    for height in 1..=3 {
        assert_eq!(
            body.grid().get(boarding.offset(Cell(0, height, 0))),
            Block::Air
        );
    }
    assert_eq!(body.split().len(), 1);
    let lift = body.count_parts(PartKind::Lift) as f32 * tuning::LIFT_NEWTONS_PER_PART;
    assert!(
        lift > body
            .mass_properties()
            .with_payload(expedition::Expedition::default().cargo_mass())
            .mass
            * tuning::GRAVITY
    );
    assert!(body.count_parts(PartKind::Propeller) >= 1);
}
#[test]
fn rejected_transaction_changes_nothing() {
    let mut b = fixtures::starter();
    let before = b.blueprint();
    let revision = b.revision();
    let mut h = History::default();
    assert!(
        h.apply(&mut b, Edit::Set(Cell(0, 0, 1), Block::Air))
            .is_err()
    );
    assert_eq!(b.blueprint(), before);
    assert_eq!(b.revision(), revision);
    assert!(!h.can_undo());
}

#[test]
fn merged_terrain_preserves_volume_and_segment_hits() {
    use aether_core::terrain::*;
    let mut seed = 7391_u64;
    for (index, island) in ISLANDS.iter().enumerate() {
        let original = island_pieces(island.center, island.radius, index as u32);
        let merged = collision_boxes(&original);
        assert!(merged.len() < original.len());
        let volume = |boxes: &[TerrainBox]| {
            boxes
                .iter()
                .map(|b| b.size.x as f64 * b.size.y as f64 * b.size.z as f64)
                .sum::<f64>()
        };
        assert!((volume(&original) - volume(&merged)).abs() < 0.01);
        for _ in 0..3000 {
            let mut point = || {
                let mut p = island.center;
                for axis in 0..3 {
                    seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                    p[axis] += ((seed >> 32) as u32 as f32 / u32::MAX as f32 - 0.5) * 60.0;
                }
                p
            };
            let a = point();
            let b = point();
            assert_eq!(
                original.iter().any(|s| segment_intersects(a, b, s)),
                merged.iter().any(|s| segment_intersects(a, b, s)),
                "island {index}: {a:?} -> {b:?}"
            );
        }
    }
}
#[test]
fn undo_redo_100_operations() {
    let mut b = fixtures::starter();
    let before = b.blueprint();
    let mut h = History::default();
    for x in 0..100 {
        h.apply(&mut b, Edit::Set(Cell(x, 10, 0), Block::Wood))
            .unwrap();
    }
    let after = b.blueprint();
    for _ in 0..100 {
        assert!(h.undo(&mut b).unwrap());
    }
    assert_eq!(b.blueprint(), before);
    for _ in 0..100 {
        assert!(h.redo(&mut b).unwrap());
    }
    assert_eq!(b.blueprint(), after);
}
#[test]
fn new_edit_discards_redo() {
    let mut b = fixtures::starter();
    let mut h = History::default();
    h.apply(&mut b, Edit::Set(Cell(4, 0, 0), Block::Wood))
        .unwrap();
    h.undo(&mut b).unwrap();
    h.apply(&mut b, Edit::Set(Cell(5, 0, 0), Block::Glass))
        .unwrap();
    assert!(!h.can_redo());
}
#[test]
fn empty_chunk_is_freed() {
    let mut b = fixtures::starter();
    let mut h = History::default();
    h.apply(&mut b, Edit::Set(Cell(32, 0, 0), Block::Wood))
        .unwrap();
    assert!(b.grid().chunk_coords().any(|c| c == Cell(2, 0, 0)));
    h.apply(&mut b, Edit::Set(Cell(32, 0, 0), Block::Air))
        .unwrap();
    assert!(!b.grid().chunk_coords().any(|c| c == Cell(2, 0, 0)));
}
#[test]
fn bounded_cell_input_rejects_integer_extremes() {
    for x in [i32::MIN, i32::MAX, -16385, 16385] {
        assert!(
            Body::from_blueprint(Blueprint {
                circuit_design: None,
                name: "Invalid".into(),
                cells: vec![(Cell(x, 0, 0), Block::Wood)],
                parts: vec![]
            })
            .is_err()
        );
    }
}
#[test]
fn duplicate_and_extent_rejected() {
    let mut b = fixtures::starter().blueprint();
    b.cells.push(b.cells[0]);
    assert!(Body::from_blueprint(b).is_err());
    let b = Blueprint {
        circuit_design: None,
        name: "Large".into(),
        cells: vec![(Cell(0, 0, 0), Block::Wood), (Cell(128, 0, 0), Block::Wood)],
        parts: vec![],
    };
    assert_eq!(Body::from_blueprint(b).unwrap_err(), DomainError::Extent);
}
#[test]
fn part_footprint_conflict_is_atomic() {
    let mut b = fixtures::starter();
    let mut h = History::default();
    let before = b.blueprint();
    assert!(
        h.apply(&mut b, Edit::Set(Cell(0, 3, -1), Block::Wood))
            .is_err()
    );
    assert_eq!(before, b.blueprint());
}
#[test]
fn mass_of_single_cube_matches_formula() {
    let p = fixtures::solid([1, 1, 1], Block::Wood).mass_properties();
    assert_eq!(p.mass, 18.0);
    assert_eq!(p.center, Vec3::ZERO);
    assert!((p.inertia.x_axis.x - 18.0 * CELL_SIZE.powi(2) / 6.0).abs() < 1e-5);
}
#[test]
fn version_one_session_migrates_without_losing_the_ship_or_pose() {
    let original = session();
    let mut old = serde_json::to_value(&original).unwrap();
    old["version"] = serde_json::json!(1);
    old.as_object_mut().unwrap().remove("expedition");
    let migrated = Session::decode(&serde_json::to_vec(&old).unwrap()).unwrap();
    assert_eq!(migrated.version, VERSION);
    assert_eq!(migrated.vessels[0].blueprint, original.vessels[0].blueprint);
    assert_eq!(migrated.vessels[0].position, original.vessels[0].position);
    assert!(migrated.expedition.validate());
    let mut future = old.clone();
    future["version"] = serde_json::json!(VERSION + 1);
    assert!(matches!(
        Session::decode(&serde_json::to_vec(&future).unwrap()),
        Err(SaveError::Version)
    ));
}

#[test]
fn version_two_migrates_to_three_without_installing_a_circuit_or_changing_cargo() {
    let original = session();
    let mut old = serde_json::to_value(&original).unwrap();
    old["version"] = serde_json::json!(2);
    let migrated = Session::decode(&serde_json::to_vec(&old).unwrap()).unwrap();
    assert_eq!(migrated.version, 3);
    assert_eq!(migrated.vessels[0].blueprint, original.vessels[0].blueprint);
    assert_eq!(migrated.vessels[0].fuel, original.vessels[0].fuel);
    assert_eq!(migrated.expedition.cargo, original.expedition.cargo);
    assert!(migrated.vessels[0].circuit.is_none());
}

#[test]
fn fleet_undo_has_an_exact_integer_budget_and_rejects_malformed_fractional_debits() {
    use aether_network::*;
    let mut saved = session();
    let mut body = fixtures::starter();
    let ports = vessel_ports(&body);
    let root = ports.iter().find(|p| p.capacity > 0).unwrap().id;
    let design = Circuit {
        links: ports
            .iter()
            .filter(|p| p.id != root)
            .map(|p| Link {
                a: root,
                b: p.id,
                open: true,
            })
            .collect(),
        ports,
    };
    History::default()
        .apply(&mut body, Edit::Circuit(design.clone()))
        .unwrap();
    let mut network = Network::empty(design).unwrap();
    network.fill(root, 123_457).unwrap();
    saved.vessels[0].blueprint = body.blueprint();
    saved.vessels[0].fuel = network.amount() as f32 / 1000.0;
    saved.vessels[0].circuit = Some(CircuitState {
        network: network.capture(),
        ..Default::default()
    });
    for id in 2..=3 {
        let mut v = saved.vessels[0].clone();
        v.id = BodyId(id);
        saved.vessels.push(v);
    }
    let mut current = saved.clone();
    for v in &mut current.vessels {
        v.circuit.as_mut().unwrap().limit_total(33_337).unwrap();
        v.fuel = 33.337;
    }
    let mut undo = saved.clone();
    undo.limit_reserve_from(&current).unwrap();
    assert_eq!(
        undo.vessels
            .iter()
            .map(|v| v
                .circuit
                .as_ref()
                .unwrap()
                .validated_network()
                .unwrap()
                .amount())
            .sum::<u64>(),
        100_011
    );
    assert!(undo.encode().is_ok());
    let before = saved.encode().unwrap();
    let mut bad = serde_json::from_slice::<serde_json::Value>(&before).unwrap();
    bad["vessels"][0]["circuit"]["recharge"]["remainder"] = serde_json::json!(1_000_000_000u32);
    assert!(Session::decode(&serde_json::to_vec(&bad).unwrap()).is_err());
    assert_eq!(saved.encode().unwrap(), before);
}
#[test]
fn expedition_roundtrip_preserves_choices_resources_and_unique_rewards() {
    let mut s = session();
    s.expedition.destination = 900;
    s.expedition.discover(900);
    s.expedition.harvest(900, 0).unwrap();
    assert!(s.expedition.catalogue(fauna::Species::Guardian));
    let expected = serde_json::to_value(&s.expedition).unwrap();
    let mut loaded = Session::decode(&s.encode().unwrap()).unwrap();
    assert_eq!(serde_json::to_value(&loaded.expedition).unwrap(), expected);
    assert!(!loaded.expedition.catalogue(fauna::Species::Guardian));
    assert!(!loaded.expedition.discover(900));
    assert!(loaded.expedition.harvest(900, 0).is_err());
    loaded.expedition.destination = u32::MAX;
    assert!(loaded.validate().is_err());
}
#[test]
fn gusts_are_seeded_smooth_bounded_and_visible_over_a_few_seconds() {
    let position = Vec3::new(17.0, 12.0, -43.0);
    let mut minimum = f32::MAX;
    let mut maximum = 0.0_f32;
    let mut previous = fields::wind_seeded(position, 0.0, 7391);
    for tick in 1..=3600 {
        let time = tick as f32 / 60.0;
        let wind = fields::wind_seeded(position, time, 7391);
        assert_eq!(wind, fields::wind_seeded(position, time, 7391));
        assert!((8.0..15.0).contains(&wind.length()));
        assert!(wind.distance(previous) < 0.06, "no wind discontinuity");
        assert!(wind.z < -8.0, "the prevailing route must remain stable");
        if tick <= 600 {
            minimum = minimum.min(wind.length());
            maximum = maximum.max(wind.length());
        }
        previous = wind;
    }
    assert!(
        maximum - minimum > 1.2,
        "gust should be perceptible in ten seconds"
    );
    assert_ne!(
        fields::wind_seeded(position, 4.0, 7391),
        fields::wind_seeded(position, 4.0, 42)
    );
}
#[test]
fn exact_connectivity_finds_distant_loop() {
    let mut cells = vec![];
    for x in 0..8 {
        cells.push((Cell(x, 0, 0), Block::Wood));
        cells.push((Cell(x, 0, 7), Block::Wood));
    }
    for z in 1..7 {
        cells.push((Cell(0, 0, z), Block::Wood));
        cells.push((Cell(7, 0, z), Block::Wood));
    }
    let mut b = Body::from_blueprint(Blueprint {
        circuit_design: None,
        name: "Loop".into(),
        cells,
        parts: vec![],
    })
    .unwrap();
    let mut h = History::default();
    h.apply(&mut b, Edit::Set(Cell(0, 0, 3), Block::Air))
        .unwrap();
    assert_eq!(b.split().len(), 1);
    h.apply(&mut b, Edit::Set(Cell(7, 0, 3), Block::Air))
        .unwrap();
    let pieces = b.split();
    assert_eq!(pieces.len(), 2);
    assert_eq!(
        pieces.iter().map(|p| p.cells.len()).sum::<usize>(),
        b.grid().len()
    );
}
#[test]
fn cube_greedy_has_six_quads() {
    let b = fixtures::solid([4, 4, 4], Block::Wood);
    let m = mesh::chunk_mesh(b.grid(), Cell::ZERO);
    assert_eq!(m[&Block::Wood].indices.len(), 36);
}
#[test]
fn winding_points_out() {
    let b = fixtures::solid([1, 1, 1], Block::Wood);
    let m = &mesh::chunk_mesh(b.grid(), Cell::ZERO)[&Block::Wood];
    for tri in m.indices.as_chunks::<3>().0 {
        let [a, b, c] = [tri[0], tri[1], tri[2]].map(|i| Vec3::from(m.positions[i as usize]));
        let normal = Vec3::from(m.normals[tri[0] as usize]);
        assert!((b - a).cross(c - a).dot(normal) > 0.0);
    }
}
#[test]
fn real_chunk_boundary_hides_interior_faces() {
    let b = fixtures::solid([17, 1, 1], Block::Wood);
    let quads: usize = b
        .grid()
        .chunk_coords()
        .map(|c| mesh::chunk_mesh(b.grid(), c)[&Block::Wood].indices.len() / 6)
        .sum();
    assert_eq!(quads, 10);
    assert!(Grid::dirty_neighbours(Cell(15, 0, 0)).contains(&Cell(1, 0, 0)));
}
#[test]
fn glass_does_not_hide_opaque() {
    assert!(mesh::visible(Block::Wood, Block::Glass));
    assert!(!mesh::visible(Block::Glass, Block::Wood));
    assert!(!mesh::visible(Block::Glass, Block::Glass));
    assert!(mesh::visible(Block::Glass, Block::Air));
}
#[test]
fn dda_selects_surface_and_rejects_nan() {
    let b = fixtures::solid([1, 1, 1], Block::Wood);
    let hit = picking::raycast(b.grid(), Vec3::Z * 4.0, -Vec3::Z, 10.0).unwrap();
    assert_eq!(hit.cell, Cell::ZERO);
    assert_eq!(hit.normal, Cell(0, 0, 1));
    assert!((hit.distance - 3.75).abs() < 1e-5);
    assert!(picking::raycast(b.grid(), Vec3::splat(f32::NAN), Vec3::X, 10.0).is_none());
    assert!(picking::raycast(b.grid(), Vec3::Z * 4.0, Vec3::Y, 10.0).is_none());
}
#[test]
fn dda_inside_and_negative_coordinates() {
    let b = Body::from_blueprint(Blueprint {
        circuit_design: None,
        name: "Negative".into(),
        cells: vec![(Cell(-17, 0, 0), Block::Wood)],
        parts: vec![],
    })
    .unwrap();
    assert_eq!(
        picking::raycast(b.grid(), Vec3::new(-8.5, 0.0, 0.0), Vec3::X, 1.0)
            .unwrap()
            .cell,
        Cell(-17, 0, 0)
    );
}
#[test]
fn session_roundtrip_preserves_pose_and_origin() {
    let mut s = session();
    s.vessels[0].position = Vec3::new(13.0, 28.0, -99.0);
    s.vessels[0].rotation = Quat::from_rotation_y(0.87);
    let decoded = Session::decode(&s.encode().unwrap()).unwrap();
    assert_eq!(decoded.vessels[0].blueprint, s.vessels[0].blueprint);
    assert!(
        decoded.vessels[0]
            .rotation
            .abs_diff_eq(s.vessels[0].rotation, 1e-6)
    );
}
#[test]
fn session_rejects_corruption_and_impossible_fuel() {
    let s = session();
    let bytes = s.encode().unwrap();
    for n in [0, 1, bytes.len() / 2, bytes.len() - 1] {
        assert!(Session::decode(&bytes[..n]).is_err());
    }
    let mut s = s;
    s.vessels[0].fuel = 10000.0;
    assert!(s.encode().is_err());
}
#[test]
fn session_rejects_unknown_version_and_duplicate_ids() {
    let mut s = session();
    s.version = 999;
    assert!(s.validate().is_err());
    s.version = VERSION;
    s.vessels.push(s.vessels[0].clone());
    assert!(s.validate().is_err());
}
#[test]
fn blueprint_does_not_include_fuel() {
    let b = fixtures::starter();
    let data = BlueprintFile::encode(&b).unwrap();
    assert!(!String::from_utf8_lossy(&data).contains("fuel"));
    assert_eq!(
        BlueprintFile::decode(&data).unwrap().blueprint(),
        b.blueprint()
    );
}
#[test]
fn field_is_bounded_continuous_and_inert_outside() {
    let c = fields::archipelago_current();
    assert_eq!(c.sample(Vec3::splat(900.0)).1, 0.0);
    let a = c.sample(c.points[1] + Vec3::X * 0.001).0;
    let b = c.sample(c.points[1] - Vec3::X * 0.001).0;
    assert!(a.distance(b) < 0.01);
    assert!(fields::current_acceleration(&c, c.points[1], Vec3::splat(1000.0)).length() <= 8.001);
}
#[test]
fn sail_force_follows_normal_and_is_bounded() {
    let flow = Vec3::Z * 12.0;
    assert_eq!(fields::sail_force(flow, Vec3::X, 18.0), Vec3::ZERO);
    assert!(fields::sail_force(flow, Vec3::Z, 18.0).z > 0.0);
    assert!(fields::sail_force(Vec3::Z * 10000.0, Vec3::Z, 1000.0).length() <= 18000.0);
}

#[test]
fn cooperative_mesher_matches_full_mesher_for_bounded_generated_shapes() {
    let mut state = 7391_u64;
    for _ in 0..20 {
        let mut cells = std::collections::BTreeMap::new();
        for _ in 0..300 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let c = Cell(
                ((state >> 8) % 40) as i32 - 20,
                ((state >> 24) % 5) as i32 - 2,
                ((state >> 40) % 40) as i32 - 20,
            );
            cells.insert(
                c,
                if state & 3 == 0 {
                    Block::Glass
                } else {
                    Block::Wood
                },
            );
        }
        let body = Body::from_blueprint(Blueprint {
            circuit_design: None,
            name: "generated".into(),
            cells: cells.into_iter().collect(),
            parts: vec![],
        })
        .unwrap();
        for coord in body.grid().chunk_coords() {
            let expected = mesh::chunk_mesh(body.grid(), coord);
            let mut cooperative = mesh::ChunkMesher::new(coord);
            let mut slices = 1;
            while !cooperative.step(body.grid()) {
                slices += 1;
                assert!(slices <= 96);
            }
            assert_eq!(expected, cooperative.finish());
        }
    }
}
#[test]
fn entire_allowed_fleet_fits_the_session_byte_budget() {
    let mut s = session();
    let body = fixtures::solid([20, 25, 20], Block::Wood);
    s.vessels[0].blueprint = body.blueprint();
    s.vessels[0].fuel = 0.0;
    let template = s.vessels[0].clone();
    s.vessels = (1..=32)
        .map(|id| {
            let mut v = template.clone();
            v.id = BodyId(id);
            v
        })
        .collect();
    let bytes = s.encode().unwrap();
    assert!(bytes.len() < MAX_SAVE_BYTES);
    let decoded = Session::decode(&bytes).unwrap();
    assert_eq!(decoded.vessels.len(), 32);
    assert_eq!(decoded.vessels[31].blueprint.cells.len(), 10_000);
    s.vessels.push(template);
    assert!(s.validate().is_err());
}
#[test]
fn invalid_generated_inputs_never_mutate_the_body() {
    let mut b = fixtures::starter();
    let original = b.blueprint();
    let mut h = History::default();
    for coordinate in [i32::MIN, -16385, 16385, i32::MAX] {
        assert!(
            h.apply(&mut b, Edit::Set(Cell(coordinate, 0, 0), Block::Wood))
                .is_err()
        );
        assert_eq!(b.blueprint(), original);
    }
    let mut seed = 7319_u64;
    for length in 0..512 {
        let bytes: Vec<u8> = (0..length)
            .map(|_| {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                (seed >> 32) as u8
            })
            .collect();
        assert!(Session::decode(&bytes).is_err());
    }
}
#[test]
fn terrain_segment_test_detects_thin_obstacles_and_parallel_rays() {
    let obstacle = terrain::TerrainBox {
        center: Vec3::ZERO,
        size: Vec3::new(0.02, 2.0, 2.0),
        surface: terrain::Surface::Rock,
    };
    assert!(terrain::segment_intersects(
        Vec3::X * -100.0,
        Vec3::X * 100.0,
        &obstacle
    ));
    assert!(!terrain::segment_intersects(
        Vec3::new(0.02, -4.0, 0.0),
        Vec3::new(0.02, 4.0, 0.0),
        &obstacle
    ));
    assert!(!terrain::line_clear(Vec3::splat(f32::NAN), Vec3::ZERO));
}
#[test]
fn saves_preserve_altitude_setpoint_and_accept_early_v1() {
    let mut s = session();
    s.vessels[0].target_altitude = Some(42.0);
    assert_eq!(
        Session::decode(&s.encode().unwrap()).unwrap().vessels[0].target_altitude,
        Some(42.0)
    );
    let mut json = serde_json::to_value(s).unwrap();
    json["vessels"][0]
        .as_object_mut()
        .unwrap()
        .remove("target_altitude");
    assert_eq!(
        Session::decode(&serde_json::to_vec(&json).unwrap())
            .unwrap()
            .vessels[0]
            .target_altitude,
        None
    );
}
#[test]
fn ray_picking_uses_local_coordinates_on_a_rotated_body() {
    let b = fixtures::solid([1, 1, 1], Block::Wood);
    let rotation = Quat::from_euler(glam::EulerRot::YXZ, 0.87, 0.2, -0.3);
    let translation = Vec3::new(-30.0, 17.0, 4.0);
    let origin = translation + rotation * (Vec3::Z * 4.0);
    let direction = rotation * (-Vec3::Z);
    let hit = picking::raycast(
        b.grid(),
        rotation.inverse() * (origin - translation),
        rotation.inverse() * direction,
        10.0,
    )
    .unwrap();
    assert_eq!(hit.cell, Cell::ZERO);
    assert!((hit.distance - 3.75).abs() < 0.00001);
}
#[test]
fn mesh_stress_does_not_relax_public_cell_limit() {
    let cells = (0..50_000)
        .map(|i| (Cell(i % 50, (i / 50) % 50, i / 2500), Block::Wood))
        .collect();
    assert_eq!(
        Body::from_blueprint(Blueprint {
            circuit_design: None,
            name: "Too large".into(),
            cells,
            parts: vec![]
        })
        .unwrap_err(),
        DomainError::CellLimit
    );
}
#[test]
fn boarding_respects_moved_helm_and_free_headroom() {
    let body = fixtures::starter();
    let c = body.boarding_cell().unwrap();
    assert_ne!(body.grid().get(c), Block::Air);
    assert!(
        body.parts()
            .iter()
            .all(|p| !p.footprint().contains(&c.offset(Cell(0, 1, 0))))
    );
    let mut blueprint = body.blueprint();
    for (cell, _) in &mut blueprint.cells {
        *cell = cell.offset(Cell(-40, 0, 80));
    }
    for part in &mut blueprint.parts {
        part.cell = part.cell.offset(Cell(-40, 0, 80));
    }
    let moved = Body::from_blueprint(blueprint).unwrap();
    assert_eq!(moved.boarding_cell().unwrap(), c.offset(Cell(-40, 0, 80)));
}

#[test]
fn tether_save_accepts_a_translated_origin_but_rejects_an_arbitrary_point() {
    let mut saved = session();
    let vessel = &mut saved.vessels[0];
    vessel.docked = false;
    for (cell, _) in &mut vessel.blueprint.cells {
        *cell = cell.offset(Cell(200, 0, -200));
    }
    for part in &mut vessel.blueprint.parts {
        part.cell = part.cell.offset(Cell(200, 0, -200));
    }
    let local_point = vessel
        .blueprint
        .parts
        .iter()
        .find(|p| p.kind == PartKind::Harpoon)
        .unwrap()
        .center();
    saved.tether = Some(aether_core::save::TetherSave {
        body: vessel.id,
        anchor: 0,
        local_point,
        length: 20.0,
    });
    let decoded = Session::decode(&saved.encode().unwrap()).unwrap();
    assert_eq!(decoded.tether.unwrap().local_point, local_point);
    saved.tether.as_mut().unwrap().local_point += Vec3::X;
    assert!(saved.validate().is_err());
}
