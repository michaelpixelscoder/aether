use crate::{
    app::Phase,
    controls::{Action, Actions, line_clear},
    session::GameSession,
};
use aether_sim::{FlightTelemetry, Tether, Vessel};
use bevy::prelude::*;

pub fn weather(
    clock: Res<aether_sim::SimClock>,
    game: Res<GameSession>,
    prefs: Res<crate::persistence::Preferences>,
    positions: Query<&avian3d::prelude::Position>,
    mut view: ResMut<aether_view::weather::WeatherView>,
) {
    let Some(position) = game
        .walker
        .or(game.active)
        .and_then(|e| positions.get(e).ok())
    else {
        return;
    };
    view.seconds = clock.tick as f32 / aether_core::tuning::TICK_HZ as f32;
    view.sample = aether_core::weather::sample(position.0, view.seconds);
    view.reduced_motion = prefs.reduce_motion;
}

pub fn animate_motors(
    clock: Res<aether_sim::SimClock>,
    prefs: Res<crate::persistence::Preferences>,
    vessels: Query<&FlightTelemetry>,
    mut motors: Query<(
        &aether_view::PartVisual,
        &mut aether_view::mechanisms::MotorVisual,
    )>,
) {
    for (part, mut motor) in &mut motors {
        if let Ok(telemetry) = vessels.get(part.owner) {
            let seconds = clock.tick as f32 / aether_core::tuning::TICK_HZ as f32;
            let dt = (seconds - motor.seconds).clamp(0.0, 0.1);
            motor.seconds = seconds;
            motor.power = telemetry.motor;
            motor.reduced_motion = prefs.reduce_motion;
            if !motor.reduced_motion {
                motor.angle =
                    (motor.angle + dt * motor.power * 26.0).rem_euclid(std::f32::consts::TAU);
            }
        }
    }
}

pub fn animate_equipment(
    clock: Res<aether_sim::SimClock>,
    prefs: Res<crate::persistence::Preferences>,
    vessels: Query<(
        &Vessel,
        &avian3d::prelude::Position,
        &avian3d::prelude::Rotation,
        &avian3d::prelude::LinearVelocity,
        &avian3d::prelude::AngularVelocity,
    )>,
    couriers: Query<(
        &aether_sim::traffic::Courier,
        &avian3d::prelude::Position,
        &avian3d::prelude::Rotation,
        &avian3d::prelude::LinearVelocity,
    )>,
    mut parts: Query<(
        &aether_view::PartVisual,
        &mut Transform,
        Option<&mut aether_view::sail::SailWind>,
    )>,
) {
    let seconds = clock.tick as f32 / aether_core::tuning::TICK_HZ as f32;
    for (visual, mut t, wind) in &mut parts {
        let (body, center, position, rotation, velocity, angular, trim) = if let Ok((
            vessel,
            position,
            rotation,
            velocity,
            angular,
        )) =
            vessels.get(visual.owner)
        {
            (
                &vessel.body,
                vessel.properties.center,
                position.0,
                rotation.0,
                velocity.0,
                angular.0,
                vessel.trim,
            )
        } else if let Ok((courier, position, rotation, velocity)) = couriers.get(visual.owner) {
            let Some(body) = courier.0.sailing_body() else {
                continue;
            };
            (
                body,
                Vec3::ZERO,
                position.0,
                rotation.0,
                velocity.0,
                Vec3::ZERO,
                0.0,
            )
        } else {
            continue;
        };
        if let Some(part) = body.parts().iter().find(|p| p.id == visual.id)
            && part.kind.is_sail()
        {
            t.rotation = Quat::from_rotation_y(
                part.quarter_turn as f32 * std::f32::consts::FRAC_PI_2 + trim,
            );
            if let Some(mut wind) = wind {
                let apparent = aether_core::fields::wind_seeded(position, seconds, clock.seed)
                    - velocity
                    - angular.cross(rotation * (part.center() - center));
                *wind = aether_view::sail::SailWind {
                    seconds,
                    normal_speed: apparent.dot(rotation * t.rotation * Vec3::Z),
                    speed: apparent.length(),
                    phase: part.center().dot(Vec3::new(0.73, 0.31, 0.57)),
                    reduced_motion: prefs.reduce_motion,
                };
            }
        }
    }
}

