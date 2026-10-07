use crate::bindings::{BindingCapture, BindingKey, Control};
use crate::{
    app::Phase,
    camera::CameraRig,
    editor::Editor,
    persistence,
    session::{self, GameSession},
};
use aether_core::{Block, PartKind};
use aether_sim::{
    self as sim, PilotIntent, Vessel,
    character::{WalkerIntent, WalkingConfig},
};
use avian3d::prelude::*;
use bevy::input::keyboard::Key;
use bevy::prelude::*;
use std::collections::VecDeque;

fn letter(code: KeyCode) -> Option<&'static str> {
    Some(match code {
        KeyCode::KeyA => "a",
        KeyCode::KeyB => "b",
        KeyCode::KeyC => "c",
        KeyCode::KeyD => "d",
        KeyCode::KeyE => "e",
        KeyCode::KeyF => "f",
        KeyCode::KeyG => "g",
        KeyCode::KeyJ => "j",
        KeyCode::KeyK => "k",
        KeyCode::KeyL => "l",
        KeyCode::KeyM => "m",
        KeyCode::KeyO => "o",
        KeyCode::KeyQ => "q",
        KeyCode::KeyR => "r",
        KeyCode::KeyS => "s",
        KeyCode::KeyT => "t",
        KeyCode::KeyV => "v",
        KeyCode::KeyW => "w",
        KeyCode::KeyX => "x",
        KeyCode::KeyY => "y",
        KeyCode::KeyZ => "z",
        _ => return None,
    })
}
fn logical_pressed(
    code: KeyCode,
    physical: &ButtonInput<KeyCode>,
    logical: &ButtonInput<Key>,
    just: bool,
) -> bool {
    if let Some(letter) = letter(code) {
        if just {
            logical
                .get_just_pressed()
                .any(|k| matches!(k,Key::Character(c) if c.eq_ignore_ascii_case(letter)))
        } else {
            logical
                .get_pressed()
                .any(|k| matches!(k,Key::Character(c) if c.eq_ignore_ascii_case(letter)))
        }
    } else if just {
        physical.just_pressed(code)
    } else {
        physical.pressed(code)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    Retry,
    NewGame,
    Resume,
    Menu,
    Pause,
    Settings,
    Save,
    Load,
    Export,
    Import,
    Edit,
    Dock,
    Recover,
    Tether,
    Walk,
    Undo,
    Redo,
    SelectBlock(Block),
    SelectPart(PartKind),
    RemovePart,
    Split,
    Volume,
    Motion,
    Invert,
    Sensitivity,
    RotatePart,
    Rebind(Control),
    SetBinding(BindingKey),
    BindingUnavailable,
    SwapBinding,
    CancelBinding,
    ResetBindings,
    Tutorial,
    Atlas,
    Travel,
    TravelTab(crate::travel::Category),
    TravelPage(bool),
    FastTravel(u32),
    Destination(u32),
    Interact,
    Trade(aether_core::expedition::Goods, bool),
    Deliver,
    MotorRefit,
    Refine,
    Engineering,
    CircuitEnable,
    CircuitPort(aether_core::PartId),
    CircuitValve(aether_core::PartId, aether_core::PartId),
    CircuitRepair(aether_core::PartId),
}
#[derive(Resource, Default)]
pub struct Actions(pub VecDeque<Action>);

pub fn collect(
    keys: Res<ButtonInput<KeyCode>>,
    logical: Res<ButtonInput<Key>>,
    phase: Res<State<Phase>>,
    game: Res<GameSession>,
    rig: Res<CameraRig>,
    prefs: Res<persistence::Preferences>,
    capture: Res<BindingCapture>,
    ui: Res<crate::interface::KeyboardUi>,
    windows: Query<&Window>,
    mut actions: ResMut<Actions>,
    mut pilots: Query<(Entity, &mut PilotIntent)>,
    mut walkers: Query<&mut WalkerIntent>,
) {
    let pressed = |key| logical_pressed(key, &keys, &logical, false);
    let just_pressed = |key| logical_pressed(key, &keys, &logical, true);
    let held = |control| prefs.bindings.pressed(control, &keys, &logical, false);
    let tapped = |control| prefs.bindings.pressed(control, &keys, &logical, true);
    let focused = windows.single().is_ok_and(|w| w.focused);
    let ctrl = [KeyCode::ControlLeft, KeyCode::ControlRight]
        .into_iter()
        .any(|key| pressed(key) || keys.just_released(key));
    let playing = *phase.get() == Phase::Playing && focused && !ctrl && !ui.active;
    for (entity, mut intent) in &mut pilots {
        *intent = PilotIntent::default();
        if playing && game.walker.is_none() && Some(entity) == game.active {
            intent.throttle = if held(Control::Backward) {
                0.0
            } else if held(Control::Reverse) {
                -0.55
            } else if held(Control::Forward) {
                1.0
            } else {
                0.0
            };
            intent.brake = held(Control::Backward);
            intent.turn = held(Control::Left) as u8 as f32 - held(Control::Right) as u8 as f32;
            intent.climb = held(Control::Ascend) as u8 as f32 - held(Control::Descend) as u8 as f32;
            intent.trim =
                held(Control::TrimRight) as u8 as f32 - held(Control::TrimLeft) as u8 as f32;
        }
    }
    for mut intent in &mut walkers {
        *intent = WalkerIntent::default();
        if playing {
            let x = held(Control::Right) as u8 as f32 - held(Control::Left) as u8 as f32;
            let z = held(Control::Backward) as u8 as f32 - held(Control::Forward) as u8 as f32;
            intent.direction =
                Quat::from_rotation_y(rig.yaw) * Vec3::new(x, 0.0, z).normalize_or_zero() * 4.0;
            intent.jump = held(Control::Jump);
        }
    }
    if !focused {
        return;
    }
    if ui.consumed {
        return;
    }
    // Buttons run before collection. Their Rebind is dispatched afterward;
    // include it now so a click and key in one frame cannot lose the key.
    let opening_capture = actions.0.iter().any(|a| matches!(a, Action::Rebind(_)));
    if *phase.get() == Phase::Settings && (capture.active.is_some() || opening_capture) {
        if just_pressed(KeyCode::Escape) {
            actions.0.push_back(Action::CancelBinding);
        } else if !ctrl
            && !opening_capture
            && capture.conflict.is_some()
            && just_pressed(KeyCode::Enter)
        {
            actions.0.push_back(Action::SwapBinding);
        } else if !ctrl
            && let Some(key) = logical
                .get_just_pressed()
                .find_map(BindingKey::from_logical)
        {
            actions.0.push_back(Action::SetBinding(key));
        } else if logical.get_just_pressed().next().is_some()
            || keys.get_just_pressed().next().is_some()
        {
            actions.0.push_back(Action::BindingUnavailable);
        }
        return;
    }
    if *phase.get() == Phase::Menu {
        if just_pressed(KeyCode::Enter) {
            actions.0.push_back(Action::NewGame);
        }
        if just_pressed(KeyCode::KeyL) {
            actions.0.push_back(Action::Load);
        }
    }
    if just_pressed(KeyCode::Escape) {
        actions.0.push_back(Action::Pause);
    }
    if tapped(Control::Atlas) && matches!(phase.get(), Phase::Playing | Phase::Atlas) {
        actions.0.push_back(Action::Atlas);
    }
    if *phase.get() == Phase::Atlas && !ctrl {
        return;
    }
    if ui.active {
        return;
    }
    if !matches!(
        phase.get(),
        Phase::Playing | Phase::Editing | Phase::Paused | Phase::Atlas
    ) {
        return;
    }
    if ctrl {
        for (key, action) in [
            (KeyCode::KeyS, Action::Save),
            (KeyCode::KeyO, Action::Load),
            (KeyCode::KeyZ, Action::Undo),
            (KeyCode::KeyY, Action::Redo),
        ] {
            if just_pressed(key) {
                actions.0.push_back(action);
            }
        }
        return;
    }
    if *phase.get() == Phase::Paused {
        return;
    }
    for (control, action) in [
        (Control::Edit, Action::Edit),
        (Control::Dock, Action::Dock),
        (Control::Recover, Action::Recover),
        (Control::Tether, Action::Tether),
        (Control::Walk, Action::Walk),
        (Control::Rotate, Action::RotatePart),
        (Control::Interact, Action::Interact),
    ] {
        if tapped(control) {
            actions.0.push_back(action);
        }
    }
    for (key, action) in [
        (KeyCode::Digit1, Action::SelectBlock(Block::Wood)),
        (KeyCode::Digit2, Action::SelectBlock(Block::Metal)),
        (KeyCode::Digit3, Action::SelectBlock(Block::Glass)),
    ] {
        if just_pressed(key) {
            actions.0.push_back(action);
        }
    }
}
pub fn dispatch(world: &mut World) {
    let actions: Vec<_> = world.resource_mut::<Actions>().0.drain(..).collect();
    for action in actions {
        let phase = *world.resource::<State<Phase>>().get();
        if matches!(phase, Phase::Boot | Phase::Loading | Phase::Error) && action != Action::Retry {
            continue;
        }
        if world.resource::<persistence::Storage>().pending
            && matches!(
                action,
                Action::NewGame
                    | Action::Recover
                    | Action::Dock
                    | Action::Edit
                    | Action::Walk
                    | Action::Load
                    | Action::Import
                    | Action::Undo
                    | Action::Redo
                    | Action::Split
                    | Action::Interact
                    | Action::Trade(_, _)
                    | Action::Deliver
                    | Action::MotorRefit
                    | Action::Refine
                    | Action::CircuitEnable
                    | Action::CircuitPort(_)
                    | Action::CircuitValve(_, _)
                    | Action::CircuitRepair(_)
                    | Action::FastTravel(_)
            )
        {
            notice(
                world,
                "Opération de stockage en cours. Patientez avant de changer la construction.",
            );
            continue;
        }
        crate::audio::feedback(
            world,
            matches!(action, Action::NewGame | Action::Dock | Action::Tether),
        );
        match action {
            Action::Travel => {
                if matches!(phase, Phase::Playing | Phase::Paused | Phase::Atlas) {
                    world.resource_mut::<NextState<Phase>>().set(Phase::Travel);
                }
            }
            Action::TravelTab(category) => {
                let mut menu = world.resource_mut::<crate::travel::TravelMenu>();
                menu.category = category;
                menu.page = 0;
                crate::interface::rebuild(world);
            }
            Action::TravelPage(next) => {
                crate::travel::page(world, next);
                crate::interface::rebuild(world);
            }
            Action::FastTravel(id) => {
                if phase == Phase::Travel {
                    crate::travel::go(world, id);
                }
            }
            Action::Atlas => {
                let target = if phase == Phase::Atlas {
                    Phase::Playing
                } else {
                    Phase::Atlas
                };
                world.resource_mut::<NextState<Phase>>().set(target);
            }
            Action::Destination(id) => {
                if aether_core::world::dock(id).is_some() {
                    world.resource_mut::<GameSession>().expedition.destination = id;
                    crate::interface::rebuild(world);
                }
            }
            Action::Interact => crate::exploration::interact(world),
            Action::Trade(goods, buy) => crate::exploration::trade(world, goods, buy),
            Action::Deliver => crate::exploration::deliver(world),
            Action::MotorRefit => crate::exploration::motor_refit(world),
            Action::Refine => crate::exploration::refine(world),
            Action::Engineering => {
                let open = !world.resource::<crate::editor::Editor>().circuit_panel;
                world.resource_mut::<crate::editor::Editor>().circuit_panel = open;
                if phase != Phase::Atlas {
                    world.resource_mut::<NextState<Phase>>().set(Phase::Atlas);
                } else {
                    crate::interface::rebuild(world);
                }
            }
            Action::CircuitEnable => crate::engineering::enable(world),
            Action::CircuitPort(id) => crate::engineering::select(world, id),
            Action::CircuitValve(a, b) => crate::engineering::valve(world, a, b),
            Action::CircuitRepair(id) => crate::engineering::repair(world, id),
            Action::Tutorial => {
                crate::tutorial::reset(world);
                world.resource_mut::<NextState<Phase>>().set(Phase::Playing);
                notice(
                    world,
                    "Conseils réinitialisés. Chaque étape se valide par une action réelle.",
                );
            }
            Action::Rebind(control) => {
                let mut capture = world.resource_mut::<BindingCapture>();
                capture.selected = Some(control);
                capture.active = Some(control);
                capture.conflict = None;
                capture.message = format!(
                    "{} : appuyez sur une touche. Échap annule.",
                    control.label()
                );
                crate::interface::sync_bindings(world);
            }
            Action::SetBinding(_) | Action::SwapBinding => {
                if let Some(control) = world.resource::<BindingCapture>().active {
                    let key = match action {
                        Action::SetBinding(key) => key,
                        Action::SwapBinding => {
                            let Some((key, _)) = world.resource::<BindingCapture>().conflict else {
                                continue;
                            };
                            key
                        }
                        _ => unreachable!(),
                    };
                    let result = {
                        let mut prefs = world.resource_mut::<persistence::Preferences>();
                        if action == Action::SwapBinding {
                            prefs.bindings.swap(control, key)
                        } else {
                            prefs.bindings.assign(control, key)
                        }
                    };
                    let success = result.is_ok();
                    let conflict = world
                        .resource::<persistence::Preferences>()
                        .bindings
                        .conflict(control, key);
                    let mut capture = world.resource_mut::<BindingCapture>();
                    capture.conflict = None;
                    match result {
                        Ok(()) => {
                            capture.active = None;
                            capture.message = format!("{} : {}", control.label(), key.label());
                        }
                        Err(error) => {
                            capture.message = error;
                            if action != Action::SwapBinding {
                                capture.conflict = conflict.map(|other| (key, other));
                            }
                        }
                    }
                    if success {
                        persistence::save_preferences(world);
                    }
                    crate::interface::sync_bindings(world);
                }
            }
            Action::BindingUnavailable => {
                let mut capture = world.resource_mut::<BindingCapture>();
                capture.conflict = None;
                capture.message = "Cette touche est réservée ou non prise en charge. Utilisez une lettre A–Z, Maj, une flèche, Espace ou Tab. Échap annule.".into();
                crate::interface::sync_bindings(world);
            }
            Action::CancelBinding | Action::ResetBindings => {
                *world.resource_mut::<BindingCapture>() = default();
                if action == Action::ResetBindings {
                    world.resource_mut::<persistence::Preferences>().bindings = default();
                    persistence::save_preferences(world);
                }
                crate::interface::sync_bindings(world);
            }
            Action::Retry => {
                let retry_leaves = world
                    .resource::<crate::app::LoadingStatus>()
                    .assets
                    .iter()
                    .filter(|(_, handle)| {
                        world
                            .resource::<AssetServer>()
                            .get_load_state(handle.id())
                            .is_some_and(|state| state.is_failed())
                    })
                    .map(|(_, handle)| handle.id())
                    .collect();
                let retry_errors = world
                    .resource::<crate::app::LoadingStatus>()
                    .assets
                    .iter()
                    .flat_map(|(_, handle)| {
                        let server = world.resource::<AssetServer>();
                        let direct = match server.get_load_state(handle.id()) {
                            Some(bevy::asset::LoadState::Failed(error)) => Some(error),
                            _ => None,
                        };
                        let dependency =
                            match server.get_recursive_dependency_load_state(handle.id()) {
                                Some(bevy::asset::RecursiveDependencyLoadState::Failed(error)) => {
                                    Some(error)
                                }
                                _ => None,
                            };
                        [direct, dependency].into_iter().flatten()
                    })
                    .collect();
                let mut paths: Vec<_> = world
                    .resource::<crate::app::LoadingStatus>()
                    .assets
                    .iter()
                    .filter(|(_, handle)| {
                        let assets = world.resource::<AssetServer>();
                        assets
                            .get_load_state(handle.id())
                            .is_some_and(|state| state.is_failed())
                    })
                    .map(|(p, _)| p.clone())
                    .collect();
                // Already loading files can finish normally. Restart only
                // failed assets; a timeout with no explicit failure retries
                // the outstanding files without invalidating loaded fonts.
                if paths.is_empty() {
                    let assets = world.resource::<AssetServer>();
                    paths.extend(
                        world
                            .resource::<crate::app::LoadingStatus>()
                            .assets
                            .iter()
                            .filter(|(_, handle)| !assets.is_loaded_with_dependencies(handle.id()))
                            .map(|(path, _)| path.clone()),
                    );
                }
                for path in paths {
                    world.resource::<AssetServer>().reload(path);
                }
                let mut loading = world.resource_mut::<crate::app::LoadingStatus>();
                loading.elapsed = 0.0;
                loading.error.clear();
                loading.retry_errors = retry_errors;
                loading.retry_leaves = retry_leaves;
                loading.retried_parents.clear();
                // The current delta predates the click and can include a long
                // first-frame upload. Start timing from the following frame.
                loading.reset_timer = true;
                world.resource_mut::<NextState<Phase>>().set(Phase::Loading);
            }
            Action::RotatePart => {
                let mut editor = world.resource_mut::<Editor>();
                editor.quarter_turn = (editor.quarter_turn + 1) % 4;
            }
            Action::NewGame => {
                if busy(world) {
                    continue;
                }
                session::replace(world, session::starter());
                world.resource_mut::<NextState<Phase>>().set(Phase::Playing);
                notice(
                    world,
                    format!(
                        "Bienvenue au chantier. {} ouvre l'atelier ; {} largue les amarres.",
                        hint(world, Control::Edit),
                        hint(world, Control::Dock)
                    ),
                );
            }
            Action::Resume => world.resource_mut::<NextState<Phase>>().set(Phase::Playing),
            Action::Menu => {
                if matches!(phase, Phase::Playing | Phase::Editing | Phase::Paused) {
                    persistence::save(world);
                }
                world.resource_mut::<NextState<Phase>>().set(Phase::Menu);
            }
            Action::Pause => {
                if !matches!(
                    phase,
                    Phase::Playing
                        | Phase::Editing
                        | Phase::Paused
                        | Phase::Settings
                        | Phase::Atlas
                        | Phase::Travel
                ) {
                    continue;
                }
                let target = if matches!(phase, Phase::Atlas | Phase::Travel) {
                    Phase::Playing
                } else if phase == Phase::Settings {
                    world.resource::<GameSession>().settings_return_phase
                } else if phase == Phase::Paused {
                    world.resource::<GameSession>().return_phase
                } else {
                    world.resource_mut::<GameSession>().return_phase = phase;
                    Phase::Paused
                };
                world.resource_mut::<NextState<Phase>>().set(target);
            }
            Action::Settings => {
                *world.resource_mut::<BindingCapture>() = default();
                world.resource_mut::<GameSession>().settings_return_phase = phase;
                world
                    .resource_mut::<NextState<Phase>>()
                    .set(Phase::Settings);
            }
            Action::Save => persistence::save(world),
            Action::Load => persistence::load(world),
            Action::Export => persistence::export(world),
            Action::Import => persistence::import(world),
            Action::Recover => {
                if !busy(world) {
                    session::recover(world);
                }
            }
            Action::Edit => {
                let docked = active_vessel(world).is_some_and(|v| v.docked);
                if phase == Phase::Editing {
                    world.resource_mut::<NextState<Phase>>().set(Phase::Playing);
                } else if docked {
                    world.resource_mut::<NextState<Phase>>().set(Phase::Editing);
                } else {
                    notice(
                        world,
                        format!(
                            "Rejoignez un quai et amarrez-vous avec {} avant de construire.",
                            hint(world, Control::Dock)
                        ),
                    );
                }
            }
            Action::Dock => dock(world),
            Action::Tether => tether(world),
            Action::Walk => walk(world),
            Action::Undo | Action::Redo => {
                let circuit_workshop = phase == Phase::Atlas
                    && world.resource::<Editor>().circuit_panel
                    && active_vessel(world).is_some_and(|v| v.docked);
                if phase == Phase::Editing || circuit_workshop {
                    crate::editor::history(world, action == Action::Redo);
                    if circuit_workshop {
                        crate::interface::rebuild(world);
                    }
                }
            }
            Action::SelectBlock(block) => {
                let mut editor = world.resource_mut::<Editor>();
                editor.block = block;
                editor.part = None;
                editor.remove_part = false;
            }
            Action::SelectPart(part) => {
                let mut editor = world.resource_mut::<Editor>();
                editor.part = Some(part);
                editor.remove_part = false;
            }
            Action::RemovePart => {
                world.resource_mut::<Editor>().remove_part = true;
            }
            Action::Split => {
                if phase == Phase::Editing {
                    crate::editor::split(world);
                }
            }
            Action::Volume | Action::Motion | Action::Invert | Action::Sensitivity => {
                {
                    let mut prefs = world.resource_mut::<persistence::Preferences>();
                    match action {
                        Action::Volume => prefs.volume = if prefs.volume > 0.0 { 0.0 } else { 0.6 },
                        Action::Motion => prefs.reduce_motion = !prefs.reduce_motion,
                        Action::Invert => prefs.invert_y = !prefs.invert_y,
                        Action::Sensitivity => {
                            prefs.sensitivity = if prefs.sensitivity >= 1.5 {
                                0.5
                            } else {
                                prefs.sensitivity + 0.5
                            }
                        }
                        _ => {}
                    }
                }
                persistence::save_preferences(world);
                crate::interface::rebuild(world);
            }
        }
    }
}
pub fn notice(world: &mut World, text: impl Into<String>) {
    world.resource_mut::<GameSession>().notice = text.into();
}
fn hint(world: &World, control: Control) -> String {
    world
        .resource::<persistence::Preferences>()
        .bindings
        .label(control)
}
fn busy(world: &mut World) -> bool {
    if world.resource::<persistence::Storage>().pending {
        notice(world, "Une sauvegarde est en cours. Patientez un instant.");
        true
    } else {
        false
    }
}
fn active_vessel(world: &World) -> Option<&Vessel> {
    world.get::<Vessel>(world.resource::<GameSession>().active?)
}
fn dock(world: &mut World) {
    let Some(entity) = world.resource::<GameSession>().active else {
        return;
    };
    if world.resource::<aether_view::MeshMetrics>().pending > 0 {
        notice(world, "La construction est en préparation…");
        return;
    }
    let Some(vessel) = world.get::<Vessel>(entity) else {
        return;
    };
    let docked = vessel.docked;
    if docked {
        if vessel.body.split().len() != 1 {
            notice(
                world,
                "Reliez la coque ou séparez les fragments avant le départ.",
            );
            return;
        }
        if [PartKind::Helm, PartKind::Lift, PartKind::Tank]
            .iter()
            .any(|p| vessel.body.count_parts(*p) == 0)
        {
            notice(
                world,
                "Il faut un poste, un réservoir et un sustentateur pour partir.",
            );
            return;
        }
        world.get_mut::<Vessel>(entity).expect("vessel").docked = false;
        world.entity_mut(entity).insert(RigidBody::Dynamic);
        world.resource_mut::<NextState<Phase>>().set(Phase::Playing);
        world.resource_mut::<GameSession>().progress =
            world.resource::<GameSession>().progress.max(1);
        notice(
            world,
            "Amarres larguées. Suivez le courant lumineux vers les jardins.",
        );
    } else {
        let Some(position) = world.get::<Position>(entity).map(|p| p.0) else {
            return;
        };
        let velocity = world
            .get::<LinearVelocity>(entity)
            .map_or(Vec3::ZERO, |v| v.0);
        let Some(checkpoint) = sim::dock_near(position, velocity) else {
            notice(
                world,
                format!(
                    "Longez le quai à son altitude : moins de 9 m, sous 3,5 m/s. Frein : {}.",
                    hint(world, Control::Backward)
                ),
            );
            return;
        };
        let dock = aether_core::world::dock(checkpoint).expect("nearest valid port");
        let mut vessel = world.get_mut::<Vessel>(entity).expect("vessel");
        vessel.docked = true;
        vessel.target_altitude = dock.y;
        world.entity_mut(entity).insert((
            RigidBody::Static,
            sim::teleport_pose(dock, aether_core::world::dock_rotation(checkpoint)),
            LinearVelocity::ZERO,
            AngularVelocity::ZERO,
            ConstantForce::default(),
            ConstantTorque::default(),
        ));
        release_tether(world);
        world.resource_mut::<GameSession>().checkpoint = checkpoint;
        if checkpoint == 1 {
            world.resource_mut::<GameSession>().progress = 4;
            notice(
                world,
                "Traversée accomplie ! Les jardins vous accueillent. Améliorez votre vaisseau ou repartez.",
            );
        } else {
            notice(
                world,
                format!(
                    "Amarré. Recharge d'Aether en cours ; {} pour construire.",
                    hint(world, Control::Edit)
                ),
            );
        }
        session::request_save(world);
    }
}
pub fn release_tether(world: &mut World) {
    let entities: Vec<_> = world
        .query_filtered::<Entity, With<sim::Tether>>()
        .iter(world)
        .collect();
    for e in entities {
        world.despawn(e);
    }
}
pub use aether_core::terrain::line_clear;
fn tether(world: &mut World) {
    if world.query::<&sim::Tether>().iter(world).next().is_some() {
        release_tether(world);
        notice(world, "Câble libéré.");
        return;
    }
    let Some(entity) = world.resource::<GameSession>().active else {
        return;
    };
    let Some(vessel) = world.get::<Vessel>(entity) else {
        return;
    };
    if vessel.docked {
        notice(world, "Larguez les amarres avant d'utiliser le harpon.");
        return;
    }
    let Some(part) = vessel
        .body
        .parts()
        .iter()
        .find(|p| p.kind == PartKind::Harpoon)
    else {
        notice(world, "Ajoutez un harpon dans l'atelier.");
        return;
    };
    let local = part.center();
    let Some((position, rotation)) = world
        .get::<Position>(entity)
        .zip(world.get::<Rotation>(entity))
    else {
        return;
    };
    let position = position.0 + rotation.0 * local;
    let target = world
        .query::<(Entity, &sim::Anchor, &Transform)>()
        .iter(world)
        .filter(|(_, _, t)| {
            t.translation.distance(position) < 70.0 && line_clear(position, t.translation)
        })
        .min_by(|a, b| {
            a.2.translation
                .distance_squared(position)
                .total_cmp(&b.2.translation.distance_squared(position))
        })
        .map(|(e, a, t)| (e, a.0, t.translation.distance(position)));
    if let Some((anchor, index, length)) = target {
        sim::attach_tether(&mut world.commands(), entity, anchor, index, local, length);
        world.resource_mut::<GameSession>().progress =
            world.resource::<GameSession>().progress.max(3);
        notice(
            world,
            format!(
                "Harpon accroché. {} pour libérer le câble et conserver votre élan.",
                hint(world, Control::Tether)
            ),
        );
    } else {
        notice(world, "Aucune ancre en ligne de vue à moins de 70 m.");
    }
}
fn walk(world: &mut World) {
    if *world.resource::<State<Phase>>().get() != Phase::Playing {
        notice(world, "Terminez l'atelier avant de quitter le poste.");
        return;
    }
    if let Some(entity) = world.resource_mut::<GameSession>().walker.take() {
        world.despawn(entity);
        notice(world, "Au poste de pilotage.");
        return;
    }
    let Some(vessel) = world.resource::<GameSession>().active else {
        return;
    };
    let Some((position, rotation)) = world
        .get::<Position>(vessel)
        .zip(world.get::<Rotation>(vessel))
    else {
        return;
    };
    let transform = Transform::from_translation(position.0).with_rotation(rotation.0);
    let Some(cell) = world
        .get::<Vessel>(vessel)
        .and_then(|v| v.body.boarding_cell())
    else {
        notice(
            world,
            "Aucun emplacement libre sur le pont pour quitter le poste.",
        );
        return;
    };
    let position = transform.transform_point(cell.center() + Vec3::Y * 0.25)
        + Vec3::Y * (aether_core::tuning::WALKER_FLOAT_HEIGHT + 0.05);
    let linear = world
        .get::<LinearVelocity>(vessel)
        .map_or(Vec3::ZERO, |v| v.0);
    let angular = world
        .get::<AngularVelocity>(vessel)
        .map_or(Vec3::ZERO, |v| v.0);
    let center = world
        .get::<Vessel>(vessel)
        .map_or(Vec3::ZERO, |v| v.properties.center);
    let velocity = linear + angular.cross(position - transform.transform_point(center));
    spawn_avatar(world, position, velocity);
    notice(
        world,
        format!(
            "Sur le pont : {} / {} / {} / {} pour marcher, {} pour sauter, {} pour piloter.",
            hint(world, Control::Forward),
            hint(world, Control::Left),
            hint(world, Control::Backward),
            hint(world, Control::Right),
            hint(world, Control::Jump),
            hint(world, Control::Walk)
        ),
    );
}
pub fn spawn_avatar(world: &mut World, position: Vec3, velocity: Vec3) {
    let mut walker = None;
    world.resource_scope(|world, mut configs: Mut<Assets<WalkingConfig>>| {
        walker = Some(sim::character::spawn_walker(
            &mut world.commands(),
            &mut configs,
            position,
        ));
    });
    world.flush();
    if let Some(entity) = walker {
        let assets = world
            .resource::<aether_view::avatar::AvatarAssets>()
            .clone();
        world.resource_scope(|world, palette: Mut<aether_view::Palette>| {
            aether_view::avatar::spawn(
                &mut world.commands(),
                &palette,
                &assets,
                entity,
                false,
                Vec3::new(0.0, -aether_core::tuning::WALKER_FLOAT_HEIGHT, 0.0),
            );
        });
        world.entity_mut(entity).insert((
            LinearVelocity(velocity),
            sim::character::InheritedMotion::from_velocity(velocity),
        ));
        world.resource_mut::<GameSession>().walker = Some(entity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn azerty_shortcut_follows_printed_letter() {
        let mut physical = ButtonInput::default();
        physical.press(KeyCode::KeyW);
        let mut logical = ButtonInput::default();
        logical.press(Key::Character("z".into()));
        assert!(logical_pressed(KeyCode::KeyZ, &physical, &logical, true));
        assert!(!logical_pressed(KeyCode::KeyW, &physical, &logical, true));
    }
}
