use aether_app::AetherAppPlugin;
use aether_runner::RunnerPlugin;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(AetherAppPlugin {
            title: "Aether Isles — Skyway Runner",
        })
        .add_plugins(RunnerPlugin)
        .run();
}
