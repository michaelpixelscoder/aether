#[cfg(target_arch = "wasm32")]
use bevy::prelude::App;
#[cfg(target_arch = "wasm32")]
use runner_game::configure as configure_runner;
#[cfg(target_arch = "wasm32")]
use sandbox_game::configure as configure_sandbox;
#[cfg(target_arch = "wasm32")]
use shipwright_game::configure as configure_shipwright;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
fn configure_game(app: &mut App, game: &str) -> Result<(), String> {
    match game {
        "sandbox" => configure_sandbox(app),
        "runner" => configure_runner(app),
        "shipwright" => configure_shipwright(app),
        _ => return Err(format!("Unknown game: {game}")),
    }
    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start_game(game: &str) -> Result<(), JsValue> {
    let mut app = App::new();
    configure_game(&mut app, game).map_err(|error| JsValue::from_str(&error))?;
    app.run();
    Ok(())
}
