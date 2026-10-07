mod common;
const STANDING_HEIGHT: f32 = aether_core::tuning::WALKER_FLOAT_HEIGHT + 0.25;
use aether_core::{Block, fixtures};
use aether_sim::{character::*, *};
use avian3d::prelude::*;
use bevy::{prelude::*, time::TimeUpdateStrategy};
use common::{app, run};
use std::time::Duration;

fn walker(a: &mut App, position: Vec3) -> Entity {
    let mut result = None;
    a.world_mut()
        .resource_scope(|w, mut config: Mut<Assets<WalkingConfig>>| {
            result = Some(spawn_walker(&mut w.commands(), &mut config, position));
        });
    a.world_mut().flush();
    result.unwrap()
}

#[test]
fn repeated_walking_does_not_leave_sensor_entities() {
    let mut a = app();
    run(&mut a, 4);
    for _ in 0..20 {
        let entity = walker(&mut a, Vec3::new(100.0, 12.0, 50.0));
        run(&mut a, 4);
        let alive = a
            .world_mut()
            .query_filtered::<Entity, With<bevy_tnua::TnuaProximitySensor>>()
            .iter(a.world())
            .count();
        assert!(
            alive > 0,
            "the production controller must create its sensors"
        );
        a.world_mut().despawn(entity);
        run(&mut a, 4);
        let left = a
            .world_mut()
            .query_filtered::<Entity, With<bevy_tnua::TnuaProximitySensor>>()
            .iter(a.world())
            .count();
        assert_eq!(left, 0, "despawned walker left sensor entities behind");
    }
}

#[test]
fn slope_support_is_stable_at_30_60_120_render_fps() {
    for fps in [30, 60, 120] {
        for degrees in [15.0_f32, 30.0, 39.0] {
            let mut a = app();
            a.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
                1.0 / fps as f64,
            )));
            let rotation = Quat::from_rotation_z(degrees.to_radians());
            a.world_mut().spawn((
                RigidBody::Kinematic,
                Collider::cuboid(30.0, 0.5, 30.0),
                Transform::from_rotation(rotation),
            ));
            let player = walker(&mut a, Vec3::Y * 1.3);
            run(&mut a, fps * 3);
            let before = a.world().get::<Position>(player).unwrap().0;
            run(&mut a, fps * 10);
            let after = a.world().get::<Position>(player).unwrap().0;
            assert!(
                after.is_finite() && after.distance(before) < 0.15,
                "slope={degrees} fps={fps}: {before:?} -> {after:?}"
            );
            assert!(
                !a.world()
                    .get::<WalkerController>(player)
                    .unwrap()
                    .is_airborne()
                    .unwrap_or(true)
            );
        }
    }
}

#[test]
fn voxel_internal_edges_do_not_stop_a_walking_character() {
    let mut a = app();
    let deck = fixtures::solid([40, 1, 8], Block::Wood);
    a.world_mut()
        .spawn((RigidBody::Static, collider(&deck), collider_offset()));
    let player = walker(&mut a, Vec3::new(1.0, STANDING_HEIGHT, 1.0));
    run(&mut a, 120);
    a.world_mut()
        .get_mut::<WalkerIntent>(player)
        .unwrap()
        .direction = Vec3::X;
    let mut max_vertical_error = 0.0_f32;
    for _ in 0..600 {
        a.update();
        let p = a.world().get::<Position>(player).unwrap().0;
        max_vertical_error = max_vertical_error.max((p.y - STANDING_HEIGHT).abs());
    }
    let p = a.world().get::<Position>(player).unwrap().0;
    assert!(
        p.x > 9.0 && max_vertical_error < 0.13,
        "{p:?}, vertical={max_vertical_error}"
    );
}

#[test]
fn sleeping_contact_wakes_and_teleported_body_rebuilds_contacts() {
    let mut a = app();
    a.world_mut().spawn((
        RigidBody::Static,
        Collider::cuboid(20.0, 1.0, 20.0),
        Transform::from_xyz(0.0, -0.5, 0.0),
    ));
    let ball = a
        .world_mut()
        .spawn((
            RigidBody::Dynamic,
            Collider::sphere(0.5),
            Transform::from_xyz(0.0, 2.0, 0.0),
        ))
        .id();
    run(&mut a, 600);
    assert!(
        a.world().get::<Sleeping>(ball).is_some(),
        "resting body should sleep"
    );
    a.world_mut()
        .entity_mut(ball)
        .insert(LinearVelocity(Vec3::X * 2.0));
    run(&mut a, 10);
    assert!(a.world().get::<Sleeping>(ball).is_none());
    assert!(a.world().get::<Position>(ball).unwrap().x > 0.1);
    a.world_mut().entity_mut(ball).insert((
        Position(Vec3::new(3.0, 3.0, 0.0)),
        Transform::from_xyz(3.0, 3.0, 0.0),
        LinearVelocity::ZERO,
        AngularVelocity::ZERO,
    ));
    run(&mut a, 240);
    let p = a.world().get::<Position>(ball).unwrap().0;
    assert!(
        (p.x - 3.0).abs() < 0.03 && (p.y - 0.5).abs() < 0.03,
        "{p:?}"
    );
}

