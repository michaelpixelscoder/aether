use aether_core::{PartId, aether_network::*};
use std::collections::BTreeMap;

#[test]
fn blueprint_wiring_undo_retains_spent_fuel_and_removed_storage_returns_empty() {
    use aether_core::{
        PartKind,
        edit::{Edit, History},
        fixtures,
    };
    let mut body = fixtures::explorer();
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
    let mut history = History::default();
    history
        .apply(&mut body, Edit::Circuit(design.clone()))
        .unwrap();
    let mut live = Network::empty(design).unwrap();
    live.fill(root, 1_123_457).unwrap();
    let engine = body
        .parts()
        .iter()
        .find(|p| p.kind == PartKind::Propeller)
        .unwrap()
        .id;
    live.withdraw(&[(engine, 97)]).unwrap();
    let before = live.amount();
    let mut closed = body.circuit_design().unwrap().clone();
    closed
        .links
        .iter_mut()
        .find(|e| e.a == engine || e.b == engine)
        .unwrap()
        .open = false;
    history.apply(&mut body, Edit::Circuit(closed)).unwrap();
    live.reconfigure(body.circuit_design().unwrap().clone())
        .unwrap();
    assert_eq!(live.segment_at(engine).unwrap().0, 0);
    history.undo(&mut body).unwrap();
    live.reconfigure(body.circuit_design().unwrap().clone())
        .unwrap();
    assert_eq!(live.amount(), before);
    assert_eq!(live.segment_at(engine).unwrap().0, before);
    history.apply(&mut body, Edit::RemovePart(root)).unwrap();
    let change = live
        .reconfigure(body.circuit_design().unwrap().clone())
        .unwrap();
    let remaining = live.amount();
    assert!(change.spilled > 0);
    history.undo(&mut body).unwrap();
    live.reconfigure(body.circuit_design().unwrap().clone())
        .unwrap();
    assert_eq!(live.amount(), remaining);
    assert!(remaining < before);
}

#[test]
fn split_filters_actual_ports_preserves_fractional_meters_and_exact_reserve() {
    use aether_core::{Block, Blueprint, Body, Cell, Part, PartKind};
    let parts = vec![
        Part {
            id: PartId(1),
            kind: PartKind::Tank,
            cell: Cell(0, 0, 0),
            quarter_turn: 0,
        },
        Part {
            id: PartId(2),
            kind: PartKind::Tank,
            cell: Cell(5, 0, 0),
            quarter_turn: 0,
        },
    ];
    let design = Circuit {
        ports: ports_for_parts(&parts),
        links: vec![Link {
            a: PartId(1),
            b: PartId(2),
            open: true,
        }],
    };
    let body = Body::from_blueprint(Blueprint {
        name: "Deux réservoirs".into(),
        cells: vec![(Cell(0, 0, 0), Block::Wood), (Cell(5, 0, 0), Block::Wood)],
        parts,
        circuit_design: Some(design.clone()),
    })
    .unwrap();
    let mut network = Network::empty(design).unwrap();
    network.fill(PartId(1), 10_001).unwrap();
    let mut meter = FlowMeter::default();
    meter.debit_nanos(137, 16_666_667).unwrap();
    let state = CircuitState {
        network: network.capture(),
        meters: BTreeMap::from([(PartId(1), meter)]),
        leaks: BTreeMap::from([(PartId(1), 137)]),
        recharge: Default::default(),
    };
    let mut total = 0;
    for fragment in body.split() {
        let fragment = Body::from_blueprint(fragment).unwrap();
        let split = state.for_fragment(&fragment).unwrap();
        total += split.validate_for_body(&fragment).unwrap().amount();
        assert!(split.network.circuit.links.is_empty());
        assert_eq!(
            split.meters.contains_key(&PartId(1)),
            fragment.parts()[0].id == PartId(1)
        );
        let mut limited = split.clone();
        limited.limit_total(2).unwrap();
        assert_eq!(limited.validated_network().unwrap().amount(), 2);
    }
    assert_eq!(total, 10_001);
}

fn circuit() -> Circuit {
    Circuit {
        ports: vec![
            Port {
                id: PartId(1),
                capacity: 100_000,
            },
            Port {
                id: PartId(2),
                capacity: 300_000,
            },
            Port {
                id: PartId(3),
                capacity: 0,
            },
            Port {
                id: PartId(4),
                capacity: 0,
            },
        ],
        links: vec![
            Link {
                a: PartId(1),
                b: PartId(3),
                open: true,
            },
            Link {
                a: PartId(3),
                b: PartId(2),
                open: true,
            },
            Link {
                a: PartId(3),
                b: PartId(4),
                open: true,
            },
        ],
    }
}
fn bytes(network: &Network) -> Vec<u8> {
    serde_json::to_vec(&network.capture()).unwrap()
}

