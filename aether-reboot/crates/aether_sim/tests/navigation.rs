#[allow(dead_code)]
mod common;
use aether_core::{BodyId, fixtures, save::VesselSave};
use aether_sim::*;
use avian3d::prelude::*;
use bevy::{prelude::*, time::TimeUpdateStrategy};
use std::time::Duration;

fn expedition(wind: Vec3) -> (App, Entity) {
    let mut a = common::app();
    a.insert_resource(FlightEnvironment {
        wind: Some(wind),
        currents: false,
    });
    let body = fixtures::explorer();
    assert_eq!(body.split().len(), 1);
    let save = VesselSave {
        circuit: None,
        id: BodyId(1),
        fuel: body.fuel_capacity(),
        blueprint: body.blueprint(),
        position: Vec3::new(0.0, 200.0, 500.0),
        rotation: Quat::IDENTITY,
        velocity: Vec3::ZERO,
        angular_velocity: Vec3::ZERO,
        trim: 0.0,
        target_altitude: Some(200.0),
        docked: false,
    };
    let e = spawn_vessel(&mut a.world_mut().commands(), save);
    a.world_mut().flush();
    a.world_mut().get_mut::<PilotIntent>(e).unwrap().throttle = 1.0;
    (a, e)
}
#[test]
fn motor_can_cruise_without_wind_and_against_a_headwind() {
    for wind in [Vec3::ZERO, Vec3::Z * 12.0, Vec3::X * 12.0] {
        let (mut a, e) = expedition(wind);
        common::run(&mut a, 60 * 35);
        let v = a.world().get::<LinearVelocity>(e).unwrap().0;
        let p = a.world().get::<Position>(e).unwrap().0;
        assert!(-v.z > 7.0, "wind {wind:?}, velocity {v:?}");
        assert!((p.y - 200.0).abs() < 1.0, "altitude {p:?}");
        assert!(p.z < 330.0, "no meaningful travel: {p:?}");
        assert!(v.x.abs() < 3.0, "uncontrolled slip: {v:?}");
    }
}
#[test]
fn favourable_wind_adds_speed_and_release_preserves_inertia() {
    let mut speeds = Vec::new();
    for wind in [Vec3::ZERO, Vec3::NEG_Z * 22.0] {
        let (mut a, e) = expedition(wind);
        common::run(&mut a, 60 * 40);
        let v = a.world().get::<LinearVelocity>(e).unwrap().0;
        speeds.push(-v.z);
        a.world_mut().get_mut::<PilotIntent>(e).unwrap().throttle = 0.0;
        common::run(&mut a, 12);
        assert!(a.world().get::<LinearVelocity>(e).unwrap().0.length() > v.length() * 0.90);
        a.world_mut().get_mut::<PilotIntent>(e).unwrap().brake = true;
        common::run(&mut a, 60 * 3);
        assert!(a.world().get::<LinearVelocity>(e).unwrap().0.length() < 1.0);
    }
    assert!(
        speeds[1] > speeds[0] * 1.12,
        "wind should reward sailing: {speeds:?}"
    );
}