#[test]
fn tether_mass_speed_length_fps_and_pause_matrix_is_bounded() {
    let mut worst = 0.0_f32;
    for mass in [1000.0, 10000.0, 900000.0] {
        for speed in [0.0, 25.0, 80.0] {
            for length in [3.0, 80.0] {
                for fps in [30, 60, 120] {
                    let mut a = app();
                    a.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
                        1.0 / fps as f64,
                    )));
                    let anchor = a
                        .world_mut()
                        .spawn((RigidBody::Static, Transform::from_xyz(0.0, 100.0, 0.0)))
                        .id();
                    let body = a
                        .world_mut()
                        .spawn((
                            RigidBody::Dynamic,
                            Collider::sphere(0.2),
                            Mass(mass),
                            Transform::from_xyz(length, 100.0, 0.0),
                            LinearVelocity(Vec3::Z * speed),
                        ))
                        .id();
                    a.world_mut()
                        .spawn(DistanceJoint::new(body, anchor).with_limits(0.0, length));
                    let mut errors = Vec::new();
                    for frame in 0..fps * 4 {
                        a.update();
                        let p = a.world().get::<Position>(body).unwrap().0;
                        assert!(p.is_finite());
                        if frame > fps {
                            errors
                                .push(((p - Vec3::Y * 100.0).length() - length).max(0.0) / length);
                        }
                    }
                    a.world_mut().resource_mut::<Time<Virtual>>().pause();
                    // One application update drains the already prepared virtual delta.
                    a.update();
                    let paused = a.world().get::<Position>(body).unwrap().0;
                    run(&mut a, 60);
                    assert_eq!(a.world().get::<Position>(body).unwrap().0, paused);
                    a.world_mut().resource_mut::<Time<Virtual>>().unpause();
                    run(&mut a, fps);
                    let p = a.world().get::<Position>(body).unwrap().0;
                    errors.push(((p - Vec3::Y * 100.0).length() - length).max(0.0) / length);
                    errors.sort_by(f32::total_cmp);
                    let p95 = errors[errors.len() * 95 / 100];
                    worst = worst.max(p95);
                    assert!(
                        p95 < 0.02,
                        "mass={mass}, speed={speed}, length={length}, fps={fps}, stretch_p95={p95}"
                    );
                }
            }
        }
    }
    println!(
        "54 cable cases: mass 1k/10k/900k kg, speed 0/25/80 m/s, length 3/80 m, render 30/60/120 Hz; worst relative p95={worst}"
    );
}

#[test]
fn loaded_vessel_with_sail_current_tether_and_low_reserve_remains_finite() {
    let mut a = app();
    let body = common::vessel(&mut a, false);
    let mut blueprint = fixtures::starter().blueprint();
    for x in 4..9 {
        for z in 0..4 {
            blueprint
                .cells
                .push((aether_core::Cell(x, 0, z), Block::Metal));
        }
    }
    let loaded = aether_core::Body::from_blueprint(blueprint).unwrap();
    // Reuse production spawning so all mass, inertia and collider updates are real.
    a.world_mut().despawn(body);
    let saved = aether_core::save::VesselSave {
        circuit: None,
        id: aether_core::BodyId(1),
        blueprint: loaded.blueprint(),
        position: Vec3::new(5.0, 14.0, -65.0),
        rotation: Quat::IDENTITY,
        velocity: Vec3::Z * -14.0,
        angular_velocity: Vec3::ZERO,
        fuel: 0.8,
        trim: 0.4,
        target_altitude: Some(14.0),
        docked: false,
    };
    let body = spawn_vessel(&mut a.world_mut().commands(), saved);
    let anchor = a
        .world_mut()
        .spawn((RigidBody::Static, Transform::from_xyz(0.0, 26.0, -65.0)))
        .id();
    let local = loaded
        .parts()
        .iter()
        .find(|p| p.kind == aether_core::PartKind::Harpoon)
        .unwrap()
        .center();
    let rope = attach_tether(&mut a.world_mut().commands(), body, anchor, 0, local, 18.0);
    a.world_mut().flush();
    run(&mut a, 600);
    assert_eq!(a.world().get::<Vessel>(body).unwrap().fuel, 0.0);
    let p = a.world().get::<Position>(body).unwrap().0;
    let r = a.world().get::<Rotation>(body).unwrap().0;
    assert!(p.is_finite() && (p + r * local - Vec3::new(0.0, 26.0, -65.0)).length() < 18.4);
    let before = a.world().get::<LinearVelocity>(body).unwrap().0;
    a.world_mut().despawn(rope);
    assert_eq!(a.world().get::<LinearVelocity>(body).unwrap().0, before);
    run(&mut a, 60);
    assert!(a.world().get::<Position>(body).unwrap().0.is_finite());
}

