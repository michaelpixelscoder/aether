use aether_core::{BodyId, fixtures, save::VesselSave};
use aether_sim::{character::*, *};
use bevy::{asset::AssetPlugin, prelude::*, time::TimeUpdateStrategy, transform::TransformPlugin};
use std::time::Duration;

pub fn app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        TransformPlugin,
        AssetPlugin::default(),
        SimulationPlugin,
        CharacterPlugin,
    ));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
        1.0 / 60.0,
    )));
    app.finish();
    app.cleanup();
    app.update();
    app
}
pub fn run(app: &mut App, ticks: usize) {
    for _ in 0..ticks {
        app.update();
    }
}
pub fn vessel(app: &mut App, docked: bool) -> Entity {
    let body = fixtures::starter();
    let save = VesselSave {
        circuit: None,
        id: BodyId(1),
        blueprint: body.blueprint(),
        position: Vec3::new(0.0, 12.0, 0.0),
        rotation: Quat::IDENTITY,
        velocity: Vec3::ZERO,
        angular_velocity: Vec3::ZERO,
        fuel: body.fuel_capacity(),
        trim: 0.0,
        target_altitude: Some(12.0),
        docked,
    };
    let entity = spawn_vessel(&mut app.world_mut().commands(), save);
    app.world_mut().flush();
    entity
}
