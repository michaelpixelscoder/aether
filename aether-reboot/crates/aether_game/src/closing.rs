//! Native close waits for the final save. A failure keeps the window and error visible.
use crate::{
    app::Phase,
    persistence::{self, Storage},
};
use bevy::{prelude::*, window::WindowCloseRequested};
#[derive(Resource, Default)]
pub struct Closing {
    requested: bool,
    attempted: bool,
    failed: bool,
}
pub fn collect(
    mut requests: MessageReader<WindowCloseRequested>,
    mut closing: ResMut<Closing>,
    mut exit: MessageWriter<AppExit>,
) {
    if requests.read().next().is_some() {
        if closing.failed {
            exit.write(AppExit::Success);
        } else {
            closing.requested = true;
        }
    }
}
pub fn finish(world: &mut World) {
    if !world.resource::<Closing>().requested || world.resource::<Storage>().pending {
        return;
    }
    let phase = *world.resource::<State<Phase>>().get();
    let settings_in_game = phase == Phase::Settings
        && world
            .resource::<crate::session::GameSession>()
            .settings_return_phase
            != Phase::Menu;
    if !matches!(
        phase,
        Phase::Playing | Phase::Editing | Phase::Paused | Phase::Atlas | Phase::Travel
    ) && !settings_in_game
    {
        world.write_message(AppExit::Success);
        return;
    }
    if !world.resource::<Closing>().attempted {
        world.resource_mut::<Closing>().attempted = true;
        world.resource_mut::<NextState<Phase>>().set(Phase::Paused);
        persistence::save(world);
        return;
    }
    if world.resource::<Storage>().last_save_ok {
        world.write_message(AppExit::Success);
    } else {
        let mut closing = world.resource_mut::<Closing>();
        closing.requested = false;
        closing.failed = true;
        let previous = world
            .resource::<crate::session::GameSession>()
            .notice
            .clone();
        crate::controls::notice(
            world,
            format!("{previous} Fermez de nouveau pour quitter sans enregistrer."),
        );
    }
}
