//! Continuous regional weather. Sampling and saving use the simulation clock.
use crate::world::REGION_CENTERS;
use glam::Vec3;
#[derive(Clone, Copy, Debug)]
pub struct Weather {
    pub storm: f32,
    pub snow: f32,
    pub ash: f32,
    pub underground: f32,
    pub daylight: f32,
    pub temperature: f32,
}
pub fn sample(position: Vec3, seconds: f32) -> Weather {
    let influence = |region: usize| {
        let d = position.distance(REGION_CENTERS[region]);
        let x = (1.0 - d / 1500.0).clamp(0.0, 1.0);
        x * x * (3.0 - 2.0 * x)
    };
    let storm = influence(6) * (0.7 + 0.3 * (seconds * 0.018).sin());
    let snow = influence(4);
    let ash = influence(3);
    // A cavern is a wide hall under a roof, not a spherical dark spot around
    // its port. Keep the same ambience across its navigable interior, then
    // blend towards daylight above the skylights and beyond the entrances.
    let underground = crate::world::vaults()
        .iter()
        .map(|vault| {
            let local = position - vault.center;
            let horizontal = ((1750.0 - local.x.hypot(local.z)) / 550.0).clamp(0.0, 1.0);
            let vertical = ((550.0 - local.y) / 250.0).clamp(0.0, 1.0);
            let smooth = |x: f32| x * x * (3.0 - 2.0 * x);
            smooth(horizontal) * smooth(vertical)
        })
        .fold(0.0_f32, f32::max);
    let daylight =
        (0.55 + 0.55 * (seconds * std::f32::consts::TAU / 2400.0).cos()).clamp(0.12, 1.0);
    Weather {
        storm,
        snow,
        ash,
        underground,
        daylight,
        temperature: 18.0 - snow * 35.0 + ash * 23.0 - underground * 8.0,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cavern_light_covers_the_hall_and_opens_above_skylights() {
        for vault in crate::world::vaults() {
            for p in [
                Vec3::ZERO,
                Vec3::new(700.0, 200.0, 0.0),
                Vec3::new(0.0, 50.0, 1100.0),
            ] {
                assert_eq!(sample(vault.center + p, 0.0).underground, 1.0);
            }
            assert_eq!(sample(vault.center + Vec3::Y * 600.0, 0.0).underground, 0.0);
            let edge = sample(vault.center + Vec3::Y * 425.0, 0.0).underground;
            assert!((0.0..1.0).contains(&edge));
        }
        assert_eq!(sample(REGION_CENTERS[0], 0.0).underground, 0.0);
    }
    #[test]
    fn regional_weather_is_continuous_bounded_and_distinct() {
        assert!(sample(REGION_CENTERS[4], 0.0).temperature < 0.0);
        assert!(sample(REGION_CENTERS[3], 0.0).temperature > 30.0);
        assert!(sample(REGION_CENTERS[6], 20.0).storm > 0.6);
        for i in 0..1000 {
            let p = Vec3::new(i as f32 * 7.0 - 3000.0, 0.0, -3000.0);
            let a = sample(p, 100.0);
            let b = sample(p + Vec3::X, 100.016);
            assert!((a.temperature - b.temperature).abs() < 0.1);
            assert!((0.12..=1.0).contains(&a.daylight));
        }
    }
}
