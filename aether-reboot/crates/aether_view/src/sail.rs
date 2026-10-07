//! GPU morph targets keep the authored voxel cloth, seams and emblem together.
//! The game supplies apparent wind; this presentation module owns no physics.
use bevy::{mesh::morph::MorphWeights, prelude::*, world_serialization::WorldInstanceReady};

#[derive(Component, Default, Clone, Copy)]
pub struct SailWind {
    pub seconds: f32,
    pub normal_speed: f32,
    pub speed: f32,
    pub phase: f32,
    pub reduced_motion: bool,
}

#[derive(Component)]
pub struct SailSurface(pub Entity);

pub fn bind(
    event: On<WorldInstanceReady>,
    mut commands: Commands,
    roots: Query<(), With<SailWind>>,
    children: Query<&Children>,
    weights: Query<&MorphWeights>,
    surfaces: Query<(&Name, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !roots.contains(event.entity) {
        return;
    }
    for entity in children.iter_descendants(event.entity) {
        if let Ok((name, handle)) = surfaces.get(entity)
            && name.starts_with("Indigo canvas")
            && let Some(mut material) = materials.get_mut(&handle.0)
        {
            // The albedo already contains the navy dye. Multiplying it by a
            // second dark tint hid the weave and wind-driven cloth folds.
            material.base_color = Color::linear_rgb(0.80, 0.86, 1.0);
            // Thin canvas admits a restrained backlight without refraction or
            // an emissive cheat. The opaque embroidery stays reflective.
            material.diffuse_transmission = 0.28;
        }
        if weights.get(entity).is_ok_and(|w| w.weights().len() == 3) {
            commands.entity(entity).insert(SailSurface(event.entity));
        }
    }
}

impl SailWind {
    pub fn weights(self) -> [f32; 3] {
        let pressure = (self.normal_speed / 12.0).clamp(-1.0, 1.0);
        let motion = if self.reduced_motion { 0.25 } else { 1.0 };
        let ripple = (self.speed / 14.0).clamp(0.0, 1.0) * motion;
        let phase = self.seconds * 2.4 + self.phase;
        [pressure, ripple * phase.sin(), ripple * phase.cos()]
    }
}

pub fn animate(wind: Query<&SailWind>, mut surfaces: Query<(&SailSurface, &mut MorphWeights)>) {
    for (surface, mut weights) in &mut surfaces {
        if let Ok(wind) = wind.get(surface.0) {
            weights.weights_mut().copy_from_slice(&wind.weights());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wind_drives_only_its_sail_and_stays_still_when_time_is_paused() {
        let mut app = App::new();
        app.add_systems(Update, animate);
        let a = app
            .world_mut()
            .spawn(SailWind {
                normal_speed: -10.0,
                speed: 10.0,
                ..default()
            })
            .id();
        let b = app.world_mut().spawn(SailWind::default()).id();
        let first = app
            .world_mut()
            .spawn((
                SailSurface(a),
                MorphWeights::new(vec![0.0; 3], None).unwrap(),
            ))
            .id();
        let second = app
            .world_mut()
            .spawn((
                SailSurface(b),
                MorphWeights::new(vec![0.0; 3], None).unwrap(),
            ))
            .id();
        app.update();
        let initial = app
            .world()
            .get::<MorphWeights>(first)
            .unwrap()
            .weights()
            .to_vec();
        assert!(initial[0] < -0.8);
        assert_eq!(
            app.world().get::<MorphWeights>(second).unwrap().weights(),
            &[0.0; 3]
        );
        app.world_mut().get_mut::<SailWind>(a).unwrap().seconds = 1.0;
        app.update();
        let moving = app
            .world()
            .get::<MorphWeights>(first)
            .unwrap()
            .weights()
            .to_vec();
        assert_ne!(initial, moving);
        app.update();
        assert_eq!(
            app.world().get::<MorphWeights>(first).unwrap().weights(),
            moving
        );
        app.world_mut()
            .get_mut::<SailWind>(a)
            .unwrap()
            .reduced_motion = true;
        app.update();
        let reduced = app.world().get::<MorphWeights>(first).unwrap().weights();
        assert_eq!(reduced[0], moving[0]);
        assert!((reduced[1] - moving[1] * 0.25).abs() < 1e-6);
        app.world_mut().get_mut::<SailWind>(a).unwrap().normal_speed = 10.0;
        app.update();
        assert!(app.world().get::<MorphWeights>(first).unwrap().weights()[0] > 0.8);
    }
}
