mod common;
const STANDING_HEIGHT: f32 = aether_core::tuning::WALKER_FLOAT_HEIGHT + 0.25;
use aether_core::{save::*, *};
use aether_sim::{character::*, *};
use avian3d::prelude::*;
use bevy::prelude::*;
use common::{app, run, vessel};

#[test]
fn moving_voxel_vessel_does_not_cross_a_thin_wall() {
    let mut a = app();
    let e = vessel(&mut a, false);
    a.world_mut().spawn((
        RigidBody::Static,
        Collider::cuboid(0.1, 30.0, 30.0),
        Transform::from_xyz(8.0, 15.0, 0.0),
    ));
    a.world_mut()
        .entity_mut(e)
        .insert(LinearVelocity(Vec3::X * 40.0));
    a.world_mut().get_mut::<PilotIntent>(e).unwrap().brake = true;
    run(&mut a, 300);
    let p = a.world().get::<Position>(e).unwrap().0;
    assert!(p.is_finite() && p.x < 7.0, "{p:?}");
}
#[test]
fn voxel_vessel_lands_on_another_voxel_vessel() {
    let mut a = app();
    vessel(&mut a, true);
    let e = vessel(&mut a, false);
    a.world_mut()
        .entity_mut(e)
        .insert(Transform::from_xyz(0.0, 20.0, 0.0));
    a.world_mut().get_mut::<Vessel>(e).unwrap().fuel = 0.0;
    run(&mut a, 600);
    let p = a.world().get::<Position>(e).unwrap().0;
    assert!(p.is_finite() && p.y > 11.0, "{p:?}");
}

