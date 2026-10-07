use bevy::{audio::Volume, prelude::*};
#[derive(Resource)]
pub struct Sounds {
    click: Handle<AudioSource>,
    bell: Handle<AudioSource>,
    warning: Handle<AudioSource>,
    tension: Handle<AudioSource>,
}
pub fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(Sounds {
        click: assets.load("audio/click.wav"),
        bell: assets.load("audio/bell.wav"),
        warning: assets.load("audio/warning.wav"),
        tension: assets.load("audio/tension.wav"),
    });
}
pub fn signals(
    mut commands: Commands,
    sounds: Res<Sounds>,
    prefs: Res<crate::persistence::Preferences>,
    game: Res<crate::session::GameSession>,
    tethers: Query<&aether_sim::Tether>,
    transforms: Query<&Transform>,
    mut state: Local<(String, bool)>,
) {
    let new_warning = state.0 != game.notice
        && (game.notice.contains("impossible")
            || game.notice.contains("basse")
            || game.notice.contains("insuffisante"));
    let taut = tethers.iter().any(|t| {
        transforms
            .get(t.vessel)
            .ok()
            .zip(transforms.get(t.anchor).ok())
            .is_some_and(|(v, a)| {
                v.transform_point(t.local_point).distance(a.translation) > t.length * 0.98
            })
    });
    let handle = if new_warning {
        Some(sounds.warning.clone())
    } else if taut && !state.1 {
        Some(sounds.tension.clone())
    } else {
        None
    };
    if prefs.volume > 0.0
        && let Some(handle) = handle
    {
        commands.spawn((
            AudioPlayer::new(handle),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(prefs.volume * 0.22)),
        ));
    }
    state.0.clone_from(&game.notice);
    state.1 = taut;
}
pub fn feedback(world: &mut World, bell: bool) {
    let volume = world.resource::<crate::persistence::Preferences>().volume;
    if volume <= 0.0 {
        return;
    }
    let Some(sounds) = world.get_resource::<Sounds>() else {
        return;
    };
    let sound = if bell {
        sounds.bell.clone()
    } else {
        sounds.click.clone()
    };
    world.spawn((
        AudioPlayer::new(sound),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(volume * 0.3)),
    ));
}
