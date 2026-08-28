use aether_app::AetherAppPlugin;
use aether_runner::RunnerPlugin;
use bevy::prelude::*;

pub fn configure(app: &mut App) {
    app.add_plugins(AetherAppPlugin {
        title: "Aether Isles — Skyway Runner",
    })
    .add_plugins(RunnerPlugin);
}

pub fn run() {
    let mut app = App::new();
    configure(&mut app);
    app.run();
}

#[cfg(not(target_arch = "wasm32"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aether_register_game(app: *mut App) {
    let app = unsafe { app.as_mut() }.expect("game loader passed a null app pointer");
    configure(app);
}