use aether_app::AetherAppPlugin;
use aether_first_person::FirstPersonPlugin;
use aether_grid_ground::GridGroundPlugin;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(AetherAppPlugin {
            title: "Aether Isles — Sandbox",
        })
        .add_plugins((FirstPersonPlugin, GridGroundPlugin))
        .run();
}
