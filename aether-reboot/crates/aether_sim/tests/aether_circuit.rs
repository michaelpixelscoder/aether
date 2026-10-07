#[allow(dead_code)]
mod common;
use aether_core::{BodyId, PartKind, aether_network::NetworkError, fixtures, save::VesselSave};
use aether_sim::{aether::AetherCircuit, *};
use avian3d::prelude::*;
use bevy::prelude::*;
use std::time::Duration;

fn ship(disconnect_motors: bool) -> (App, Entity) {
    let mut app = common::app();
    app.insert_resource(FlightEnvironment {
        wind: Some(Vec3::ZERO),
        currents: false,
    });
    let body = fixtures::explorer();
    let mut circuit = AetherCircuit::manifold(&body, body.fuel_capacity()).unwrap();
    if disconnect_motors {
        let engines: Vec<_> = body
            .parts()
            .iter()
            .filter(|p| p.kind == PartKind::Propeller)
            .map(|p| p.id)
            .collect();
        let mut topology = circuit.capture().circuit;
        for edge in &mut topology.links {
            if engines.contains(&edge.a) || engines.contains(&edge.b) {
                edge.open = false;
            }
        }
        circuit.reconfigure(&body, topology).unwrap();
    }
    let save = VesselSave {
        circuit: None,
        id: BodyId(1),
        blueprint: body.blueprint(),
        position: Vec3::new(0.0, 200.0, 500.0),
        rotation: Quat::IDENTITY,
        velocity: Vec3::ZERO,
        angular_velocity: Vec3::ZERO,
        fuel: body.fuel_capacity(),
        trim: 0.0,
        target_altitude: Some(200.0),
        docked: false,
    };
    let entity = spawn_vessel(&mut app.world_mut().commands(), save);
    app.world_mut().flush();
    app.world_mut().entity_mut(entity).insert(circuit);
    app.world_mut()
        .get_mut::<PilotIntent>(entity)
        .unwrap()
        .throttle = 1.0;
    (app, entity)
}

#[test]
fn uninstalling_distribution_on_a_maximum_storage_ship_cannot_round_fuel_up() {
    use aether_core::{Block, Blueprint, Body, Cell, Part, PartId};
    let body = Body::from_blueprint(Blueprint {
        name: "256 réservoirs".into(),
        cells: (0..32)
            .flat_map(|x| (0..32).map(move |z| (Cell(x, 0, z), Block::Wood)))
            .collect(),
        parts: (0..256)
            .map(|i| Part {
                id: PartId(i as u64 + 1),
                kind: PartKind::Tank,
                cell: Cell((i % 16) * 2, 0, (i / 16) * 2),
                quarter_turn: 0,
            })
            .collect(),
        circuit_design: None,
    })
    .unwrap();
    let mut supply = AetherCircuit::manifold(&body, body.fuel_capacity()).unwrap();
    for charge in [1, 17, 31, 41, 123_457, 1_999_999] {
        supply.pay_charge(charge).unwrap();
        let scalar = supply.conservative_common_fuel();
        assert!(f64::from(scalar) * 1000.0 <= supply.remaining_milli() as f64);
        let reinstalled = AetherCircuit::manifold(&body, scalar).unwrap();
        assert!(reinstalled.remaining_milli() <= supply.remaining_milli());
    }
}

#[test]
fn portal_charges_debit_authoritative_storage_and_refuse_insufficient_fuel_atomically() {
    let body = fixtures::explorer();
    let mut supply = AetherCircuit::manifold(&body, 123.457).unwrap();
    supply.pay_charge(80_000).unwrap();
    assert_eq!(supply.remaining_milli(), 43_457);
    let before = supply.capture_state();
    assert!(supply.pay_charge(80_000).is_err());
    assert_eq!(supply.capture_state(), before);
    let mut restored = AetherCircuit::restore(&body, supply.capture_state()).unwrap();
    restored.pay_charge(43_457).unwrap();
    assert_eq!(restored.remaining_milli(), 0);
    assert!(restored.pay_charge(1).is_err());
}