#[test]
fn interpolated_landmark_lags_by_at_most_one_tick_and_teleport_resets_it() {
    for fps in [30, 60, 120] {
        let mut a = app();
        a.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1.0 / fps as f64,
        )));
        let vessel = common::vessel(&mut a, false);
        a.world_mut()
            .get_mut::<PilotIntent>(vessel)
            .unwrap()
            .throttle = 0.8;
        let local = Vec3::new(1.0, 1.8, -2.0);
        let mut maximum_error = 0.0_f32;
        for _ in 0..fps * 5 {
            a.update();
            let w = a.world();
            let p = w.get::<Position>(vessel).unwrap().0;
            let r = w.get::<Rotation>(vessel).unwrap().0;
            let t = w.get::<Transform>(vessel).unwrap();
            let v = w.get::<LinearVelocity>(vessel).unwrap().0.length();
            let angular = w.get::<AngularVelocity>(vessel).unwrap().0.length();
            let error = t.transform_point(local).distance(p + r * local);
            maximum_error = maximum_error.max(error);
            assert!(
                error < (v + angular * local.length()) / 60.0 * 1.1 + 0.003,
                "fps={fps} error={error} velocity={v}"
            );
        }
        assert!(a.world().get::<TransformInterpolation>(vessel).is_some());
        if fps == 120 {
            assert!(maximum_error > 0.001, "test must observe actual easing");
        }
        let dock = Vec3::new(-18.0, 12.0, 12.0);
        a.world_mut().entity_mut(vessel).insert((
            teleport_pose(dock, Quat::IDENTITY),
            LinearVelocity::ZERO,
            AngularVelocity::ZERO,
            RigidBody::Static,
        ));
        a.world_mut().get_mut::<Vessel>(vessel).unwrap().docked = true;
        a.update();
        assert!(
            a.world()
                .get::<Transform>(vessel)
                .unwrap()
                .translation
                .distance(dock)
                < 0.001,
            "dock {dock:?}, transform {:?}, position {:?}",
            a.world().get::<Transform>(vessel),
            a.world().get::<Position>(vessel)
        );
        println!("render {fps} Hz landmark max lag {maximum_error:.6} m");
    }
}

#[test]
fn moving_sail_sideways_changes_measured_torque_and_course() {
    let mut courses = Vec::new();
    for offset in [-1, 1] {
        let mut a = app();
        let mut blueprint = fixtures::starter().blueprint();
        blueprint
            .parts
            .iter_mut()
            .find(|p| p.kind == aether_core::PartKind::Sail)
            .unwrap()
            .cell = aether_core::Cell(offset, 0, -4);
        let body = aether_core::Body::from_blueprint(blueprint).unwrap();
        let save = aether_core::save::VesselSave {
            circuit: None,
            id: aether_core::BodyId(1),
            blueprint: body.blueprint(),
            position: Vec3::new(100.0, 12.0, 50.0),
            rotation: Quat::IDENTITY,
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            fuel: 180.0,
            trim: 0.0,
            target_altitude: Some(12.0),
            docked: false,
        };
        let entity = spawn_vessel(&mut a.world_mut().commands(), save);
        a.world_mut().flush();
        a.world_mut()
            .get_mut::<PilotIntent>(entity)
            .unwrap()
            .throttle = 0.8;
        run(&mut a, 120);
        let yaw = a
            .world()
            .get::<Rotation>(entity)
            .unwrap()
            .to_euler(EulerRot::YXZ)
            .0;
        courses.push(yaw);
        println!(
            "sail lateral offset {} m: yaw after 2 s = {} degrees",
            offset as f32 * 0.5,
            yaw.to_degrees()
        );
    }
    assert!(
        courses[0] * courses[1] < 0.0 && (courses[0] - courses[1]).abs() > 0.02,
        "{courses:?}"
    );
}
