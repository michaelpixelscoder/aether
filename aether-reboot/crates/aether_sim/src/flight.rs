use crate::{FlightTelemetry, PilotIntent, SimClock, Vessel};
use aether_core::{PartKind, fields, tuning::*};
use avian3d::prelude::*;
use bevy::prelude::*;

/// Deterministic environments for the physics laboratory and authored scenarios.
/// A normal voyage has neither override and samples the world fields.
#[derive(Resource, Default)]
pub struct FlightEnvironment {
    pub wind: Option<Vec3>,
    pub currents: bool,
}

pub fn flight_forces(
    time: Res<Time<Fixed>>,
    clock: Res<SimClock>,
    environment: Option<Res<FlightEnvironment>>,
    mut vessels: Query<(
        &mut Vessel,
        &PilotIntent,
        &Position,
        &Rotation,
        &LinearVelocity,
        &AngularVelocity,
        &mut ConstantForce,
        &mut ConstantTorque,
        &mut FlightTelemetry,
        Option<&mut crate::aether::AetherCircuit>,
    )>,
) {
    let dt = time.delta_secs();
    for (
        mut vessel,
        intent,
        position,
        rotation,
        velocity,
        angular,
        mut force,
        mut torque,
        mut telemetry,
        mut circuit,
    ) in &mut vessels
    {
        force.0 = Vec3::ZERO;
        torque.0 = Vec3::ZERO;
        let wind = environment
            .as_ref()
            .and_then(|e| e.wind)
            .unwrap_or_else(|| {
                fields::wind_seeded(position.0, clock.tick as f32 / TICK_HZ as f32, clock.seed)
            });
        telemetry.wind = wind;
        if vessel.docked {
            *telemetry = FlightTelemetry { wind, ..default() };
            if let Some(supply) = circuit.as_deref_mut() {
                telemetry.circuit_fault = supply.dock_recharge(time.delta()).is_err();
                match supply.burn(&vessel.body, time.delta(), 0.0, 0.0) {
                    Ok(burn) => {
                        telemetry.leakage = if dt > 0.0 {
                            burn.leaked_milli as f32 / 1000.0 / dt
                        } else {
                            0.0
                        };
                    }
                    Err(_) => telemetry.circuit_fault = true,
                }
                vessel.fuel = supply.remaining_milli() as f32 / 1000.0;
            } else {
                vessel.fuel = (vessel.fuel + dt * RECHARGE_AETHER_PER_SECOND)
                    .min(vessel.body.fuel_capacity());
            }
            continue;
        }
        let mass = vessel.properties;
        let engines = vessel.body.count_parts(PartKind::Propeller);
        let throttle = if intent.brake {
            0.0
        } else {
            intent.throttle.clamp(-1.0, 1.0)
        };
        let (current_velocity, current_weight) = if environment.as_ref().is_none_or(|e| e.currents)
        {
            fields::sample_currents(position.0)
        } else {
            (Vec3::ZERO, 0.0)
        };
        // A vertical route carries the altitude hold along with it. Manual
        // climb/descent remains available to leave the flow at any point.
        let active_motor = engines > 0 && throttle.abs() > 0.0;
        let manual_vertical = intent.climb.abs() > 0.0 || intent.brake;
        let vertical_flow = current_weight
            * if manual_vertical {
                0.1
            } else if active_motor {
                0.25
            } else {
                1.0
            };
        if current_weight > 0.15 && !manual_vertical && !active_motor {
            vessel.target_altitude += (position.0.y - vessel.target_altitude)
                * (current_weight * dt * 2.5).clamp(0.0, 1.0);
        }
        let center = position.0 + rotation.0 * mass.center;
        vessel.trim = (vessel.trim + intent.trim * dt * TRIM_RADIANS_PER_SECOND)
            .clamp(-MAX_TRIM_RADIANS, MAX_TRIM_RADIANS);
        vessel.target_altitude = (vessel.target_altitude
            + intent.climb * dt * CLIMB_COMMAND_METERS_PER_SECOND)
            .clamp(ALTITUDE_MIN_METERS, ALTITUDE_MAX_METERS);
        let lift_count = vessel.body.count_parts(PartKind::Lift) as f32;
        let requested = (mass.mass
            * (GRAVITY + (vessel.target_altitude - position.0.y) * ALTITUDE_GAIN
                - (velocity.0.y - current_velocity.y * vertical_flow) * VERTICAL_DAMPING))
            .max(0.0);
        let available = requested.min(lift_count * LIFT_NEWTONS_PER_PART);
        let forward = (rotation.0 * Vec3::NEG_Z).with_y(0.0).normalize_or_zero();
        let consumption = (if available > 0.0 {
            IDLE_AETHER_PER_SECOND + available * AETHER_PER_NEWTON_SECOND
        } else {
            0.0
        }) + engines as f32 * throttle.abs() * MOTOR_AETHER_PER_SECOND;
        let common_fraction = if consumption * dt > 0.0 {
            (vessel.fuel / (consumption * dt)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let mut circuit_fault = false;
        let circuit_burn = circuit.as_deref_mut().map(|supply| {
            let result = supply.burn(&vessel.body, time.delta(), available, throttle);
            vessel.fuel = supply.remaining_milli() as f32 / 1000.0;
            match result {
                Ok(burn) => burn,
                Err(_) => {
                    circuit_fault = true;
                    crate::aether::Burn::default()
                }
            }
        });
        if circuit_burn.is_none() {
            vessel.fuel = (vessel.fuel - consumption * dt).max(0.0);
        }
        let power = |id| {
            circuit_burn.as_ref().map_or(common_fraction, |burn| {
                burn.fractions.get(&id).copied().unwrap_or(0.0)
            })
        };
        let fraction = if circuit_burn.is_some() {
            vessel
                .body
                .parts()
                .iter()
                .filter(|p| p.kind == PartKind::Lift)
                .map(|p| power(p.id))
                .sum::<f32>()
                / lift_count.max(1.0)
        } else {
            common_fraction
        };
        let motor_fraction = if circuit_burn.is_some() {
            vessel
                .body
                .parts()
                .iter()
                .filter(|p| p.kind == PartKind::Propeller)
                .map(|p| power(p.id))
                .sum::<f32>()
                / (engines as f32).max(1.0)
        } else {
            common_fraction
        };
        let lift = Vec3::Y * available * fraction;
        force.0 += lift;
        for engine in vessel
            .body
            .parts()
            .iter()
            .filter(|p| p.kind == PartKind::Propeller)
        {
            let direction = (rotation.0
                * Quat::from_rotation_y(engine.quarter_turn as f32 * std::f32::consts::FRAC_PI_2)
                * Vec3::NEG_Z)
                .with_y(0.0)
                .normalize_or_zero()
                * if throttle < 0.0 { -1.0 } else { 1.0 };
            let reverse_taper = if throttle < 0.0 {
                ((6.0 - velocity.0.dot(direction)) / 3.0).clamp(0.0, 1.0)
            } else {
                1.0
            };
            let push = direction
                * motor_thrust(
                    velocity.0.dot(direction),
                    mass.mass,
                    engines,
                    throttle.abs(),
                )
                * reverse_taper
                / engines as f32
                * power(engine.id);
            force.0 += push;
            torque.0 += (rotation.0 * (engine.center() - mass.center)).cross(push);
        }
        // Rudder assistance removes lateral slip gradually at low speed. At high
        // speed it weakens: turning the bow does not instantly turn momentum.
        if engines > 0 && throttle.abs() > 0.0 {
            let lateral = Vec3::new(-forward.z, 0.0, forward.x);
            let authority = LATERAL_DAMPING / (1.0 + velocity.0.length_squared() / 400.0);
            force.0 -= lateral * velocity.0.dot(lateral) * mass.mass * authority * motor_fraction;
        }
        let lift_center = vessel
            .body
            .parts()
            .iter()
            .filter(|p| p.kind == PartKind::Lift)
            .fold(Vec3::ZERO, |sum, p| sum + p.center())
            / lift_count.max(1.0);
        if circuit_burn.is_some() {
            for device in vessel
                .body
                .parts()
                .iter()
                .filter(|p| p.kind == PartKind::Lift)
            {
                let powered_lift = Vec3::Y * available / lift_count.max(1.0) * power(device.id);
                torque.0 += (rotation.0 * (device.center() - mass.center)).cross(powered_lift);
            }
        } else {
            torque.0 += (rotation.0 * (lift_center - mass.center)).cross(lift);
        }
        let trim = Quat::from_rotation_y(vessel.trim);
        for sail in vessel.body.parts().iter().filter(|p| p.kind.is_sail()) {
            let point = position.0 + rotation.0 * sail.center();
            let point_velocity = velocity.0 + angular.0.cross(point - center);
            let normal = rotation.0
                * Quat::from_rotation_y(sail.quarter_turn as f32 * std::f32::consts::FRAC_PI_2)
                * trim
                * Vec3::Z;
            let mut push = fields::sail_force(
                wind - point_velocity,
                normal,
                sail.kind.size().x
                    * sail.kind.size().y
                    * if engines > 0 {
                        if intent.brake || throttle < 0.0 {
                            0.0
                        } else {
                            0.8
                        }
                    } else {
                        intent.throttle.clamp(0.0, 1.0)
                    },
            );
            // Automatic reefing in adverse apparent wind makes motor travel
            // reliable. Off-centre rigs still create their real force torque.
            if engines > 0 && push.dot(forward) < 0.0 {
                push *= 0.25;
            }
            force.0 += push;
            torque.0 += (point - center).cross(push);
        }
        let mut flow_acceleration = ((current_velocity - velocity.0) * current_weight
            / CURRENT_RELAXATION_SECONDS)
            .clamp_length_max(CURRENT_MAX_ACCELERATION);
        // A powered rudder feathers across/opposite the stream, so its 8 m/s²
        // acceleration cannot overpower the small motor. Favourable flow keeps
        // its full boost and all changes remain forces, preserving momentum.
        if active_motor {
            let heading = forward * throttle.signum();
            let along = flow_acceleration.dot(heading);
            let lateral = flow_acceleration.with_y(0.0) - heading * along;
            flow_acceleration = heading * along.max(-0.35)
                + lateral.clamp_length_max(0.6)
                + Vec3::Y * flow_acceleration.y;
        }
        if intent.brake {
            flow_acceleration *= 0.15;
        }
        if manual_vertical || active_motor {
            flow_acceleration.y *= 0.1;
        }
        force.0 += flow_acceleration * mass.mass;
        force.0 -= velocity.0 * mass.mass * DRAG_PER_SECOND;
        if intent.brake {
            force.0 -=
                velocity.0.with_y(0.0) * mass.mass * (BRAKE_DRAG_PER_SECOND - DRAG_PER_SECOND);
        }
        // Aether stabilisers supply a bounded restoring torque; sail offsets remain physical.
        let inertia_scale = mass.inertia.y_axis.y.max(1.0);
        let up = rotation.0 * Vec3::Y;
        torque.0 +=
            (up.cross(Vec3::Y) * inertia_scale * 6.5 - angular.0 * inertia_scale * 1.6) * fraction;
        let turning = if engines > 0 {
            1.5 / (1.0 + velocity.0.length_squared() / 900.0)
        } else {
            0.9
        };
        torque.0 += Vec3::Y * intent.turn * inertia_scale * turning * fraction;
        force.0 = force.0.clamp_length_max(mass.mass * FORCE_ACCELERATION_CAP);
        torque.0 = torque
            .0
            .clamp_length_max(inertia_scale * TORQUE_ACCELERATION_CAP);
        *telemetry = FlightTelemetry {
            lift_ratio: available * fraction / (mass.mass * GRAVITY),
            consumption: circuit_burn.as_ref().map_or(consumption * fraction, |b| {
                if dt > 0.0 {
                    b.used_milli as f32 / 1000.0 / dt
                } else {
                    0.0
                }
            }),
            wind,
            current_weight,
            motor: if engines > 0 {
                throttle * motor_fraction
            } else {
                0.0
            },
            leakage: circuit_burn.as_ref().map_or(0.0, |b| {
                if dt > 0.0 {
                    b.leaked_milli as f32 / 1000.0 / dt
                } else {
                    0.0
                }
            }),
            circuit_fault,
        };
    }
}
