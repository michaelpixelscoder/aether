use crate::SessionEntity;
use avian3d::prelude::*;
use bevy::prelude::*;

pub use aether_core::terrain::{ANCHORS, ISLANDS, IslandSpec};
#[derive(Component, Clone, Copy)]
pub struct Anchor(pub u32);
#[derive(Component)]
pub struct Tether {
    pub vessel: Entity,
    pub anchor: Entity,
    pub anchor_index: u32,
    pub length: f32,
    pub local_point: Vec3,
}

pub fn spawn_world(commands: &mut Commands) {
    commands.spawn((
        SessionEntity,
        RigidBody::Static,
        Collider::compound(
            aether_core::terrain::sanctuary_pieces()
                .iter()
                .map(|p| {
                    (
                        p.center - aether_core::terrain::SANCTUARY,
                        Quat::IDENTITY,
                        Collider::cuboid(p.size.x, p.size.y, p.size.z),
                    )
                })
                .collect(),
        ),
        Transform::from_translation(aether_core::terrain::SANCTUARY),
    ));
    for (index, island) in ISLANDS.into_iter().enumerate() {
        commands.spawn((
            Name::new(island.name),
            SessionEntity,
            RigidBody::Static,
            Collider::compound(
                aether_core::terrain::collision_boxes(&aether_core::terrain::island_pieces(
                    island.center,
                    island.radius,
                    index as u32,
                ))
                .iter()
                .map(|p| {
                    (
                        p.center - island.center,
                        Quat::IDENTITY,
                        Collider::cuboid(p.size.x, p.size.y, p.size.z),
                    )
                })
                .collect(),
            ),
            Transform::from_translation(island.center),
        ));
        commands.spawn((
            SessionEntity,
            RigidBody::Static,
            Collider::compound(
                aether_core::terrain::dock_boxes(island.dock)
                    .iter()
                    .map(|p| {
                        (
                            p.center - island.dock,
                            Quat::IDENTITY,
                            Collider::cuboid(p.size.x, p.size.y, p.size.z),
                        )
                    })
                    .collect(),
            ),
            Transform::from_translation(island.dock),
        ));
    }
    for (index, position) in aether_core::world::anchors() {
        commands.spawn((
            SessionEntity,
            Anchor(*index),
            RigidBody::Static,
            Collider::sphere(0.8),
            Transform::from_translation(*position),
        ));
    }
}
pub fn attach_tether(
    commands: &mut Commands,
    vessel: Entity,
    anchor: Entity,
    anchor_index: u32,
    local_point: Vec3,
    length: f32,
) -> Entity {
    let length = length.clamp(0.5, 80.0);
    commands
        .spawn((
            SessionEntity,
            Tether {
                vessel,
                anchor,
                anchor_index,
                length,
                local_point,
            },
            DistanceJoint::new(vessel, anchor)
                .with_local_anchor1(local_point)
                .with_local_anchor2(Vec3::ZERO)
                .with_limits(0.0, length),
        ))
        .id()
}
pub fn dock_near(position: Vec3, velocity: Vec3) -> Option<u32> {
    let (id, _, distance) = aether_core::world::nearest_port(position);
    (distance < 9.0 && velocity.length() < 3.5).then_some(id)
}
