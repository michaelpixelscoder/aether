//! Isolated contact-type timings; no renderer, no claims about GPU or a whole game frame.
use aether_core::{Block, fixtures};
use aether_sim::{SimulationPlugin, collider, collider_offset};
use avian3d::{collision::CollisionDiagnostics, prelude::*};
use bevy::{prelude::*, time::TimeUpdateStrategy, transform::TransformPlugin};
use std::time::Duration;
fn percentiles(values: &mut [f64]) -> [f64; 3] {
    values.sort_by(f64::total_cmp);
    [50, 95, 99].map(|p| values[(values.len() * p / 100).min(values.len() - 1)])
}
fn main() {
    println!(
        "{{\"units\":\"milliseconds\",\"profile\":\"{}\",\"warmup_ticks\":120,\"sample_ticks\":600,\"cases\":[",
        if cfg!(debug_assertions) {
            "dev"
        } else {
            "release"
        }
    );
    for (index, name) in ["sphere_voxel", "voxel_box", "voxel_voxel"]
        .into_iter()
        .enumerate()
    {
        let mut a = App::new();
        a.add_plugins((MinimalPlugins, TransformPlugin, SimulationPlugin))
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
                1.0 / 60.0,
            )));
        a.finish();
        a.cleanup();
        a.update();
        let floor = fixtures::solid([24, 1, 24], Block::Wood);
        let block = fixtures::solid([4, 2, 4], Block::Wood);
        let bottom = if index == 1 {
            Collider::cuboid(12.0, 0.5, 12.0)
        } else {
            collider(&floor)
        };
        let bottom_transform = if index == 1 {
            Transform::from_xyz(5.5, 0.0, 5.5)
        } else {
            collider_offset()
        };
        a.world_mut()
            .spawn((RigidBody::Static, bottom, bottom_transform));
        let top = if index == 0 {
            Collider::sphere(0.5)
        } else {
            collider(&block)
        };
        let top_transform = if index == 0 {
            Transform::from_xyz(5.0, 0.74, 5.0)
        } else {
            Transform::from_xyz(4.75, 0.24, 4.75)
        };
        let body = a
            .world_mut()
            .spawn((RigidBody::Dynamic, top, top_transform, Mass(100.0)))
            .id();
        let mut narrow = Vec::new();
        let mut broad = Vec::new();
        let mut contacts = 0;
        for tick in 0..720 {
            // Keep this pair awake without repeatedly rebuilding its collider.
            a.world_mut().entity_mut(body).remove::<Sleeping>();
            a.update();
            let d = a.world().resource::<CollisionDiagnostics>();
            if tick >= 120 {
                narrow.push(d.narrow_phase.as_secs_f64() * 1000.0);
                broad.push(d.broad_phase.as_secs_f64() * 1000.0);
            }
            contacts = contacts.max(
                a.world()
                    .resource::<ContactGraph>()
                    .iter_active_touching()
                    .count(),
            );
        }
        assert!(contacts > 0, "fixture must produce contacts");
        let n = percentiles(&mut narrow);
        let b = percentiles(&mut broad);
        println!(
            "{}{{\"pair\":\"{}\",\"narrow_p50_p95_p99\":{:?},\"broad_p50_p95_p99\":{:?},\"max_touching_pairs\":{}}}",
            if index > 0 { "," } else { "" },
            name,
            n,
            b,
            contacts
        );
    }
    println!("]}}");
}
