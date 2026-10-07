//! Kinematic patrols have real collision and advance only on simulation ticks.
use avian3d::prelude::*;
use bevy::prelude::*;
#[derive(Component)]
pub struct Resident(pub aether_core::fauna::Resident);
fn offset(r: aether_core::fauna::Resident) -> Vec3 {
    if r.species == aether_core::fauna::Species::Guardian {
        Vec3::Y * 0.9 * r.scale()
    } else {
        Vec3::ZERO
    }
}
pub fn spawn(commands: &mut Commands, seconds: f32) {
    for r in aether_core::fauna::RESIDENTS {
        let (p, q) = r.pose(seconds);
        let size = match r.species {
            aether_core::fauna::Species::Guardian => Vec3::new(1.2, 1.8, 1.2),
            aether_core::fauna::Species::Cavewing => Vec3::new(4.0, 0.7, 1.6),
        } * r.scale();
        commands.spawn((
            crate::SessionEntity,
            Resident(r),
            RigidBody::Kinematic,
            TransformInterpolation,
            Collider::cuboid(size.x, size.y, size.z),
            Transform::from_translation(p + offset(r)).with_rotation(q),
            LinearVelocity::ZERO,
            Visibility::Inherited,
        ));
    }
}
pub fn update(
    clock: Res<crate::SimClock>,
    time: Res<Time<Fixed>>,
    mut residents: Query<(&Resident, &Position, &mut LinearVelocity, &mut Rotation)>,
) {
    let dt = time.delta_secs();
    let seconds = clock.tick as f32 / aether_core::tuning::TICK_HZ as f32;
    for (r, p, mut v, mut q) in &mut residents {
        let (next, rotation) = r.0.pose(seconds + dt);
        v.0 = (next + offset(r.0) - p.0) / dt;
        q.0 = rotation;
    }
}
pub fn visual_offset(r: aether_core::fauna::Resident) -> Vec3 {
    -offset(r)
}