#[test]
fn cutting_a_pipe_and_rejoining_preserves_exact_quantity_and_fill_ratio() {
    let mut network = Network::empty(circuit()).unwrap();
    assert_eq!(network.fill(PartId(1), 123_457).unwrap(), 123_457);
    assert_eq!(network.segment_at(PartId(4)), Some((123_457, 400_000)));
    let before = network.capture();
    let mut separated = circuit();
    separated.links[1].open = false;
    let delta = network.reconfigure(separated).unwrap();
    assert_eq!(
        delta,
        TopologyChange {
            before: 123_457,
            after: 123_457,
            spilled: 0
        }
    );
    assert_eq!(
        network.segment_at(PartId(1)).unwrap().0,
        before.contents[&PartId(1)]
    );
    assert_eq!(
        network.segment_at(PartId(2)).unwrap().0,
        before.contents[&PartId(2)]
    );
    assert_eq!(network.segment_at(PartId(4)).unwrap().1, 100_000);
    network.reconfigure(circuit()).unwrap();
    assert_eq!(network.capture(), before);
}

#[test]
fn removing_storage_spills_only_its_share_without_undo_creating_fuel() {
    let mut network = Network::empty(circuit()).unwrap();
    network.fill(PartId(1), 200_000).unwrap();
    let mut reduced = circuit();
    reduced.ports.retain(|p| p.id != PartId(2));
    reduced
        .links
        .retain(|p| p.a != PartId(2) && p.b != PartId(2));
    assert_eq!(network.reconfigure(reduced).unwrap().spilled, 150_000);
    assert_eq!(network.amount(), 50_000);
    assert_eq!(network.reconfigure(circuit()).unwrap().spilled, 0);
    assert_eq!(network.amount(), 50_000, "Re-added tanks are empty");
}

#[test]
fn starved_consumers_are_fair_and_independent_of_all_insertion_orders() {
    let mut a = Network::empty(circuit()).unwrap();
    let mut reversed = circuit();
    reversed.ports.reverse();
    reversed.links.reverse();
    for edge in &mut reversed.links {
        std::mem::swap(&mut edge.a, &mut edge.b);
    }
    let mut b = Network::empty(reversed).unwrap();
    for network in [&mut a, &mut b] {
        network.fill(PartId(1), 11).unwrap();
    }
    let delivered = a.withdraw(&[(PartId(3), 10), (PartId(4), 20)]).unwrap();
    assert_eq!(
        delivered,
        b.withdraw(&[(PartId(4), 20), (PartId(3), 10)]).unwrap()
    );
    assert_eq!(delivered, BTreeMap::from([(PartId(3), 4), (PartId(4), 7)]));
    assert_eq!(a.amount(), 0);
    assert_eq!(bytes(&a), bytes(&b));
}

#[test]
fn pump_is_directed_bounded_by_both_endpoints_and_never_creates_quantity() {
    let mut separated = circuit();
    separated.links[1].open = false;
    let mut network = Network::empty(separated).unwrap();
    network.fill(PartId(1), 90_000).unwrap();
    network.fill(PartId(2), 299_993).unwrap();
    let total = network.amount();
    assert_eq!(network.pump(PartId(3), PartId(2), 8_000).unwrap(), 7);
    assert_eq!(network.amount(), total);
    assert_eq!(network.pump(PartId(2), PartId(3), 30_000).unwrap(), 10_007);
    assert_eq!(network.amount(), total);
    assert_eq!(network.pump(PartId(1), PartId(4), 1_000).unwrap(), 0);
}

#[test]
fn every_invalid_edit_or_debit_is_atomic_including_late_bad_requests() {
    let mut network = Network::empty(circuit()).unwrap();
    network.fill(PartId(1), 99).unwrap();
    let before = bytes(&network);
    let mut bad = circuit();
    bad.links.push(bad.links[0].clone());
    assert_eq!(network.reconfigure(bad), Err(NetworkError::Link));
    let mut bad = circuit();
    bad.ports[0].capacity = MAX_PORT_CAPACITY + 1;
    assert_eq!(network.reconfigure(bad), Err(NetworkError::Bounds));
    let mut bad = circuit();
    bad.ports[0].id = PartId(0);
    assert_eq!(network.reconfigure(bad), Err(NetworkError::Port));
    assert!(
        network
            .withdraw(&[(PartId(3), 10), (PartId(999), 1)])
            .is_err()
    );
    assert!(network.withdraw(&[(PartId(3), 1), (PartId(3), 2)]).is_err());
    assert!(network.withdraw(&[(PartId(3), MAX_TRANSFER + 1)]).is_err());
    assert!(network.pump(PartId(1), PartId(999), 10).is_err());
    assert!(network.fill(PartId(1), MAX_TRANSFER + 1).is_err());
    assert_eq!(before, bytes(&network));
}

