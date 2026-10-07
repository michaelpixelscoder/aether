use glam::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Current {
    pub points: Vec<Vec3>,
    pub radius: f32,
    pub speed: f32,
}
impl Current {
    pub fn sample(&self, position: Vec3) -> (Vec3, f32) {
        let mut velocity = Vec3::ZERO;
        let mut sum = 0.0_f32;
        let mut influence = 0.0_f32;
        let radius = self.radius.max(0.01);
        // Conservative rejection: no square root or normalization is needed
        // for a segment that cannot contribute. Keep a margin around the edge
        // so floating-point rounding cannot remove a nonzero legacy weight.
        let outside_squared = radius * radius * 1.000_001;
        for segment in self.points.windows(2) {
            let d = segment[1] - segment[0];
            let length = d.length_squared();
            if length < 1e-6 {
                continue;
            }
            let t = ((position - segment[0]).dot(d) / length).clamp(0.0, 1.0);
            let distance_squared = position.distance_squared(segment[0] + d * t);
            if distance_squared > outside_squared {
                continue;
            }
            let distance = distance_squared.sqrt();
            let x = (1.0 - distance / radius).clamp(0.0, 1.0);
            let weight = x * x * (3.0 - 2.0 * x);
            velocity += d.normalize() * self.speed * weight;
            sum += weight;
            influence = influence.max(weight);
        }
        if sum > 0.0 {
            (velocity / sum, influence)
        } else {
            (Vec3::ZERO, 0.0)
        }
    }
}
pub fn wind(position: Vec3, time: f32) -> Vec3 {
    wind_seeded(position, time, 7391)
}
pub fn wind_seeded(position: Vec3, time: f32, seed: u64) -> Vec3 {
    let time = time + (seed.wrapping_sub(7391) % 10007) as f32 * 0.013;
    let prevailing = Vec3::new(
        3.0 + (time * 0.11 + position.z * 0.006).sin(),
        0.2 * (time * 0.07).sin(),
        -11.0 + (time * 0.08).cos(),
    );
    // Smooth, spatially coherent gusts; no per-frame randomness or abrupt turns.
    // Keep the prevailing route and bound the speed modulation to ±13.5%.
    let gust = 1.0
        + 0.09 * (time * 0.67 + position.z * 0.012).sin()
        + 0.045 * (time * 1.37 - position.x * 0.018).sin();
    let veer = 0.045 * (time * 0.31 + position.x * 0.008).sin();
    let weather = crate::weather::sample(position, time);
    glam::Quat::from_rotation_y(veer + weather.storm * (time * 0.23).sin() * 0.12)
        * prevailing
        * gust
        * (1.0 + weather.storm * 0.65 + weather.ash * 0.2)
}
pub fn sail_force(relative_wind: Vec3, normal: Vec3, area: f32) -> Vec3 {
    let n = normal.normalize_or_zero();
    let speed = relative_wind.dot(n);
    n * (0.5 * 1.2 * area.max(0.0) * 1.3 * speed * speed.abs()).clamp(-18_000.0, 18_000.0)
}
pub fn current_acceleration(current: &Current, position: Vec3, velocity: Vec3) -> Vec3 {
    let (target, weight) = current.sample(position);
    ((target - velocity) * weight / 2.0).clamp_length_max(8.0)
}
pub fn archipelago_current() -> Current {
    Current {
        points: vec![
            Vec3::new(0.0, 12.0, -18.0),
            Vec3::new(5.0, 14.0, -65.0),
            Vec3::new(28.0, 18.0, -115.0),
            Vec3::new(68.0, 20.0, -150.0),
        ],
        radius: 20.0,
        speed: 14.0,
    }
}
pub fn refuge_current() -> Current {
    Current {
        points: vec![
            Vec3::new(-15.0, 12.0, -63.0),
            Vec3::new(-25.0, 9.0, -85.0),
            Vec3::new(-28.0, 8.0, -101.0),
        ],
        radius: 13.0,
        speed: 9.0,
    }
}
pub fn sample_currents(position: Vec3) -> (Vec3, f32) {
    static LEGACY: std::sync::LazyLock<(Current, Current)> =
        std::sync::LazyLock::new(|| (archipelago_current(), refuge_current()));
    let (a, wa) = LEGACY.0.sample(position);
    let (b, wb) = LEGACY.1.sample(position);
    let large = crate::world::sample_current(position);
    if large.1 > wa.max(wb) {
        return large;
    }
    if wa + wb > 0.0 {
        ((a * wa + b * wb) / (wa + wb), wa.max(wb))
    } else {
        (Vec3::ZERO, 0.0)
    }
}
