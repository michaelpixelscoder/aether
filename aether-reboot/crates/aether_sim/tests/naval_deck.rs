#[allow(dead_code)]
mod common;
use aether_core::{BodyId, fixtures, save::VesselSave, tuning::WALKER_FLOAT_HEIGHT};
use aether_sim::{character::*, spawn_vessel};
use avian3d::prelude::*;
use bevy::prelude::*;

fn walk_to(app: &mut App, player: Entity, target: Vec2) {
    for _ in 0..600 {
        let position = app.world().get::<Position>(player).unwrap().0;
        let delta = target - Vec2::new(position.x, position.z);
        if delta.length() < 0.08 {
            app.world_mut()
                .get_mut::<WalkerIntent>(player)
                .unwrap()
                .direction = Vec3::ZERO;
            common::run(app, 90);
            return;
        }
        app.world_mut()
            .get_mut::<WalkerIntent>(player)
            .unwrap()
            .direction = Vec3::new(delta.x, 0.0, delta.y).clamp_length_max(1.5);
        app.update();
    }
    panic!(
        "blocked quarterdeck path to {target:?}: {:?}",
        app.world().get::<Position>(player).unwrap().0
    );
}

#[test]
fn player_walks_both_real_quarterdeck_steps_and_around_the_mizzen() {
    for side in [-1.0, 1.0] {
        let mut app = common::app();
        let body = fixtures::explorer();
        spawn_vessel(
            &mut app.world_mut().commands(),
            VesselSave {
                circuit: None,
                id: BodyId(1),
                blueprint: body.blueprint(),
                position: Vec3::ZERO,
                rotation: Quat::IDENTITY,
                velocity: Vec3::ZERO,
                angular_velocity: Vec3::ZERO,
                fuel: body.fuel_capacity(),
                trim: 0.0,
                target_altitude: Some(0.0),
                docked: true,
            },
        );
        app.world_mut().flush();
        let mut player = None;
        app.world_mut()
            .resource_scope(|world, mut configs: Mut<Assets<WalkingConfig>>| {
                player = Some(spawn_walker(
                    &mut world.commands(),
                    &mut configs,
                    Vec3::new(0.0, WALKER_FLOAT_HEIGHT + 0.25, 3.0),
                ));
            });
        app.world_mut().flush();
        let player = player.unwrap();
        common::run(&mut app, 120);
        // Enter narrow stairs centrally, then use either side of the wider
        // upper deck to pass the actual mast. Rails remain solid throughout.
        for (x, z, top) in [
            (0.0, 4.4, 1.25),
            (side * 0.60, 4.4, 1.25),
            (side * 0.60, 5.4, 1.25),
            (0.0, 5.6, 1.25),
            (0.0, 6.65, 0.25),
            (0.0, 5.6, 1.25),
            (side * 0.60, 5.4, 1.25),
            (side * 0.60, 4.4, 1.25),
            (0.0, 4.4, 1.25),
            (0.0, 3.0, 0.25),
        ] {
            walk_to(&mut app, player, Vec2::new(x, z));
            let p = app.world().get::<Position>(player).unwrap().0;
            assert!(
                (p.y - top - WALKER_FLOAT_HEIGHT).abs() < 0.10,
                "wrong support height: {p:?}"
            );
            assert!(
                app.world()
                    .get::<WalkerController>(player)
                    .unwrap()
                    .basis_memory
                    .standing_on_entity()
                    .is_some(),
                "lost support at {p:?}"
            );
        }
    }
}
