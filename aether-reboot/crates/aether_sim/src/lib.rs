//! Fixed-step physical integration. Rendering is not a dependency.
pub mod aether;
pub mod character;
pub mod fauna;
pub mod flight;
pub mod streaming;
pub mod traffic;
pub mod vessel;
pub mod world;

use avian3d::prelude::*;
use bevy::prelude::*;
pub use flight::*;
pub use vessel::*;
pub use world::*;

#[derive(Resource)]
pub struct SimClock {
    pub tick: u64,
    pub seed: u64,
}
impl Default for SimClock {
    fn default() -> Self {
        Self {
            tick: 0,
            seed: 7391,
        }
    }
}
pub struct SimulationPlugin;
impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Time::<Fixed>::from_hz(aether_core::tuning::TICK_HZ))
            .insert_resource(Gravity(Vec3::new(0.0, -aether_core::tuning::GRAVITY, 0.0)))
            .init_resource::<SimClock>()
            .add_plugins(PhysicsPlugins::default())
            .add_systems(
                FixedPostUpdate,
                flight::flight_forces
                    .after(PhysicsSystems::Prepare)
                    .before(PhysicsSystems::StepSimulation),
            )
            .add_systems(FixedPostUpdate, advance_clock.after(PhysicsSystems::Last));
        app.add_systems(
            FixedPostUpdate,
            (fauna::update, traffic::update)
                .after(PhysicsSystems::Prepare)
                .before(PhysicsSystems::StepSimulation),
        );
    }
}
fn advance_clock(mut clock: ResMut<SimClock>) {
    clock.tick = clock.tick.saturating_add(1);
}
