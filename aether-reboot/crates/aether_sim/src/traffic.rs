//! Visible courier airships are kinematic bodies, including their gondola and
//! balloon. The validated route keeps their entire envelope away from cliffs.
use avian3d::prelude::*;
use bevy::prelude::*;
#[derive(Component)]
pub struct Courier(pub aether_core::traffic::Courier);
pub fn spawn(commands: &mut Commands, seconds: f32) {
    for courier in aether_core::traffic::COURIERS {
        let (position, rotation) = courier.pose(seconds);
        let owner = commands
            .spawn((
                crate::SessionEntity,
                Courier(courier),
                RigidBody::Kinematic,
                TransformInterpolation,
                Visibility::Inherited,
                Transform::from_translation(position)
                    .with_rotation(rotation)
                    .with_scale(Vec3::splat(courier.scale())),
                LinearVelocity::ZERO,
            ))
            .id();
        if let Some(body) = courier.sailing_body() {
            commands.spawn((
                crate::vessel::collider(body),
                crate::vessel::collider_offset(),
                Friction::new(0.7),
                ChildOf(owner),
            ));
            crate::vessel::spawn_equipment(commands, owner, body);
        } else {
            commands.entity(owner).insert(Collider::compound(
                aether_core::traffic::collisions()
                    .iter()
                    .map(|b| {
                        (
                            b.center,
                            Quat::IDENTITY,
                            Collider::cuboid(b.size.x, b.size.y, b.size.z),
                        )
                    })
                    .collect(),
            ));
        }
    }
}
pub fn update(
    clock: Res<crate::SimClock>,
    time: Res<Time<Fixed>>,
    mut couriers: Query<(&Courier, &Position, &mut LinearVelocity, &mut Rotation)>,
) {
    let dt = time.delta_secs();
    let seconds = clock.tick as f32 / aether_core::tuning::TICK_HZ as f32;
    for (courier, p, mut v, mut q) in &mut couriers {
        let (next, rotation) = courier.0.pose(seconds + dt);
        v.0 = (next - p.0) / dt;
        q.0 = rotation;
    }
}