#[test]
fn sub_unit_leaks_conserve_over_arbitrary_time_partitions_and_resume() {
    let mut a = FlowMeter::default();
    let mut total = 0;
    for _ in 0..60 {
        total += a.debit(7, 16_000).unwrap();
    }
    total += a.debit(7, 40_000).unwrap();
    assert_eq!(total, 7);
    let mut b = FlowMeter::default();
    assert_eq!(b.debit(7, 1_000_000).unwrap(), total);
    assert_eq!(a, b);
    a.debit(7, 15_000).unwrap();
    let mut restored: FlowMeter = serde_json::from_slice(&serde_json::to_vec(&a).unwrap()).unwrap();
    assert_eq!(a.debit(7, 985_000), restored.debit(7, 985_000));
    let before = restored;
    assert!(restored.debit(7, 1_000_001).is_err());
    assert_eq!(restored, before);
}

#[test]
fn untrusted_save_rejects_hidden_overfull_or_unknown_storage() {
    let mut save = Network::empty(circuit()).unwrap().capture();
    save.contents.insert(PartId(1), 100_001);
    assert!(matches!(
        Network::restore(save),
        Err(NetworkError::Contents)
    ));
    let mut save = Network::empty(circuit()).unwrap().capture();
    save.contents.insert(PartId(999), 0);
    assert!(matches!(
        Network::restore(save),
        Err(NetworkError::Contents)
    ));
    let encoded = serde_json::to_vec(&Network::empty(circuit()).unwrap().capture()).unwrap();
    let restored = Network::restore(serde_json::from_slice(&encoded).unwrap()).unwrap();
    assert_eq!(bytes(&restored), encoded);
}

#[test]
fn repeated_splits_leaks_pumps_and_refills_have_an_exact_closed_budget() {
    let mut network = Network::empty(circuit()).unwrap();
    let mut received = 0;
    let mut consumed = 0;
    let mut spilled = 0;
    let mut meter = FlowMeter::default();
    for tick in 0..10_000 {
        received += network.fill(PartId(1), (tick * 7919) % 1301).unwrap();
        let mut topology = circuit();
        topology.links[1].open = tick % 11 < 6;
        if tick % 19 == 0 {
            topology.ports[1].capacity = 35_000;
        }
        spilled += network.reconfigure(topology).unwrap().spilled;
        network.pump(PartId(1), PartId(2), tick % 103).unwrap();
        let leak = meter.debit(137, 16_667).unwrap();
        let delivered = network
            .withdraw(&[(PartId(3), leak), (PartId(4), tick % 61)])
            .unwrap();
        consumed += delivered.values().sum::<u64>();
        assert_eq!(
            received,
            consumed + spilled + network.amount(),
            "tick {tick}"
        );
        if tick % 97 == 0 {
            network = Network::restore(serde_json::from_slice(&bytes(&network)).unwrap()).unwrap();
        }
    }
}

#[test]
fn maximum_circuit_is_iterative_and_conservative_with_large_integer_demands() {
    let topology = Circuit {
        ports: (1..=MAX_PORTS as u64)
            .map(|id| Port {
                id: PartId(id),
                capacity: MAX_PORT_CAPACITY,
            })
            .collect(),
        links: (2..=MAX_PORTS as u64)
            .map(|id| Link {
                a: PartId(id - 1),
                b: PartId(id),
                open: true,
            })
            .collect(),
    };
    let mut network = Network::empty(topology).unwrap();
    assert_eq!(network.fill(PartId(1), MAX_TRANSFER).unwrap(), MAX_TRANSFER);
    let demands: Vec<_> = (1..=MAX_PORTS as u64)
        .map(|id| (PartId(id), MAX_TRANSFER))
        .collect();
    let delivered = network.withdraw(&demands).unwrap();
    assert!(delivered.values().all(|n| *n == MAX_PORT_CAPACITY));
    assert_eq!(delivered.values().sum::<u64>(), MAX_TRANSFER);
    assert_eq!(network.amount(), 0);
}