pub fn animate_avatars(
    game: Res<GameSession>,
    vessels: Query<&Vessel>,
    walkers: Query<(
        &aether_sim::character::WalkerIntent,
        &aether_sim::character::WalkerController,
    )>,
    mut avatars: Query<(
        &mut aether_view::avatar::AvatarVisual,
        &mut Transform,
        &mut Visibility,
    )>,
) {
    for (mut avatar, mut transform, mut visibility) in &mut avatars {
        if avatar.pilot {
            *visibility = if game.active == Some(avatar.owner) && game.walker.is_none() {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
            if let Ok(vessel) = vessels.get(avatar.owner) {
                if let Some(helm) = vessel
                    .body
                    .parts()
                    .iter()
                    .find(|p| p.kind == aether_core::PartKind::Helm)
                {
                    transform.translation = helm.cell.center() + Vec3::new(0.0, 0.25, 0.5);
                } else {
                    *visibility = Visibility::Hidden;
                }
            }
        } else if let Ok((intent, controller)) = walkers.get(avatar.owner) {
            avatar.motion = if controller.is_airborne().unwrap_or(false) {
                2
            } else if intent.direction.length_squared() > 0.01 {
                1
            } else {
                0
            };
            if intent.direction.length_squared() > 0.01 {
                transform.look_to(intent.direction, Vec3::Y);
            }
        }
    }
}

pub fn update(
    time: Res<Time>,
    phase: Res<State<Phase>>,
    mut game: ResMut<GameSession>,
    mut actions: ResMut<Actions>,
    vessels: Query<(&Vessel, &Transform, &FlightTelemetry)>,
    transforms: Query<&Transform>,
    diagnostics: Res<crate::diagnostics::Diagnostics>,
    prefs: Res<crate::persistence::Preferences>,
) {
    if !matches!(phase.get(), Phase::Playing | Phase::Editing) {
        return;
    }
    let Some(active) = game.active else {
        return;
    };
    let Ok((vessel, transform, telemetry)) = vessels.get(active) else {
        return;
    };
    if transform.translation.y < aether_core::tuning::ALTITUDE_MIN_METERS - 120.0
        || !transform.translation.is_finite()
    {
        actions.0.push_back(Action::Recover);
        return;
    }
    if let Some(walker) = game.walker
        && transforms.get(walker).is_ok_and(|t| {
            !t.translation.is_finite()
                || t.translation.y < aether_core::tuning::ALTITUDE_MIN_METERS - 80.0
        })
    {
        actions.0.push_back(Action::Walk);
    }
    if telemetry.current_weight > 0.5 && game.progress < 2 {
        game.progress = 2;
        game.notice = "Le courant vous porte. Une ancre dorée approche sur votre droite.".into();
    }
    if vessel.fuel < vessel.body.fuel_capacity() * 0.2 && !game.fuel_warning && !vessel.docked {
        game.fuel_warning = true;
        game.notice = format!(
            "Réserve basse ! Rejoignez un quai ou utilisez {} pour demander secours.",
            prefs.bindings.label(crate::bindings::Control::Recover)
        );
    }
    if vessel.docked {
        game.fuel_warning = false;
    }
    if diagnostics.benchmark {
        return;
    }
    game.autosave_elapsed += time.delta_secs();
    if game.autosave_elapsed > 30.0 {
        game.autosave_elapsed = 0.0;
        actions.0.push_back(Action::Save);
    }
}
pub fn draw_tether(
    mut commands: Commands,
    tethers: Query<(Entity, &Tether)>,
    transforms: Query<&Transform>,
    mut gizmos: Gizmos,
    physics: Query<(&avian3d::prelude::Position, &avian3d::prelude::Rotation)>,
    mut game: ResMut<GameSession>,
) {
    for (entity, tether) in &tethers {
        let (Ok(vessel), Ok(anchor)) =
            (transforms.get(tether.vessel), transforms.get(tether.anchor))
        else {
            commands.entity(entity).despawn();
            continue;
        };
        let a = vessel.transform_point(tether.local_point);
        let b = anchor.translation;
        let physical_clear = physics
            .get(tether.vessel)
            .ok()
            .zip(physics.get(tether.anchor).ok())
            .is_some_and(|((p, r), (anchor, _))| {
                line_clear(p.0 + r.0 * tether.local_point, anchor.0)
            });
        if !physical_clear {
            commands.entity(entity).despawn();
            game.notice = "Le câble a rencontré un obstacle et s'est libéré.".into();
            continue;
        }
        let slack = (tether.length - a.distance(b)).max(0.0);
        let color = if a.distance(b) > tether.length * 0.98 {
            Color::srgb(1.0, 0.77, 0.28)
        } else {
            Color::srgb(0.8, 0.72, 1.0)
        };
        let mut previous = a;
        for i in 1..=24 {
            let t = i as f32 / 24.0;
            let point = a.lerp(b, t) - Vec3::Y * (slack * 0.5 * (std::f32::consts::PI * t).sin());
            gizmos.line(previous, point, color);
            previous = point;
        }
    }
}