#[test]
fn child_voxel_collider_matches_domain_mass_and_surface() {
    let mut a = app();
    let e = vessel(&mut a, true);
    run(&mut a, 2);
    let body = a.world().get::<Vessel>(e).unwrap();
    let expected = body.properties;
    assert_eq!(a.world().get::<Mass>(e).unwrap().0, expected.mass);
    let ball = a
        .world_mut()
        .spawn((
            RigidBody::Dynamic,
            Collider::sphere(0.2),
            Transform::from_xyz(0.5, 15.0, 1.0),
        ))
        .id();
    run(&mut a, 600);
    let p = a.world().get::<Position>(ball).unwrap().0;
    assert!((p.y - 12.45).abs() < 0.03, "{p:?}");
}
#[test]
fn heavy_construction_cannot_hover_with_two_lifts() {
    let mut a = app();
    let e = vessel(&mut a, false);
    let mut blueprint = a.world().get::<Vessel>(e).unwrap().body.blueprint();
    blueprint.cells.retain(|(cell, _)| {
        !(cell.0.abs() <= 5 && cell.2.abs() <= 5 && (-5..=-2).contains(&cell.1))
    });
    for z in -5..=5 {
        for x in -5..=5 {
            for y in -5..=-2 {
                blueprint.cells.push((Cell(x, y, z), Block::Metal));
            }
        }
    }
    let body = Body::from_blueprint(blueprint).unwrap();
    let mut v = a.world_mut().get_mut::<Vessel>(e).unwrap();
    v.body = body;
    v.properties = v.body.mass_properties();
    let copy = v.clone();
    replace_geometry(&mut a.world_mut().commands(), e, &copy);
    a.world_mut().flush();
    run(&mut a, 120);
    assert!(a.world().get::<FlightTelemetry>(e).unwrap().lift_ratio < 0.2);
    assert!(a.world().get::<Position>(e).unwrap().y < 0.0);
}
#[test]
fn construction_mass_increases_consumption() {
    fn consumption(heavy: bool) -> f32 {
        let mut a = app();
        let e = vessel(&mut a, false);
        if heavy {
            let mut v = a.world_mut().get_mut::<Vessel>(e).unwrap();
            let mut b = v.body.blueprint();
            for (c, m) in &mut b.cells {
                if c.1 < 0 {
                    *m = Block::Metal;
                }
            }
            v.body.replace(b).unwrap();
            v.properties = v.body.mass_properties();
            let copy = v.clone();
            replace_geometry(&mut a.world_mut().commands(), e, &copy);
            a.world_mut().flush();
        }
        run(&mut a, 600);
        180.0 - a.world().get::<Vessel>(e).unwrap().fuel
    }
    let light = consumption(false);
    let heavy = consumption(true);
    println!("Aether consumed in 10 s: cedar hull {light:.6}, metal lower hull {heavy:.6}");
    assert!(heavy > light * 1.1, "light={light}, heavy={heavy}");
}
#[test]
fn player_walk_jump_and_landing_on_a_static_deck() {
    let mut a = app();
    a.world_mut().spawn((
        RigidBody::Static,
        Collider::cuboid(40.0, 0.5, 40.0),
        Transform::default(),
    ));
    let mut e = None;
    a.world_mut()
        .resource_scope(|w, mut config: Mut<Assets<WalkingConfig>>| {
            e = Some(spawn_walker(
                &mut w.commands(),
                &mut config,
                Vec3::new(0.0, STANDING_HEIGHT, 0.0),
            ));
        });
    a.world_mut().flush();
    let e = e.unwrap();
    run(&mut a, 120);
    a.world_mut().get_mut::<WalkerIntent>(e).unwrap().direction = Vec3::X * 4.0;
    run(&mut a, 60);
    let before = a.world().get::<Position>(e).unwrap().0;
    assert!(before.x > 2.0 && before.x < 4.2);
    {
        let mut i = a.world_mut().get_mut::<WalkerIntent>(e).unwrap();
        i.direction = Vec3::ZERO;
        i.jump = true;
    }
    run(&mut a, 10);
    a.world_mut().get_mut::<WalkerIntent>(e).unwrap().jump = false;
    run(&mut a, 12);
    assert!(a.world().get::<Position>(e).unwrap().y > before.y + 0.6);
    run(&mut a, 180);
    let p = a.world().get::<Position>(e).unwrap().0;
    assert!((p.y - STANDING_HEIGHT).abs() < 0.05, "{p:?}");
}
#[test]
fn sphere_ccd_stops_at_a_thin_wall_at_100_meters_per_second() {
    let mut a = app();
    a.world_mut().spawn((
        RigidBody::Static,
        Collider::cuboid(0.1, 20.0, 20.0),
        Transform::default(),
    ));
    let e = a
        .world_mut()
        .spawn((
            RigidBody::Dynamic,
            Collider::sphere(0.15),
            SweptCcd::LINEAR,
            GravityScale(0.0),
            Transform::from_xyz(-5.0, 0.0, 0.0),
            LinearVelocity(Vec3::X * 100.0),
        ))
        .id();
    run(&mut a, 120);
    let p = a.world().get::<Position>(e).unwrap().0;
    assert!(p.x < 0.1 && p.is_finite(), "{p:?}");
}
#[test]
fn tether_is_bounded_for_ten_simulated_minutes() {
    let mut a = app();
    let anchor = a
        .world_mut()
        .spawn((RigidBody::Static, Transform::from_xyz(0.0, 30.0, 0.0)))
        .id();
    let body = a
        .world_mut()
        .spawn((
            RigidBody::Dynamic,
            Collider::sphere(0.5),
            Mass(10000.0),
            Transform::from_xyz(20.0, 30.0, 0.0),
            LinearVelocity(Vec3::Z * 25.0),
        ))
        .id();
    a.world_mut()
        .spawn(DistanceJoint::new(body, anchor).with_limits(0.0, 20.0));
    let mut errors = Vec::new();
    for step in 0..36_000 {
        a.update();
        let p = a.world().get::<Position>(body).unwrap().0;
        assert!(p.is_finite());
        if step > 120 {
            errors.push(((p - Vec3::Y * 30.0).length() - 20.0).max(0.0) / 20.0);
        }
    }
    errors.sort_by(f32::total_cmp);
    let p95 = errors[errors.len() * 95 / 100];
    assert!(p95 < 0.02, "relative p95 rope stretch={p95}");
}
#[test]
fn thirty_minutes_of_navigation_recovery_and_serialization_remain_finite() {
    let mut a = app();
    let e = vessel(&mut a, false);
    let mut saves = 0;
    for frame in 0..108_000 {
        if frame % 1800 == 0 {
            a.world_mut().entity_mut(e).insert((
                Position(Vec3::new(0.0, 12.0, 0.0)),
                Rotation(Quat::IDENTITY),
                Transform::from_xyz(0.0, 12.0, 0.0),
                LinearVelocity::ZERO,
                AngularVelocity::ZERO,
            ));
            a.world_mut().get_mut::<Vessel>(e).unwrap().fuel = 180.0;
        }
        {
            let mut i = a.world_mut().get_mut::<PilotIntent>(e).unwrap();
            i.throttle = 0.8;
            i.turn = ((frame as f32 / 600.0).sin()) * 0.4;
            i.brake = frame % 1800 > 1500;
        }
        a.update();
        if frame % 600 == 0 {
            let v = a.world().get::<Vessel>(e).unwrap();
            let p = a.world().get::<Position>(e).unwrap().0;
            let r = a.world().get::<Rotation>(e).unwrap().0;
            let velocity = a.world().get::<LinearVelocity>(e).unwrap().0;
            let w = a.world().get::<AngularVelocity>(e).unwrap().0;
            let snapshot = Session {
                expedition: Default::default(),
                version: VERSION,
                seed: 7391,
                tick: frame,
                active: v.id,
                vessels: vec![capture(v, p, r, velocity, w)],
                checkpoint: 0,
                progress: 1,
                tether: None,
                walker: None,
            };
            let bytes = snapshot.encode().unwrap();
            let loaded = Session::decode(&bytes).unwrap();
            assert!(loaded.vessels[0].position.is_finite());
            saves += 1;
        }
    }
    assert_eq!(saves, 180);
}
#[test]
fn current_and_sail_reach_the_gardens_without_pose_teleportation() {
    let mut a = app();
    let e = vessel(&mut a, false);
    let target = ISLANDS[1].dock;
    spawn_world(&mut a.world_mut().commands());
    a.world_mut().flush();
    // Recorded simple helmsman: open sail, join the current, climb to destination,
    // then brake. Only public control intentions are used during the crossing.
    let mut closest = f32::MAX;
    let mut arrived = false;
    let waypoints = [
        Vec3::new(5.0, 14.0, -65.0),
        Vec3::new(28.0, 18.0, -115.0),
        target,
    ];
    let mut waypoint = 0;
    for step in 0..18000 {
        let p = a
            .world()
            .get::<Position>(e)
            .map_or(Vec3::new(0.0, 12.0, 0.0), |p| p.0);
        let speed = a
            .world()
            .get::<LinearVelocity>(e)
            .map_or(0.0, |v| v.0.length());
        let distance = p.distance(target);
        closest = closest.min(distance);
        if distance < 9.0 && speed < 3.5 {
            arrived = true;
            break;
        }
        if waypoint < 2 && (p - waypoints[waypoint]).with_y(0.0).length() < 14.0 {
            waypoint += 1;
        }
        let direction = (waypoints[waypoint] - p).with_y(0.0).normalize_or_zero();
        let desired_yaw = (-direction.x).atan2(-direction.z);
        let yaw = a
            .world()
            .get::<Rotation>(e)
            .map_or(0.0, |r| r.0.to_euler(EulerRot::YXZ).0);
        let angle = (desired_yaw - yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        let angular = a.world().get::<AngularVelocity>(e).map_or(0.0, |v| v.y);
        if step % 600 == 0 && std::env::var_os("AETHER_TRACE").is_some() {
            eprintln!(
                "t={} p={p:?} speed={speed:.2} yaw={yaw:.2} desired={desired_yaw:.2} waypoint={waypoint} distance={distance:.2}",
                step / 60
            );
        }
        let altitude = a.world().get::<Vessel>(e).unwrap().target_altitude;
        let mut intent = a.world_mut().get_mut::<PilotIntent>(e).unwrap();
        intent.turn = (angle * 2.0 - angular * 1.5).clamp(-1.0, 1.0);
        intent.throttle = if distance < 35.0 { 0.0 } else { 0.8 };
        intent.brake = distance < 35.0;
        intent.climb = (waypoints[waypoint].y - altitude).clamp(-1.0, 1.0);
        a.update();
    }
    assert!(arrived, "closest distance to destination={closest}");
    assert!(a.world().get::<Vessel>(e).unwrap().fuel > 0.0);
}

#[test]
fn harpoon_swing_reaches_the_refuge_branch_and_releases_continuously() {
    for reaction_ticks in [0, 30, 90] {
        let mut a = app();
        let e = vessel(&mut a, false);
        spawn_world(&mut a.world_mut().commands());
        a.world_mut().flush();
        let target = ISLANDS[2].dock;
        let anchor = a
            .world_mut()
            .query::<(Entity, &Anchor)>()
            .iter(a.world())
            .find(|(_, a)| a.0 == 2)
            .unwrap()
            .0;
        let local = a
            .world()
            .get::<Vessel>(e)
            .unwrap()
            .body
            .parts()
            .iter()
            .find(|p| p.kind == PartKind::Harpoon)
            .unwrap()
            .center();
        let mut cable = None;
        let mut attached = false;
        let mut released = false;
        let mut closest = f32::MAX;
        let mut arrived = false;
        let mut opportunity = None;
        for step in 0..18_000 {
            let p = a
                .world()
                .get::<Position>(e)
                .map_or(ISLANDS[0].dock, |p| p.0);
            let rotation = a.world().get::<Rotation>(e).map_or(Quat::IDENTITY, |r| r.0);
            let velocity = a
                .world()
                .get::<LinearVelocity>(e)
                .map_or(Vec3::ZERO, |v| v.0);
            if !attached && p.z < -42.0 {
                let first = *opportunity.get_or_insert(step);
                if step - first >= reaction_ticks {
                    let length = (p + rotation * local).distance(ANCHORS[2]);
                    assert!(
                        length <= 70.0
                            && aether_core::terrain::line_clear(p + rotation * local, ANCHORS[2])
                    );
                    cable = Some(attach_tether(
                        &mut a.world_mut().commands(),
                        e,
                        anchor,
                        2,
                        local,
                        length,
                    ));
                    a.world_mut().flush();
                    attached = true;
                }
            }
            if !released && attached && p.x < -8.0 && p.z < -70.0 {
                a.world_mut().despawn(cable.take().unwrap());
                assert_eq!(velocity, a.world().get::<LinearVelocity>(e).unwrap().0);
                released = true;
            }
            let distance = p.distance(target);
            closest = closest.min(distance);
            if distance < 9.0 && velocity.length() < 3.5 {
                arrived = true;
                break;
            }
            let direction = (target - p).with_y(0.0).normalize_or_zero();
            let yaw = rotation.to_euler(EulerRot::YXZ).0;
            let angle = ((-direction.x).atan2(-direction.z) - yaw + std::f32::consts::PI)
                .rem_euclid(std::f32::consts::TAU)
                - std::f32::consts::PI;
            let angular = a.world().get::<AngularVelocity>(e).map_or(0.0, |w| w.y);
            let altitude = a.world().get::<Vessel>(e).unwrap().target_altitude;
            let mut i = a.world_mut().get_mut::<PilotIntent>(e).unwrap();
            i.turn = (angle * 2.0 - angular * 1.5).clamp(-1.0, 1.0);
            i.throttle = if released { 0.0 } else { 0.8 };
            i.brake = released && distance < 15.0;
            i.climb = ((if released { 8.0 } else { 12.0 }) - altitude).clamp(-1.0, 1.0);
            a.update();
            if step % 600 == 0 && std::env::var_os("AETHER_TRACE").is_some() {
                eprintln!(
                    "refuge t={} p={p:?} v={velocity:?} attached={attached} released={released} distance={distance}",
                    step / 60
                );
            }
        }
        assert!(
            attached && released && arrived,
            "closest={closest}, attached={attached}, released={released}"
        );
        println!(
            "refuge reaction delay {} s: arrived, closest={closest:.3} m, reserve={:.3}",
            reaction_ticks as f32 / 60.0,
            a.world().get::<Vessel>(e).unwrap().fuel
        );
    }
}

fn walker(a: &mut App, position: Vec3) -> Entity {
    let mut e = None;
    a.world_mut()
        .resource_scope(|w, mut config: Mut<Assets<WalkingConfig>>| {
            e = Some(spawn_walker(&mut w.commands(), &mut config, position));
        });
    a.world_mut().flush();
    e.unwrap()
}
#[test]
fn jumping_inherits_translation_of_the_support() {
    let mut a = app();
    let deck = a
        .world_mut()
        .spawn((
            RigidBody::Kinematic,
            Collider::cuboid(30.0, 0.5, 30.0),
            Transform::default(),
            LinearVelocity::ZERO,
        ))
        .id();
    let player = walker(&mut a, Vec3::Y * STANDING_HEIGHT);
    run(&mut a, 120);
    a.world_mut()
        .entity_mut(deck)
        .insert(LinearVelocity(Vec3::X * 5.0));
    run(&mut a, 120);
    let before = a.world().get::<Position>(player).unwrap().0;
    // Keep the jump held through the apex: Tnua intentionally shortens a released jump.
    a.world_mut().get_mut::<WalkerIntent>(player).unwrap().jump = true;
    run(&mut a, 30);
    let airborne = a.world().get::<Position>(player).unwrap().0;
    assert!(airborne.y > before.y + 0.5);
    assert!(
        (airborne.x - before.x - 2.5).abs() < 0.5,
        "{before:?} -> {airborne:?}"
    );
    a.world_mut().get_mut::<WalkerIntent>(player).unwrap().jump = false;
    run(&mut a, 180);
    assert!((a.world().get::<Position>(player).unwrap().y - STANDING_HEIGHT).abs() < 0.05);
}
#[test]
fn controller_stays_bounded_under_a_low_ceiling() {
    let mut a = app();
    a.world_mut().spawn((
        RigidBody::Static,
        Collider::cuboid(20.0, 0.5, 20.0),
        Transform::default(),
    ));
    a.world_mut().spawn((
        RigidBody::Static,
        Collider::cuboid(20.0, 0.2, 20.0),
        Transform::from_xyz(0.0, 2.4, 0.0),
    ));
    let e = walker(&mut a, Vec3::Y * STANDING_HEIGHT);
    run(&mut a, 120);
    a.world_mut().get_mut::<WalkerIntent>(e).unwrap().jump = true;
    for _ in 0..60 {
        a.update();
        let p = a.world().get::<Position>(e).unwrap().0;
        assert!(p.is_finite() && p.y < 1.85, "{p:?}");
    }
    a.world_mut().get_mut::<WalkerIntent>(e).unwrap().jump = false;
    run(&mut a, 180);
    assert!((a.world().get::<Position>(e).unwrap().y - STANDING_HEIGHT).abs() < 0.05);
}
#[test]
fn controller_walks_up_small_steps() {
    let mut a = app();
    a.world_mut().spawn((
        RigidBody::Static,
        Collider::cuboid(40.0, 0.5, 40.0),
        Transform::default(),
    ));
    for (x, width, top) in [(1.5, 1.0, 0.45), (2.5, 1.0, 0.65), (6.0, 6.0, 0.85)] {
        a.world_mut().spawn((
            RigidBody::Static,
            Collider::cuboid(width, top, 4.0),
            Transform::from_xyz(x, top * 0.5, 0.0),
        ));
    }
    let e = walker(&mut a, Vec3::Y * STANDING_HEIGHT);
    run(&mut a, 120);
    a.world_mut().get_mut::<WalkerIntent>(e).unwrap().direction = Vec3::X * 2.0;
    run(&mut a, 210);
    let p = a.world().get::<Position>(e).unwrap().0;
    assert!(
        p.x > 5.0 && (p.y - (0.85 + aether_core::tuning::WALKER_FLOAT_HEIGHT)).abs() < 0.10,
        "{p:?}"
    );
}
#[test]
fn character_mass_does_not_capsize_a_floating_vessel() {
    let mut a = app();
    let e = vessel(&mut a, false);
    let player = walker(&mut a, Vec3::new(0.0, 13.1, 2.2));
    run(&mut a, 120);
    a.world_mut()
        .get_mut::<WalkerIntent>(player)
        .unwrap()
        .direction = Vec3::NEG_Z;
    run(&mut a, 90);
    a.world_mut()
        .get_mut::<WalkerIntent>(player)
        .unwrap()
        .direction = Vec3::ZERO;
    run(&mut a, 300);
    let p = a.world().get::<Position>(e).unwrap().0;
    let up = a.world().get::<Rotation>(e).unwrap().0 * Vec3::Y;
    assert!(
        p.is_finite() && p.y > 10.0 && up.dot(Vec3::Y) > 0.9,
        "position={p:?}, up={up:?}"
    );
}

#[test]
fn restored_airborne_character_keeps_horizontal_momentum() {
    let mut a = app();
    let player = walker(&mut a, Vec3::Y * 20.0);
    let velocity = Vec3::new(5.0, 2.0, -3.0);
    a.world_mut().entity_mut(player).insert((
        LinearVelocity(velocity),
        aether_sim::character::InheritedMotion::from_velocity(velocity),
    ));
    run(&mut a, 30);
    let after = a.world().get::<LinearVelocity>(player).unwrap().0;
    assert!((after.x - velocity.x).abs() < 0.01 && (after.z - velocity.z).abs() < 0.01);
}