#[test]
fn full_session_roundtrip_restores_real_component_and_rejects_inconsistent_design() {
    use aether_core::{
        edit::{Edit, History},
        save::{Session, VERSION},
    };
    let mut body = fixtures::explorer();
    let mut circuit = AetherCircuit::manifold(&body, 1234.567).unwrap();
    History::default()
        .apply(&mut body, Edit::Circuit(circuit.capture().circuit))
        .unwrap();
    let tank = body
        .parts()
        .iter()
        .find(|p| p.kind == PartKind::Tank)
        .unwrap()
        .id;
    circuit.set_tank_leak(tank, 137).unwrap();
    circuit
        .burn(&body, Duration::from_nanos(16_666_667), 100_000.0, 0.7)
        .unwrap();
    let saved = Session {
        version: VERSION,
        seed: 7391,
        tick: 42,
        vessels: vec![VesselSave {
            id: BodyId(1),
            blueprint: body.blueprint(),
            position: Vec3::new(0.0, 200.0, 500.0),
            rotation: Quat::IDENTITY,
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            fuel: circuit.remaining_milli() as f32 / 1000.0,
            trim: 0.0,
            target_altitude: Some(200.0),
            docked: false,
            circuit: Some(circuit.capture_state()),
        }],
        active: BodyId(1),
        checkpoint: 0,
        progress: 0,
        tether: None,
        walker: None,
        expedition: Default::default(),
    };
    let loaded = Session::decode(&saved.encode().unwrap()).unwrap();
    let mut app = common::app();
    let entity = spawn_vessel(&mut app.world_mut().commands(), loaded.vessels[0].clone());
    app.world_mut().flush();
    assert_eq!(
        app.world()
            .get::<AetherCircuit>(entity)
            .unwrap()
            .capture_state(),
        circuit.capture_state()
    );
    let mut bad = loaded.clone();
    bad.vessels[0].circuit = None;
    assert!(bad.encode().is_err());
    let mut bad = loaded.clone();
    bad.vessels[0].fuel += 1.0;
    assert!(bad.encode().is_err());
    let mut bad = loaded.clone();
    bad.vessels[0]
        .blueprint
        .circuit_design
        .as_mut()
        .unwrap()
        .links[0]
        .open = false;
    assert!(bad.encode().is_err());
    let mut current = loaded.clone();
    current.vessels[0]
        .circuit
        .as_mut()
        .unwrap()
        .limit_total(123_457)
        .unwrap();
    current.vessels[0].fuel = 123.457;
    let mut undo = loaded.clone();
    undo.limit_reserve_from(&current).unwrap();
    assert_eq!(
        undo.vessels[0]
            .circuit
            .as_ref()
            .unwrap()
            .validated_network()
            .unwrap()
            .amount(),
        123_457
    );
    assert!(undo.encode().is_ok());
}

#[test]
fn closing_engine_valves_removes_real_thrust_while_connected_lifts_hold_altitude() {
    let mut speeds = Vec::new();
    for isolated in [false, true] {
        let (mut app, entity) = ship(isolated);
        common::run(&mut app, 60 * 12);
        let vessel = app.world().get::<Vessel>(entity).unwrap();
        let network = app.world().get::<AetherCircuit>(entity).unwrap();
        let velocity = app.world().get::<LinearVelocity>(entity).unwrap().0;
        let position = app.world().get::<Position>(entity).unwrap().0;
        let telemetry = app.world().get::<FlightTelemetry>(entity).unwrap();
        assert!(!telemetry.circuit_fault);
        assert!(vessel.fuel > 0.0);
        assert!((vessel.fuel - network.remaining_milli() as f32 / 1000.0).abs() < 0.001);
        assert!((position.y - 200.0).abs() < 1.0, "{position:?}");
        if isolated {
            assert_eq!(telemetry.motor, 0.0);
        }
        speeds.push(-velocity.z);
    }
    assert!(speeds[0] > 5.0 && speeds[1].abs() < 0.5, "{speeds:?}");
}

