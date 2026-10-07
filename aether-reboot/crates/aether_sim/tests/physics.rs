mod common;
const STANDING_HEIGHT: f32 = aether_core::tuning::WALKER_FLOAT_HEIGHT + 0.25;
use aether_core::fixtures;
use aether_sim::{character::*, *};
use avian3d::prelude::*;
use bevy::{prelude::*, time::TimeUpdateStrategy};
use common::{app, run, vessel};
use std::time::Duration;
#[test]
fn gravity_and_static_contacts_are_finite() {
    let mut a = app();
    a.world_mut().spawn((
        RigidBody::Static,
        Collider::cuboid(100.0, 1.0, 100.0),
        Transform::from_xyz(0.0, -0.5, 0.0),
    ));
    let ball = a
        .world_mut()
        .spawn((
            RigidBody::Dynamic,
            Collider::sphere(0.5),
            Transform::from_xyz(0.0, 5.0, 0.0),
        ))
        .id();
    run(&mut a, 600);
    let p = a.world().get::<Position>(ball).unwrap().0;
    assert!(p.is_finite());
    assert!((p.y - 0.5).abs() < 0.05, "{p:?}");
}
#[test]
fn voxel_collider_supports_a_sphere() {
    let mut a = app();
    let body = fixtures::solid([8, 1, 8], aether_core::Block::Wood);
    a.world_mut()
        .spawn((RigidBody::Static, collider(&body), collider_offset()));
    let ball = a
        .world_mut()
        .spawn((
            RigidBody::Dynamic,
            Collider::sphere(0.25),
            Transform::from_xyz(1.0, 4.0, 1.0),
        ))
        .id();
    run(&mut a, 360);
    let p = a.world().get::<Position>(ball).unwrap().0;
    assert!((p.y - 0.5).abs() < 0.08, "{p:?}");
}
#[test]
fn lift_consumes_energy_and_stays_bounded() {
    let mut a = app();
    let e = vessel(&mut a, false);
    run(&mut a, 600);
    let v = a.world().get::<Vessel>(e).unwrap();
    assert!(v.fuel < v.body.fuel_capacity() && v.fuel > 0.0);
    let p = a.world().get::<Position>(e).unwrap().0;
    assert!(p.is_finite());
    assert!(
        (p.y - v.target_altitude).abs() < 1.0,
        "altitude hold diverges: {p:?}, target {}",
        v.target_altitude
    );
    let w = a.world().get::<AngularVelocity>(e).unwrap().0;
    assert!(w.length() < 2.0, "{w:?}");
}
#[test]
fn tank_empty_removes_sustentation() {
    let mut a = app();
    let e = vessel(&mut a, false);
    a.world_mut().get_mut::<Vessel>(e).unwrap().fuel = 0.0;
    run(&mut a, 60);
    assert!(a.world().get::<Position>(e).unwrap().y < 9.0);
    assert_eq!(a.world().get::<Vessel>(e).unwrap().fuel, 0.0);
}
#[test]
fn dock_recharges_but_does_not_move() {
    let mut a = app();
    let e = vessel(&mut a, true);
    let capacity = a.world().get::<Vessel>(e).unwrap().body.fuel_capacity();
    a.world_mut().get_mut::<Vessel>(e).unwrap().fuel = capacity - 5.0;
    run(&mut a, 120);
    assert_eq!(a.world().get::<Vessel>(e).unwrap().fuel, capacity);
    assert_eq!(
        a.world().get::<Position>(e).unwrap().0,
        Vec3::new(0.0, 12.0, 0.0)
    );
}
#[test]
fn distance_joint_is_unilateral_and_bounded() {
    let mut a = app();
    let anchor = a
        .world_mut()
        .spawn((RigidBody::Static, Transform::IDENTITY))
        .id();
    let body = a
        .world_mut()
        .spawn((
            RigidBody::Dynamic,
            Collider::sphere(0.2),
            Transform::from_xyz(0.0, -2.0, 0.0),
            Mass(10.0),
        ))
        .id();
    a.world_mut()
        .spawn(DistanceJoint::new(body, anchor).with_limits(0.0, 5.0));
    run(&mut a, 1);
    assert!(a.world().get::<Position>(body).unwrap().0.length() < 3.0);
    run(&mut a, 600);
    let p = a.world().get::<Position>(body).unwrap().0;
    assert!(p.length() < 5.1, "{p:?}");
    assert!(p.length() > 4.8);
}
#[test]
fn fixed_ticks_ignore_render_rate() {
    fn simulate(pattern: &[Duration]) -> (Vec3, Quat, f32) {
        let mut a = app();
        let e = vessel(&mut a, false);
        a.world_mut().get_mut::<PilotIntent>(e).unwrap().throttle = 0.8;
        let target = a.world().resource::<Time<Fixed>>().timestep() * 300;
        let mut elapsed = Duration::ZERO;
        let mut frame = 0;
        while elapsed < target {
            let dt = pattern[frame % pattern.len()].min(target - elapsed);
            a.insert_resource(TimeUpdateStrategy::ManualDuration(dt));
            a.update();
            elapsed += dt;
            frame += 1;
        }
        assert_eq!(a.world().resource::<SimClock>().tick, 300);
        (
            a.world().get::<Position>(e).unwrap().0,
            a.world().get::<Rotation>(e).unwrap().0,
            a.world().get::<Vessel>(e).unwrap().fuel,
        )
    }
    let a = simulate(&[Duration::from_secs_f64(1.0 / 30.0)]);
    let patterns = vec![
        vec![Duration::from_secs_f64(1.0 / 60.0)],
        vec![Duration::from_secs_f64(1.0 / 120.0)],
        vec![
            Duration::from_millis(2),
            Duration::from_millis(55),
            Duration::from_millis(8),
            Duration::from_millis(11),
            Duration::from_millis(40),
        ],
    ];
    for pattern in patterns {
        let b = simulate(&pattern);
        assert!(a.0.distance(b.0) < 0.01, "{:?} {:?}", a.0, b.0);
        assert!(a.1.angle_between(b.1).abs() < 0.5_f32.to_radians());
        assert!((a.2 - b.2).abs() < 0.001);
    }
}
#[test]
fn controller_follows_translating_rotating_deck() {
    let mut a = app();
    let platform = a
        .world_mut()
        .spawn((
            RigidBody::Kinematic,
            Collider::cuboid(12.0, 0.5, 12.0),
            Transform::default(),
            LinearVelocity::ZERO,
            AngularVelocity::ZERO,
        ))
        .id();
    let mut entity = None;
    a.world_mut()
        .resource_scope(|world, mut configs: Mut<Assets<WalkingConfig>>| {
            entity = Some(spawn_walker(
                &mut world.commands(),
                &mut configs,
                Vec3::new(1.0, STANDING_HEIGHT, 0.0),
            ));
        });
    a.world_mut().flush();
    let character = entity.unwrap();
    run(&mut a, 120);
    let before = a.world().get::<Position>(character).unwrap().0;
    a.world_mut().entity_mut(platform).insert((
        LinearVelocity(Vec3::new(5.0, 0.0, 0.0)),
        AngularVelocity(Vec3::Y * std::f32::consts::FRAC_PI_6),
    ));
    run(&mut a, 1800);
    let center = a.world().get::<Position>(platform).unwrap().0;
    let rotation = a.world().get::<Rotation>(platform).unwrap().0;
    let local = rotation.inverse() * (a.world().get::<Position>(character).unwrap().0 - center);
    assert!(
        (local.x - before.x).abs() < 0.05 && (local.z - before.z).abs() < 0.05,
        "relative drift: before={before:?} after={local:?}"
    );
}
