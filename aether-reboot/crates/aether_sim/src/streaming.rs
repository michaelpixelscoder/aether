//! Proximity enables interaction; distant islands retain their visual silhouette.
use crate::{SessionEntity, Vessel, character::Walker};
use aether_core::{world, world_geometry};
use avian3d::prelude::*;
use bevy::prelude::*;
use std::collections::BTreeSet;
#[derive(Component)]
pub struct IslandCollision(pub u32);
#[derive(Component)]
pub struct VaultCollision(pub u32);
pub fn vault_update(
    mut commands: Commands,
    targets: Query<&Position, Or<(With<Vessel>, With<Walker>)>>,
    loaded: Query<(Entity, &VaultCollision)>,
) {
    let positions: Vec<_> = targets.iter().map(|p| p.0).collect();
    for vault in world::vaults() {
        let distance = positions
            .iter()
            .map(|p| p.distance(vault.center))
            .fold(f32::INFINITY, f32::min);
        let existing = loaded.iter().find(|(_, v)| v.0 == vault.id).map(|(e, _)| e);
        if distance < 2600.0 && existing.is_none() {
            spawn_vault(&mut commands, vault);
        } else if distance > 2900.0
            && let Some(e) = existing
        {
            commands.entity(e).despawn();
        }
    }
}
fn spawn_vault(commands: &mut Commands, vault: world::Vault) {
    let collider = Collider::compound(
        world_geometry::vault_pieces()
            .iter()
            .map(|p| {
                (
                    p.center,
                    Quat::IDENTITY,
                    Collider::cuboid(p.size.x, p.size.y, p.size.z),
                )
            })
            .collect(),
    );
    commands.spawn((
        SessionEntity,
        VaultCollision(vault.id),
        RigidBody::Static,
        collider,
        Transform::from_translation(vault.center).with_rotation(Quat::from_rotation_y(vault.yaw)),
        Friction::new(0.7),
    ));
}
pub fn update(
    mut commands: Commands,
    targets: Query<&Position, Or<(With<Vessel>, With<Walker>)>>,
    loaded: Query<(Entity, &IslandCollision)>,
) {
    let positions: Vec<Vec3> = targets.iter().map(|p| p.0).collect();
    if positions.is_empty() {
        return;
    }
    let loaded_ids: BTreeSet<_> = loaded.iter().map(|(_, i)| i.0).collect();
    for (entity, id) in &loaded {
        if let Some(island) = world::island(id.0)
            && positions
                .iter()
                .all(|p| p.distance(island.center) > island.radius() + 1300.0)
        {
            commands.entity(entity).despawn();
        }
    }
    // Work is bounded during fast transitions; load at least 25 seconds of
    // travel ahead of the fastest current, well outside collision distance.
    let mut wanted: Vec<_> = world::islands()
        .iter()
        .filter(|i| !loaded_ids.contains(&i.id))
        .filter_map(|i| {
            let distance = positions
                .iter()
                .map(|p| p.distance(i.center))
                .fold(f32::INFINITY, f32::min);
            (distance < i.radius() + 1100.0).then_some((i, distance))
        })
        .collect();
    wanted.sort_by(|a, b| a.1.total_cmp(&b.1));
    for (island, _) in wanted.into_iter().take(2) {
        spawn(&mut commands, island);
    }
}
/// Portal arrival has no streaming delay: collision exists before the move.
pub fn ensure(world_ecs: &mut World, id: u32) {
    if let Some(position) = world::dock(id) {
        for vault in world::vaults() {
            if position.distance(vault.center) < 2600.0
                && !world_ecs
                    .query::<&VaultCollision>()
                    .iter(world_ecs)
                    .any(|v| v.0 == vault.id)
            {
                spawn_vault(&mut world_ecs.commands(), vault);
                world_ecs.flush();
            }
        }
    }
    if world_ecs
        .query::<&IslandCollision>()
        .iter(world_ecs)
        .any(|i| i.0 == id)
    {
        return;
    }
    if let Some(island) = world::island(id) {
        spawn(&mut world_ecs.commands(), island);
        world_ecs.flush();
    }
}
fn spawn(commands: &mut Commands, island: &world::WorldIsland) {
    let collider = Collider::compound(
        world_geometry::pieces_for(island.asset_key())
            .iter()
            .map(|p| {
                (
                    p.center * island.scale,
                    Quat::IDENTITY,
                    Collider::cuboid(
                        p.size.x * island.scale,
                        p.size.y * island.scale,
                        p.size.z * island.scale,
                    ),
                )
            })
            .collect(),
    );
    commands.spawn((
        SessionEntity,
        IslandCollision(island.id),
        RigidBody::Static,
        collider,
        Transform::from_translation(island.center).with_rotation(Quat::from_rotation_y(island.yaw)),
        Friction::new(0.7),
    ));
}
