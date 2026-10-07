#[allow(dead_code)]
mod common;
use aether_core::world;
use aether_sim::{character::*, streaming::*};
use avian3d::prelude::*;
use bevy::prelude::*;

fn walker(app: &mut App, p: Vec3) -> Entity {
    let mut entity = None;
    app.world_mut()
        .resource_scope(|world, mut configs: Mut<Assets<WalkingConfig>>| {
            entity = Some(spawn_walker(&mut world.commands(), &mut configs, p));
        });
    app.world_mut().flush();
    entity.unwrap()
}
#[test]
fn every_capital_dock_has_real_support_and_portal_collision_is_immediate() {
    for island in world::islands().iter().filter(|i| i.capital) {
        let mut app = common::app();
        ensure(app.world_mut(), island.id);
        ensure(app.world_mut(), island.id);
        assert_eq!(
            app.world_mut()
                .query::<&IslandCollision>()
                .iter(app.world())
                .count(),
            1
        );
        let e = walker(&mut app, island.dock() + Vec3::Y);
        common::run(&mut app, 240);
        let p = app.world().get::<Position>(e).unwrap().0;
        assert!(
            (p.y - (island.dock().y - 1.25 + aether_core::tuning::WALKER_FLOAT_HEIGHT)).abs()
                < 0.12,
            "{} unsupported dock: {p:?}",
            island.id
        );
        assert!(
            app.world()
                .get::<WalkerController>(e)
                .unwrap()
                .basis_memory
                .standing_on_entity()
                .is_some()
        );
    }
}
#[test]
fn player_can_climb_the_capital_stairs_with_walking_input() {
    let mut app = common::app();
    let island = world::island(100).unwrap();
    ensure(app.world_mut(), 100);
    let start = island.transform_point(Vec3::new(0.0, 0.0, 7.0))
        + Vec3::Y * (aether_core::tuning::WALKER_FLOAT_HEIGHT + 0.05);
    let e = walker(&mut app, start);
    common::run(&mut app, 90);
    app.world_mut()
        .get_mut::<WalkerIntent>(e)
        .unwrap()
        .direction = Vec3::NEG_Z * 3.5;
    for _ in 0..60 * 35 {
        app.update();
        if app.world().get::<Position>(e).unwrap().z < island.center.z - 11.0 * island.scale {
            break;
        }
    }
    app.world_mut()
        .get_mut::<WalkerIntent>(e)
        .unwrap()
        .direction = Vec3::ZERO;
    common::run(&mut app, 120);
    let p = app.world().get::<Position>(e).unwrap().0;
    assert!(
        p.z < island.center.z - 10.0 * island.scale,
        "blocked staircase: {p:?}"
    );
    assert!(
        (p.y - (island.center.y + 7.0 * island.scale + aether_core::tuning::WALKER_FLOAT_HEIGHT))
            .abs()
            < 0.18,
        "wrong landing: {p:?}"
    );
}
#[test]
fn streaming_unloads_distant_collision_and_keeps_stable_port_ids() {
    let mut app = common::app();
    app.add_systems(Update, update);
    let e = walker(&mut app, world::dock(100).unwrap() + Vec3::Y);
    common::run(&mut app, 100);
    assert!(
        app.world_mut()
            .query::<&IslandCollision>()
            .iter(app.world())
            .any(|i| i.0 == 100)
    );
    let p = world::dock(600).unwrap() + Vec3::Y;
    app.world_mut().entity_mut(e).insert((
        aether_sim::teleport_pose(p, Quat::IDENTITY),
        LinearVelocity::ZERO,
    ));
    ensure(app.world_mut(), 600);
    common::run(&mut app, 100);
    let ids: Vec<_> = app
        .world_mut()
        .query::<&IslandCollision>()
        .iter(app.world())
        .map(|i| i.0)
        .collect();
    assert!(ids.contains(&600));
    assert!(!ids.contains(&100));
    let distinct: std::collections::BTreeSet<_> = ids.iter().collect();
    assert_eq!(distinct.len(), ids.len());
    assert!(ids.len() < world::islands().len() / 2);
}
#[test]
fn residents_follow_the_saved_clock_and_pause_with_the_simulation() {
    let mut app = common::app();
    app.world_mut().resource_mut::<aether_sim::SimClock>().tick = 54_000;
    aether_sim::fauna::spawn(&mut app.world_mut().commands(), 900.0);
    app.world_mut().flush();
    common::run(&mut app, 120);
    let tick = app.world().resource::<aether_sim::SimClock>().tick;
    let residents: Vec<_> = app
        .world_mut()
        .query::<(Entity, &aether_sim::fauna::Resident, &Position)>()
        .iter(app.world())
        .map(|(e, r, p)| (e, r.0, p.0))
        .collect();
    assert_eq!(residents.len(), 6);
    for (_, r, p) in &residents {
        let expected = r.pose(tick as f32 / 60.0).0 - aether_sim::fauna::visual_offset(*r);
        assert!(
            p.distance(expected) < 0.06,
            "resident {}: {p:?} != {expected:?}",
            r.id
        );
    }
    app.world_mut().resource_mut::<Time<Virtual>>().pause();
    common::run(&mut app, 120);
    assert_eq!(app.world().resource::<aether_sim::SimClock>().tick, tick);
    for (e, _, p) in residents {
        assert_eq!(app.world().get::<Position>(e).unwrap().0, p);
    }
}

#[test]
fn courier_traffic_resumes_its_route_and_stops_during_pause() {
    let mut app = common::app();
    app.world_mut().resource_mut::<aether_sim::SimClock>().tick = 54_000;
    aether_sim::traffic::spawn(&mut app.world_mut().commands(), 900.0);
    app.world_mut().flush();
    common::run(&mut app, 120);
    let tick = app.world().resource::<aether_sim::SimClock>().tick;
    let couriers: Vec<_> = app
        .world_mut()
        .query::<(Entity, &aether_sim::traffic::Courier, &Position)>()
        .iter(app.world())
        .map(|(e, r, p)| (e, r.0, p.0))
        .collect();
    assert_eq!(couriers.len(), aether_core::traffic::COURIERS.len());
    for (_, r, p) in &couriers {
        let expected = r.pose(tick as f32 / 60.0).0;
        assert!(
            p.distance(expected) < 0.06,
            "courier {}: {p:?} != {expected:?}",
            r.id
        );
    }
    app.world_mut().resource_mut::<Time<Virtual>>().pause();
    common::run(&mut app, 120);
    assert_eq!(app.world().resource::<aether_sim::SimClock>().tick, tick);
    for (e, _, p) in couriers {
        assert_eq!(app.world().get::<Position>(e).unwrap().0, p);
    }
}
