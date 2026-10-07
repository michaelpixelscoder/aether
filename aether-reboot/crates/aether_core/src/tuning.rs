//! Version 2 flight envelope. SI units except the explicitly fictional Aether unit.
pub const VERSION: u32 = 2;
pub const TICK_HZ: f64 = 60.0;
// Leave 0.55 m beneath the capsule for editable half-metre steps. The
// avatar offset uses the same constant so its feet remain on the deck.
pub const WALKER_FLOAT_HEIGHT: f32 = 1.10;
pub const GRAVITY: f32 = 9.81;
pub const LIFT_NEWTONS_PER_PART: f32 = 22_000.0;
pub const ALTITUDE_GAIN: f32 = 2.0;
pub const VERTICAL_DAMPING: f32 = 2.8;
pub const ALTITUDE_MIN_METERS: f32 = -3000.0;
pub const ALTITUDE_MAX_METERS: f32 = 2200.0;
pub const CLIMB_COMMAND_METERS_PER_SECOND: f32 = 12.0;
pub const IDLE_AETHER_PER_SECOND: f32 = 0.1;
pub const AETHER_PER_NEWTON_SECOND: f32 = 0.000020;
pub const RECHARGE_AETHER_PER_SECOND: f32 = 25.0;
pub const DRAG_PER_SECOND: f32 = 0.1;
pub const BRAKE_DRAG_PER_SECOND: f32 = 1.8;
pub const CURRENT_RELAXATION_SECONDS: f32 = 2.0;
pub const CURRENT_MAX_ACCELERATION: f32 = 8.0;
pub const FORCE_ACCELERATION_CAP: f32 = 35.0;
pub const TORQUE_ACCELERATION_CAP: f32 = 12.0;
pub const TRIM_RADIANS_PER_SECOND: f32 = 0.8;
pub const MAX_TRIM_RADIANS: f32 = 1.3;
/// Small craft are acceleration-limited, heavy designs still need more engines.
pub const MOTOR_NEWTONS: f32 = 12_000.0;
pub const MOTOR_ACCELERATION: f32 = 1.45;
pub const MOTOR_AETHER_PER_SECOND: f32 = 0.10;
pub const MOTOR_CRUISE_METERS_PER_SECOND: f32 = 17.0;
pub const LATERAL_DAMPING: f32 = 0.42;

/// Force only: never writes velocity. Thrust fades above engine cruise speed,
/// leaving the sails and Mainstream free to carry a vessel faster.
pub fn motor_thrust(speed_forward: f32, mass: f32, engines: usize, throttle: f32) -> f32 {
    let available = (engines as f32 * MOTOR_NEWTONS).min(mass * MOTOR_ACCELERATION);
    let taper = ((MOTOR_CRUISE_METERS_PER_SECOND + 8.0 - speed_forward) / 8.0).clamp(0.0, 1.0);
    available * throttle.clamp(0.0, 1.0) * taper
}