#[test]
fn reverse_propulsion_clears_a_manoeuvre_without_becoming_a_fast_travel_mode() {
    let (mut a, e) = expedition(Vec3::NEG_Z * 12.0);
    a.world_mut().get_mut::<PilotIntent>(e).unwrap().throttle = -0.55;
    common::run(&mut a, 60 * 30);
    let position = a.world().get::<Position>(e).unwrap().0;
    let speed = a.world().get::<LinearVelocity>(e).unwrap().z;
    assert!(position.z > 540.0, "reverse made no progress: {position:?}");
    assert!((2.0..6.1).contains(&speed), "reverse speed {speed}");
    assert!(
        a.world().get::<Vessel>(e).unwrap().fuel
            < aether_core::fixtures::explorer().fuel_capacity()
    );
    a.world_mut().get_mut::<PilotIntent>(e).unwrap().brake = true;
    common::run(&mut a, 180);
    assert!(a.world().get::<LinearVelocity>(e).unwrap().0.length() < 1.0);
}
#[test]
fn propulsion_does_not_depend_on_render_frequency() {
    let mut end = Vec::new();
    for fps in [30, 60, 120] {
        let (mut a, e) = expedition(Vec3::ZERO);
        a.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1.0 / fps as f64,
        )));
        common::run(&mut a, fps * 12);
        end.push(a.world().get::<Position>(e).unwrap().0);
    }
    for p in &end[1..] {
        assert!(p.distance(end[0]) < 0.35, "{end:?}");
    }
}
#[test]
fn high_speed_is_not_clamped_to_motor_cruise() {
    let (mut a, e) = expedition(Vec3::ZERO);
    a.world_mut().get_mut::<LinearVelocity>(e).unwrap().0 = Vec3::NEG_Z * 45.0;
    a.world_mut().get_mut::<PilotIntent>(e).unwrap().turn = 1.0;
    common::run(&mut a, 12);
    let v = a.world().get::<LinearVelocity>(e).unwrap().0;
    assert!(v.length() > 40.0, "instant speed clamp: {v:?}");
    assert!(v.z < -39.0, "instant turn of momentum: {v:?}");
}
#[test]
fn an_ascending_current_carries_altitude_hold_and_can_be_left_manually() {
    let (mut a, e) = expedition(Vec3::ZERO);
    a.world_mut().resource_mut::<FlightEnvironment>().currents = true;
    let route = &aether_core::world::routes()[4];
    let (start, end) = route
        .points
        .windows(2)
        .filter(|s| s[1].y - s[0].y > 4.0)
        .map(|p| (p[0], p[1]))
        .next()
        .expect("ascending route");
    a.world_mut().entity_mut(e).insert((
        teleport_pose(start, Quat::IDENTITY),
        LinearVelocity((end - start).normalize() * 20.0),
    ));
    a.world_mut().get_mut::<Vessel>(e).unwrap().target_altitude = start.y;
    a.world_mut().get_mut::<PilotIntent>(e).unwrap().throttle = 0.0;
    common::run(&mut a, 60 * 8);
    let p = a.world().get::<Position>(e).unwrap().0;
    let hold = a.world().get::<Vessel>(e).unwrap().target_altitude;
    assert!(
        p.y > start.y + 8.0,
        "current fought altitude hold: {start:?} -> {p:?}"
    );
    assert!(hold > start.y + 5.0, "hold did not follow flow");
    {
        let mut intent = a.world_mut().get_mut::<PilotIntent>(e).unwrap();
        intent.brake = true;
        intent.climb = 1.0;
    }
    common::run(&mut a, 60 * 16);
    let after = a.world().get::<Position>(e).unwrap().0;
    assert!(
        after.is_finite() && after.y > p.y + 100.0,
        "cannot climb out: {p:?} -> {after:?}"
    );
    assert!(a.world().get::<FlightTelemetry>(e).unwrap().current_weight < 0.2);
}
#[test]
fn extra_cargo_is_real_mass_and_uses_more_lift_energy() {
    let mut consumed = vec![];
    for payload in [0.0, 1000.0] {
        let (mut a, e) = expedition(Vec3::ZERO);
        {
            let mut v = a.world_mut().get_mut::<Vessel>(e).unwrap();
            v.payload = payload;
            v.properties = v.body.mass_properties().with_payload(payload);
        }
        let vessel = a.world().get::<Vessel>(e).unwrap().clone();
        replace_geometry(&mut a.world_mut().commands(), e, &vessel);
        a.world_mut().flush();
        let initial = vessel.fuel;
        common::run(&mut a, 600);
        let v = a.world().get::<Vessel>(e).unwrap();
        assert!((a.world().get::<Mass>(e).unwrap().0 - v.properties.mass).abs() < 0.1);
        consumed.push(initial - v.fuel);
    }
    assert!(
        consumed[1] > consumed[0] * 1.02,
        "cargo energy: {consumed:?}"
    );
}

#[test]
fn powered_pilot_can_leave_a_current_towards_the_chosen_heading() {
    // Reproduces the Dawn water approach: the old full current force pulled
    // the ship south-west even with its bow and both engines facing north.
    let (mut a, e) = expedition(Vec3::ZERO);
    a.world_mut().resource_mut::<FlightEnvironment>().currents = true;
    let start = Vec3::new(-360.0, 94.0, -686.0);
    let rotation = Quat::from_rotation_y(-3.02);
    a.world_mut().entity_mut(e).insert((
        teleport_pose(start, rotation),
        LinearVelocity::ZERO,
        AngularVelocity::ZERO,
    ));
    a.world_mut().get_mut::<Vessel>(e).unwrap().target_altitude = start.y;
    common::run(&mut a, 60 * 18);
    let end = a.world().get::<Position>(e).unwrap().0;
    assert!(
        end.z > start.z + 45.0,
        "current defeated engines: {start:?} -> {end:?}"
    );
    assert!(
        (end.y - start.y).abs() < 15.0,
        "current dragged pilot vertically: {end:?}"
    );
}

#[test]
fn compact_vortex_eye_can_be_left_under_power_in_each_cardinal_heading() {
    let start = Vec3::new(172.0, 122.0, -482.0);
    for yaw in [
        0.0,
        std::f32::consts::FRAC_PI_2,
        std::f32::consts::PI,
        -std::f32::consts::FRAC_PI_2,
    ] {
        let (mut app, entity) = expedition(Vec3::ZERO);
        app.world_mut().resource_mut::<FlightEnvironment>().currents = true;
        let rotation = Quat::from_rotation_y(yaw);
        let forward = rotation * Vec3::NEG_Z;
        app.world_mut().entity_mut(entity).insert((
            teleport_pose(start, rotation),
            LinearVelocity::ZERO,
            AngularVelocity::ZERO,
        ));
        app.world_mut()
            .get_mut::<Vessel>(entity)
            .unwrap()
            .target_altitude = start.y;
        common::run(&mut app, 60 * 20);
        let end = app.world().get::<Position>(entity).unwrap().0;
        assert!(
            end.is_finite() && (end - start).dot(forward) > 65.0,
            "vortex trapped the chosen heading {yaw}: {start:?} -> {end:?}"
        );
        assert!(
            (end.y - start.y).abs() < 15.0,
            "uncontrolled vertical drift: {end:?}"
        );
    }
}

