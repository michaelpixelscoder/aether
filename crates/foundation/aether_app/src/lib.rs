use bevy::prelude::*;
use bevy::window::{PresentMode, WindowResolution};

/// Shared window and engine configuration used by every Aether game.
pub struct AetherAppPlugin {
    pub title: &'static str,
}

impl Plugin for AetherAppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: self.title.into(),
                resolution: WindowResolution::new(1280, 720),
                present_mode: PresentMode::AutoVsync,
                canvas: Some("#aether-canvas".into()),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: true,
                transparent: true,
                ..default()
            }),
            ..default()
        }));
    }
}