#[test]
fn leaks_repairs_and_dock_recharge_have_a_conservative_budget_on_the_real_ship() {
    let body = fixtures::explorer();
    let mut supply = AetherCircuit::manifold(&body, 100.0).unwrap();
    let tank = body
        .parts()
        .iter()
        .find(|p| p.kind == PartKind::Tank)
        .unwrap()
        .id;
    supply.set_tank_leak(tank, 137).unwrap();
    let initial = supply.remaining_milli();
    let mut used = 0;
    let mut leaked = 0;
    for i in 0..600_u64 {
        let start = i * 1_000_000_000 / 60;
        let end = (i + 1) * 1_000_000_000 / 60;
        let result = supply
            .burn(&body, Duration::from_nanos(end - start), 100_000.0, 1.0)
            .unwrap();
        used += result.used_milli;
        leaked += result.leaked_milli;
        assert_eq!(initial, supply.remaining_milli() + used + leaked);
    }
    assert_eq!(leaked, 1_370);
    supply.set_tank_leak(tank, 0).unwrap();
    let result = supply
        .burn(&body, Duration::from_secs(1), 100_000.0, 1.0)
        .unwrap();
    assert_eq!(result.leaked_milli, 0);
    let before = supply.remaining_milli();
    assert_eq!(
        supply.dock_recharge(Duration::from_secs(1)).unwrap(),
        25_000
    );
    assert_eq!(supply.remaining_milli(), before + 25_000);
}

#[test]
fn missing_support_ports_or_invalid_durations_never_debit_any_supply() {
    let body = fixtures::explorer();
    let mut supply = AetherCircuit::manifold(&body, 12.0).unwrap();
    let before = supply.capture();
    let mut topology = before.circuit.clone();
    topology.ports[0].capacity += 1;
    assert_eq!(supply.reconfigure(&body, topology), Err(NetworkError::Port));
    assert!(
        supply
            .burn(&body, Duration::from_secs(2), 100_000.0, 1.0)
            .is_err()
    );
    assert!(
        supply
            .burn(&body, Duration::from_millis(20), f32::NAN, 1.0)
            .is_err()
    );
    assert!(
        supply
            .burn(&body, Duration::from_millis(20), 100_000.0, 2.0)
            .is_err()
    );
    assert!(supply.dock_recharge(Duration::from_secs(2)).is_err());
    assert_eq!(before, supply.capture());
}

#[test]
fn exact_elapsed_time_partitions_produce_the_same_device_debits() {
    let body = fixtures::explorer();
    let mut endings = Vec::new();
    for hz in [30_u64, 60, 120] {
        let mut supply = AetherCircuit::manifold(&body, 100.0).unwrap();
        for i in 0..hz * 12 {
            let nanos = (i + 1) * 1_000_000_000 / hz - i * 1_000_000_000 / hz;
            supply
                .burn(&body, Duration::from_nanos(nanos), 100_000.0, 0.7)
                .unwrap();
        }
        endings.push(supply.capture());
    }
    assert_eq!(endings[0], endings[1]);
    assert_eq!(endings[1], endings[2]);
}

#[test]
fn restoring_a_circuit_preserves_fractional_consumption_and_leaks() {
    let body = fixtures::explorer();
    let mut a = AetherCircuit::manifold(&body, 100.0).unwrap();
    let tank = body
        .parts()
        .iter()
        .find(|p| p.kind == PartKind::Tank)
        .unwrap()
        .id;
    a.set_tank_leak(tank, 137).unwrap();
    a.burn(&body, Duration::from_nanos(16_666_667), 100_000.0, 0.7)
        .unwrap();
    let mut b = AetherCircuit::restore(&body, a.capture_state()).unwrap();
    for _ in 0..300 {
        let first = a
            .burn(&body, Duration::from_nanos(16_666_667), 100_000.0, 0.7)
            .unwrap();
        let second = b
            .burn(&body, Duration::from_nanos(16_666_667), 100_000.0, 0.7)
            .unwrap();
        assert_eq!(first.used_milli, second.used_milli);
        assert_eq!(first.leaked_milli, second.leaked_milli);
        assert_eq!(first.fractions, second.fractions);
    }
    assert_eq!(a.capture_state(), b.capture_state());
    let mut bad = a.capture_state();
    bad.meters
        .insert(aether_core::PartId(999), Default::default());
    assert!(AetherCircuit::restore(&body, bad).is_err());
}