#[test]
fn expedition_hull_can_moor_alongside_the_gardens_with_keyboard_intentions() {
    let mut app = common::app();
    let body = fixtures::explorer();
    let entity = spawn_vessel(
        &mut app.world_mut().commands(),
        VesselSave {
            circuit: None,
            id: BodyId(1),
            fuel: body.fuel_capacity(),
            blueprint: body.blueprint(),
            position: ISLANDS[0].dock,
            rotation: Quat::IDENTITY,
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            trim: 0.0,
            target_altitude: Some(ISLANDS[0].dock.y),
            docked: false,
        },
    );
    spawn_world(&mut app.world_mut().commands());
    app.world_mut().flush();
    // Stop west of the jetty, then enter parallel to its long side. A 16.5 m
    // hull cannot drive bow-first to the platform centre. No pose is injected
    // after creation; these are the same binary intentions as keyboard input.
    let route = [
        Vec3::new(5.0, 14.0, -65.0),
        Vec3::new(28.0, 18.0, -115.0),
        Vec3::new(55.0, 20.0, -132.0),
        Vec3::new(57.0, 20.0, -154.0),
    ];
    let mut waypoint = 0;
    let mut closest = f32::MAX;
    let mut arrived = false;
    for tick in 0..60 * 85 {
        let position = app.world().get::<Position>(entity).unwrap().0;
        let speed = app
            .world()
            .get::<LinearVelocity>(entity)
            .unwrap()
            .0
            .length();
        closest = closest.min(position.distance(ISLANDS[1].dock));
        if dock_near(
            position,
            app.world().get::<LinearVelocity>(entity).unwrap().0,
        ) == Some(1)
        {
            arrived = true;
            break;
        }
        if tick % 12 == 0 {
            let segment = (route[waypoint] - position).with_y(0.0).length();
            let tolerance = if waypoint == 2 { 2.0 } else { 14.0 };
            if waypoint < 3 && segment < tolerance && (waypoint != 2 || speed < 1.5) {
                waypoint += 1;
            }
            let goal = route[waypoint];
            let delta = goal - position;
            let velocity = app.world().get::<LinearVelocity>(entity).unwrap().0;
            let steering = delta - velocity.with_y(0.0) * 1.5;
            let segment = delta.with_y(0.0).length();
            let yaw = app
                .world()
                .get::<Rotation>(entity)
                .unwrap()
                .0
                .to_euler(EulerRot::YXZ)
                .0;
            let angle = ((-steering.x).atan2(-steering.z) - yaw + std::f32::consts::PI)
                .rem_euclid(std::f32::consts::TAU)
                - std::f32::consts::PI;
            let angular = app.world().get::<AngularVelocity>(entity).unwrap().y;
            let altitude = app.world().get::<Vessel>(entity).unwrap().target_altitude;
            let turn = angle * 2.0 - angular * 1.5;
            let desired = 11.0_f32.min(segment * 0.3);
            if tick % 60 == 0 && std::env::var_os("AETHER_TRACE").is_some() {
                eprintln!(
                    "t={} p={position:?} speed={speed:.2} yaw={yaw:.2} angle={angle:.2} waypoint={waypoint} segment={segment:.2}",
                    tick / 60
                );
            }
            let mut intent = app.world_mut().get_mut::<PilotIntent>(entity).unwrap();
            intent.turn = if turn > 0.13 {
                1.0
            } else if turn < -0.13 {
                -1.0
            } else {
                0.0
            };
            intent.throttle =
                if speed < desired + 0.3 && angle.abs() < if segment < 30.0 { 0.2 } else { 0.7 } {
                    1.0
                } else {
                    0.0
                };
            intent.climb = if goal.y - altitude > 0.4 {
                1.0
            } else if goal.y - altitude < -0.4 {
                -1.0
            } else {
                0.0
            };
            intent.brake =
                speed > desired + 0.25 || segment < 1.8 || (segment < 30.0 && angle.abs() > 0.25);
        }
        app.update();
    }
    assert!(
        arrived,
        "expedition hull failed to berth: closest={closest}, waypoint={waypoint}, position={:?}",
        app.world().get::<Position>(entity).unwrap().0
    );
    assert!(app.world().get::<Vessel>(entity).unwrap().fuel > 0.0);
}
